//! utd-rs: satu binary berisi server aplikasi dan perintah admin.

mod akun;
mod audit;
mod auth;
mod cli;
mod config;
mod error;
#[cfg(test)]
mod uji;

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
        .fallback(|| async { Galat::TidakDitemukan("Alamat API tidak dikenal.".into()) })
        .layer(middleware::from_fn(hanya_json));
    let web = ServeDir::new(&state.cfg.dir_web).fallback(ServeFile::new(format!("{}/index.html", state.cfg.dir_web)));
    Router::new()
        .nest("/api", api)
        .fallback_service(web)
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
    res.headers_mut().insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
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
            "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; frame-ancestors 'none'",
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
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() && args[0] != "serve" {
        return cli::jalankan(&db, &args).await;
    }
    let alamat: SocketAddr = cfg.alamat.parse()?;
    let state = AppState { db, cfg: Arc::new(cfg) };
    tracing::info!(%alamat, "utd-rs berjalan");
    let listener = tokio::net::TcpListener::bind(alamat).await?;
    axum::serve(listener, router(state).into_make_service_with_connect_info::<SocketAddr>()).await?;
    Ok(())
}
