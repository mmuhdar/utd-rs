//! Pengeluaran, verifikasi samping tempat tidur, transfusi, observasi, reaksi, dan insiden (SPEK M4; FR-14..FR-17,
//! FR-34; Permenkes 91/2015 §5.8–§5.9, §6.1).

use crate::AppState;
use crate::audit::{self, Jejak};
use crate::auth::Pengguna;
use crate::error::{Galat, Hasil, tolak};
use crate::master::{teks, wajib_teks};
use crate::pratransfusi::kompatibel;
use crate::stok::ubah_status;
use crate::tanda_tangan;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::PgConnection;
use time::{Date, Duration, OffsetDateTime};
use uuid::Uuid;

pub fn rute() -> Router<AppState> {
    Router::new()
        .route("/alokasi/{id}/keluarkan", post(keluarkan))
        .route("/pengeluaran/aktif", get(pengeluaran_aktif))
        .route("/pengeluaran/{id}/kembali", post(kembali))
        .route("/transfusi/verifikasi", post(verifikasi_samping))
        .route("/transfusi/aktif", get(transfusi_aktif))
        .route("/transfusi/{id}", get(detail))
        .route("/transfusi/{id}/observasi", post(observasi))
        .route("/transfusi/{id}/selesai", post(selesai))
        .route("/transfusi/{id}/reaksi", post(lapor_reaksi))
        .route("/observasi/{id}/koreksi", post(koreksi_observasi))
        .route("/reaksi", get(daftar_reaksi))
        .route("/reaksi/{id}", get(detail_reaksi))
        .route("/reaksi/{id}/telusur", post(telusur))
        .route("/reaksi/{id}/tutup", post(tutup_reaksi))
        .route("/insiden", get(daftar_insiden))
}

/// Batas waktu sejak keluar dari BDRS sampai transfusi dimulai (§5.8.2 no. 5).
pub const MENIT_MAKS_KELUAR: i64 = 30;

#[derive(Deserialize)]
struct IsianKeluar {
    penerima_ruang: Option<String>,
}

/// POST /api/alokasi/{id}/keluarkan — hanya kantong teralokasi, belum kedaluwarsa, dengan uji silang terverifikasi
/// (atau alokasi tanpa uji silang yang sah: aturan komponen `tidak_perlu` atau darurat yang disetujui).
async fn keluarkan(
    State(s): State<AppState>,
    p: Pengguna,
    Path(alokasi): Path<Uuid>,
    Json(i): Json<IsianKeluar>,
) -> Hasil<Json<Value>> {
    p.wajib("penyerahan.kelola")?;
    let penerima =
        wajib_teks(&i.penerima_ruang, "penerima_ruang", "Nama petugas ruang yang menerima (bukan keluarga pasien)")?;
    let mut tx = s.db.begin().await?;
    let r: Option<(Uuid, Option<Uuid>, OffsetDateTime, String, bool)> = sqlx::query_as(
        "select a.kantong_id, a.uji_silang_id, k.kedaluwarsa, r.status,
                coalesce((select u.tanda_tangan_id is not null from uji_silang u where u.id = a.uji_silang_id), false)
         from alokasi a join kantong k on k.id = a.kantong_id join permintaan r on r.id = a.permintaan_id
         where a.id = $1 and a.dilepas_pada is null for update of a",
    )
    .bind(alokasi)
    .fetch_optional(&mut *tx)
    .await?;
    let (kantong, uji, kedaluwarsa, status, uji_ok) =
        r.ok_or_else(|| Galat::TidakDitemukan("Alokasi aktif tidak ditemukan.".into()))?;
    if kedaluwarsa <= OffsetDateTime::now_utc() {
        return Err(Galat::Konflik("Kantong sudah kedaluwarsa; jangan dikeluarkan.".into()));
    }
    if uji.is_some() && !uji_ok {
        return Err(Galat::Konflik("Uji silang kantong ini belum diverifikasi.".into()));
    }
    if !matches!(status.as_str(), "diproses" | "siap") {
        return Err(Galat::Konflik(format!("Permintaan berstatus {status}.")));
    }
    ubah_status(&mut tx, &p, kantong, &["dialokasikan"], "dikeluarkan", None).await?;
    let v: Value = sqlx::query_scalar(
        "insert into pengeluaran (alokasi_id, kantong_id, dikeluarkan_oleh, penerima_ruang) values ($1, $2, $3, $4) returning to_jsonb(pengeluaran)",
    )
    .bind(alokasi)
    .bind(kantong)
    .bind(p.akun_id)
    .bind(penerima)
    .fetch_one(&mut *tx)
    .await?;
    audit::ubah(&mut tx, &p, "keluarkan", "kantong", kantong, v.clone()).await?;
    tx.commit().await?;
    Ok(Json(v))
}

/// Kantong yang sudah keluar tetapi belum ditransfusikan; `terlambat` bila lewat 30 menit.
async fn pengeluaran_aktif(State(s): State<AppState>, p: Pengguna) -> Hasil<Json<Value>> {
    p.wajib("permintaan.lihat")?;
    let v: Value = sqlx::query_scalar(
        "select coalesce(jsonb_agg(to_jsonb(g) || jsonb_build_object('nomor_kantong', k.nomor, 'komponen', k.komponen,
                    'abo', k.abo, 'rhesus', k.rhesus, 'permintaan', r.nomor, 'ruang', r.ruang, 'pasien', to_jsonb(ps),
                    'menit', floor(extract(epoch from now() - g.dikeluarkan_pada) / 60),
                    'terlambat', now() - g.dikeluarkan_pada > make_interval(mins => $1))
                order by g.dikeluarkan_pada), '[]')
         from pengeluaran g join kantong k on k.id = g.kantong_id join alokasi a on a.id = g.alokasi_id
         join permintaan r on r.id = a.permintaan_id join pasien ps on ps.no_rm = r.no_rm
         where k.status = 'dikeluarkan'",
    )
    .bind(MENIT_MAKS_KELUAR as i32)
    .fetch_one(&s.db)
    .await?;
    Ok(Json(v))
}

/// Kantong kembali dari ruang tanpa ditransfusikan: dikarantina, lalu dikembalikan ke UTD atau dimusnahkan.
async fn kembali(
    State(s): State<AppState>,
    p: Pengguna,
    Path(id): Path<Uuid>,
    Json(a): Json<crate::permintaan::Alasan>,
) -> Hasil<Json<Value>> {
    p.wajib("penyerahan.kelola")?;
    let alasan = wajib_teks(&a.alasan, "alasan", "Alasan kantong dikembalikan dari ruang")?;
    let mut tx = s.db.begin().await?;
    let kantong: Option<Uuid> = sqlx::query_scalar("select kantong_id from pengeluaran where id = $1")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;
    let kantong = kantong.ok_or_else(|| Galat::TidakDitemukan("Pengeluaran tidak ditemukan.".into()))?;
    ubah_status(&mut tx, &p, kantong, &["dikeluarkan"], "karantina", Some(&format!("Kembali dari ruang: {alasan}")))
        .await?;
    tx.commit().await?;
    Ok(Json(json!({})))
}

// ---------- Verifikasi samping tempat tidur ----------

#[derive(Deserialize)]
struct IsianVerifikasi {
    no_rm: Option<String>,
    tanggal_lahir: Option<Date>,
    nomor_kantong: Option<String>,
    petugas_2: Option<String>,
    sandi_petugas_2: Option<String>,
    rawat_jalan: Option<bool>,
    alasan_terlambat: Option<String>,
}

struct Cocok {
    pengeluaran: Uuid,
    kantong: Uuid,
    no_rm: String,
    dikeluarkan: OffsetDateTime,
}

/// Pencocokan pasien dan kantong (§6.1.2). Mengembalikan daftar ketidaksesuaian (kosong = cocok).
async fn cocokkan(conn: &mut PgConnection, i: &IsianVerifikasi) -> Hasil<(Option<Cocok>, Vec<String>, Option<Uuid>)> {
    let nomor = teks(&i.nomor_kantong).ok_or_else(|| tolak("nomor_kantong", "Pindai label kantong."))?;
    let no_rm = teks(&i.no_rm).ok_or_else(|| tolak("no_rm", "Pindai gelang pasien."))?;
    let lahir = i
        .tanggal_lahir
        .ok_or_else(|| tolak("tanggal_lahir", "Konfirmasi tanggal lahir pasien (tanya pasien atau lihat gelang)."))?;
    type Baris = (
        Option<Uuid>,
        Uuid,
        String,
        String,
        String,
        String,
        OffsetDateTime,
        Option<OffsetDateTime>,
        Option<String>,
        Option<Date>,
    );
    let r: Option<Baris> = sqlx::query_as(
        "select g.id, k.id, k.status, k.komponen::text, k.abo::text, k.rhesus::text, k.kedaluwarsa, g.dikeluarkan_pada, r.no_rm, ps.tanggal_lahir
         from kantong k left join pengeluaran g on g.kantong_id = k.id left join alokasi a on a.id = g.alokasi_id
         left join permintaan r on r.id = a.permintaan_id left join pasien ps on ps.no_rm = r.no_rm
         where k.nomor = $1",
    )
    .bind(nomor)
    .fetch_optional(&mut *conn)
    .await?;
    let Some((peng, kantong, status, komponen, abo, rh, exp, keluar, rm_tujuan, lahir_tujuan)) = r else {
        return Ok((None, vec![format!("Kantong {nomor} tidak dikenal oleh bank darah.")], None));
    };
    let (peng, keluar, rm_tujuan) =
        (peng.unwrap_or_default(), keluar.unwrap_or(OffsetDateTime::UNIX_EPOCH), rm_tujuan.unwrap_or_default());
    let lahir_tujuan = lahir_tujuan.unwrap_or(Date::MIN);
    let mut beda = Vec::new();
    if status != "dikeluarkan" {
        beda.push(format!("Kantong berstatus {status}, bukan dikeluarkan dari BDRS."));
    }
    if rm_tujuan != no_rm {
        beda.push(format!("Kantong ini untuk pasien No. RM {rm_tujuan}, bukan {no_rm}."));
    }
    if rm_tujuan == no_rm && lahir_tujuan != lahir {
        beda.push("Tanggal lahir pasien tidak sesuai data permintaan.".into());
    }
    if exp <= OffsetDateTime::now_utc() {
        beda.push("Kantong sudah kedaluwarsa.".into());
    }
    if rm_tujuan == no_rm {
        let gol = crate::permintaan::golongan_terverifikasi(conn, no_rm).await?;
        let darurat: bool = sqlx::query_scalar("select coalesce((select a.darurat from pengeluaran g join alokasi a on a.id = g.alokasi_id where g.id = $1), false)")
            .bind(peng)
            .fetch_one(&mut *conn)
            .await?;
        if let Some((pa, pr)) = gol
            && !darurat
            && !kompatibel(&komponen, &pa, &pr, &abo, &rh)
        {
            beda.push(format!("Golongan kantong {abo}{rh} tidak kompatibel dengan golongan pasien {pa}{pr}."));
        }
    }
    let cocok =
        beda.is_empty().then(|| Cocok { pengeluaran: peng, kantong, no_rm: no_rm.to_string(), dikeluarkan: keluar });
    Ok((cocok, beda, Some(kantong)))
}

/// POST /api/transfusi/verifikasi — dua petugas di samping tempat tidur. Ketidaksesuaian memblokir transfusi dan
/// dicatat sebagai kejadian nyaris cedera (KNC).
async fn verifikasi_samping(
    State(s): State<AppState>,
    p: Pengguna,
    Json(i): Json<IsianVerifikasi>,
) -> Hasil<Json<Value>> {
    p.wajib("transfusi.catat")?;
    let mut tx = s.db.begin().await?;
    let (cocok, beda, kantong) = cocokkan(&mut tx, &i).await?;
    let Some(c) = cocok else {
        let uraian = format!("Verifikasi samping tempat tidur gagal: {}", beda.join(" "));
        let pasien_ada: bool = sqlx::query_scalar("select exists (select 1 from pasien where no_rm = $1)")
            .bind(teks(&i.no_rm))
            .fetch_one(&mut *tx)
            .await?;
        let id: Uuid = sqlx::query_scalar(
            "insert into insiden (jenis, uraian, no_rm, kantong_id, dilaporkan_oleh) values ('knc', $1, $2, $3, $4) returning id",
        )
        .bind(&uraian)
        .bind(if pasien_ada { teks(&i.no_rm) } else { None })
        .bind(kantong)
        .bind(p.akun_id)
        .fetch_one(&mut *tx)
        .await?;
        audit::ubah(&mut tx, &p, "lapor", "insiden", id, json!({ "jenis": "knc", "uraian": uraian })).await?;
        tx.commit().await?;
        return Err(Galat::Konflik(format!(
            "JANGAN TRANSFUSIKAN. {} Kejadian ini dicatat sebagai kejadian nyaris cedera; hubungi bank darah.",
            beda.join(" ")
        )));
    };
    let menit = (OffsetDateTime::now_utc() - c.dikeluarkan).whole_minutes();
    let terlambat = menit > MENIT_MAKS_KELUAR;
    if terlambat && teks(&i.alasan_terlambat).is_none() {
        return Err(tolak(
            "alasan_terlambat",
            format!(
                "Kantong sudah {menit} menit di luar bank darah (batas {MENIT_MAKS_KELUAR} menit). Isi alasan bila tetap ditransfusikan, atau kembalikan ke BDRS."
            ),
        ));
    }
    let petugas_2 = wajib_teks(&i.petugas_2, "petugas_2", "Nama pengguna petugas kedua")?;
    let isi = json!({ "no_rm": c.no_rm, "nomor_kantong": i.nomor_kantong, "tanggal_lahir": i.tanggal_lahir });
    let ttd = tanda_tangan::petugas_lain(
        &mut tx,
        &p,
        petugas_2,
        i.sandi_petugas_2.as_deref().unwrap_or(""),
        "transfusi.catat",
        "verifikasi samping tempat tidur",
        "kantong",
        c.kantong,
        &isi,
    )
    .await?;
    ubah_status(&mut tx, &p, c.kantong, &["dikeluarkan"], "ditransfusikan", None).await?;
    let v: Value = sqlx::query_scalar(
        "insert into transfusi (pengeluaran_id, kantong_id, no_rm, petugas_1, ttd_petugas_2, rawat_jalan, alasan_terlambat)
         values ($1, $2, $3, $4, $5, $6, $7) returning to_jsonb(transfusi)",
    )
    .bind(c.pengeluaran)
    .bind(c.kantong)
    .bind(&c.no_rm)
    .bind(p.akun_id)
    .bind(ttd)
    .bind(i.rawat_jalan.unwrap_or(false))
    .bind(if terlambat { teks(&i.alasan_terlambat) } else { None })
    .fetch_one(&mut *tx)
    .await?;
    audit::ubah(&mut tx, &p, "mulai", "transfusi", v["id"].as_str().unwrap_or_default(), v.clone()).await?;
    // Permintaan selesai otomatis bila semua kantong yang diminta sudah ditransfusikan.
    sqlx::query(
        "update permintaan r set status = 'selesai', alasan_status = 'Semua kantong ditransfusikan'
         where r.id = (select a.permintaan_id from pengeluaran g join alokasi a on a.id = g.alokasi_id where g.id = $1)
           and r.status in ('diproses', 'siap')
           and (select coalesce(sum((b->>'jumlah')::int), 0) from jsonb_array_elements(r.baris) b)
               <= (select count(*) from transfusi t join pengeluaran g on g.id = t.pengeluaran_id join alokasi a on a.id = g.alokasi_id
                   where a.permintaan_id = r.id)",
    )
    .bind(c.pengeluaran)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(v))
}

async fn transfusi_aktif(State(s): State<AppState>, p: Pengguna) -> Hasil<Json<Value>> {
    p.wajib("permintaan.lihat")?;
    let v: Value = sqlx::query_scalar(
        "select coalesce(jsonb_agg(to_jsonb(t) || jsonb_build_object('nomor_kantong', k.nomor, 'komponen', k.komponen,
                    'pasien', to_jsonb(ps), 'observasi_terakhir', (select max(o.waktu) from observasi o where o.transfusi_id = t.id),
                    'ada_15_menit', exists (select 1 from observasi o where o.transfusi_id = t.id and o.tahap = '15_menit'))
                order by t.mulai), '[]')
         from transfusi t join kantong k on k.id = t.kantong_id join pasien ps on ps.no_rm = t.no_rm
         where t.selesai is null or (not exists (select 1 from observasi o where o.transfusi_id = t.id and o.tahap = 'pasca')
                                     and t.selesai > now() - interval '1 day')",
    )
    .fetch_one(&s.db)
    .await?;
    Ok(Json(v))
}

async fn ambil(conn: &mut PgConnection, id: Uuid) -> Hasil<Value> {
    let v: Option<Value> = sqlx::query_scalar(
        "select to_jsonb(t) || jsonb_build_object(
            'kantong', to_jsonb(k), 'pasien', to_jsonb(ps), 'petugas_1', a1.nama,
            'petugas_2', (select a.nama from tanda_tangan x join akun a on a.id = x.akun_id where x.id = t.ttd_petugas_2),
            'permintaan', (select jsonb_build_object('id', r.id, 'nomor', r.nomor, 'dokter', r.dokter, 'indikasi', r.indikasi, 'ruang', r.ruang)
                           from pengeluaran g join alokasi a on a.id = g.alokasi_id join permintaan r on r.id = a.permintaan_id
                           where g.id = t.pengeluaran_id),
            'dikeluarkan_pada', (select g.dikeluarkan_pada from pengeluaran g where g.id = t.pengeluaran_id),
            'observasi', (select coalesce(jsonb_agg(to_jsonb(o) || jsonb_build_object('petugas', a.nama) order by o.waktu), '[]')
                          from observasi o join akun a on a.id = o.dicatat_oleh where o.transfusi_id = t.id),
            'reaksi', (select coalesce(jsonb_agg(to_jsonb(r)), '[]') from reaksi r where r.transfusi_id = t.id))
         from transfusi t join kantong k on k.id = t.kantong_id join pasien ps on ps.no_rm = t.no_rm
         join akun a1 on a1.id = t.petugas_1 where t.id = $1",
    )
    .bind(id)
    .fetch_optional(conn)
    .await?;
    v.ok_or_else(|| Galat::TidakDitemukan("Transfusi tidak ditemukan.".into()))
}

async fn detail(State(s): State<AppState>, p: Pengguna, Path(id): Path<Uuid>) -> Hasil<Json<Value>> {
    p.wajib("permintaan.lihat")?;
    let mut tx = s.db.begin().await?;
    let v = ambil(&mut tx, id).await?;
    audit::catat(
        &mut tx,
        Some(&p),
        Jejak { aksi: "lihat", objek: "transfusi", objek_id: Some(id.to_string()), ..Default::default() },
    )
    .await?;
    tx.commit().await?;
    Ok(Json(v))
}

#[derive(Deserialize)]
struct IsianObservasi {
    tahap: String,
    suhu: Option<f64>,
    nadi: Option<i32>,
    sistolik: Option<i32>,
    diastolik: Option<i32>,
    napas: Option<i32>,
    keluhan: Option<String>,
    alasan: Option<String>,
}

fn periksa_tanda_vital(i: &IsianObservasi) -> Hasil<()> {
    if !matches!(i.tahap.as_str(), "pra" | "15_menit" | "berkala" | "selesai" | "pasca") {
        return Err(tolak("tahap", "Pilih tahap observasi."));
    }
    if i.suhu.is_none() && i.nadi.is_none() && i.sistolik.is_none() {
        return Err(tolak("suhu", "Isi minimal suhu, nadi, atau tekanan darah."));
    }
    if i.suhu.is_some_and(|t| !(30.0..=45.0).contains(&t))
        || i.nadi.is_some_and(|n| !(20..=250).contains(&n))
        || i.sistolik.is_some_and(|n| !(40..=300).contains(&n))
        || i.diastolik.is_some_and(|n| !(20..=200).contains(&n))
        || i.napas.is_some_and(|n| !(4..=80).contains(&n))
    {
        return Err(tolak("suhu", "Nilai tanda vital di luar rentang yang mungkin; periksa kembali."));
    }
    Ok(())
}

/// Observasi saat mulai, 15 menit, berkala, selesai, dan 4 jam (rawat inap) / 1 jam (rawat jalan) sesudahnya (§6.1.5).
async fn observasi(
    State(s): State<AppState>,
    p: Pengguna,
    Path(id): Path<Uuid>,
    Json(i): Json<IsianObservasi>,
) -> Hasil<Json<Value>> {
    p.wajib("transfusi.catat")?;
    periksa_tanda_vital(&i)?;
    let mut tx = s.db.begin().await?;
    let selesai: Option<Option<OffsetDateTime>> =
        sqlx::query_scalar("select selesai from transfusi where id = $1").bind(id).fetch_optional(&mut *tx).await?;
    let selesai = selesai.ok_or_else(|| Galat::TidakDitemukan("Transfusi tidak ditemukan.".into()))?;
    if selesai.is_some() && i.tahap != "pasca" {
        return Err(Galat::Konflik(
            "Transfusi sudah selesai; hanya observasi pasca-transfusi yang bisa ditambahkan.".into(),
        ));
    }
    if selesai.is_none() && i.tahap == "pasca" {
        return Err(Galat::Konflik("Selesaikan transfusi dulu sebelum mencatat observasi pasca-transfusi.".into()));
    }
    let v: Value = sqlx::query_scalar(
        "insert into observasi (transfusi_id, tahap, suhu, nadi, sistolik, diastolik, napas, keluhan, dicatat_oleh)
         values ($1, $2, $3::float8::numeric, $4, $5, $6, $7, $8, $9) returning to_jsonb(observasi)",
    )
    .bind(id)
    .bind(&i.tahap)
    .bind(i.suhu)
    .bind(i.nadi)
    .bind(i.sistolik)
    .bind(i.diastolik)
    .bind(i.napas)
    .bind(teks(&i.keluhan))
    .bind(p.akun_id)
    .fetch_one(&mut *tx)
    .await?;
    audit::ubah(&mut tx, &p, "catat", "observasi", v["id"].to_string(), v.clone()).await?;
    tx.commit().await?;
    Ok(Json(v))
}

/// Koreksi observasi (FR-34): penginput dalam 2×24 jam; lewat itu hanya penanggung jawab. Nilai lama di audit.
async fn koreksi_observasi(
    State(s): State<AppState>,
    p: Pengguna,
    Path(id): Path<i64>,
    Json(i): Json<IsianObservasi>,
) -> Hasil<Json<Value>> {
    p.wajib("transfusi.catat")?;
    periksa_tanda_vital(&i)?;
    let alasan = wajib_teks(&i.alasan, "alasan", "Alasan koreksi")?;
    let mut tx = s.db.begin().await?;
    let r: Option<(Uuid, OffsetDateTime, Value)> =
        sqlx::query_as("select dicatat_oleh, waktu, to_jsonb(o) from observasi o where id = $1 for update")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
    let (oleh, waktu, sebelum) = r.ok_or_else(|| Galat::TidakDitemukan("Observasi tidak ditemukan.".into()))?;
    let dalam_jendela = OffsetDateTime::now_utc() - waktu <= Duration::hours(48);
    if !(oleh == p.akun_id && dalam_jendela) && !p.boleh("koreksi.lanjut") {
        return Err(Galat::Terlarang(
            "Koreksi oleh penginput hanya dalam 2×24 jam. Setelah itu koreksi dilakukan penanggung jawab BDRS (Permenkes 24/2022 Pasal 30).".into(),
        ));
    }
    let sesudah: Value = sqlx::query_scalar(
        "update observasi o set tahap = $2, suhu = $3::float8::numeric, nadi = $4, sistolik = $5, diastolik = $6, napas = $7, keluhan = $8,
                dikoreksi_pada = now() where id = $1 returning to_jsonb(o)",
    )
    .bind(id)
    .bind(&i.tahap)
    .bind(i.suhu)
    .bind(i.nadi)
    .bind(i.sistolik)
    .bind(i.diastolik)
    .bind(i.napas)
    .bind(teks(&i.keluhan))
    .fetch_one(&mut *tx)
    .await?;
    audit::catat(
        &mut tx,
        Some(&p),
        Jejak {
            aksi: "koreksi",
            objek: "observasi",
            objek_id: Some(id.to_string()),
            sebelum: Some(sebelum),
            sesudah: Some(sesudah.clone()),
            alasan: Some(alasan),
            ..Default::default()
        },
    )
    .await?;
    tx.commit().await?;
    Ok(Json(sesudah))
}

#[derive(Deserialize)]
struct IsianSelesai {
    volume_masuk_ml: Option<i32>,
    dihentikan: Option<bool>,
    catatan: Option<String>,
}

async fn selesai(
    State(s): State<AppState>,
    p: Pengguna,
    Path(id): Path<Uuid>,
    Json(i): Json<IsianSelesai>,
) -> Hasil<Json<Value>> {
    p.wajib("transfusi.catat")?;
    let dihentikan = i.dihentikan.unwrap_or(false);
    if dihentikan && teks(&i.catatan).is_none() {
        return Err(tolak("catatan", "Isi alasan transfusi dihentikan."));
    }
    if i.volume_masuk_ml.is_some_and(|v| !(0..=2000).contains(&v)) {
        return Err(tolak("volume_masuk_ml", "Volume masuk tidak wajar."));
    }
    let mut tx = s.db.begin().await?;
    let v: Option<Value> = sqlx::query_scalar(
        "update transfusi t set selesai = now(), dihentikan = $2, volume_masuk_ml = $3, catatan = $4 where id = $1 and selesai is null
         returning to_jsonb(t)",
    )
    .bind(id)
    .bind(dihentikan)
    .bind(i.volume_masuk_ml)
    .bind(teks(&i.catatan))
    .fetch_optional(&mut *tx)
    .await?;
    let v = v.ok_or_else(|| Galat::Konflik("Transfusi tidak ditemukan atau sudah selesai.".into()))?;
    audit::ubah(&mut tx, &p, if dihentikan { "hentikan" } else { "selesai" }, "transfusi", id, v.clone()).await?;
    tx.commit().await?;
    Ok(Json(v))
}

// ---------- Reaksi transfusi ----------

#[derive(Deserialize)]
struct IsianReaksi {
    #[serde(default, with = "time::serde::rfc3339::option")]
    waktu_reaksi: Option<OffsetDateTime>,
    gejala: Vec<String>,
    tindakan: Option<String>,
}

pub const GEJALA: [&str; 14] = [
    "demam",
    "menggigil",
    "urtikaria/gatal",
    "sesak napas",
    "hipotensi",
    "takikardia",
    "nyeri dada",
    "nyeri pinggang",
    "urin gelap/hemoglobinuria",
    "mual/muntah",
    "syok",
    "perdarahan tidak terkontrol",
    "ikterus",
    "lainnya",
];

/// POST /api/transfusi/{id}/reaksi — dilaporkan dari ruang (keluhan tertulis, §5.9). Akut bila ≤ 24 jam sejak mulai.
async fn lapor_reaksi(
    State(s): State<AppState>,
    p: Pengguna,
    Path(id): Path<Uuid>,
    Json(i): Json<IsianReaksi>,
) -> Hasil<Json<Value>> {
    p.wajib("reaksi.lapor")?;
    let tindakan =
        wajib_teks(&i.tindakan, "tindakan", "Tindakan yang sudah dilakukan (mis. transfusi dihentikan, infus NaCl)")?;
    if i.gejala.is_empty() || i.gejala.iter().any(|g| !GEJALA.contains(&g.as_str())) {
        return Err(tolak("gejala", "Pilih minimal satu gejala dari daftar."));
    }
    let sekarang = OffsetDateTime::now_utc();
    let waktu = i.waktu_reaksi.unwrap_or(sekarang);
    let mut tx = s.db.begin().await?;
    let mulai: Option<OffsetDateTime> =
        sqlx::query_scalar("select mulai from transfusi where id = $1").bind(id).fetch_optional(&mut *tx).await?;
    let mulai = mulai.ok_or_else(|| Galat::TidakDitemukan("Transfusi tidak ditemukan.".into()))?;
    if waktu < mulai || waktu > sekarang + Duration::minutes(5) {
        return Err(tolak("waktu_reaksi", "Waktu reaksi harus setelah transfusi dimulai dan tidak di masa depan."));
    }
    let kategori = if waktu - mulai <= Duration::hours(24) { "akut" } else { "lambat" };
    let v: Value = sqlx::query_scalar(
        "insert into reaksi (transfusi_id, waktu_reaksi, gejala, tindakan, kategori, dilaporkan_oleh) values ($1, $2, $3, $4, $5, $6)
         returning to_jsonb(reaksi)",
    )
    .bind(id)
    .bind(waktu)
    .bind(&i.gejala)
    .bind(tindakan)
    .bind(kategori)
    .bind(p.akun_id)
    .fetch_one(&mut *tx)
    .await?;
    audit::ubah(&mut tx, &p, "lapor", "reaksi", v["id"].as_str().unwrap_or_default(), v.clone()).await?;
    tx.commit().await?;
    Ok(Json(v))
}

async fn daftar_reaksi(State(s): State<AppState>, p: Pengguna) -> Hasil<Json<Value>> {
    if !p.boleh("reaksi.telusur") {
        p.wajib("laporan.lihat")?;
    }
    let v: Value = sqlx::query_scalar(
        "select coalesce(jsonb_agg(to_jsonb(r) || jsonb_build_object('nomor_kantong', k.nomor, 'komponen', k.komponen,
                    'pasien', to_jsonb(ps)) order by r.status = 'ditutup', r.dilaporkan_pada desc), '[]')
         from reaksi r join transfusi t on t.id = r.transfusi_id join kantong k on k.id = t.kantong_id
         join pasien ps on ps.no_rm = t.no_rm",
    )
    .fetch_one(&s.db)
    .await?;
    Ok(Json(v))
}

async fn detail_reaksi(State(s): State<AppState>, p: Pengguna, Path(id): Path<Uuid>) -> Hasil<Json<Value>> {
    if !p.boleh("reaksi.telusur") {
        p.wajib("laporan.lihat")?;
    }
    let mut tx = s.db.begin().await?;
    let r: Option<(Value, Uuid)> = sqlx::query_as(
        "select to_jsonb(r) || jsonb_build_object('pelapor', a.nama,
                'penutup', (select ak.nama from tanda_tangan x join akun ak on ak.id = x.akun_id where x.id = r.ttd_penutupan)),
                r.transfusi_id
         from reaksi r join akun a on a.id = r.dilaporkan_oleh where r.id = $1",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?;
    let (mut v, transfusi) = r.ok_or_else(|| Galat::TidakDitemukan("Laporan reaksi tidak ditemukan.".into()))?;
    let t = ambil(&mut tx, transfusi).await?;
    // Arsip uji kantong untuk penelusuran: uji silang beserta waktu, metode, dan petugasnya (§5.9).
    v["uji_silang"] = sqlx::query_scalar(
        "select coalesce(jsonb_agg(to_jsonb(u) || jsonb_build_object('petugas', a.nama)), '[]') from uji_silang u
         join akun a on a.id = u.dicatat_oleh where u.kantong_id = $1 and u.batal_alasan is null",
    )
    .bind(t["kantong_id"].as_str().and_then(|x| x.parse::<Uuid>().ok()))
    .fetch_one(&mut *tx)
    .await?;
    v["transfusi"] = t;
    audit::catat(
        &mut tx,
        Some(&p),
        Jejak { aksi: "lihat", objek: "reaksi", objek_id: Some(id.to_string()), ..Default::default() },
    )
    .await?;
    tx.commit().await?;
    Ok(Json(v))
}

#[derive(Deserialize)]
struct IsianTelusur {
    sisa_volume_ml: Option<i32>,
    abo_pasien_ulang: Option<String>,
    rhesus_pasien_ulang: Option<String>,
    abo_kantong_ulang: Option<String>,
    rhesus_kantong_ulang: Option<String>,
    uji_silang_ulang: String,
    kesimpulan: Option<String>,
}

/// Penelusuran reaksi oleh BDRS (§5.9): pemeriksaan ulang golongan pasien dan kantong, uji silang ulang, kesimpulan.
async fn telusur(
    State(s): State<AppState>,
    p: Pengguna,
    Path(id): Path<Uuid>,
    Json(i): Json<IsianTelusur>,
) -> Hasil<Json<Value>> {
    p.wajib("reaksi.telusur")?;
    let kesimpulan = wajib_teks(&i.kesimpulan, "kesimpulan", "Kesimpulan penelusuran")?;
    let abo_ok = |a: &Option<String>| a.as_deref().is_none_or(|a| crate::stok::ABO.contains(&a));
    let rh_ok = |r: &Option<String>| r.as_deref().is_none_or(|r| matches!(r, "pos" | "neg"));
    if !abo_ok(&i.abo_pasien_ulang)
        || !abo_ok(&i.abo_kantong_ulang)
        || !rh_ok(&i.rhesus_pasien_ulang)
        || !rh_ok(&i.rhesus_kantong_ulang)
    {
        return Err(tolak("abo_pasien_ulang", "Golongan darah tidak dikenal."));
    }
    if !matches!(i.uji_silang_ulang.as_str(), "serasi" | "tidak_serasi" | "tidak_dilakukan") {
        return Err(tolak("uji_silang_ulang", "Pilih hasil uji silang ulang."));
    }
    let mut tx = s.db.begin().await?;
    let v: Option<Value> = sqlx::query_scalar(
        "update reaksi r set sisa_volume_ml = $2, abo_pasien_ulang = $3, rhesus_pasien_ulang = $4, abo_kantong_ulang = $5,
                rhesus_kantong_ulang = $6, uji_silang_ulang = $7, kesimpulan = $8, ditelusuri_oleh = $9, ditelusuri_pada = now(),
                status = 'ditelusuri'
         where id = $1 and status in ('dilaporkan', 'ditelusuri') returning to_jsonb(r)",
    )
    .bind(id)
    .bind(i.sisa_volume_ml)
    .bind(teks(&i.abo_pasien_ulang))
    .bind(teks(&i.rhesus_pasien_ulang))
    .bind(teks(&i.abo_kantong_ulang))
    .bind(teks(&i.rhesus_kantong_ulang))
    .bind(&i.uji_silang_ulang)
    .bind(kesimpulan)
    .bind(p.akun_id)
    .fetch_optional(&mut *tx)
    .await?;
    let v = v.ok_or_else(|| Galat::Konflik("Laporan reaksi tidak ditemukan atau sudah ditutup.".into()))?;
    audit::ubah(&mut tx, &p, "telusur", "reaksi", id, v.clone()).await?;
    tx.commit().await?;
    Ok(Json(v))
}

#[derive(Deserialize)]
struct Penutupan {
    sandi: Option<String>,
}

/// Penutupan penelusuran oleh penanggung jawab BDRS dengan tanda tangan (§5.9: divalidasi pemeriksa dan penanggung jawab).
async fn tutup_reaksi(
    State(s): State<AppState>,
    p: Pengguna,
    Path(id): Path<Uuid>,
    Json(t): Json<Penutupan>,
) -> Hasil<Json<Value>> {
    p.wajib("reaksi.tutup")?;
    let mut tx = s.db.begin().await?;
    let isi: Option<Value> =
        sqlx::query_scalar("select to_jsonb(r) - 'ttd_penutupan' - 'status' from reaksi r where id = $1 and status = 'ditelusuri' for update")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
    let isi = isi.ok_or_else(|| Galat::Konflik("Hanya laporan yang sudah ditelusuri yang bisa ditutup.".into()))?;
    let ttd = tanda_tangan::sendiri(
        &mut tx,
        &p,
        t.sandi.as_deref().unwrap_or(""),
        "penutupan penelusuran reaksi transfusi",
        "reaksi",
        id,
        &isi,
    )
    .await?;
    sqlx::query("update reaksi set status = 'ditutup', ttd_penutupan = $2 where id = $1")
        .bind(id)
        .bind(ttd)
        .execute(&mut *tx)
        .await?;
    audit::ubah(&mut tx, &p, "tutup", "reaksi", id, json!({ "tanda_tangan_id": ttd })).await?;
    tx.commit().await?;
    Ok(Json(json!({})))
}

async fn daftar_insiden(State(s): State<AppState>, p: Pengguna) -> Hasil<Json<Value>> {
    if !p.boleh("reaksi.telusur") {
        p.wajib("laporan.lihat")?;
    }
    let v: Value = sqlx::query_scalar(
        "select coalesce(jsonb_agg(to_jsonb(i) || jsonb_build_object('pelapor', a.nama,
                    'nomor_kantong', (select k.nomor from kantong k where k.id = i.kantong_id)) order by i.waktu desc), '[]')
         from (select * from insiden order by waktu desc limit 500) i join akun a on a.id = i.dilaporkan_oleh",
    )
    .fetch_one(&s.db)
    .await?;
    Ok(Json(v))
}
