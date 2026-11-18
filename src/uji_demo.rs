//! Data demo harus selalu bisa dibuat lewat aturan domain yang berlaku.

use crate::{AppState, config, demo};
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test]
async fn seed_demo_berhasil(db: PgPool) {
    demo::jalankan(AppState { db: db.clone(), cfg: Arc::new(config::Config::uji()) }).await.unwrap();
    let n: i64 = sqlx::query_scalar("select count(*) from transfusi").fetch_one(&db).await.unwrap();
    assert_eq!(n, 2);
    let putus: Option<i64> = sqlx::query_scalar("select audit_periksa()").fetch_one(&db).await.unwrap();
    assert_eq!(putus, None);
}

#[sqlx::test]
async fn cache_berkas_web(db: PgPool) {
    use axum::http::Request;
    use tower::ServiceExt;
    let app = crate::uji::Uji::baru(db).app;
    let minta = |uri: &str| Request::builder().uri(uri).body(axum::body::Body::empty()).unwrap();
    let res = app.clone().oneshot(minta("/permintaan/123")).await.unwrap();
    assert_eq!(res.headers()["cache-control"], "no-cache");
    let res = app.clone().oneshot(minta("/_app/immutable/entry/app.js")).await.unwrap();
    assert_eq!(res.headers()["cache-control"], "public, max-age=31536000, immutable");
    let res = app.oneshot(minta("/api/sehat")).await.unwrap();
    assert_eq!(res.headers()["cache-control"], "no-store");
}
