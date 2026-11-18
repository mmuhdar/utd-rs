//! Alat uji API ujung ke ujung dan uji fondasi. Setiap `#[sqlx::test]` mendapat database baru yang sudah dimigrasi.

use crate::{AppState, auth, config, router};
use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;

pub const SANDI: &str = "sandi-uji-panjang";

pub struct Uji {
    pub app: Router,
    pub db: PgPool,
    cookie: Option<String>,
}

impl Uji {
    pub fn baru(db: PgPool) -> Self {
        let app = router(AppState { db: db.clone(), cfg: Arc::new(config::Config::uji()) });
        Uji { app, db, cookie: None }
    }

    /// Membuat akun dengan peran tertentu (sandi sudah diganti) lalu masuk.
    pub async fn sebagai(db: &PgPool, nama_pengguna: &str, peran: &[&str]) -> Self {
        sqlx::query("insert into akun (nama_pengguna, nama, peran, sandi_hash, wajib_ganti_sandi) values ($1, $2, $3, $4, false)")
            .bind(nama_pengguna)
            .bind(format!("Uji {nama_pengguna}"))
            .bind(peran.iter().map(|p| p.to_string()).collect::<Vec<_>>())
            .bind(auth::hash_sandi(SANDI).unwrap())
            .execute(db)
            .await
            .unwrap();
        let mut u = Uji::baru(db.clone());
        let (st, _) = u.post("/api/auth/masuk", json!({ "nama_pengguna": nama_pengguna, "sandi": SANDI })).await;
        assert_eq!(st, StatusCode::OK, "masuk {nama_pengguna}");
        u
    }

    pub async fn kirim(&mut self, metode: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
        let mut req = Request::builder().method(metode).uri(uri);
        if let Some(c) = &self.cookie {
            req = req.header(header::COOKIE, c);
        }
        let req = match body {
            Some(b) => req.header(header::CONTENT_TYPE, "application/json").body(Body::from(b.to_string())),
            None => req.body(Body::empty()),
        }
        .unwrap();
        let res = self.app.clone().oneshot(req).await.unwrap();
        if let Some(c) = res.headers().get(header::SET_COOKIE) {
            self.cookie = Some(c.to_str().unwrap().split(';').next().unwrap().to_string());
        }
        let st = res.status();
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        (st, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
    }

    pub async fn get(&mut self, uri: &str) -> (StatusCode, Value) {
        self.kirim("GET", uri, None).await
    }

    pub async fn post(&mut self, uri: &str, body: Value) -> (StatusCode, Value) {
        self.kirim("POST", uri, Some(body)).await
    }

    /// POST yang harus berhasil; mengembalikan isi respons.
    pub async fn ok(&mut self, uri: &str, body: Value) -> Value {
        let (st, v) = self.post(uri, body).await;
        assert_eq!(st, StatusCode::OK, "POST {uri}: {v}");
        v
    }
}

#[sqlx::test]
async fn masuk_kunci_dan_ganti_sandi(db: PgPool) {
    let mut admin = Uji::sebagai(&db, "admin1", &["admin"]).await;
    let v = admin.ok("/api/akun", json!({ "nama_pengguna": "budi", "nama": "Budi", "peran": ["teknisi"], "sandi_sementara": "sementara-123" })).await;
    assert!(v["id"].is_string());

    // Akun baru wajib ganti sandi sebelum memakai fitur.
    let mut budi = Uji::baru(db.clone());
    let (st, _) = budi.post("/api/auth/masuk", json!({ "nama_pengguna": "budi", "sandi": "sementara-123" })).await;
    assert_eq!(st, StatusCode::OK);
    let (st, _) = budi.get("/api/audit").await;
    assert_eq!(st, StatusCode::FORBIDDEN);
    let (st, v) =
        budi.post("/api/auth/ganti-sandi", json!({ "sandi_lama": "sementara-123", "sandi_baru": "pendek" })).await;
    assert_eq!(st, StatusCode::UNPROCESSABLE_ENTITY, "{v}");
    budi.ok("/api/auth/ganti-sandi", json!({ "sandi_lama": "sementara-123", "sandi_baru": "sandi-baru-budi" })).await;

    // Lima kali salah mengunci akun, walau sandi benar sesudahnya.
    let mut lain = Uji::baru(db.clone());
    for _ in 0..5 {
        let (st, _) =
            lain.post("/api/auth/masuk", json!({ "nama_pengguna": "budi", "sandi": "salah-salah-salah" })).await;
        assert_eq!(st, StatusCode::UNAUTHORIZED);
    }
    let (st, v) = lain.post("/api/auth/masuk", json!({ "nama_pengguna": "budi", "sandi": "sandi-baru-budi" })).await;
    assert_eq!(st, StatusCode::UNAUTHORIZED);
    assert!(v["pesan"].as_str().unwrap().contains("dikunci"));

    // Teknisi tidak boleh mengelola akun.
    let (st, _) = budi.get("/api/akun").await;
    assert_eq!(st, StatusCode::FORBIDDEN);
}

#[sqlx::test]
async fn audit_berantai_dan_tidak_bisa_diubah(db: PgPool) {
    let mut admin = Uji::sebagai(&db, "admin1", &["admin"]).await;
    admin.ok("/api/akun", json!({ "nama_pengguna": "sari", "nama": "Sari", "peran": ["perawat"], "sandi_sementara": "sementara-123" })).await;
    let (_, v) = admin.get("/api/audit/periksa").await;
    assert_eq!(v["utuh"], true, "{v}");
    assert!(v["jumlah"].as_i64().unwrap() >= 2);

    assert!(sqlx::query("update audit set aksi = 'x'").execute(&db).await.is_err());
    assert!(sqlx::query("delete from audit").execute(&db).await.is_err());

    // Pemalsuan lewat jalur di luar aplikasi (trigger dimatikan) tetap terdeteksi oleh rantai hash.
    sqlx::query("alter table audit disable trigger audit_append_only").execute(&db).await.unwrap();
    sqlx::query("update audit set aksi = 'palsu' where urutan = 1").execute(&db).await.unwrap();
    let (_, v) = admin.get("/api/audit/periksa").await;
    assert_eq!(v["utuh"], false);
    assert_eq!(v["putus_di_urutan"], 1);
}

#[sqlx::test]
async fn mutasi_wajib_json(db: PgPool) {
    let u = Uji::baru(db);
    let req = Request::builder().method("POST").uri("/api/auth/masuk").body(Body::from("nama_pengguna=a")).unwrap();
    let res = u.app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}
