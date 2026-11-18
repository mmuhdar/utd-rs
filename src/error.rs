//! Galat API. Pesan ditulis untuk pengguna: apa yang salah dan apa yang harus dilakukan.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct GalatIsian {
    pub isian: String,
    pub pesan: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Galat {
    #[error("Sesi tidak ditemukan atau sudah berakhir. Silakan masuk kembali.")]
    BelumMasuk,
    #[error("{0}")]
    Autentikasi(String),
    #[error("{0}")]
    Terlarang(String),
    #[error("{0}")]
    TidakDitemukan(String),
    #[error("{0}")]
    Konflik(String),
    #[error("{0}")]
    Permintaan(String),
    #[error("Data belum valid. Periksa isian yang ditandai.")]
    Validasi(Vec<GalatIsian>),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

pub type Hasil<T> = Result<T, Galat>;

/// Galat validasi untuk satu isian.
pub fn tolak(isian: &str, pesan: impl Into<String>) -> Galat {
    Galat::Validasi(vec![GalatIsian { isian: isian.into(), pesan: pesan.into() }])
}

impl From<sqlx::Error> for Galat {
    fn from(e: sqlx::Error) -> Self {
        if let sqlx::Error::Database(db) = &e {
            let aturan = db.constraint().unwrap_or("aturan data").to_string();
            match db.code().as_deref() {
                Some("23505") => return Galat::Konflik(format!("Data dengan nilai yang sama sudah ada ({aturan}).")),
                Some("23503") => {
                    return Galat::Permintaan("Data yang dirujuk tidak ditemukan atau masih dipakai data lain.".into());
                }
                Some("23514") => return Galat::Permintaan(format!("Nilai tidak diizinkan ({aturan}).")),
                _ => {}
            }
        }
        Galat::Internal(e.into())
    }
}

impl IntoResponse for Galat {
    fn into_response(self) -> Response {
        let status = match &self {
            Galat::BelumMasuk | Galat::Autentikasi(_) => StatusCode::UNAUTHORIZED,
            Galat::Terlarang(_) => StatusCode::FORBIDDEN,
            Galat::TidakDitemukan(_) => StatusCode::NOT_FOUND,
            Galat::Konflik(_) => StatusCode::CONFLICT,
            Galat::Permintaan(_) => StatusCode::BAD_REQUEST,
            Galat::Validasi(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Galat::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = match self {
            Galat::Validasi(isian) => {
                serde_json::json!({ "pesan": "Data belum valid. Periksa isian yang ditandai.", "isian": isian })
            }
            Galat::Internal(e) => {
                let kode = uuid::Uuid::now_v7();
                tracing::error!(%kode, galat = ?e, "galat internal");
                serde_json::json!({
                    "pesan": format!("Terjadi kesalahan di server. Coba lagi; bila berulang, laporkan kode {kode} ke admin TI.")
                })
            }
            lain => serde_json::json!({ "pesan": lain.to_string() }),
        };
        (status, Json(body)).into_response()
    }
}
