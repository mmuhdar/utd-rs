//! Mode UTD-RS (ADR 0001; SPEK M5; FR-18, FR-29..FR-31): donor, cekal, seleksi, donasi, golongan donasi,
//! uji saring IMLTD dengan verifikasi kedua, pengolahan menjadi kantong, pelulusan, dan look-back.
//! Look-back atas nomor donasi dari UTD mitra juga tersedia di mode BDRS.

use crate::AppState;
use crate::audit::{self, Jejak};
use crate::auth::Pengguna;
use crate::error::{Galat, Hasil, tolak};
use crate::master::{teks, wajib_teks};
use crate::pratransfusi::{Verifikasi, tafsir_abo, verifikasi};
use crate::stok::{KOMPONEN, ubah_status};
use crate::tanda_tangan;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::PgConnection;
use time::{Date, OffsetDateTime};
use uuid::Uuid;

pub fn rute() -> Router<AppState> {
    Router::new()
        .route("/donor", get(cari).post(simpan))
        .route("/donor/{id}", get(detail))
        .route("/donor/{id}/cekal", post(cekal))
        .route("/donor/{id}/seleksi", post(seleksi))
        .route("/cekal/{id}/cabut", post(cabut_cekal))
        .route("/seleksi/{id}/donasi", post(ambil_darah))
        .route("/donasi", get(daftar_donasi))
        .route("/donasi/{id}", get(detail_donasi))
        .route("/donasi/{id}/golongan", post(golongan))
        .route("/golongan-donasi/{id}/verifikasi", post(verifikasi_golongan))
        .route("/donasi/{id}/imltd", post(imltd))
        .route("/imltd/{id}/verifikasi", post(verifikasi_imltd))
        .route("/donasi/{id}/olah", post(olah))
        .route("/kantong/{id}/luluskan", post(luluskan))
        .route("/aturan-donor", get(aturan).put(ubah_aturan))
        .route("/lookback", get(daftar_lookback).post(lookback_utd))
        .route("/lookback/{id}", get(detail_lookback))
        .route("/lookback/{id}/selesai", post(selesai_lookback))
}

/// Uji saring IMLTD minimal per donasi (Permenkes 91/2015 §3.8.1 A; BPOM 11/2025 butir 6.128).
pub const IMLTD_WAJIB: [&str; 4] = ["hiv", "hbsag", "hcv", "sifilis"];

async fn wajib_mode_utd(conn: &mut PgConnection) -> Hasil<()> {
    let jenis: String = sqlx::query_scalar("select jenis from profil_instalasi").fetch_one(conn).await?;
    if jenis != "utd_rs" {
        return Err(Galat::Konflik(
            "Fitur donor hanya aktif pada mode UTD-RS. Ubah profil instalasi bila RS menjalankan UTD.".into(),
        ));
    }
    Ok(())
}

/// Cekal yang berlaku hari ini, bila ada.
async fn cekal_aktif(conn: &mut PgConnection, donor: Uuid) -> Hasil<Option<String>> {
    Ok(sqlx::query_scalar(
        "select case when jenis = 'permanen' then 'permanen: ' || alasan else 'sampai ' || to_char(sampai, 'DD-MM-YYYY') || ': ' || alasan end
         from cekal where donor_id = $1 and dicabut_pada is null and mulai <= current_date and (sampai is null or sampai >= current_date)
         order by jenis = 'permanen' desc limit 1",
    )
    .bind(donor)
    .fetch_optional(conn)
    .await?)
}

// ---------- Donor ----------

#[derive(Deserialize)]
struct IsianDonor {
    nik: Option<String>,
    nama: Option<String>,
    tanggal_lahir: Option<Date>,
    jenis_kelamin: Option<String>,
    alamat: Option<String>,
    telepon: Option<String>,
}

async fn simpan(State(s): State<AppState>, p: Pengguna, Json(i): Json<IsianDonor>) -> Hasil<Json<Value>> {
    p.wajib("donor.kelola")?;
    let nik = wajib_teks(&i.nik, "nik", "NIK")?;
    if nik.len() != 16 || !nik.bytes().all(|b| b.is_ascii_digit()) {
        return Err(tolak("nik", "NIK terdiri dari 16 angka."));
    }
    let nama = wajib_teks(&i.nama, "nama", "Nama donor")?;
    let lahir = i.tanggal_lahir.ok_or_else(|| tolak("tanggal_lahir", "Tanggal lahir wajib diisi."))?;
    let jk = i
        .jenis_kelamin
        .as_deref()
        .filter(|j| matches!(*j, "L" | "P"))
        .ok_or_else(|| tolak("jenis_kelamin", "Pilih L atau P."))?;
    let mut tx = s.db.begin().await?;
    wajib_mode_utd(&mut tx).await?;
    let sebelum: Option<Value> = sqlx::query_scalar("select to_jsonb(d) from donor d where nik = $1 for update")
        .bind(nik)
        .fetch_optional(&mut *tx)
        .await?;
    let v: Value = sqlx::query_scalar(
        "insert into donor (nik, nama, tanggal_lahir, jenis_kelamin, alamat, telepon) values ($1, $2, $3, $4, $5, $6)
         on conflict (nik) do update set nama = $2, tanggal_lahir = $3, jenis_kelamin = $4, alamat = $5, telepon = $6, diperbarui_pada = now()
         returning to_jsonb(donor)",
    )
    .bind(nik)
    .bind(nama)
    .bind(lahir)
    .bind(jk)
    .bind(teks(&i.alamat))
    .bind(teks(&i.telepon))
    .fetch_one(&mut *tx)
    .await?;
    audit::catat(
        &mut tx,
        Some(&p),
        Jejak {
            aksi: "simpan",
            objek: "donor",
            objek_id: v["id"].as_str().map(String::from),
            sebelum,
            sesudah: Some(v.clone()),
            ..Default::default()
        },
    )
    .await?;
    tx.commit().await?;
    Ok(Json(v))
}

#[derive(Deserialize)]
struct Cari {
    q: Option<String>,
}

async fn cari(State(s): State<AppState>, p: Pengguna, Query(c): Query<Cari>) -> Hasil<Json<Value>> {
    p.wajib("donor.kelola")?;
    let q = teks(&c.q).unwrap_or("");
    if q.chars().count() < 3 {
        return Ok(Json(json!([])));
    }
    let v: Value = sqlx::query_scalar(
        "select coalesce(jsonb_agg(to_jsonb(d) order by d.nama), '[]') from
           (select * from donor where nik = $1 or nama ilike '%' || $1 || '%' order by nama limit 20) d",
    )
    .bind(q)
    .fetch_one(&s.db)
    .await?;
    Ok(Json(v))
}

async fn detail(State(s): State<AppState>, p: Pengguna, Path(id): Path<Uuid>) -> Hasil<Json<Value>> {
    p.wajib("donor.kelola")?;
    let mut tx = s.db.begin().await?;
    let v: Option<Value> = sqlx::query_scalar(
        "select to_jsonb(d) || jsonb_build_object(
            'cekal', (select coalesce(jsonb_agg(to_jsonb(c) order by c.dibuat_pada desc), '[]') from cekal c where c.donor_id = d.id),
            'seleksi', (select coalesce(jsonb_agg(to_jsonb(x) || jsonb_build_object('donasi_id', (select n.id from donasi n where n.seleksi_id = x.id))
                           order by x.waktu desc), '[]') from seleksi x where x.donor_id = d.id),
            'donasi', (select coalesce(jsonb_agg(to_jsonb(n) order by n.waktu desc), '[]') from donasi n where n.donor_id = d.id))
         from donor d where d.id = $1",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?;
    let mut v = v.ok_or_else(|| Galat::TidakDitemukan("Donor tidak ditemukan.".into()))?;
    v["cekal_aktif"] = json!(cekal_aktif(&mut tx, id).await?);
    audit::catat(
        &mut tx,
        Some(&p),
        Jejak { aksi: "lihat", objek: "donor", objek_id: Some(id.to_string()), ..Default::default() },
    )
    .await?;
    tx.commit().await?;
    Ok(Json(v))
}

#[derive(Deserialize)]
struct IsianCekal {
    jenis: String,
    sampai: Option<Date>,
    alasan: Option<String>,
}

async fn buat_cekal(
    conn: &mut PgConnection,
    p: &Pengguna,
    donor: Uuid,
    jenis: &str,
    sampai: Option<Date>,
    alasan: &str,
    sumber: &str,
) -> Hasil<Uuid> {
    let id: Uuid = sqlx::query_scalar(
        "insert into cekal (donor_id, jenis, sampai, alasan, sumber, dibuat_oleh) values ($1, $2, $3, $4, $5, $6) returning id",
    )
    .bind(donor)
    .bind(jenis)
    .bind(if jenis == "sementara" { sampai } else { None })
    .bind(alasan)
    .bind(sumber)
    .bind(p.akun_id)
    .fetch_one(&mut *conn)
    .await?;
    audit::ubah(
        conn,
        p,
        "cekal",
        "donor",
        donor,
        json!({ "jenis": jenis, "sampai": sampai, "alasan": alasan, "sumber": sumber }),
    )
    .await?;
    Ok(id)
}

async fn cekal(
    State(s): State<AppState>,
    p: Pengguna,
    Path(donor): Path<Uuid>,
    Json(i): Json<IsianCekal>,
) -> Hasil<Json<Value>> {
    p.wajib("donor.kelola")?;
    let alasan = wajib_teks(&i.alasan, "alasan", "Alasan cekal")?;
    match (i.jenis.as_str(), i.sampai) {
        ("sementara", None) => return Err(tolak("sampai", "Isi tanggal berakhirnya cekal sementara.")),
        ("sementara", Some(d)) if d <= OffsetDateTime::now_utc().date() => {
            return Err(tolak("sampai", "Tanggal berakhir harus setelah hari ini."));
        }
        ("sementara" | "permanen", _) => {}
        _ => return Err(tolak("jenis", "Pilih cekal sementara atau permanen.")),
    }
    let mut tx = s.db.begin().await?;
    wajib_mode_utd(&mut tx).await?;
    let id = buat_cekal(&mut tx, &p, donor, &i.jenis, i.sampai, alasan, "manual").await?;
    tx.commit().await?;
    Ok(Json(json!({ "id": id })))
}

#[derive(Deserialize)]
struct Cabut {
    alasan: Option<String>,
    sandi: Option<String>,
}

/// Mencabut cekal hanya oleh penanggung jawab, dengan tanda tangan.
async fn cabut_cekal(
    State(s): State<AppState>,
    p: Pengguna,
    Path(id): Path<Uuid>,
    Json(c): Json<Cabut>,
) -> Hasil<Json<Value>> {
    p.wajib("donor.lulus")?;
    let alasan = wajib_teks(&c.alasan, "alasan", "Alasan pencabutan cekal (mis. hasil uji konfirmasi negatif)")?;
    let mut tx = s.db.begin().await?;
    let donor: Option<Uuid> =
        sqlx::query_scalar("select donor_id from cekal where id = $1 and dicabut_pada is null for update")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
    let donor = donor.ok_or_else(|| Galat::Konflik("Cekal tidak ditemukan atau sudah dicabut.".into()))?;
    let ttd = tanda_tangan::sendiri(
        &mut tx,
        &p,
        c.sandi.as_deref().unwrap_or(""),
        "pencabutan cekal donor",
        "cekal",
        id,
        &json!({ "alasan": alasan }),
    )
    .await?;
    sqlx::query("update cekal set dicabut_pada = now(), dicabut_ttd = $2, alasan_cabut = $3 where id = $1")
        .bind(id)
        .bind(ttd)
        .bind(alasan)
        .execute(&mut *tx)
        .await?;
    audit::ubah(&mut tx, &p, "cabut_cekal", "donor", donor, json!({ "cekal_id": id, "alasan": alasan })).await?;
    tx.commit().await?;
    Ok(Json(json!({})))
}

// ---------- Seleksi dan donasi ----------

#[derive(Deserialize)]
struct IsianSeleksi {
    berat_kg: f64,
    hb: f64,
    sistolik: i32,
    diastolik: i32,
    nadi: i32,
    suhu: f64,
    kuesioner_lolos: bool,
    keputusan: String,
    alasan: Option<String>,
    /// Untuk ditolak sementara: sampai kapan donor dicekal.
    sampai: Option<Date>,
}

#[derive(sqlx::FromRow)]
struct AturanDonor {
    umur_min: i32,
    umur_maks: i32,
    berat_min: f64,
    hb_min: f64,
    hb_maks: f64,
    sistolik_min: i32,
    sistolik_maks: i32,
    diastolik_min: i32,
    diastolik_maks: i32,
    nadi_min: i32,
    nadi_maks: i32,
    suhu_maks: f64,
    hari_antar_donasi: i32,
}

/// Alasan donor tidak memenuhi syarat menurut ambang `aturan_donor`; kosong bila memenuhi.
fn tidak_memenuhi(a: &AturanDonor, umur: i32, i: &IsianSeleksi) -> Vec<String> {
    let mut v = Vec::new();
    if umur < a.umur_min || umur > a.umur_maks {
        v.push(format!("umur {umur} tahun di luar {}–{}", a.umur_min, a.umur_maks));
    }
    if i.berat_kg < a.berat_min {
        v.push(format!("berat {} kg < {}", i.berat_kg, a.berat_min));
    }
    if i.hb < a.hb_min || i.hb > a.hb_maks {
        v.push(format!("Hb {} g/dL di luar {}–{}", i.hb, a.hb_min, a.hb_maks));
    }
    if i.sistolik < a.sistolik_min
        || i.sistolik > a.sistolik_maks
        || i.diastolik < a.diastolik_min
        || i.diastolik > a.diastolik_maks
    {
        v.push(format!("tekanan darah {}/{} di luar batas", i.sistolik, i.diastolik));
    }
    if i.nadi < a.nadi_min || i.nadi > a.nadi_maks {
        v.push(format!("nadi {} di luar {}–{}", i.nadi, a.nadi_min, a.nadi_maks));
    }
    if i.suhu > a.suhu_maks {
        v.push(format!("suhu {} °C > {}", i.suhu, a.suhu_maks));
    }
    if !i.kuesioner_lolos {
        v.push("kuesioner tidak lolos".into());
    }
    v
}

/// POST /api/donor/{id}/seleksi — donor dicekal tidak bisa diseleksi; hasil ditolak otomatis membuat cekal.
async fn seleksi(
    State(s): State<AppState>,
    p: Pengguna,
    Path(donor): Path<Uuid>,
    Json(i): Json<IsianSeleksi>,
) -> Hasil<Json<Value>> {
    p.wajib("donor.kelola")?;
    let mut tx = s.db.begin().await?;
    wajib_mode_utd(&mut tx).await?;
    if let Some(c) = cekal_aktif(&mut tx, donor).await? {
        return Err(Galat::Konflik(format!("Donor sedang dicekal ({c}).")));
    }
    let a: AturanDonor = sqlx::query_as(
        "select umur_min, umur_maks, berat_min::float8, hb_min::float8, hb_maks::float8, sistolik_min, sistolik_maks, diastolik_min,
                diastolik_maks, nadi_min, nadi_maks, suhu_maks::float8, hari_antar_donasi from aturan_donor",
    )
    .fetch_one(&mut *tx)
    .await?;
    let r: Option<(i32, Option<i32>)> = sqlx::query_as(
        "select extract(year from age(current_date, d.tanggal_lahir))::int,
                (select (current_date - max(n.waktu)::date) from donasi n where n.donor_id = d.id and n.status <> 'gagal')
         from donor d where d.id = $1",
    )
    .bind(donor)
    .fetch_optional(&mut *tx)
    .await?;
    let (umur, hari_sejak) = r.ok_or_else(|| Galat::TidakDitemukan("Donor tidak ditemukan.".into()))?;
    let mut kurang = tidak_memenuhi(&a, umur, &i);
    if let Some(h) = hari_sejak.filter(|h| *h < a.hari_antar_donasi) {
        kurang.push(format!("donasi terakhir {h} hari lalu (minimal {} hari)", a.hari_antar_donasi));
    }
    match i.keputusan.as_str() {
        "diterima" if !kurang.is_empty() => {
            return Err(tolak(
                "keputusan",
                format!("Donor tidak memenuhi syarat: {}. Tolak sementara atau permanen.", kurang.join("; ")),
            ));
        }
        "ditolak_sementara" if i.sampai.is_none() => {
            return Err(tolak("sampai", "Isi sampai kapan donor ditolak sementara."));
        }
        "diterima" | "ditolak_sementara" | "ditolak_permanen" => {}
        _ => return Err(tolak("keputusan", "Pilih keputusan seleksi.")),
    }
    let alasan = if i.keputusan == "diterima" {
        None
    } else {
        Some(teks(&i.alasan).map(String::from).unwrap_or_else(|| kurang.join("; "))).filter(|a| !a.is_empty())
    };
    if i.keputusan != "diterima" && alasan.is_none() {
        return Err(tolak("alasan", "Isi alasan penolakan."));
    }
    let v: Value = sqlx::query_scalar(
        "insert into seleksi (donor_id, berat_kg, hb, sistolik, diastolik, nadi, suhu, kuesioner_lolos, keputusan, alasan, dicatat_oleh)
         values ($1, $2::float8::numeric, $3::float8::numeric, $4, $5, $6, $7::float8::numeric, $8, $9, $10, $11) returning to_jsonb(seleksi)",
    )
    .bind(donor)
    .bind(i.berat_kg)
    .bind(i.hb)
    .bind(i.sistolik)
    .bind(i.diastolik)
    .bind(i.nadi)
    .bind(i.suhu)
    .bind(i.kuesioner_lolos)
    .bind(&i.keputusan)
    .bind(&alasan)
    .bind(p.akun_id)
    .fetch_one(&mut *tx)
    .await?;
    audit::ubah(&mut tx, &p, "seleksi", "donor", donor, v.clone()).await?;
    if i.keputusan != "diterima" {
        let jenis = if i.keputusan == "ditolak_permanen" { "permanen" } else { "sementara" };
        buat_cekal(&mut tx, &p, donor, jenis, i.sampai, alasan.as_deref().unwrap_or_default(), "seleksi").await?;
    }
    tx.commit().await?;
    Ok(Json(v))
}

#[derive(Deserialize)]
struct IsianDonasi {
    nomor: Option<String>,
    jenis: String,
    untuk_no_rm: Option<String>,
    volume_ml: i32,
    jenis_kantong: String,
    catatan: Option<String>,
}

/// POST /api/seleksi/{id}/donasi — pengambilan darah setelah seleksi diterima pada hari yang sama.
async fn ambil_darah(
    State(s): State<AppState>,
    p: Pengguna,
    Path(seleksi): Path<Uuid>,
    Json(i): Json<IsianDonasi>,
) -> Hasil<Json<Value>> {
    p.wajib("donor.kelola")?;
    let nomor = wajib_teks(&i.nomor, "nomor", "Nomor donasi (dari label barcode kantong)")?;
    let mut tx = s.db.begin().await?;
    wajib_mode_utd(&mut tx).await?;
    let r: Option<(Uuid, String, bool)> =
        sqlx::query_as("select donor_id, keputusan, waktu > now() - interval '12 hours' from seleksi where id = $1")
            .bind(seleksi)
            .fetch_optional(&mut *tx)
            .await?;
    let (donor, keputusan, baru) = r.ok_or_else(|| Galat::TidakDitemukan("Seleksi tidak ditemukan.".into()))?;
    if keputusan != "diterima" {
        return Err(Galat::Konflik("Donor tidak lolos seleksi; darah tidak boleh diambil.".into()));
    }
    if !baru {
        return Err(Galat::Konflik("Seleksi sudah lebih dari 12 jam; lakukan seleksi ulang.".into()));
    }
    if let Some(c) = cekal_aktif(&mut tx, donor).await? {
        return Err(Galat::Konflik(format!("Donor sedang dicekal ({c}).")));
    }
    let v: Value = sqlx::query_scalar(
        "insert into donasi (nomor, donor_id, seleksi_id, jenis, untuk_no_rm, volume_ml, jenis_kantong, catatan, dicatat_oleh)
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9) returning to_jsonb(donasi)",
    )
    .bind(nomor)
    .bind(donor)
    .bind(seleksi)
    .bind(&i.jenis)
    .bind(teks(&i.untuk_no_rm))
    .bind(i.volume_ml)
    .bind(&i.jenis_kantong)
    .bind(teks(&i.catatan))
    .bind(p.akun_id)
    .fetch_one(&mut *tx)
    .await?;
    audit::ubah(&mut tx, &p, "ambil", "donasi", v["id"].as_str().unwrap_or_default(), v.clone()).await?;
    tx.commit().await?;
    Ok(Json(v))
}

#[derive(Deserialize)]
struct FilterDonasi {
    status: Option<String>,
}

async fn daftar_donasi(State(s): State<AppState>, p: Pengguna, Query(f): Query<FilterDonasi>) -> Hasil<Json<Value>> {
    p.wajib("donor.kelola")?;
    let v: Value = sqlx::query_scalar(
        "select coalesce(jsonb_agg(to_jsonb(n) || jsonb_build_object('donor', d.nama,
                    'golongan', (select g.abo || g.rhesus from golongan_donasi g where g.donasi_id = n.id and g.tanda_tangan_id is not null and g.batal_alasan is null),
                    'imltd_lengkap', (select count(distinct u.parameter) from uji_imltd u where u.donasi_id = n.id and u.tanda_tangan_id is not null
                                      and u.parameter = any($2)) = 4,
                    'reaktif', exists (select 1 from uji_imltd u where u.donasi_id = n.id and u.hasil = 'reaktif' and u.batal_alasan is null),
                    'karantina', (select count(*) from kantong k where k.donasi_id = n.id and k.status = 'karantina'))
                order by n.waktu desc), '[]')
         from (select * from donasi where $1::text is null or status = $1 order by waktu desc limit 300) n join donor d on d.id = n.donor_id",
    )
    .bind(teks(&f.status))
    .bind(IMLTD_WAJIB.map(String::from).to_vec())
    .fetch_one(&s.db)
    .await?;
    Ok(Json(v))
}

async fn detail_donasi(State(s): State<AppState>, p: Pengguna, Path(id): Path<Uuid>) -> Hasil<Json<Value>> {
    p.wajib("donor.kelola")?;
    let v: Option<Value> = sqlx::query_scalar(
        "select to_jsonb(n) || jsonb_build_object('donor', to_jsonb(d),
            'golongan', (select to_jsonb(g) || jsonb_build_object('petugas', a.nama, 'terverifikasi', g.tanda_tangan_id is not null)
                         from golongan_donasi g join akun a on a.id = g.dicatat_oleh where g.donasi_id = n.id and g.batal_alasan is null),
            'imltd', (select coalesce(jsonb_agg(to_jsonb(u) || jsonb_build_object('petugas', a.nama, 'terverifikasi', u.tanda_tangan_id is not null)
                         order by u.parameter), '[]') from uji_imltd u join akun a on a.id = u.dicatat_oleh
                      where u.donasi_id = n.id and u.batal_alasan is null),
            'kantong', (select coalesce(jsonb_agg(to_jsonb(k) || jsonb_build_object('lulus',
                           exists (select 1 from tanda_tangan t where t.objek = 'kantong' and t.objek_id = k.id::text and t.makna = 'pelulusan'))
                           order by k.komponen), '[]') from kantong k where k.donasi_id = n.id))
         from donasi n join donor d on d.id = n.donor_id where n.id = $1",
    )
    .bind(id)
    .fetch_optional(&s.db)
    .await?;
    Ok(Json(v.ok_or_else(|| Galat::TidakDitemukan("Donasi tidak ditemukan.".into()))?))
}

// ---------- Golongan dan IMLTD donasi ----------

#[derive(Deserialize)]
struct IsianGolongan {
    anti_a: String,
    anti_b: String,
    sel_a: String,
    sel_b: String,
    anti_d: String,
    metode: String,
}

async fn golongan(
    State(s): State<AppState>,
    p: Pengguna,
    Path(donasi): Path<Uuid>,
    Json(i): Json<IsianGolongan>,
) -> Hasil<Json<Value>> {
    p.wajib("uji.input")?;
    if ![&i.anti_a, &i.anti_b, &i.sel_a, &i.sel_b, &i.anti_d].iter().all(|h| matches!(h.as_str(), "pos" | "neg")) {
        return Err(tolak("anti_a", "Isi semua hasil reaksi dengan positif atau negatif."));
    }
    let (abo, cocok) = tafsir_abo(&i.anti_a, &i.anti_b, &i.sel_a, &i.sel_b);
    if !cocok {
        return Err(tolak(
            "sel_a",
            format!("Cell grouping menunjukkan {abo} tetapi serum grouping tidak sesuai. Ulangi pemeriksaan."),
        ));
    }
    let rh = if i.anti_d == "pos" { "pos" } else { "neg" };
    let mut tx = s.db.begin().await?;
    wajib_mode_utd(&mut tx).await?;
    let r: Option<(Option<String>, Option<String>)> = sqlx::query_as(
        "select d.abo::text, d.rhesus::text from donasi n join donor d on d.id = n.donor_id where n.id = $1",
    )
    .bind(donasi)
    .fetch_optional(&mut *tx)
    .await?;
    let (abo_donor, rh_donor) = r.ok_or_else(|| Galat::TidakDitemukan("Donasi tidak ditemukan.".into()))?;
    if let (Some(a), Some(r)) = (&abo_donor, &rh_donor)
        && (a != abo || r != rh)
    {
        return Err(Galat::Konflik(format!(
            "Hasil {abo}{rh} berbeda dari golongan donor sebelumnya ({a}{r}). Periksa identitas sampel dan ulangi; jangan olah donasi ini dulu."
        )));
    }
    let v: Value = sqlx::query_scalar(
        "insert into golongan_donasi (donasi_id, anti_a, anti_b, sel_a, sel_b, anti_d, abo, rhesus, metode, dicatat_oleh)
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) returning to_jsonb(golongan_donasi)",
    )
    .bind(donasi)
    .bind(&i.anti_a)
    .bind(&i.anti_b)
    .bind(&i.sel_a)
    .bind(&i.sel_b)
    .bind(&i.anti_d)
    .bind(abo)
    .bind(rh)
    .bind(&i.metode)
    .bind(p.akun_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| match Galat::from(e) {
        Galat::Konflik(_) => Galat::Konflik("Donasi ini sudah punya hasil golongan. Bila salah, minta verifikator menolaknya dulu.".into()),
        g => g,
    })?;
    audit::ubah(&mut tx, &p, "input", "golongan_donasi", v["id"].as_str().unwrap_or_default(), v.clone()).await?;
    tx.commit().await?;
    Ok(Json(v))
}

async fn verifikasi_golongan(
    State(s): State<AppState>,
    p: Pengguna,
    Path(id): Path<Uuid>,
    Json(v): Json<Verifikasi>,
) -> Hasil<Json<Value>> {
    let mut tx = s.db.begin().await?;
    let (setuju, isi) =
        verifikasi(&mut tx, &p, "golongan_donasi", "verifikasi kedua golongan darah donasi", id, &v).await?;
    if setuju {
        sqlx::query(
            "update donor d set abo = $2, rhesus = $3, diperbarui_pada = now() from donasi n where n.id = $1::text::uuid and d.id = n.donor_id",
        )
        .bind(isi["donasi_id"].as_str().unwrap_or_default())
        .bind(isi["abo"].as_str())
        .bind(isi["rhesus"].as_str())
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(Json(json!({ "setuju": setuju })))
}

#[derive(Deserialize)]
struct IsianImltd {
    parameter: String,
    metode: String,
    hasil: String,
}

async fn imltd(
    State(s): State<AppState>,
    p: Pengguna,
    Path(donasi): Path<Uuid>,
    Json(i): Json<IsianImltd>,
) -> Hasil<Json<Value>> {
    p.wajib("uji.input")?;
    let mut tx = s.db.begin().await?;
    wajib_mode_utd(&mut tx).await?;
    let v: Value = sqlx::query_scalar(
        "insert into uji_imltd (donasi_id, parameter, metode, hasil, dicatat_oleh) values ($1, $2, $3, $4, $5) returning to_jsonb(uji_imltd)",
    )
    .bind(donasi)
    .bind(&i.parameter)
    .bind(&i.metode)
    .bind(&i.hasil)
    .bind(p.akun_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| match Galat::from(e) {
        Galat::Konflik(_) => Galat::Konflik(format!("Hasil {} untuk donasi ini sudah ada.", i.parameter.to_uppercase())),
        g => g,
    })?;
    audit::ubah(&mut tx, &p, "input", "uji_imltd", v["id"].as_str().unwrap_or_default(), v.clone()).await?;
    tx.commit().await?;
    Ok(Json(v))
}

/// Hasil reaktif yang terverifikasi: donor dicekal (permanen sampai uji konfirmasi), kantong donasi ini tetap
/// karantina dan ditandai reaktif, dan look-back dibuka untuk donasi-donasi sebelumnya.
async fn verifikasi_imltd(
    State(s): State<AppState>,
    p: Pengguna,
    Path(id): Path<Uuid>,
    Json(v): Json<Verifikasi>,
) -> Hasil<Json<Value>> {
    let mut tx = s.db.begin().await?;
    let (setuju, isi) = verifikasi(&mut tx, &p, "uji_imltd", "verifikasi kedua uji saring IMLTD", id, &v).await?;
    let mut lookback = None;
    if setuju && isi["hasil"] == "reaktif" {
        let donasi: Uuid = serde_json::from_value(isi["donasi_id"].clone()).map_err(anyhow::Error::from)?;
        let donor: Uuid =
            sqlx::query_scalar("select donor_id from donasi where id = $1").bind(donasi).fetch_one(&mut *tx).await?;
        let param = isi["parameter"].as_str().unwrap_or_default().to_uppercase();
        let alasan = format!("Reaktif {param} pada donasi; menunggu uji konfirmasi");
        buat_cekal(&mut tx, &p, donor, "permanen", None, &alasan, "imltd").await?;
        let kantong: Vec<Uuid> =
            sqlx::query_scalar("select id from kantong where donasi_id = $1").bind(donasi).fetch_all(&mut *tx).await?;
        for k in kantong {
            sqlx::query("update kantong set imltd_nonreaktif = false, alasan_status = $2 where id = $1 and status = 'karantina'")
                .bind(k)
                .bind(format!("Reaktif {param}"))
                .execute(&mut *tx)
                .await?;
        }
        lookback = Some(buka_lookback(&mut tx, &p, Some(donor), None, Some(donasi), &alasan).await?);
    }
    tx.commit().await?;
    Ok(Json(json!({ "setuju": setuju, "lookback_id": lookback })))
}

// ---------- Pengolahan dan pelulusan ----------

#[derive(Deserialize)]
struct IsianOlah {
    komponen: Vec<KomponenOlah>,
}

#[derive(Deserialize)]
struct KomponenOlah {
    komponen: String,
    volume_ml: i32,
    alat_id: Uuid,
}

/// POST /api/donasi/{id}/olah — satu donasi menjadi kantong komponen (karantina sampai pelulusan). Golongan donasi
/// harus sudah terverifikasi karena tercetak di label.
async fn olah(
    State(s): State<AppState>,
    p: Pengguna,
    Path(donasi): Path<Uuid>,
    Json(i): Json<IsianOlah>,
) -> Hasil<Json<Value>> {
    p.wajib("donor.kelola")?;
    if i.komponen.is_empty() || i.komponen.iter().any(|k| !KOMPONEN.contains(&k.komponen.as_str())) {
        return Err(tolak("komponen", "Pilih komponen hasil pengolahan."));
    }
    let mut jenis: Vec<&str> = i.komponen.iter().map(|k| k.komponen.as_str()).collect();
    jenis.sort_unstable();
    jenis.dedup();
    if jenis.len() != i.komponen.len() || (jenis.contains(&"wb") && jenis.len() > 1) {
        return Err(tolak(
            "komponen",
            "Setiap komponen hanya sekali per donasi, dan darah lengkap (WB) tidak bisa digabung dengan komponen lain.",
        ));
    }
    let mut tx = s.db.begin().await?;
    wajib_mode_utd(&mut tx).await?;
    type Baris = (String, String, OffsetDateTime, Option<String>, Option<String>);
    let r: Option<Baris> = sqlx::query_as(
        "select n.nomor, n.status, n.waktu, g.abo::text, g.rhesus::text from donasi n
         left join golongan_donasi g on g.donasi_id = n.id and g.tanda_tangan_id is not null and g.batal_alasan is null
         where n.id = $1 for update of n",
    )
    .bind(donasi)
    .fetch_optional(&mut *tx)
    .await?;
    let (nomor, status, waktu, abo, rh) = r.ok_or_else(|| Galat::TidakDitemukan("Donasi tidak ditemukan.".into()))?;
    if status != "diambil" {
        return Err(Galat::Konflik(format!("Donasi berstatus {status}; tidak bisa diolah lagi.")));
    }
    let (Some(abo), Some(rh)) = (abo, rh) else {
        return Err(Galat::Konflik("Golongan donasi belum terverifikasi; label kantong belum bisa dicetak.".into()));
    };
    let mut kantong = Vec::new();
    for k in &i.komponen {
        let jam: i32 = sqlx::query_scalar("select masa_simpan_jam from aturan_komponen where komponen = $1")
            .bind(&k.komponen)
            .fetch_one(&mut *tx)
            .await?;
        let kedaluwarsa = waktu + time::Duration::hours(jam as i64);
        let v: Value = sqlx::query_scalar(
            "insert into kantong (nomor, nomor_donasi, komponen, abo, rhesus, volume_ml, tanggal_ambil, kedaluwarsa, kedaluwarsa_label,
                                  asal, donasi_id, imltd_nonreaktif, status, alasan_status, alat_id)
             values ($1, $2, $3, $4, $5, $6, $7, $8, $8, 'donasi', $9, false, 'karantina', 'Menunggu uji saring dan pelulusan', $10)
             returning to_jsonb(kantong)",
        )
        .bind(format!("{nomor}-{}", k.komponen.to_uppercase()))
        .bind(&nomor)
        .bind(&k.komponen)
        .bind(&abo)
        .bind(&rh)
        .bind(k.volume_ml)
        .bind(waktu.date())
        .bind(kedaluwarsa)
        .bind(donasi)
        .bind(k.alat_id)
        .fetch_one(&mut *tx)
        .await?;
        audit::ubah(&mut tx, &p, "olah", "kantong", v["id"].as_str().unwrap_or_default(), v.clone()).await?;
        kantong.push(v);
    }
    sqlx::query("update donasi set status = 'diolah' where id = $1").bind(donasi).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(json!(kantong)))
}

#[derive(Deserialize)]
struct Pelulusan {
    sandi: Option<String>,
}

/// POST /api/kantong/{id}/luluskan — hanya penanggung jawab dengan tanda tangan (CPOB Aneks 2 butir 40, 45).
/// Ditolak bila IMLTD wajib belum lengkap/terverifikasi, ada yang reaktif, golongan belum terverifikasi, donor dicekal,
/// atau kantong kedaluwarsa.
async fn luluskan(
    State(s): State<AppState>,
    p: Pengguna,
    Path(id): Path<Uuid>,
    Json(l): Json<Pelulusan>,
) -> Hasil<Json<Value>> {
    p.wajib("donor.lulus")?;
    let mut tx = s.db.begin().await?;
    let r: Option<(Option<Uuid>, String, OffsetDateTime, Value)> =
        sqlx::query_as("select donasi_id, status, kedaluwarsa, to_jsonb(k) from kantong k where id = $1 for update")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
    let (donasi, status, kedaluwarsa, isi) =
        r.ok_or_else(|| Galat::TidakDitemukan("Kantong tidak ditemukan.".into()))?;
    let donasi = donasi.ok_or_else(|| Galat::Konflik("Pelulusan hanya untuk kantong hasil donasi sendiri.".into()))?;
    if status != "karantina" {
        return Err(Galat::Konflik(format!("Kantong berstatus {status}.")));
    }
    if kedaluwarsa <= OffsetDateTime::now_utc() {
        return Err(Galat::Konflik("Kantong sudah kedaluwarsa.".into()));
    }
    let (lengkap, reaktif, golongan, donor): (Vec<String>, bool, bool, Uuid) = sqlx::query_as(
        "select array(select distinct u.parameter from uji_imltd u where u.donasi_id = n.id and u.tanda_tangan_id is not null and u.hasil = 'nonreaktif'),
                exists (select 1 from uji_imltd u where u.donasi_id = n.id and u.hasil = 'reaktif' and u.batal_alasan is null),
                exists (select 1 from golongan_donasi g where g.donasi_id = n.id and g.tanda_tangan_id is not null and g.batal_alasan is null),
                n.donor_id
         from donasi n where n.id = $1",
    )
    .bind(donasi)
    .fetch_one(&mut *tx)
    .await?;
    let kurang: Vec<&str> = IMLTD_WAJIB.iter().copied().filter(|w| !lengkap.iter().any(|l| l == w)).collect();
    if reaktif {
        return Err(Galat::Konflik(
            "Donasi ini reaktif IMLTD; kantong tidak boleh diluluskan dan harus dimusnahkan.".into(),
        ));
    }
    if !kurang.is_empty() {
        return Err(Galat::Konflik(format!(
            "Uji saring belum lengkap atau belum diverifikasi: {}.",
            kurang.join(", ").to_uppercase()
        )));
    }
    if !golongan {
        return Err(Galat::Konflik("Golongan donasi belum terverifikasi.".into()));
    }
    if let Some(c) = cekal_aktif(&mut tx, donor).await? {
        return Err(Galat::Konflik(format!("Donor sedang dicekal ({c}); kantong tidak boleh diluluskan.")));
    }
    tanda_tangan::sendiri(&mut tx, &p, l.sandi.as_deref().unwrap_or(""), "pelulusan", "kantong", id, &isi).await?;
    sqlx::query("update kantong set imltd_nonreaktif = true where id = $1").bind(id).execute(&mut *tx).await?;
    let v = ubah_status(&mut tx, &p, id, &["karantina"], "tersedia", Some("Diluluskan")).await?;
    crate::integrasi::antre_produk(&mut tx, id, "lulus").await?;
    tx.commit().await?;
    Ok(Json(v))
}

async fn aturan(State(s): State<AppState>, _p: Pengguna) -> Hasil<Json<Value>> {
    Ok(Json(sqlx::query_scalar("select to_jsonb(a) from aturan_donor a").fetch_one(&s.db).await?))
}

#[derive(Deserialize)]
struct IsianAturanDonor {
    nilai: Value,
    alasan: Option<String>,
}

/// Mengubah ambang seleksi donor; hanya kolom yang dikenal, nilai numerik.
async fn ubah_aturan(State(s): State<AppState>, p: Pengguna, Json(i): Json<IsianAturanDonor>) -> Hasil<Json<Value>> {
    p.wajib("aturan.kelola")?;
    let alasan = wajib_teks(&i.alasan, "alasan", "Alasan perubahan aturan")?;
    let mut tx = s.db.begin().await?;
    let sebelum: Value = sqlx::query_scalar("select to_jsonb(a) - 'id' - 'diubah_pada' from aturan_donor a for update")
        .fetch_one(&mut *tx)
        .await?;
    let mut baru = sebelum.clone();
    for (k, v) in i.nilai.as_object().into_iter().flatten() {
        if sebelum.get(k).is_none() || !v.is_number() {
            return Err(tolak(k, "Isian aturan tidak dikenal atau bukan angka."));
        }
        baru[k] = v.clone();
    }
    // jsonb_populate_record memakai tipe kolom tabel, jadi nilai di luar tipe ditolak database.
    let sesudah: Value = sqlx::query_scalar(
        "update aturan_donor a set (umur_min, umur_maks, berat_min, hb_min, hb_maks, sistolik_min, sistolik_maks, diastolik_min, diastolik_maks,
                                    nadi_min, nadi_maks, suhu_maks, hari_antar_donasi, diubah_pada) =
           (select r.umur_min, r.umur_maks, r.berat_min, r.hb_min, r.hb_maks, r.sistolik_min, r.sistolik_maks, r.diastolik_min, r.diastolik_maks,
                   r.nadi_min, r.nadi_maks, r.suhu_maks, r.hari_antar_donasi, now()
            from jsonb_populate_record(null::aturan_donor, $1) r)
         returning to_jsonb(a)",
    )
    .bind(&baru)
    .fetch_one(&mut *tx)
    .await?;
    audit::catat(
        &mut tx,
        Some(&p),
        Jejak {
            aksi: "ubah",
            objek: "aturan_donor",
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

// ---------- Look-back ----------

/// Membuka look-back dan mengarantina kantong terkait yang masih ada di stok.
async fn buka_lookback(
    conn: &mut PgConnection,
    p: &Pengguna,
    donor: Option<Uuid>,
    nomor_donasi: Option<&str>,
    pemicu: Option<Uuid>,
    alasan: &str,
) -> Hasil<Uuid> {
    let id: Uuid = sqlx::query_scalar(
        "insert into lookback (donor_id, nomor_donasi, donasi_pemicu, alasan, dibuat_oleh) values ($1, $2, $3, $4, $5) returning id",
    )
    .bind(donor)
    .bind(nomor_donasi)
    .bind(pemicu)
    .bind(alasan)
    .bind(p.akun_id)
    .fetch_one(&mut *conn)
    .await?;
    let di_stok: Vec<Uuid> = sqlx::query_scalar(
        "select k.id from kantong k left join donasi n on n.id = k.donasi_id
         where (n.donor_id = $1 or k.nomor_donasi = $2) and k.status in ('tersedia', 'dialokasikan') for update of k",
    )
    .bind(donor)
    .bind(nomor_donasi)
    .fetch_all(&mut *conn)
    .await?;
    for k in di_stok {
        sqlx::query(
            "update alokasi set dilepas_pada = now(), alasan_lepas = $2 where kantong_id = $1 and dilepas_pada is null",
        )
        .bind(k)
        .bind(format!("Look-back: {alasan}"))
        .execute(&mut *conn)
        .await?;
        ubah_status(conn, p, k, &["tersedia", "dialokasikan"], "karantina", Some(&format!("Look-back: {alasan}")))
            .await?;
    }
    audit::ubah(
        conn,
        p,
        "buka",
        "lookback",
        id,
        json!({ "donor_id": donor, "nomor_donasi": nomor_donasi, "alasan": alasan }),
    )
    .await?;
    Ok(id)
}

#[derive(Deserialize)]
struct IsianLookbackUtd {
    nomor_donasi: Option<String>,
    alasan: Option<String>,
}

/// POST /api/lookback — notifikasi dari UTD mitra bahwa nomor donasi tertentu bermasalah (berlaku juga di mode BDRS).
async fn lookback_utd(State(s): State<AppState>, p: Pengguna, Json(i): Json<IsianLookbackUtd>) -> Hasil<Json<Value>> {
    p.wajib("stok.kelola")?;
    let nomor = wajib_teks(&i.nomor_donasi, "nomor_donasi", "Nomor donasi dari notifikasi UTD")?;
    let alasan = wajib_teks(&i.alasan, "alasan", "Isi notifikasi UTD (mis. donor reaktif HCV pada donasi berikutnya)")?;
    let mut tx = s.db.begin().await?;
    let id = buka_lookback(&mut tx, &p, None, Some(nomor), None, alasan).await?;
    tx.commit().await?;
    Ok(Json(json!({ "id": id })))
}

async fn daftar_lookback(State(s): State<AppState>, p: Pengguna) -> Hasil<Json<Value>> {
    p.wajib("stok.lihat")?;
    let v: Value = sqlx::query_scalar(
        "select coalesce(jsonb_agg(to_jsonb(l) || jsonb_build_object('donor', d.nama) order by l.status = 'terbuka' desc, l.dibuat_pada desc), '[]')
         from lookback l left join donor d on d.id = l.donor_id",
    )
    .fetch_one(&s.db)
    .await?;
    Ok(Json(v))
}

/// Semua kantong terkait (dari donor yang sama, atau nomor donasi yang dilaporkan) beserta penerima dan dokternya.
async fn detail_lookback(State(s): State<AppState>, p: Pengguna, Path(id): Path<Uuid>) -> Hasil<Json<Value>> {
    p.wajib("stok.lihat")?;
    let mut tx = s.db.begin().await?;
    let v: Option<Value> = sqlx::query_scalar(
        "select to_jsonb(l) || jsonb_build_object('donor', (select jsonb_build_object('nama', d.nama, 'nik', d.nik) from donor d where d.id = l.donor_id),
            'kantong', (select coalesce(jsonb_agg(jsonb_build_object('nomor', k.nomor, 'komponen', k.komponen, 'status', k.status,
                            'donasi', coalesce(n.nomor, k.nomor_donasi), 'waktu_donasi', n.waktu,
                            'penerima', (select jsonb_build_object('no_rm', ps.no_rm, 'nama', ps.nama, 'transfusi', t.mulai, 'dokter', r.dokter, 'ruang', r.ruang)
                                         from transfusi t join pasien ps on ps.no_rm = t.no_rm join pengeluaran g on g.id = t.pengeluaran_id
                                         join alokasi a on a.id = g.alokasi_id join permintaan r on r.id = a.permintaan_id where t.kantong_id = k.id))
                            order by n.waktu desc nulls last, k.nomor), '[]')
                        from kantong k left join donasi n on n.id = k.donasi_id
                        where (l.donor_id is not null and n.donor_id = l.donor_id) or k.nomor_donasi = l.nomor_donasi))
         from lookback l where l.id = $1",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?;
    let v = v.ok_or_else(|| Galat::TidakDitemukan("Look-back tidak ditemukan.".into()))?;
    audit::catat(
        &mut tx,
        Some(&p),
        Jejak { aksi: "lihat", objek: "lookback", objek_id: Some(id.to_string()), ..Default::default() },
    )
    .await?;
    tx.commit().await?;
    Ok(Json(v))
}

#[derive(Deserialize)]
struct Selesai {
    catatan: Option<String>,
}

async fn selesai_lookback(
    State(s): State<AppState>,
    p: Pengguna,
    Path(id): Path<Uuid>,
    Json(c): Json<Selesai>,
) -> Hasil<Json<Value>> {
    p.wajib("stok.kelola")?;
    let catatan = wajib_teks(
        &c.catatan,
        "catatan",
        "Tindak lanjut (dokter penerima yang sudah diberi tahu, kantong yang dimusnahkan)",
    )?;
    let mut tx = s.db.begin().await?;
    let n = sqlx::query("update lookback set status = 'selesai', catatan = $2, ditutup_oleh = $3, ditutup_pada = now() where id = $1 and status = 'terbuka'")
        .bind(id)
        .bind(catatan)
        .bind(p.akun_id)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    if n == 0 {
        return Err(Galat::Konflik("Look-back tidak ditemukan atau sudah selesai.".into()));
    }
    audit::ubah(&mut tx, &p, "selesai", "lookback", id, json!({ "catatan": catatan })).await?;
    tx.commit().await?;
    Ok(Json(json!({})))
}
