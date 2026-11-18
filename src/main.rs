//! utd-rs: satu binary berisi server aplikasi dan perintah admin.

mod akun;
mod audit;
mod auth;
mod cli;
mod config;
mod demo;
mod donor;
mod error;
mod integrasi;
mod laporan;
mod master;
mod permintaan;
mod pratransfusi;
mod stok;
mod suhu;
mod tanda_tangan;
mod transfusi;
#[cfg(test)]
mod uji;
#[cfg(test)]
mod uji_demo;
#[cfg(test)]
mod uji_donor;
#[cfg(test)]
mod uji_stok;
#[cfg(test)]
mod uji_transfusi;

use axum::extract::Request;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::http::{HeaderValue, Method};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use error::Galat;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub cfg: Arc<config::Config>,
}

pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/sehat", get(sehat))
        .route("/auth/masuk", post(auth::masuk))
        .route("/auth/keluar", post(auth::keluar))
        .route("/auth/saya", get(auth::saya))
        .route("/auth/ganti-sandi", post(auth::ganti_sandi))
        .route("/audit", get(audit::daftar))
        .route("/audit/periksa", get(audit::periksa))
        .merge(akun::rute())
        .merge(master::rute())
        .merge(stok::rute())
        .merge(suhu::rute())
        .merge(permintaan::rute())
        .merge(pratransfusi::rute())
        .merge(transfusi::rute())
        .merge(donor::rute())
        .merge(integrasi::rute())
        .merge(laporan::rute())
        .fallback(|| async { Galat::TidakDitemukan("Alamat API tidak dikenal.".into()) })
        .layer(middleware::from_fn(hanya_json));
    let web = ServeDir::new(&state.cfg.dir_web).fallback(ServeFile::new(format!("{}/index.html", state.cfg.dir_web)));
    Router::new()
        .nest("/api", api)
        .fallback_service(web)
        .layer(middleware::from_fn(cache_web))
        .layer(middleware::from_fn(header_keamanan))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn sehat(axum::extract::State(s): axum::extract::State<AppState>) -> Json<serde_json::Value> {
    let db = sqlx::query("select 1").execute(&s.db).await.is_ok();
    Json(serde_json::json!({ "status": if db { "ok" } else { "db_gagal" } }))
}

/// Permintaan yang mengubah data wajib JSON. Form lintas situs tidak bisa mengirim application/json tanpa
/// preflight CORS, jadi ini menutup CSRF bersama cookie SameSite=Strict.
async fn hanya_json(req: Request, next: Next) -> Response {
    let ubah = !matches!(*req.method(), Method::GET | Method::HEAD | Method::OPTIONS);
    let json = req
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("application/json"));
    if ubah && !json {
        return Galat::Permintaan("Permintaan harus berformat JSON.".into()).into_response();
    }
    let mut res = next.run(req).await;
    // Penolakan ekstraktor axum (JSON/path/query rusak) berupa teks berbahasa Inggris; dibungkus menjadi galat
    // berbahasa Indonesia supaya antarmuka selalu menerima `{pesan}`.
    let teks_polos =
        res.headers().get(CONTENT_TYPE).and_then(|v| v.to_str().ok()).is_some_and(|v| v.starts_with("text/plain"));
    if teks_polos && res.status().is_client_error() {
        let rincian = axum::body::to_bytes(res.into_body(), 4096)
            .await
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .unwrap_or_default();
        let mut r = Galat::Permintaan(format!(
            "Data yang dikirim tidak sesuai format yang diharapkan ({rincian}). Periksa isian, lalu coba lagi."
        ))
        .into_response();
        r.headers_mut().insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
        return r;
    }
    res.headers_mut().insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    res
}

/// Berkas web: aset ber-hash di /_app/immutable/ boleh di-cache selamanya; index.html dan berkas lain harus
/// divalidasi ulang, supaya setelah pembaruan versi browser tidak memuat index lama yang merujuk aset yang sudah hilang.
async fn cache_web(req: Request, next: Next) -> Response {
    let abadi = req.uri().path().starts_with("/_app/immutable/");
    let mut res = next.run(req).await;
    if !res.headers().contains_key(CACHE_CONTROL) {
        let nilai = if abadi { "public, max-age=31536000, immutable" } else { "no-cache" };
        res.headers_mut().insert(CACHE_CONTROL, HeaderValue::from_static(nilai));
    }
    res
}

async fn header_keamanan(req: Request, next: Next) -> Response {
    let mut res = next.run(req).await;
    let h = res.headers_mut();
    h.insert("x-content-type-options", HeaderValue::from_static("nosniff"));
    h.insert("x-frame-options", HeaderValue::from_static("DENY"));
    h.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    h.insert(
        "content-security-policy",
        HeaderValue::from_static(
            "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self' data:; frame-ancestors 'none'",
        ),
    );
    res
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();
    let cfg = config::Config::dari_env()?;
    let db = sqlx::postgres::PgPoolOptions::new().max_connections(20).connect(&cfg.database_url).await?;
    sqlx::migrate!().run(&db).await?;
    let alamat: SocketAddr = cfg.alamat.parse()?;
    let satusehat = cfg.satusehat.clone();
    let state = AppState { db: db.clone(), cfg: Arc::new(cfg) };
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() && args[0] != "serve" {
        return cli::jalankan(state, &args).await;
    }
    if let Some(ss) = satusehat {
        tokio::spawn(integrasi::jalankan(db, ss));
    }
    tracing::info!(%alamat, "utd-rs berjalan");
    let listener = tokio::net::TcpListener::bind(alamat).await?;
    axum::serve(listener, router(state).into_make_service_with_connect_info::<SocketAddr>()).await?;
    Ok(())
}
