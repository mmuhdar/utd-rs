//! Pasien dan permintaan darah (SPEK M3; FR-01, FR-02, FR-26; ADR 0002). Salinan identitas pasien dengan kunci
//! no. RM; permintaan darah dengan isian wajib Permenkes 91/2015 §5.6.1; sampel dengan batas usia §5.6.2 no. 4.

use crate::AppState;
use crate::audit::{self, Jejak};
use crate::auth::Pengguna;
use crate::error::{Galat, GalatIsian, Hasil, tolak};
use crate::master::{teks, wajib_teks};
use crate::stok::{ABO, KOMPONEN};
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::PgConnection;
use time::{Date, Duration, OffsetDateTime};
use uuid::Uuid;

pub fn rute() -> Router<AppState> {
    Router::new()
        .route("/pasien", get(cari_pasien).post(simpan_pasien))
        .route("/pasien/{no_rm}", get(detail_pasien))
        .route("/permintaan", get(daftar).post(buat))
        .route("/permintaan/{id}", get(detail))
        .route("/permintaan/{id}/tolak", post(tolak_permintaan))
        .route("/permintaan/{id}/batal", post(batal))
        .route("/permintaan/{id}/selesai", post(selesai))
        .route("/permintaan/{id}/sampel", post(terima_sampel))
}

// ---------- Pasien ----------

#[derive(Deserialize, Clone)]
pub struct IsianPasien {
    no_rm: Option<String>,
    nama: Option<String>,
    tanggal_lahir: Option<Date>,
    jenis_kelamin: Option<String>,
}

/// Menyimpan salinan identitas pasien; SIMRS adalah sumber kebenaran (ADR 0002).
pub async fn upsert_pasien(conn: &mut PgConnection, p: &Pengguna, i: &IsianPasien) -> Hasil<String> {
    let no_rm = wajib_teks(&i.no_rm, "no_rm", "No. rekam medis")?;
    let nama = wajib_teks(&i.nama, "nama", "Nama pasien")?;
    let lahir = i.tanggal_lahir.ok_or_else(|| tolak("tanggal_lahir", "Tanggal lahir wajib diisi."))?;
    if lahir > OffsetDateTime::now_utc().date() {
        return Err(tolak("tanggal_lahir", "Tanggal lahir tidak boleh di masa depan."));
    }
    let jk = i
        .jenis_kelamin
        .as_deref()
        .filter(|j| matches!(*j, "L" | "P"))
        .ok_or_else(|| tolak("jenis_kelamin", "Pilih L atau P."))?;
    let sumber = if p.peran.iter().any(|r| r == "simrs") { "simrs" } else { "manual" };
    let sebelum: Option<Value> = sqlx::query_scalar("select to_jsonb(p) from pasien p where no_rm = $1 for update")
        .bind(no_rm)
        .fetch_optional(&mut *conn)
        .await?;
    let sesudah: Value = sqlx::query_scalar(
        "insert into pasien (no_rm, nama, tanggal_lahir, jenis_kelamin, sumber) values ($1, $2, $3, $4, $5)
         on conflict (no_rm) do update set nama = $2, tanggal_lahir = $3, jenis_kelamin = $4, sumber = $5, diperbarui_pada = now()
         returning to_jsonb(pasien)",
    )
    .bind(no_rm)
    .bind(nama)
    .bind(lahir)
    .bind(jk)
    .bind(sumber)
    .fetch_one(&mut *conn)
    .await?;
    let berubah = sebelum.as_ref().is_none_or(|s| {
        s["nama"] != sesudah["nama"]
            || s["tanggal_lahir"] != sesudah["tanggal_lahir"]
            || s["jenis_kelamin"] != sesudah["jenis_kelamin"]
    });
    if berubah {
        audit::catat(
            conn,
            Some(p),
            Jejak {
                aksi: "simpan",
                objek: "pasien",
                objek_id: Some(no_rm.into()),
                sebelum,
                sesudah: Some(sesudah),
                ..Default::default()
            },
        )
        .await?;
    }
    Ok(no_rm.to_string())
}

async fn simpan_pasien(State(s): State<AppState>, p: Pengguna, Json(i): Json<IsianPasien>) -> Hasil<Json<Value>> {
    p.wajib("pasien.tulis")?;
    let mut tx = s.db.begin().await?;
    let no_rm = upsert_pasien(&mut tx, &p, &i).await?;
    tx.commit().await?;
    Ok(Json(json!({ "no_rm": no_rm })))
}

#[derive(Deserialize)]
struct Cari {
    q: Option<String>,
}

async fn cari_pasien(State(s): State<AppState>, p: Pengguna, Query(c): Query<Cari>) -> Hasil<Json<Value>> {
    p.wajib("pasien.lihat")?;
    let q = teks(&c.q).unwrap_or("");
    if q.chars().count() < 3 {
        return Ok(Json(json!([])));
    }
    let v: Value = sqlx::query_scalar(
        "select coalesce(jsonb_agg(to_jsonb(p) order by p.nama), '[]') from
           (select * from pasien where no_rm = $1 or nama ilike '%' || $1 || '%' order by no_rm = $1 desc, nama limit 20) p",
    )
    .bind(q)
    .fetch_one(&s.db)
    .await?;
    Ok(Json(v))
}

/// Golongan pasien terakhir yang terverifikasi (tidak dibatalkan, tidak diskrepansi).
pub async fn golongan_terverifikasi(conn: &mut PgConnection, no_rm: &str) -> Hasil<Option<(String, String)>> {
    Ok(sqlx::query_as(
        "select abo::text, rhesus::text from pemeriksaan_golongan
         where no_rm = $1 and tanda_tangan_id is not null and batal_alasan is null and not diskrepansi
         order by dicatat_pada desc limit 1",
    )
    .bind(no_rm)
    .fetch_optional(conn)
    .await?)
}

/// GET /api/pasien/{no_rm} — identitas, riwayat golongan, permintaan, transfusi, dan reaksi. Pembukaannya dicatat.
async fn detail_pasien(State(s): State<AppState>, p: Pengguna, Path(no_rm): Path<String>) -> Hasil<Json<Value>> {
    p.wajib("pasien.lihat")?;
    let mut tx = s.db.begin().await?;
    let v: Option<Value> = sqlx::query_scalar(
        "select to_jsonb(ps) || jsonb_build_object(
            'golongan', (select coalesce(jsonb_agg(jsonb_build_object('abo', g.abo, 'rhesus', g.rhesus, 'waktu', g.dicatat_pada,
                            'terverifikasi', g.tanda_tangan_id is not null, 'diskrepansi', g.diskrepansi) order by g.dicatat_pada desc), '[]')
                         from pemeriksaan_golongan g where g.no_rm = ps.no_rm and g.batal_alasan is null),
            'permintaan', (select coalesce(jsonb_agg(jsonb_build_object('id', r.id, 'nomor', r.nomor, 'status', r.status,
                              'jenis', r.jenis, 'ruang', r.ruang, 'diminta_pada', r.diminta_pada) order by r.diminta_pada desc), '[]')
                           from permintaan r where r.no_rm = ps.no_rm),
            'transfusi', (select coalesce(jsonb_agg(jsonb_build_object('id', t.id, 'nomor_kantong', k.nomor, 'komponen', k.komponen,
                             'abo', k.abo, 'rhesus', k.rhesus, 'mulai', t.mulai, 'selesai', t.selesai, 'dihentikan', t.dihentikan,
                             'reaksi', exists (select 1 from reaksi x where x.transfusi_id = t.id)) order by t.mulai desc), '[]')
                          from transfusi t join kantong k on k.id = t.kantong_id where t.no_rm = ps.no_rm))
         from pasien ps where ps.no_rm = $1",
    )
    .bind(no_rm.trim())
    .fetch_optional(&mut *tx)
    .await?;
    let v = v.ok_or_else(|| Galat::TidakDitemukan("Pasien tidak ditemukan.".into()))?;
    audit::catat(
        &mut tx,
        Some(&p),
        Jejak { aksi: "lihat", objek: "pasien", objek_id: Some(no_rm.trim().into()), ..Default::default() },
    )
    .await?;
    tx.commit().await?;
    Ok(Json(v))
}

// ---------- Permintaan ----------

#[derive(Deserialize)]
struct IsianPermintaan {
    no_rm: Option<String>,
    /// Identitas pasien sekaligus (dari SIMRS atau ketik manual); disimpan lebih dulu.
    pasien: Option<IsianPasien>,
    ruang: Option<String>,
    dokter: Option<String>,
    diagnosis: Option<String>,
    indikasi: Option<String>,
    jenis: String,
    alasan_darurat: Option<String>,
    hb: Option<f64>,
    trombosit: Option<i32>,
    abo_klinis: Option<String>,
    rhesus_klinis: Option<String>,
    riwayat_transfusi: Option<bool>,
    riwayat_reaksi: Option<bool>,
    rencana_transfusi: Option<Date>,
    baris: Vec<BarisPermintaan>,
    rujukan_simrs: Option<String>,
}

#[derive(Deserialize, serde::Serialize)]
struct BarisPermintaan {
    komponen: String,
    jumlah: i32,
    volume_ml: Option<i32>,
}

/// POST /api/permintaan — dari dokter di utd-rs atau dari SIMRS (FR-01).
async fn buat(State(s): State<AppState>, p: Pengguna, Json(i): Json<IsianPermintaan>) -> Hasil<Json<Value>> {
    p.wajib("permintaan.buat")?;
    let simrs = p.peran.iter().any(|r| r == "simrs");
    let dokter_sendiri = p.peran.iter().any(|r| r == "dokter");
    let mut galat = Vec::new();
    let mut wajib = |isian: &str, label: &str, t: &Option<String>| {
        if teks(t).is_none() {
            galat.push(GalatIsian { isian: isian.into(), pesan: format!("{label} wajib diisi.") });
        }
    };
    wajib("ruang", "Ruang perawatan", &i.ruang);
    wajib("diagnosis", "Diagnosis klinis", &i.diagnosis);
    wajib("indikasi", "Indikasi transfusi", &i.indikasi);
    if !dokter_sendiri {
        wajib("dokter", "Nama DPJP/dokter peminta", &i.dokter);
    }
    if i.jenis == "darurat" {
        wajib("alasan_darurat", "Alasan permintaan darurat", &i.alasan_darurat);
    }
    if !matches!(i.jenis.as_str(), "elektif" | "rutin" | "darurat") {
        galat.push(GalatIsian { isian: "jenis".into(), pesan: "Pilih elektif, rutin, atau darurat.".into() });
    }
    if i.riwayat_transfusi.is_none() {
        galat.push(GalatIsian { isian: "riwayat_transfusi".into(), pesan: "Isi riwayat transfusi (ya/tidak).".into() });
    }
    if i.riwayat_reaksi.is_none() {
        galat.push(GalatIsian {
            isian: "riwayat_reaksi".into(),
            pesan: "Isi riwayat reaksi transfusi (ya/tidak).".into(),
        });
    }
    if i.hb.is_none() && i.trombosit.is_none() {
        galat.push(GalatIsian { isian: "hb".into(), pesan: "Isi kadar Hb atau trombosit terakhir.".into() });
    }
    if i.abo_klinis.as_deref().is_some_and(|a| !ABO.contains(&a))
        || i.rhesus_klinis.as_deref().is_some_and(|r| !matches!(r, "pos" | "neg"))
    {
        galat.push(GalatIsian { isian: "abo_klinis".into(), pesan: "Golongan darah tidak dikenal.".into() });
    }
    let hari_ini = OffsetDateTime::now_utc().to_offset(time::macros::offset!(+7)).date();
    match i.rencana_transfusi {
        None => {
            galat.push(GalatIsian { isian: "rencana_transfusi".into(), pesan: "Isi tanggal rencana transfusi.".into() })
        }
        Some(d) if d < hari_ini || d > hari_ini + Duration::days(3) => galat.push(GalatIsian {
            isian: "rencana_transfusi".into(),
            pesan: "Permintaan diajukan paling cepat 3 hari sebelum rencana transfusi (Permenkes 91/2015 §5.6.1)."
                .into(),
        }),
        _ => {}
    }
    if i.baris.is_empty()
        || i.baris.iter().any(|b| !KOMPONEN.contains(&b.komponen.as_str()) || !(1..=20).contains(&b.jumlah))
    {
        galat.push(GalatIsian {
            isian: "baris".into(),
            pesan: "Isi komponen dan jumlah kantong (1–20) yang diminta.".into(),
        });
    }
    if !galat.is_empty() {
        return Err(Galat::Validasi(galat));
    }
    let mut tx = s.db.begin().await?;
    let no_rm = match &i.pasien {
        Some(ps) => upsert_pasien(&mut tx, &p, ps).await?,
        None => {
            let no_rm = wajib_teks(&i.no_rm, "no_rm", "No. rekam medis")?;
            let ada: bool = sqlx::query_scalar("select exists (select 1 from pasien where no_rm = $1)")
                .bind(no_rm)
                .fetch_one(&mut *tx)
                .await?;
            if !ada {
                return Err(tolak("no_rm", "Pasien belum terdaftar di bank darah; isi identitasnya."));
            }
            no_rm.to_string()
        }
    };
    let dokter = if dokter_sendiri { p.nama.clone() } else { teks(&i.dokter).unwrap_or_default().to_string() };
    let v: Value = sqlx::query_scalar(
        "insert into permintaan (no_rm, ruang, dokter, dokter_akun_id, diagnosis, indikasi, jenis, alasan_darurat, hb, trombosit,
                                 abo_klinis, rhesus_klinis, riwayat_transfusi, riwayat_reaksi, rencana_transfusi, baris, sumber,
                                 rujukan_simrs, diminta_oleh)
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9::float8::numeric, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
         returning to_jsonb(permintaan)",
    )
    .bind(&no_rm)
    .bind(teks(&i.ruang))
    .bind(&dokter)
    .bind(if dokter_sendiri { Some(p.akun_id) } else { None })
    .bind(teks(&i.diagnosis))
    .bind(teks(&i.indikasi))
    .bind(&i.jenis)
    .bind(teks(&i.alasan_darurat))
    .bind(i.hb)
    .bind(i.trombosit)
    .bind(teks(&i.abo_klinis))
    .bind(teks(&i.rhesus_klinis))
    .bind(i.riwayat_transfusi)
    .bind(i.riwayat_reaksi)
    .bind(i.rencana_transfusi)
    .bind(serde_json::to_value(&i.baris).map_err(anyhow::Error::from)?)
    .bind(if simrs { "simrs" } else { "utd_rs" })
    .bind(teks(&i.rujukan_simrs))
    .bind(p.akun_id)
    .fetch_one(&mut *tx)
    .await?;
    audit::ubah(&mut tx, &p, "buat", "permintaan", v["id"].as_str().unwrap_or_default(), v.clone()).await?;
    tx.commit().await?;
    Ok(Json(v))
}

#[derive(Deserialize)]
struct FilterPermintaan {
    status: Option<String>,
    no_rm: Option<String>,
}

/// GET /api/permintaan — bawaan: permintaan yang masih berjalan; darurat lebih dulu, lalu yang terlama.
async fn daftar(State(s): State<AppState>, p: Pengguna, Query(f): Query<FilterPermintaan>) -> Hasil<Json<Value>> {
    p.wajib("permintaan.lihat")?;
    let v: Value = sqlx::query_scalar(
        "select coalesce(jsonb_agg(x order by x->>'jenis' = 'darurat' desc, x->>'diminta_pada'), '[]') from (
           select to_jsonb(r) || jsonb_build_object('pasien', to_jsonb(ps),
                    'dialokasikan', (select count(*) from alokasi a where a.permintaan_id = r.id and a.dilepas_pada is null),
                    'dikeluarkan', (select count(*) from pengeluaran g join alokasi a on a.id = g.alokasi_id where a.permintaan_id = r.id),
                    'diminta', (select sum((b->>'jumlah')::int) from jsonb_array_elements(r.baris) b),
                    'golongan_terverifikasi', (select jsonb_build_object('abo', g.abo, 'rhesus', g.rhesus) from pemeriksaan_golongan g
                        where g.no_rm = r.no_rm and g.tanda_tangan_id is not null and g.batal_alasan is null and not g.diskrepansi
                        order by g.dicatat_pada desc limit 1)) x
           from permintaan r join pasien ps on ps.no_rm = r.no_rm
           where r.status = any(coalesce(string_to_array($1, ','), array['diminta', 'diproses', 'siap']))
             and ($2::text is null or r.no_rm = $2)
           order by r.diminta_pada desc limit 300) t",
    )
    .bind(teks(&f.status))
    .bind(teks(&f.no_rm))
    .fetch_one(&s.db)
    .await?;
    Ok(Json(v))
}

/// Detail permintaan beserta sampel, golongan, uji silang, alokasi, pengeluaran, dan transfusi.
pub async fn ambil_detail(conn: &mut PgConnection, id: Uuid) -> Hasil<Value> {
    let v: Option<Value> = sqlx::query_scalar(
        "select to_jsonb(r) || jsonb_build_object(
            'pasien', to_jsonb(ps),
            'sampel', (select coalesce(jsonb_agg(to_jsonb(sp) || jsonb_build_object('layak', sp.status = 'diterima' and sp.batas_pakai > now(),
                          'golongan', (select to_jsonb(g) || jsonb_build_object('petugas', a.nama, 'terverifikasi', g.tanda_tangan_id is not null)
                                       from pemeriksaan_golongan g join akun a on a.id = g.dicatat_oleh
                                       where g.sampel_id = sp.id and g.batal_alasan is null))
                          order by sp.diterima_pada desc), '[]') from sampel sp where sp.permintaan_id = r.id),
            'uji_silang', (select coalesce(jsonb_agg(to_jsonb(u) || jsonb_build_object('nomor_kantong', k.nomor, 'komponen', k.komponen,
                              'abo_label', k.abo, 'rhesus_label', k.rhesus, 'petugas', a.nama, 'terverifikasi', u.tanda_tangan_id is not null)
                              order by u.dicatat_pada), '[]')
                           from uji_silang u join kantong k on k.id = u.kantong_id join akun a on a.id = u.dicatat_oleh
                           where u.permintaan_id = r.id and u.batal_alasan is null),
            'alokasi', (select coalesce(jsonb_agg(to_jsonb(al) || jsonb_build_object('kantong', to_jsonb(k),
                            'pengeluaran', (select to_jsonb(g) from pengeluaran g where g.alokasi_id = al.id),
                            'transfusi', (select to_jsonb(t) from transfusi t join pengeluaran g on g.id = t.pengeluaran_id
                                          where g.alokasi_id = al.id))
                            order by al.dialokasikan_pada), '[]')
                        from alokasi al join kantong k on k.id = al.kantong_id where al.permintaan_id = r.id))
         from permintaan r join pasien ps on ps.no_rm = r.no_rm where r.id = $1",
    )
    .bind(id)
    .fetch_optional(conn)
    .await?;
    v.ok_or_else(|| Galat::TidakDitemukan("Permintaan darah tidak ditemukan.".into()))
}

async fn detail(State(s): State<AppState>, p: Pengguna, Path(id): Path<Uuid>) -> Hasil<Json<Value>> {
    p.wajib("permintaan.lihat")?;
    let mut tx = s.db.begin().await?;
    let mut v = ambil_detail(&mut tx, id).await?;
    let no_rm = v["no_rm"].as_str().unwrap_or_default().to_string();
    v["golongan_terverifikasi"] = match golongan_terverifikasi(&mut tx, &no_rm).await? {
        Some((abo, rh)) => json!({ "abo": abo, "rhesus": rh }),
        None => Value::Null,
    };
    audit::catat(
        &mut tx,
        Some(&p),
        Jejak { aksi: "lihat", objek: "permintaan", objek_id: Some(id.to_string()), ..Default::default() },
    )
    .await?;
    tx.commit().await?;
    Ok(Json(v))
}

#[derive(Deserialize)]
pub struct Alasan {
    pub alasan: Option<String>,
}

/// Melepas semua alokasi aktif permintaan; kantong kembali tersedia.
pub async fn lepas_semua(conn: &mut PgConnection, p: &Pengguna, permintaan: Uuid, alasan: &str) -> Hasil<()> {
    let kantong: Vec<Uuid> = sqlx::query_scalar(
        "update alokasi a set dilepas_pada = now(), alasan_lepas = $2
         where a.permintaan_id = $1 and a.dilepas_pada is null
           and not exists (select 1 from pengeluaran g where g.alokasi_id = a.id)
         returning a.kantong_id",
    )
    .bind(permintaan)
    .bind(alasan)
    .fetch_all(&mut *conn)
    .await?;
    for k in kantong {
        crate::stok::ubah_status(conn, p, k, &["dialokasikan"], "tersedia", Some(alasan)).await?;
    }
    Ok(())
}

async fn akhiri(s: &AppState, p: &Pengguna, id: Uuid, dari: &[&str], ke: &str, alasan: &str) -> Hasil<Json<Value>> {
    let mut tx = s.db.begin().await?;
    let status: Option<String> = sqlx::query_scalar("select status from permintaan where id = $1 for update")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;
    let status = status.ok_or_else(|| Galat::TidakDitemukan("Permintaan darah tidak ditemukan.".into()))?;
    if !dari.contains(&status.as_str()) {
        return Err(Galat::Konflik(format!("Permintaan berstatus {status}; tidak bisa diubah menjadi {ke}.")));
    }
    lepas_semua(&mut tx, p, id, &format!("Permintaan {ke}: {alasan}")).await?;
    sqlx::query("update permintaan set status = $2, alasan_status = $3 where id = $1")
        .bind(id)
        .bind(ke)
        .bind(alasan)
        .execute(&mut *tx)
        .await?;
    audit::catat(
        &mut tx,
        Some(p),
        Jejak {
            aksi: "status",
            objek: "permintaan",
            objek_id: Some(id.to_string()),
            sebelum: Some(json!({ "status": status })),
            sesudah: Some(json!({ "status": ke })),
            alasan: Some(alasan),
            ..Default::default()
        },
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({})))
}

/// Formulir tidak lengkap atau tidak sesuai dikembalikan ke ruang (§5.6.2 no. 3).
async fn tolak_permintaan(
    State(s): State<AppState>,
    p: Pengguna,
    Path(id): Path<Uuid>,
    Json(a): Json<Alasan>,
) -> Hasil<Json<Value>> {
    p.wajib("permintaan.proses")?;
    let alasan = wajib_teks(&a.alasan, "alasan", "Alasan penolakan")?;
    akhiri(&s, &p, id, &["diminta", "diproses"], "ditolak", alasan).await
}

async fn batal(
    State(s): State<AppState>,
    p: Pengguna,
    Path(id): Path<Uuid>,
    Json(a): Json<Alasan>,
) -> Hasil<Json<Value>> {
    if !p.boleh("permintaan.buat") {
        p.wajib("permintaan.proses")?;
    }
    let alasan = wajib_teks(&a.alasan, "alasan", "Alasan pembatalan")?;
    akhiri(&s, &p, id, &["diminta", "diproses", "siap"], "batal", alasan).await
}

/// Menutup permintaan; kantong yang belum dikeluarkan kembali ke stok.
async fn selesai(
    State(s): State<AppState>,
    p: Pengguna,
    Path(id): Path<Uuid>,
    Json(a): Json<Alasan>,
) -> Hasil<Json<Value>> {
    p.wajib("permintaan.proses")?;
    let alasan = teks(&a.alasan).unwrap_or("Kebutuhan terpenuhi");
    akhiri(&s, &p, id, &["diproses", "siap"], "selesai", alasan).await
}

// ---------- Sampel ----------

#[derive(Deserialize)]
struct IsianSampel {
    #[serde(with = "time::serde::rfc3339")]
    diambil_pada: OffsetDateTime,
    diambil_oleh: Option<String>,
    jenis: String,
    penyimpanan: String,
    diterima: bool,
    alasan_tolak: Option<String>,
}

/// Batas usia sampel sejak diambil (Permenkes 91/2015 §5.6.2 no. 4).
pub fn batas_usia(jenis: &str, penyimpanan: &str) -> Option<Duration> {
    match (jenis, penyimpanan) {
        ("edta", "suhu_ruang") => Some(Duration::hours(24)),
        ("edta", "dingin") => Some(Duration::days(3)),
        ("serum", "dingin") => Some(Duration::days(7)),
        _ => None,
    }
}

pub const ALASAN_TOLAK_SAMPEL: [&str; 5] = ["lisis", "bekuan", "volume kurang", "label tidak sesuai", "tanpa label"];

async fn terima_sampel(
    State(s): State<AppState>,
    p: Pengguna,
    Path(id): Path<Uuid>,
    Json(i): Json<IsianSampel>,
) -> Hasil<Json<Value>> {
    p.wajib("permintaan.proses")?;
    let diambil_oleh = wajib_teks(&i.diambil_oleh, "diambil_oleh", "Nama pengambil sampel")?;
    let usia = batas_usia(&i.jenis, &i.penyimpanan)
        .ok_or_else(|| tolak("penyimpanan", "Serum/plasma hanya boleh disimpan dingin (4 °C)."))?;
    let sekarang = OffsetDateTime::now_utc();
    if i.diambil_pada > sekarang + Duration::minutes(5) {
        return Err(tolak("diambil_pada", "Waktu pengambilan tidak boleh di masa depan."));
    }
    let batas = i.diambil_pada + usia;
    if i.diterima && batas <= sekarang {
        return Err(tolak("diambil_pada", "Sampel sudah melewati batas usia; minta sampel baru dari ruang."));
    }
    if !i.diterima && !teks(&i.alasan_tolak).is_some_and(|a| ALASAN_TOLAK_SAMPEL.contains(&a)) {
        return Err(tolak("alasan_tolak", "Pilih alasan penolakan sampel."));
    }
    let mut tx = s.db.begin().await?;
    let status: Option<String> = sqlx::query_scalar("select status from permintaan where id = $1 for update")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;
    match status.as_deref() {
        None => return Err(Galat::TidakDitemukan("Permintaan darah tidak ditemukan.".into())),
        Some("diminta" | "diproses" | "siap") => {}
        Some(st) => return Err(Galat::Konflik(format!("Permintaan berstatus {st}; tidak menerima sampel."))),
    }
    let v: Value = sqlx::query_scalar(
        "insert into sampel (permintaan_id, diambil_pada, diambil_oleh, jenis, penyimpanan, batas_pakai, status, alasan_tolak, diterima_oleh)
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9) returning to_jsonb(sampel)",
    )
    .bind(id)
    .bind(i.diambil_pada)
    .bind(diambil_oleh)
    .bind(&i.jenis)
    .bind(&i.penyimpanan)
    .bind(batas)
    .bind(if i.diterima { "diterima" } else { "ditolak" })
    .bind(if i.diterima { None } else { teks(&i.alasan_tolak) })
    .bind(p.akun_id)
    .fetch_one(&mut *tx)
    .await?;
    if i.diterima {
        sqlx::query("update permintaan set status = 'diproses' where id = $1 and status = 'diminta'")
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    audit::ubah(
        &mut tx,
        &p,
        if i.diterima { "terima" } else { "tolak" },
        "sampel",
        v["id"].as_str().unwrap_or_default(),
        v.clone(),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(v))
}
