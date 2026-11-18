//! Data induk (SPEK M1): profil instalasi, kop surat, UTD mitra, alat simpan, aturan komponen (ADR 0004).

use crate::AppState;
use crate::audit::{self, Jejak};
use crate::auth::Pengguna;
use crate::error::{Galat, Hasil, tolak};
use axum::extract::{Path, State};
use axum::routing::{get, put};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

pub fn rute() -> Router<AppState> {
    Router::new()
        .route("/master/profil", get(profil).put(simpan_profil))
        .route("/master/kop", get(kop).put(simpan_kop))
        .route("/auth/info", get(info_instalasi))
        .route("/auth/logo", get(logo_aplikasi))
        .route("/master/utd", get(daftar_utd).post(buat_utd))
        .route("/master/utd/{id}", put(ubah_utd))
        .route("/master/alat", get(daftar_alat).post(buat_alat))
        .route("/master/alat/{id}", put(ubah_alat))
        .route("/master/aturan-komponen", get(daftar_aturan))
        .route("/master/aturan-komponen/{komponen}", put(ubah_aturan))
}

/// Teks yang dipangkas; kosong menjadi None.
pub fn teks(t: &Option<String>) -> Option<&str> {
    t.as_deref().map(str::trim).filter(|t| !t.is_empty())
}

/// Teks wajib.
pub fn wajib_teks<'a>(t: &'a Option<String>, isian: &str, label: &str) -> Hasil<&'a str> {
    teks(t).ok_or_else(|| tolak(isian, format!("{label} wajib diisi.")))
}

async fn profil(State(s): State<AppState>, _p: Pengguna) -> Hasil<Json<Value>> {
    let v: Value = sqlx::query_scalar("select to_jsonb(p) from profil_instalasi p").fetch_one(&s.db).await?;
    Ok(Json(v))
}

#[derive(Deserialize)]
struct IsianProfil {
    nama_rs: Option<String>,
    kode_faskes: Option<String>,
    jenis: String,
}

async fn simpan_profil(State(s): State<AppState>, p: Pengguna, Json(i): Json<IsianProfil>) -> Hasil<Json<Value>> {
    p.wajib("master.kelola")?;
    let nama = wajib_teks(&i.nama_rs, "nama_rs", "Nama rumah sakit")?;
    if !matches!(i.jenis.as_str(), "bdrs" | "utd_rs") {
        return Err(tolak("jenis", "Pilih BDRS atau UTD-RS."));
    }
    let mut tx = s.db.begin().await?;
    let sebelum: Value =
        sqlx::query_scalar("select to_jsonb(p) from profil_instalasi p for update").fetch_one(&mut *tx).await?;
    let sesudah: Value = sqlx::query_scalar(
        "update profil_instalasi p set nama_rs = $1, kode_faskes = $2, jenis = $3, diubah_pada = now() returning to_jsonb(p)",
    )
    .bind(nama)
    .bind(teks(&i.kode_faskes))
    .bind(&i.jenis)
    .fetch_one(&mut *tx)
    .await?;
    audit::catat(
        &mut tx,
        Some(&p),
        Jejak {
            aksi: "ubah",
            objek: "profil_instalasi",
            sebelum: Some(sebelum),
            sesudah: Some(sesudah.clone()),
            ..Default::default()
        },
    )
    .await?;
    tx.commit().await?;
    Ok(Json(sesudah))
}

/// GET /api/master/kop — kop surat untuk dokumen cetak; dibaca semua pengguna yang mencetak.
async fn kop(State(s): State<AppState>, _p: Pengguna) -> Hasil<Json<Value>> {
    let v: Value = sqlx::query_scalar(
        "select to_jsonb(k) || jsonb_build_object('nama_rs_profil', p.nama_rs) from kop_surat k, profil_instalasi p",
    )
    .fetch_one(&s.db)
    .await?;
    Ok(Json(v))
}

/// GET /api/auth/info — publik (halaman masuk belum punya sesi): judul aplikasi dan unit dari kop surat,
/// plus `versi` untuk memperbarui logo yang di-cache browser. Hanya nama, tidak ada data pasien.
async fn info_instalasi(State(s): State<AppState>) -> Hasil<Json<Value>> {
    let v: Value = sqlx::query_scalar(
        "select jsonb_build_object('nama_rs', coalesce(nullif(k.nama_rs, ''), p.nama_rs), 'unit', k.unit,
                'ada_logo', k.logo_kanan <> '' or k.logo_kiri <> '',
                'versi', extract(epoch from greatest(k.diubah_pada, p.diubah_pada))::bigint)
         from kop_surat k, profil_instalasi p",
    )
    .fetch_one(&s.db)
    .await?;
    Ok(Json(v))
}

/// GET /api/auth/logo — publik: logo aplikasi = logo kanan kop (kosong → logo kiri); tanpa logo → 404 (ikon bawaan).
async fn logo_aplikasi(State(s): State<AppState>, headers: axum::http::HeaderMap) -> Hasil<axum::response::Response> {
    use axum::http::{StatusCode, header};
    use axum::response::IntoResponse;
    use base64::Engine;
    let logo: String = sqlx::query_scalar("select coalesce(nullif(logo_kanan, ''), logo_kiri) from kop_surat")
        .fetch_one(&s.db)
        .await?;
    let Some((jenis, isi)) = logo.strip_prefix("data:").and_then(|r| r.split_once(";base64,")) else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };
    // ETag dari isi: browser memvalidasi ulang (no-cache), jadi logo baru langsung terlihat tanpa unduh ulang yang lama.
    let etag = {
        use sha2::{Digest, Sha256};
        format!("\"{}\"", &hex::encode(Sha256::digest(isi.as_bytes()))[..16])
    };
    if headers.get(header::IF_NONE_MATCH).and_then(|v| v.to_str().ok()) == Some(etag.as_str()) {
        return Ok((StatusCode::NOT_MODIFIED, [(header::ETAG, etag)]).into_response());
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(isi)
        .map_err(|_| Galat::Konflik("Logo di kop surat rusak. Unggah ulang di Pengaturan → Kop surat.".into()))?;
    Ok((
        [(header::CONTENT_TYPE, jenis.to_string()), (header::CACHE_CONTROL, "no-cache".into()), (header::ETAG, etag)],
        bytes,
    )
        .into_response())
}

#[derive(Deserialize)]
struct IsianKop {
    pemerintah: Option<String>,
    nama_rs: Option<String>,
    unit: Option<String>,
    alamat: Option<String>,
    kontak: Option<String>,
    akreditasi: Option<String>,
    logo_kiri: Option<String>,
    logo_kanan: Option<String>,
}

const LOGO_MAKS: usize = 512 * 1024;

/// Baris teks kop: dipangkas, boleh kosong, paling panjang 200 karakter.
fn baris_kop<'a>(t: &'a Option<String>, isian: &str, label: &str) -> Hasil<&'a str> {
    let t = t.as_deref().map(str::trim).unwrap_or("");
    if t.chars().count() > 200 {
        return Err(tolak(isian, format!("{label} paling panjang 200 karakter.")));
    }
    Ok(t)
}

/// Logo: kosong, atau data URI PNG/JPEG yang isinya benar-benar PNG/JPEG dan paling besar 512 KB.
pub fn logo_kop<'a>(t: &'a Option<String>, isian: &str, label: &str) -> Hasil<&'a str> {
    use base64::Engine;
    let t = t.as_deref().unwrap_or("");
    if t.is_empty() {
        return Ok(t);
    }
    let salah = || tolak(isian, format!("{label}: unggah berkas PNG atau JPEG."));
    let (jenis, isi) = t.strip_prefix("data:image/").and_then(|r| r.split_once(";base64,")).ok_or_else(salah)?;
    let bytes = base64::engine::general_purpose::STANDARD.decode(isi).map_err(|_| salah())?;
    if bytes.len() > LOGO_MAKS {
        return Err(tolak(isian, format!("{label}: ukuran paling besar 512 KB.")));
    }
    let cocok = match jenis {
        "png" => bytes.starts_with(&[0x89, b'P', b'N', b'G']),
        "jpeg" => bytes.starts_with(&[0xff, 0xd8, 0xff]),
        _ => false,
    };
    if !cocok {
        return Err(salah());
    }
    Ok(t)
}

/// Di jejak audit logo dicatat sebagai sidik SHA-256, bukan isi gambarnya.
fn ringkas_logo(mut v: Value) -> Value {
    use sha2::{Digest, Sha256};
    for k in ["logo_kiri", "logo_kanan"] {
        if let Some(s) = v.get(k).and_then(Value::as_str).filter(|s| !s.is_empty()) {
            v[k] = Value::String(format!("sha256:{}", hex::encode(Sha256::digest(s.as_bytes()))));
        }
    }
    v
}

async fn simpan_kop(State(s): State<AppState>, p: Pengguna, Json(i): Json<IsianKop>) -> Hasil<Json<Value>> {
    p.wajib("master.kelola")?;
    let pemerintah = baris_kop(&i.pemerintah, "pemerintah", "Pemerintah daerah")?;
    let nama_rs = baris_kop(&i.nama_rs, "nama_rs", "Nama rumah sakit")?;
    let unit = baris_kop(&i.unit, "unit", "Unit")?;
    let alamat = baris_kop(&i.alamat, "alamat", "Alamat")?;
    let kontak = baris_kop(&i.kontak, "kontak", "Kontak")?;
    let akreditasi = baris_kop(&i.akreditasi, "akreditasi", "Akreditasi")?;
    let logo_kiri = logo_kop(&i.logo_kiri, "logo_kiri", "Logo kiri")?;
    let logo_kanan = logo_kop(&i.logo_kanan, "logo_kanan", "Logo kanan")?;
    let mut tx = s.db.begin().await?;
    let sebelum: Value =
        sqlx::query_scalar("select to_jsonb(k) from kop_surat k for update").fetch_one(&mut *tx).await?;
    let sesudah: Value = sqlx::query_scalar(
        "update kop_surat k set pemerintah = $1, nama_rs = $2, unit = $3, alamat = $4, kontak = $5, akreditasi = $6,
             logo_kiri = $7, logo_kanan = $8, diubah_pada = now() returning to_jsonb(k)",
    )
    .bind(pemerintah)
    .bind(nama_rs)
    .bind(unit)
    .bind(alamat)
    .bind(kontak)
    .bind(akreditasi)
    .bind(logo_kiri)
    .bind(logo_kanan)
    .fetch_one(&mut *tx)
    .await?;
    audit::catat(
        &mut tx,
        Some(&p),
        Jejak {
            aksi: "ubah",
            objek: "kop_surat",
            sebelum: Some(ringkas_logo(sebelum)),
            sesudah: Some(ringkas_logo(sesudah.clone())),
            ..Default::default()
        },
    )
    .await?;
    tx.commit().await?;
    Ok(Json(sesudah))
}

async fn daftar_utd(State(s): State<AppState>, _p: Pengguna) -> Hasil<Json<Value>> {
    let v: Value = sqlx::query_scalar(
        "select coalesce(jsonb_agg(to_jsonb(u) order by u.aktif desc, u.nama), '[]') from utd_mitra u",
    )
    .fetch_one(&s.db)
    .await?;
    Ok(Json(v))
}

#[derive(Deserialize)]
struct IsianUtd {
    kode: Option<String>,
    nama: Option<String>,
    alamat: Option<String>,
    format_label: String,
    aktif: Option<bool>,
}

async fn simpan_utd(s: &AppState, p: &Pengguna, id: Option<Uuid>, i: IsianUtd) -> Hasil<Json<Value>> {
    p.wajib("master.kelola")?;
    let kode = wajib_teks(&i.kode, "kode", "Kode UTD")?;
    let nama = wajib_teks(&i.nama, "nama", "Nama UTD")?;
    if !matches!(i.format_label.as_str(), "isbt128" | "lokal") {
        return Err(tolak("format_label", "Pilih format label ISBT 128 atau lokal."));
    }
    let mut tx = s.db.begin().await?;
    let sql = if id.is_some() {
        "update utd_mitra set kode = $2, nama = $3, alamat = $4, format_label = $5, aktif = $6 where id = $1
         returning to_jsonb(utd_mitra)"
    } else {
        "insert into utd_mitra (id, kode, nama, alamat, format_label, aktif)
         values (coalesce($1::uuid, gen_random_uuid()), $2, $3, $4, $5, $6) returning to_jsonb(utd_mitra)"
    };
    let v: Option<Value> = sqlx::query_scalar(sql)
        .bind(id)
        .bind(kode)
        .bind(nama)
        .bind(teks(&i.alamat))
        .bind(&i.format_label)
        .bind(i.aktif.unwrap_or(true))
        .fetch_optional(&mut *tx)
        .await?;
    let v = v.ok_or_else(|| Galat::TidakDitemukan("UTD tidak ditemukan.".into()))?;
    audit::ubah(
        &mut tx,
        p,
        if id.is_some() { "ubah" } else { "buat" },
        "utd_mitra",
        v["id"].as_str().unwrap_or_default(),
        v.clone(),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(v))
}

async fn buat_utd(State(s): State<AppState>, p: Pengguna, Json(i): Json<IsianUtd>) -> Hasil<Json<Value>> {
    simpan_utd(&s, &p, None, i).await
}

async fn ubah_utd(
    State(s): State<AppState>,
    p: Pengguna,
    Path(id): Path<Uuid>,
    Json(i): Json<IsianUtd>,
) -> Hasil<Json<Value>> {
    simpan_utd(&s, &p, Some(id), i).await
}

async fn daftar_alat(State(s): State<AppState>, _p: Pengguna) -> Hasil<Json<Value>> {
    // Suhu terakhir dan penyimpangan terbuka ditampilkan bersama alat.
    let v: Value = sqlx::query_scalar(
        "select coalesce(jsonb_agg(to_jsonb(a) || jsonb_build_object(
                'suhu_terakhir', (select to_jsonb(l) from log_suhu l where l.alat_id = a.id order by l.waktu desc limit 1),
                'penyimpangan_id', (select ps.id from penyimpangan_suhu ps where ps.alat_id = a.id and ps.status = 'terbuka'),
                'jumlah_kantong', (select count(*) from kantong k where k.alat_id = a.id
                                   and k.status in ('karantina', 'tersedia', 'dialokasikan')))
            order by a.aktif desc, a.kode), '[]')
         from alat_simpan a",
    )
    .fetch_one(&s.db)
    .await?;
    Ok(Json(v))
}

#[derive(Deserialize)]
struct IsianAlat {
    kode: Option<String>,
    nama: Option<String>,
    nomor_seri: Option<String>,
    jenis: String,
    suhu_min: f64,
    suhu_maks: f64,
    lokasi: Option<String>,
    aktif: Option<bool>,
}

async fn simpan_alat(s: &AppState, p: &Pengguna, id: Option<Uuid>, i: IsianAlat) -> Hasil<Json<Value>> {
    p.wajib("master.kelola")?;
    let kode = wajib_teks(&i.kode, "kode", "Kode alat")?;
    let nama = wajib_teks(&i.nama, "nama", "Nama alat")?;
    if !matches!(i.jenis.as_str(), "lemari" | "freezer" | "inkubator") {
        return Err(tolak("jenis", "Pilih jenis alat: lemari darah, freezer plasma, atau inkubator trombosit."));
    }
    if i.suhu_maks <= i.suhu_min {
        return Err(tolak("suhu_maks", "Suhu maksimum harus lebih tinggi dari suhu minimum."));
    }
    let mut tx = s.db.begin().await?;
    let sql = if id.is_some() {
        "update alat_simpan set kode = $2, nama = $3, nomor_seri = $4, jenis = $5, suhu_min = $6::float8::numeric,
             suhu_maks = $7::float8::numeric, lokasi = $8, aktif = $9 where id = $1 returning to_jsonb(alat_simpan)"
    } else {
        "insert into alat_simpan (id, kode, nama, nomor_seri, jenis, suhu_min, suhu_maks, lokasi, aktif)
         values (coalesce($1::uuid, gen_random_uuid()), $2, $3, $4, $5, $6::float8::numeric, $7::float8::numeric, $8, $9)
         returning to_jsonb(alat_simpan)"
    };
    let v: Option<Value> = sqlx::query_scalar(sql)
        .bind(id)
        .bind(kode)
        .bind(nama)
        .bind(teks(&i.nomor_seri))
        .bind(&i.jenis)
        .bind(i.suhu_min)
        .bind(i.suhu_maks)
        .bind(teks(&i.lokasi))
        .bind(i.aktif.unwrap_or(true))
        .fetch_optional(&mut *tx)
        .await?;
    let v = v.ok_or_else(|| Galat::TidakDitemukan("Alat simpan tidak ditemukan.".into()))?;
    audit::ubah(
        &mut tx,
        p,
        if id.is_some() { "ubah" } else { "buat" },
        "alat_simpan",
        v["id"].as_str().unwrap_or_default(),
        v.clone(),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(v))
}

async fn buat_alat(State(s): State<AppState>, p: Pengguna, Json(i): Json<IsianAlat>) -> Hasil<Json<Value>> {
    simpan_alat(&s, &p, None, i).await
}

async fn ubah_alat(
    State(s): State<AppState>,
    p: Pengguna,
    Path(id): Path<Uuid>,
    Json(i): Json<IsianAlat>,
) -> Hasil<Json<Value>> {
    simpan_alat(&s, &p, Some(id), i).await
}

async fn daftar_aturan(State(s): State<AppState>, _p: Pengguna) -> Hasil<Json<Value>> {
    let v: Value = sqlx::query_scalar(
        "select jsonb_agg(to_jsonb(a) order by array_position(array['wb','prc','tc','ffp','cryo'], a.komponen::text))
         from aturan_komponen a",
    )
    .fetch_one(&s.db)
    .await?;
    Ok(Json(v))
}

#[derive(Deserialize)]
struct IsianAturan {
    uji_silang: String,
    masa_simpan_jam: i32,
    jam_setelah_cair: Option<i32>,
    jam_peringatan: i32,
    alasan: Option<String>,
}

/// Aturan klinis hanya diubah penanggung jawab BDRS, dengan alasan (ADR 0004).
async fn ubah_aturan(
    State(s): State<AppState>,
    p: Pengguna,
    Path(komponen): Path<String>,
    Json(i): Json<IsianAturan>,
) -> Hasil<Json<Value>> {
    p.wajib("aturan.kelola")?;
    let alasan = wajib_teks(&i.alasan, "alasan", "Alasan perubahan aturan")?;
    if !matches!(i.uji_silang.as_str(), "mayor_minor" | "minor" | "tidak_perlu") {
        return Err(tolak("uji_silang", "Pilih aturan uji silang yang dikenal."));
    }
    let mut tx = s.db.begin().await?;
    let sebelum: Option<Value> =
        sqlx::query_scalar("select to_jsonb(a) from aturan_komponen a where komponen = $1 for update")
            .bind(&komponen)
            .fetch_optional(&mut *tx)
            .await?;
    let sebelum = sebelum.ok_or_else(|| Galat::TidakDitemukan("Komponen tidak dikenal.".into()))?;
    if sebelum["berisi_sel_merah"] == true && i.uji_silang != "mayor_minor" {
        return Err(tolak(
            "uji_silang",
            "Komponen berisi sel darah merah wajib uji silang mayor dan minor (Permenkes 91/2015 §5.7.3 B).",
        ));
    }
    let sesudah: Value = sqlx::query_scalar(
        "update aturan_komponen a set uji_silang = $2, masa_simpan_jam = $3, jam_setelah_cair = $4, jam_peringatan = $5,
                diubah_pada = now() where komponen = $1 returning to_jsonb(a)",
    )
    .bind(&komponen)
    .bind(&i.uji_silang)
    .bind(i.masa_simpan_jam)
    .bind(i.jam_setelah_cair)
    .bind(i.jam_peringatan)
    .fetch_one(&mut *tx)
    .await?;
    audit::catat(
        &mut tx,
        Some(&p),
        Jejak {
            aksi: "ubah",
            objek: "aturan_komponen",
            objek_id: Some(komponen),
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
