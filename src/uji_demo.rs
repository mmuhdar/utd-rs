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
