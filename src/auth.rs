//! Masuk, sesi, dan hak akses per peran.
//!
//! Sesi disimpan di server: cookie hanya berisi token acak, database menyimpan hash-nya. Setelah 5 kali gagal,
//! akun dikunci 15 menit. Akun baru dan akun yang sandinya direset wajib mengganti sandi sebelum memakai fitur lain.

use crate::AppState;
use crate::audit::{self, Jejak};
use crate::error::{Galat, Hasil, tolak};
use argon2::Argon2;
use argon2::password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash};
use axum::Json;
use axum::extract::{ConnectInfo, FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::{HeaderMap, HeaderValue, header};
use axum::response::{AppendHeaders, IntoResponse};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::convert::Infallible;
use std::net::SocketAddr;
use uuid::Uuid;

pub const COOKIE: &str = "utd_sesi";
const MENIT_DIAM: i64 = 30;
const JAM_MAKS: i64 = 12;
const BATAS_GAGAL: i32 = 5;

/// Hak tiap peran. Pemisahan tugas: dokter meminta, teknisi menguji dan menyerahkan, perawat memberikan,
/// kepala (penanggung jawab pelayanan darah) memverifikasi dan menutup laporan, auditor hanya membaca.
pub fn hak_peran(peran: &str) -> &'static [&'static str] {
    match peran {
        "admin" => &["akun.kelola", "master.kelola", "audit.lihat", "laporan.lihat"],
        "dokter" => &["permintaan.buat", "permintaan.lihat", "transfusi.catat", "reaksi.lapor", "pasien.lihat"],
        "perawat" => &["permintaan.lihat", "transfusi.catat", "reaksi.lapor", "pasien.lihat"],
        "teknisi" => &[
            "permintaan.lihat",
            "permintaan.proses",
            "pasien.lihat",
            "stok.kelola",
            "uji.input",
            "penyerahan.kelola",
            "donor.kelola",
            "laporan.lihat",
        ],
        "kepala" => &[
            "permintaan.lihat",
            "permintaan.proses",
            "pasien.lihat",
            "stok.kelola",
            "uji.input",
            "uji.verifikasi",
            "penyerahan.kelola",
            "donor.kelola",
            "reaksi.tinjau",
            "master.kelola",
            "audit.lihat",
            "laporan.lihat",
        ],
        "auditor" => &["audit.lihat", "laporan.lihat", "permintaan.lihat", "pasien.lihat"],
        _ => &[],
    }
}

/// Alamat dan perangkat klien, untuk jejak audit.
#[derive(Clone, Debug, Default)]
pub struct Klien {
    pub alamat: Option<String>,
    pub perangkat: Option<String>,
}

impl FromRequestParts<AppState> for Klien {
    type Rejection = Infallible;
    async fn from_request_parts(parts: &mut Parts, s: &AppState) -> Result<Self, Infallible> {
        let peer = parts.extensions.get::<ConnectInfo<SocketAddr>>().map(|c| c.0.ip().to_string());
        // Entri terakhir X-Forwarded-For ditambahkan oleh reverse proxy itu sendiri; entri sebelumnya bisa dipalsukan.
        let diteruskan = parts
            .headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.rsplit(',').next())
            .map(|v| v.trim().to_string());
        Ok(Klien {
            alamat: if s.cfg.di_balik_proxy { diteruskan.or(peer) } else { peer },
            perangkat: parts
                .headers
                .get(header::USER_AGENT)
                .and_then(|v| v.to_str().ok())
                .map(|v| v.chars().take(200).collect()),
        })
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Pengguna {
    pub akun_id: Uuid,
    pub nama: String,
    pub nama_pengguna: String,
    pub peran: Vec<String>,
    pub wajib_ganti_sandi: bool,
    #[serde(skip)]
    pub alamat: Option<String>,
    #[serde(skip)]
    pub perangkat: Option<String>,
}

impl Pengguna {
    pub fn boleh(&self, hak: &str) -> bool {
        self.peran.iter().any(|p| hak_peran(p).contains(&hak))
    }

    pub fn wajib(&self, hak: &str) -> Hasil<()> {
        if self.wajib_ganti_sandi {
            return Err(Galat::Terlarang("Ganti kata sandi sementara Anda terlebih dahulu.".into()));
        }
        if !self.boleh(hak) {
            return Err(Galat::Terlarang(format!(
                "Peran Anda tidak mencakup hak '{hak}'. Hubungi kepala bank darah bila tugas Anda memerlukannya."
            )));
        }
        Ok(())
    }
}

fn hash_token(token: &[u8]) -> Vec<u8> {
    Sha256::digest(token).to_vec()
}

fn token_dari_cookie(headers: &HeaderMap) -> Option<Vec<u8>> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .find_map(|c| c.trim().strip_prefix("utd_sesi="))
        .and_then(|t| URL_SAFE_NO_PAD.decode(t).ok())
}

#[derive(sqlx::FromRow)]
struct BarisPengguna {
    id: Uuid,
    nama: String,
    nama_pengguna: String,
    peran: Vec<String>,
    wajib_ganti_sandi: bool,
}

impl FromRequestParts<AppState> for Pengguna {
    type Rejection = Galat;
    async fn from_request_parts(parts: &mut Parts, s: &AppState) -> Result<Self, Galat> {
        let klien = Klien::from_request_parts(parts, s).await.unwrap_or_default();
        let token = token_dari_cookie(&parts.headers).ok_or(Galat::BelumMasuk)?;
        // Sesi diperpanjang sekaligus diperiksa; sesi yang diam terlalu lama atau melewati batas maksimum ditolak.
        let b: BarisPengguna = sqlx::query_as(
            "update sesi s set terakhir_pada = now() from akun a
             where s.hash = $1 and a.id = s.akun_id and a.aktif
               and s.berakhir_pada > now() and s.terakhir_pada > now() - make_interval(mins => $2)
             returning a.id, a.nama, a.nama_pengguna, a.peran, a.wajib_ganti_sandi",
        )
        .bind(hash_token(&token))
        .bind(MENIT_DIAM as i32)
        .fetch_optional(&s.db)
        .await?
        .ok_or(Galat::BelumMasuk)?;
        Ok(Pengguna {
            akun_id: b.id,
            nama: b.nama,
            nama_pengguna: b.nama_pengguna,
            peran: b.peran,
            wajib_ganti_sandi: b.wajib_ganti_sandi,
            alamat: klien.alamat,
            perangkat: klien.perangkat,
        })
    }
}

pub fn hash_sandi(sandi: &str) -> anyhow::Result<String> {
    Argon2::default()
        .hash_password(sandi.as_bytes())
        .map(|h| h.to_string())
        .map_err(|e| anyhow::anyhow!("hash sandi gagal: {e}"))
}

pub fn sandi_cocok(hash: &str, sandi: &str) -> bool {
    PasswordHash::new(hash).map(|h| Argon2::default().verify_password(sandi.as_bytes(), &h).is_ok()).unwrap_or(false)
}

/// Panjang adalah syarat yang berarti; aturan komposisi tidak dipakai.
pub fn periksa_sandi(sandi: &str) -> Result<(), String> {
    if sandi.chars().count() < 10 {
        return Err("Kata sandi minimal 10 karakter. Kalimat pendek yang mudah diingat lebih baik.".into());
    }
    Ok(())
}

fn cookie_sesi(token: Option<&[u8]>, secure: bool) -> HeaderValue {
    let (nilai, umur) = match token {
        Some(t) => (URL_SAFE_NO_PAD.encode(t), JAM_MAKS * 3600),
        None => (String::new(), 0),
    };
    let secure = if secure { "; Secure" } else { "" };
    HeaderValue::from_str(&format!("{COOKIE}={nilai}; Path=/; HttpOnly; SameSite=Strict; Max-Age={umur}{secure}"))
        .expect("cookie selalu ASCII")
}

#[derive(Deserialize)]
pub struct Masuk {
    nama_pengguna: String,
    sandi: String,
}

#[derive(sqlx::FromRow)]
struct AkunMasuk {
    id: Uuid,
    sandi_hash: String,
    aktif: bool,
    terkunci: bool,
}

/// POST /api/auth/masuk
pub async fn masuk(State(s): State<AppState>, klien: Klien, Json(m): Json<Masuk>) -> Hasil<impl IntoResponse> {
    let nama = m.nama_pengguna.trim().to_lowercase();
    let mut tx = s.db.begin().await?;
    let akun: Option<AkunMasuk> = sqlx::query_as(
        "select id, sandi_hash, aktif, coalesce(terkunci_sampai > now(), false) as terkunci
         from akun where nama_pengguna = $1 for update",
    )
    .bind(&nama)
    .fetch_optional(&mut *tx)
    .await?;
    let gagal = || Galat::Autentikasi("Nama pengguna atau kata sandi salah.".into());
    let Some(a) = akun else {
        // Hash tetap dihitung supaya waktu respons tidak membocorkan apakah akun ada.
        sandi_cocok(
            "$argon2id$v=19$m=19456,t=2,p=1$c2FsdHNhbHRzYWx0$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            &m.sandi,
        );
        let jejak = Jejak {
            aksi: "masuk_gagal",
            objek: "akun",
            objek_id: Some(nama),
            alamat: klien.alamat,
            perangkat: klien.perangkat,
            ..Default::default()
        };
        audit::catat(&mut tx, None, jejak).await?;
        tx.commit().await?;
        return Err(gagal());
    };
    if a.terkunci {
        return Err(Galat::Autentikasi(
            "Akun dikunci sementara karena terlalu banyak percobaan gagal. Coba lagi 15 menit lagi atau minta admin membukanya.".into(),
        ));
    }
    if !a.aktif {
        return Err(Galat::Autentikasi("Akun ini sudah dinonaktifkan. Hubungi admin.".into()));
    }
    if !sandi_cocok(&a.sandi_hash, &m.sandi) {
        sqlx::query(
            "update akun set gagal_masuk = gagal_masuk + 1,
                    terkunci_sampai = case when gagal_masuk + 1 >= $2 then now() + interval '15 minutes' end
             where id = $1",
        )
        .bind(a.id)
        .bind(BATAS_GAGAL)
        .execute(&mut *tx)
        .await?;
        let jejak = Jejak {
            aksi: "masuk_gagal",
            objek: "akun",
            objek_id: Some(a.id.to_string()),
            alamat: klien.alamat,
            perangkat: klien.perangkat,
            ..Default::default()
        };
        audit::catat(&mut tx, None, jejak).await?;
        tx.commit().await?;
        return Err(gagal());
    }
    let mut token = [0u8; 32];
    getrandom::fill(&mut token).map_err(|e| anyhow::anyhow!("acak gagal: {e}"))?;
    sqlx::query("update akun set gagal_masuk = 0, terkunci_sampai = null where id = $1")
        .bind(a.id)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "insert into sesi (hash, akun_id, berakhir_pada, alamat, perangkat)
         values ($1, $2, now() + make_interval(hours => $3), $4, $5)",
    )
    .bind(hash_token(&token))
    .bind(a.id)
    .bind(JAM_MAKS as i32)
    .bind(&klien.alamat)
    .bind(&klien.perangkat)
    .execute(&mut *tx)
    .await?;
    let jejak = Jejak {
        aksi: "masuk",
        objek: "akun",
        objek_id: Some(a.id.to_string()),
        alamat: klien.alamat,
        perangkat: klien.perangkat,
        ..Default::default()
    };
    audit::catat(&mut tx, None, jejak).await?;
    tx.commit().await?;
    let p = profil(&s, a.id).await?;
    Ok((AppendHeaders([(header::SET_COOKIE, cookie_sesi(Some(&token), s.cfg.cookie_secure))]), Json(p)))
}

/// POST /api/auth/keluar
pub async fn keluar(State(s): State<AppState>, headers: HeaderMap, p: Pengguna) -> Hasil<impl IntoResponse> {
    let mut tx = s.db.begin().await?;
    if let Some(t) = token_dari_cookie(&headers) {
        sqlx::query("delete from sesi where hash = $1").bind(hash_token(&t)).execute(&mut *tx).await?;
    }
    audit::catat(
        &mut tx,
        Some(&p),
        Jejak { aksi: "keluar", objek: "akun", objek_id: Some(p.akun_id.to_string()), ..Default::default() },
    )
    .await?;
    tx.commit().await?;
    Ok((AppendHeaders([(header::SET_COOKIE, cookie_sesi(None, s.cfg.cookie_secure))]), Json(serde_json::json!({}))))
}

#[derive(Serialize)]
pub struct Profil {
    #[serde(flatten)]
    pengguna: Pengguna,
    hak: Vec<&'static str>,
    jenis_instalasi: String,
    nama_rs: String,
}

async fn profil(s: &AppState, akun_id: Uuid) -> Hasil<Profil> {
    let b: BarisPengguna =
        sqlx::query_as("select id, nama, nama_pengguna, peran, wajib_ganti_sandi from akun where id = $1")
            .bind(akun_id)
            .fetch_one(&s.db)
            .await?;
    let (jenis, nama_rs): (String, String) =
        sqlx::query_as("select jenis, nama_rs from profil_instalasi").fetch_one(&s.db).await?;
    let mut hak: Vec<&'static str> = b.peran.iter().flat_map(|p| hak_peran(p).iter().copied()).collect();
    hak.sort_unstable();
    hak.dedup();
    let pengguna = Pengguna {
        akun_id: b.id,
        nama: b.nama,
        nama_pengguna: b.nama_pengguna,
        peran: b.peran,
        wajib_ganti_sandi: b.wajib_ganti_sandi,
        alamat: None,
        perangkat: None,
    };
    Ok(Profil { pengguna, hak, jenis_instalasi: jenis, nama_rs })
}

/// GET /api/auth/saya
pub async fn saya(State(s): State<AppState>, p: Pengguna) -> Hasil<Json<Profil>> {
    Ok(Json(profil(&s, p.akun_id).await?))
}

#[derive(Deserialize)]
pub struct GantiSandi {
    sandi_lama: String,
    sandi_baru: String,
}

/// POST /api/auth/ganti-sandi — sesi lain milik akun ini diakhiri.
pub async fn ganti_sandi(
    State(s): State<AppState>,
    headers: HeaderMap,
    p: Pengguna,
    Json(g): Json<GantiSandi>,
) -> Hasil<Json<Profil>> {
    let hash: String =
        sqlx::query_scalar("select sandi_hash from akun where id = $1").bind(p.akun_id).fetch_one(&s.db).await?;
    if !sandi_cocok(&hash, &g.sandi_lama) {
        return Err(tolak("sandi_lama", "Kata sandi lama salah."));
    }
    periksa_sandi(&g.sandi_baru).map_err(|e| tolak("sandi_baru", e))?;
    if g.sandi_baru == g.sandi_lama {
        return Err(tolak("sandi_baru", "Kata sandi baru harus berbeda dari yang lama."));
    }
    let mut tx = s.db.begin().await?;
    sqlx::query("update akun set sandi_hash = $2, wajib_ganti_sandi = false where id = $1")
        .bind(p.akun_id)
        .bind(hash_sandi(&g.sandi_baru)?)
        .execute(&mut *tx)
        .await?;
    let sekarang = token_dari_cookie(&headers).map(|t| hash_token(&t)).unwrap_or_default();
    sqlx::query("delete from sesi where akun_id = $1 and hash <> $2")
        .bind(p.akun_id)
        .bind(sekarang)
        .execute(&mut *tx)
        .await?;
    audit::catat(
        &mut tx,
        Some(&p),
        Jejak { aksi: "ganti_sandi", objek: "akun", objek_id: Some(p.akun_id.to_string()), ..Default::default() },
    )
    .await?;
    tx.commit().await?;
    Ok(Json(profil(&s, p.akun_id).await?))
}
