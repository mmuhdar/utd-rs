//! Uji penerimaan, stok, suhu, dan berita acara (SPEK M1–M2).

use crate::uji::{Bdrs, Uji};
use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test]
async fn penerimaan_menolak_dan_mengembalikan(db: PgPool) {
    let mut b = Bdrs::siapkan(&db).await;
    let lewat = "2020-01-01T00:00:00Z";
    let nanti = "2099-01-01T00:00:00Z";
    // Kantong kedaluwarsa tidak bisa diterima sebagai tersedia.
    let (st, v) = b
        .teknisi
        .post(
            "/api/penerimaan",
            json!({ "utd_id": b.utd, "petugas_utd": "X", "kantong": [
                { "nomor": "K1", "komponen": "prc", "abo": "O", "rhesus": "pos", "kedaluwarsa": lewat,
                  "imltd_nonreaktif": true, "diterima": true, "alat_id": b.lemari }]}),
        )
        .await;
    assert_eq!(st, StatusCode::UNPROCESSABLE_ENTITY, "{v}");
    assert_eq!(v["isian"][0]["isian"], "kantong.0.kedaluwarsa");
    // PRC tidak boleh disimpan di freezer.
    let (st, v) = b
        .teknisi
        .post(
            "/api/penerimaan",
            json!({ "utd_id": b.utd, "petugas_utd": "X", "kantong": [
                { "nomor": "K1", "komponen": "prc", "abo": "O", "rhesus": "pos", "kedaluwarsa": nanti,
                  "imltd_nonreaktif": true, "diterima": true, "alat_id": b.freezer }]}),
        )
        .await;
    assert_eq!(st, StatusCode::UNPROCESSABLE_ENTITY, "{v}");
    // Satu diterima, satu ditolak → berita acara pengembalian otomatis.
    let v = b
        .teknisi
        .ok(
            "/api/penerimaan",
            json!({ "utd_id": b.utd, "petugas_utd": "X", "suhu_min_transport": 3.0, "suhu_maks_transport": 8.0, "kantong": [
                { "nomor": "K1", "komponen": "prc", "abo": "O", "rhesus": "pos", "kedaluwarsa": nanti,
                  "imltd_nonreaktif": true, "diterima": true, "alat_id": b.lemari },
                { "nomor": "K2", "komponen": "prc", "abo": "A", "rhesus": "pos", "kedaluwarsa": nanti,
                  "imltd_nonreaktif": true, "diterima": false, "alasan_tolak": "bocor" }]}),
        )
        .await;
    assert!(v["berita_acara_tolak"].is_string());
    let (_, k2) = b.teknisi.get("/api/kantong/lacak/K2").await;
    assert_eq!(k2["status"], "dikembalikan");
    assert_eq!(k2["berita_acara"]["jenis"], "pengembalian");
    // Nomor kantong unik selamanya.
    let (st, _) = b
        .teknisi
        .post(
            "/api/penerimaan",
            json!({ "utd_id": b.utd, "petugas_utd": "X", "kantong": [
                { "nomor": "K1", "komponen": "prc", "abo": "O", "rhesus": "pos", "kedaluwarsa": nanti,
                  "imltd_nonreaktif": true, "diterima": true, "alat_id": b.lemari }]}),
        )
        .await;
    assert_eq!(st, StatusCode::CONFLICT);
    let (_, r) = b.teknisi.get("/api/stok/ringkasan").await;
    assert_eq!(r[0]["tersedia"], 1);
}

#[sqlx::test]
async fn pencairan_mempersingkat_kedaluwarsa(db: PgPool) {
    let mut b = Bdrs::siapkan(&db).await;
    let ids = b.terima(&[("F1", "ffp", "AB", "pos"), ("P1", "prc", "O", "neg")]).await;
    let v = b.teknisi.ok(&format!("/api/kantong/{}/cairkan", ids[0]), json!({ "alat_id": b.lemari })).await;
    let exp = time::OffsetDateTime::parse(v["kedaluwarsa"].as_str().unwrap(), &time::format_description::well_known::Rfc3339).unwrap();
    assert!(exp <= time::OffsetDateTime::now_utc() + time::Duration::hours(6));
    // Tidak bisa dicairkan dua kali, dan PRC tidak dicairkan.
    let (st, _) = b.teknisi.post(&format!("/api/kantong/{}/cairkan", ids[0]), json!({})).await;
    assert_eq!(st, StatusCode::CONFLICT);
    let (st, _) = b.teknisi.post(&format!("/api/kantong/{}/cairkan", ids[1]), json!({})).await;
    assert_eq!(st, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn penyimpangan_suhu_mengarantina_lalu_dinilai(db: PgPool) {
    let mut b = Bdrs::siapkan(&db).await;
    let ids = b.terima(&[("A1", "prc", "A", "pos"), ("A2", "prc", "A", "pos")]).await;
    let v = b.teknisi.ok("/api/suhu", json!({ "alat_id": b.lemari, "suhu": 4.0 })).await;
    assert_eq!(v["menyimpang"], false);

    // Data logger memakai token API.
    let mut admin = Uji::sebagai(&db, "admin", &["admin"]).await;
    let akun = admin.ok("/api/akun", json!({ "nama_pengguna": "logger1", "nama": "Logger L1", "peran": ["logger"] })).await;
    let token = admin.ok(&format!("/api/akun/{}/token", akun["id"].as_str().unwrap()), json!({})).await;
    let mut logger = Uji::baru(db.clone());
    logger.bearer(token["token"].as_str().unwrap());
    let v = logger.ok("/api/suhu", json!({ "alat_kode": "L1", "suhu": 9.5 })).await;
    assert_eq!(v["menyimpang"], true);
    let pid = v["penyimpangan_id"].as_str().unwrap().to_string();
    let (_, k) = b.teknisi.get("/api/kantong/lacak/A1").await;
    assert_eq!(k["status"], "karantina");
    // Logger tidak boleh membaca stok.
    let (st, _) = logger.get("/api/stok").await;
    assert_eq!(st, StatusCode::FORBIDDEN);

    // Alat yang menyimpang tidak bisa menerima kantong baru.
    let (st, _) = b.teknisi.post(&format!("/api/kantong/{}/pindah", ids[0]), json!({ "alat_id": b.lemari })).await;
    assert_eq!(st, StatusCode::UNPROCESSABLE_ENTITY);

    // Teknisi tidak berwenang menilai; kepala menilai satu layak satu tidak.
    let (st, _) = b.teknisi.post(&format!("/api/penyimpangan-suhu/{pid}/nilai"), json!({ "kantong_id": ids[0], "keputusan": "layak", "alasan": "x" })).await;
    assert_eq!(st, StatusCode::FORBIDDEN);
    b.kepala.ok(&format!("/api/penyimpangan-suhu/{pid}/nilai"), json!({ "kantong_id": ids[0], "keputusan": "layak", "alasan": "15 menit, 9,5 °C" })).await;
    let (st, _) = b.kepala.post(&format!("/api/penyimpangan-suhu/{pid}/tutup"), json!({ "kesimpulan": "pintu terbuka", "sandi": crate::uji::SANDI })).await;
    assert_eq!(st, StatusCode::CONFLICT, "masih ada kantong belum dinilai");
    b.kepala.ok(&format!("/api/penyimpangan-suhu/{pid}/nilai"), json!({ "kantong_id": ids[1], "keputusan": "tidak_layak", "alasan": "dekat sumber panas" })).await;
    let (st, _) = b.kepala.post(&format!("/api/penyimpangan-suhu/{pid}/tutup"), json!({ "kesimpulan": "pintu terbuka", "sandi": "salah" })).await;
    assert_eq!(st, StatusCode::UNPROCESSABLE_ENTITY);
    b.kepala.ok(&format!("/api/penyimpangan-suhu/{pid}/tutup"), json!({ "kesimpulan": "pintu terbuka", "sandi": crate::uji::SANDI })).await;
    let (_, k) = b.teknisi.get("/api/kantong/lacak/A1").await;
    assert_eq!(k["status"], "tersedia");
    let (_, k) = b.teknisi.get("/api/kantong/lacak/A2").await;
    assert_eq!(k["status"], "karantina");

    // Kantong tidak layak dimusnahkan dengan berita acara.
    let ba = b
        .teknisi
        .ok("/api/berita-acara", json!({ "jenis": "pemusnahan", "metode": "insinerator", "kantong": [{ "id": ids[1], "alasan": "penyimpangan suhu" }] }))
        .await;
    let (_, d) = b.teknisi.get(&format!("/api/berita-acara/{}", ba["id"].as_str().unwrap())).await;
    assert_eq!(d["kantong"][0]["nomor"], "A2");
}
