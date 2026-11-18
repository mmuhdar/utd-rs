//! Uji mode UTD-RS (SPEK M5) dan pengiriman SATUSEHAT ke server tiruan (SPEK M7).

use crate::config::SatuSehat;
use crate::integrasi::Pengirim;
use crate::uji::{Bdrs, SANDI};
use axum::http::StatusCode;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde_json::{Value, json};
use sqlx::PgPool;

fn seleksi(hb: f64, keputusan: &str) -> Value {
    json!({ "berat_kg": 60.0, "hb": hb, "sistolik": 120, "diastolik": 80, "nadi": 80, "suhu": 36.6, "kuesioner_lolos": true,
            "keputusan": keputusan, "sampai": "2099-01-01" })
}

/// Donor baru sampai donasi diambil dan golongannya terverifikasi; mengembalikan (donor, donasi).
async fn donasi_siap(b: &mut Bdrs, nik: &str, nomor: &str) -> (String, String) {
    let d = b
        .teknisi
        .ok(
            "/api/donor",
            json!({ "nik": nik, "nama": "Donor Uji", "tanggal_lahir": "1990-01-01", "jenis_kelamin": "L" }),
        )
        .await;
    let did = d["id"].as_str().unwrap().to_string();
    let s = b.teknisi.ok(&format!("/api/donor/{did}/seleksi"), seleksi(14.0, "diterima")).await;
    let n = b
        .teknisi
        .ok(
            &format!("/api/seleksi/{}/donasi", s["id"].as_str().unwrap()),
            json!({ "nomor": nomor, "jenis": "sukarela", "volume_ml": 350, "jenis_kantong": "double" }),
        )
        .await;
    let nid = n["id"].as_str().unwrap().to_string();
    let g = b
        .teknisi
        .ok(&format!("/api/donasi/{nid}/golongan"), json!({ "anti_a": "neg", "anti_b": "neg", "sel_a": "pos", "sel_b": "pos", "anti_d": "pos", "metode": "gel" }))
        .await;
    b.teknisi2
        .ok(
            &format!("/api/golongan-donasi/{}/verifikasi", g["id"].as_str().unwrap()),
            json!({ "setuju": true, "sandi": SANDI }),
        )
        .await;
    (did, nid)
}

async fn imltd(b: &mut Bdrs, donasi: &str, parameter: &str, hasil: &str) -> Value {
    let u = b
        .teknisi
        .ok(
            &format!("/api/donasi/{donasi}/imltd"),
            json!({ "parameter": parameter, "metode": "chlia", "hasil": hasil }),
        )
        .await;
    b.teknisi2
        .ok(&format!("/api/imltd/{}/verifikasi", u["id"].as_str().unwrap()), json!({ "setuju": true, "sandi": SANDI }))
        .await
}

#[sqlx::test]
async fn donor_sampai_pelulusan_dan_satusehat(db: PgPool) {
    let mut b = Bdrs::siapkan(&db).await;
    // Mode BDRS: fitur donor tidak aktif.
    let (st, _) = b
        .teknisi
        .post(
            "/api/donor",
            json!({ "nik": "9900000000000001", "nama": "X", "tanggal_lahir": "1990-01-01", "jenis_kelamin": "L" }),
        )
        .await;
    assert_eq!(st, StatusCode::CONFLICT);
    let (st, _) =
        b.kepala.kirim("PUT", "/api/master/profil", Some(json!({ "nama_rs": "RS Uji", "jenis": "utd_rs" }))).await;
    assert_eq!(st, StatusCode::OK);

    // Seleksi: Hb rendah tidak bisa diterima.
    let d = b.teknisi.ok("/api/donor", json!({ "nik": "9900000000000001", "nama": "Donor Satu", "tanggal_lahir": "1990-01-01", "jenis_kelamin": "L" })).await;
    let did = d["id"].as_str().unwrap();
    let (st, v) = b.teknisi.post(&format!("/api/donor/{did}/seleksi"), seleksi(11.0, "diterima")).await;
    assert_eq!(st, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(v["isian"][0]["pesan"].as_str().unwrap().contains("Hb"));

    let (_, nid) = donasi_siap(&mut b, "9900000000000002", "D0001").await;
    let k = b
        .teknisi
        .ok(
            &format!("/api/donasi/{nid}/olah"),
            json!({ "komponen": [
            { "komponen": "prc", "volume_ml": 200, "alat_id": b.lemari },
            { "komponen": "ffp", "volume_ml": 150, "alat_id": b.freezer }] }),
        )
        .await;
    let prc = k[0]["id"].as_str().unwrap().to_string();
    assert_eq!(k[0]["status"], "karantina");
    assert_eq!(k[0]["abo"], "O");

    // Pelulusan: teknisi tidak berwenang; kepala ditolak selama IMLTD belum lengkap.
    let (st, _) = b.teknisi.post(&format!("/api/kantong/{prc}/luluskan"), json!({ "sandi": SANDI })).await;
    assert_eq!(st, StatusCode::FORBIDDEN);
    for p in ["hiv", "hbsag", "hcv"] {
        imltd(&mut b, &nid, p, "nonreaktif").await;
    }
    let (st, v) = b.kepala.post(&format!("/api/kantong/{prc}/luluskan"), json!({ "sandi": SANDI })).await;
    assert_eq!(st, StatusCode::CONFLICT);
    assert!(v["pesan"].as_str().unwrap().contains("SIFILIS"));
    // Hasil yang belum diverifikasi tidak dihitung.
    b.teknisi
        .ok(
            &format!("/api/donasi/{nid}/imltd"),
            json!({ "parameter": "sifilis", "metode": "rapid", "hasil": "nonreaktif" }),
        )
        .await;
    let (st, _) = b.kepala.post(&format!("/api/kantong/{prc}/luluskan"), json!({ "sandi": SANDI })).await;
    assert_eq!(st, StatusCode::CONFLICT);
    let (_, det) = b.teknisi.get(&format!("/api/donasi/{nid}")).await;
    let sifilis = det["imltd"].as_array().unwrap().iter().find(|u| u["parameter"] == "sifilis").unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    b.teknisi2.ok(&format!("/api/imltd/{sifilis}/verifikasi"), json!({ "setuju": true, "sandi": SANDI })).await;
    let v = b.kepala.ok(&format!("/api/kantong/{prc}/luluskan"), json!({ "sandi": SANDI })).await;
    assert_eq!(v["status"], "tersedia");

    // Outbox SATUSEHAT terisi dan terkirim ke server tiruan.
    let (st, out) = b.kepala.get("/api/integrasi/outbox").await;
    assert_eq!(st, StatusCode::FORBIDDEN, "hanya admin");
    let antre: i64 =
        sqlx::query_scalar("select count(*) from outbox where status = 'antre'").fetch_one(&db).await.unwrap();
    assert_eq!(antre, 1, "{out}");
    let tiruan = Router::new()
        .route("/oauth/accesstoken", post(|| async { Json(json!({ "access_token": "tkn", "expires_in": "3599" })) }))
        .route("/fhir/Patient", get(|| async { Json(json!({ "entry": [{ "resource": { "id": "P0001" } }] })) }))
        .route(
            "/fhir/BiologicallyDerivedProduct",
            post(|Json(r): Json<Value>| async move {
                assert_eq!(r["collection"]["source"]["reference"], "Patient/P0001");
                Json(json!({ "id": "BDP-1" }))
            }),
        )
        .route(
            "/fhir/BiologicallyDerivedProduct/{id}",
            put(|Json(r): Json<Value>| async move { Json(json!({ "id": r["id"] })) }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let alamat = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, tiruan).await.unwrap() });
    let mut pengirim = Pengirim::baru(SatuSehat {
        auth_url: format!("http://{alamat}/oauth"),
        fhir_url: format!("http://{alamat}/fhir"),
        client_id: "id".into(),
        client_secret: "rahasia".into(),
        org_id: "org".into(),
    });
    assert_eq!(pengirim.proses(&db).await.unwrap(), 1);
    let sid: Option<String> = sqlx::query_scalar("select satusehat_id from kantong where id = $1::uuid")
        .bind(&prc)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(sid.as_deref(), Some("BDP-1"));

    // Pemusnahan kantong yang sudah dilaporkan ikut dikirim.
    b.teknisi
        .ok(
            "/api/berita-acara",
            json!({ "jenis": "pemusnahan", "metode": "insinerator", "kantong": [{ "id": prc, "alasan": "rusak" }] }),
        )
        .await;
    assert_eq!(pengirim.proses(&db).await.unwrap(), 1);
}

#[sqlx::test]
async fn reaktif_mencekal_donor_dan_membuka_lookback(db: PgPool) {
    let mut b = Bdrs::siapkan(&db).await;
    b.kepala.kirim("PUT", "/api/master/profil", Some(json!({ "nama_rs": "RS Uji", "jenis": "utd_rs" }))).await;
    let (did, nid) = donasi_siap(&mut b, "9900000000000003", "D0002").await;
    b.teknisi
        .ok(
            &format!("/api/donasi/{nid}/olah"),
            json!({ "komponen": [{ "komponen": "wb", "volume_ml": 350, "alat_id": b.lemari }] }),
        )
        .await;
    let v = imltd(&mut b, &nid, "hcv", "reaktif").await;
    let lb = v["lookback_id"].as_str().expect("look-back dibuka").to_string();
    let (_, d) = b.teknisi.get(&format!("/api/donor/{did}")).await;
    assert!(d["cekal_aktif"].as_str().unwrap().starts_with("permanen"));
    // Donor dicekal tidak bisa diseleksi.
    let (st, _) = b.teknisi.post(&format!("/api/donor/{did}/seleksi"), seleksi(14.0, "diterima")).await;
    assert_eq!(st, StatusCode::CONFLICT);
    // Kantong reaktif tidak bisa diluluskan.
    let (_, det) = b.teknisi.get(&format!("/api/donasi/{nid}")).await;
    let wb = det["kantong"][0]["id"].as_str().unwrap();
    let (st, _) = b.kepala.post(&format!("/api/kantong/{wb}/luluskan"), json!({ "sandi": SANDI })).await;
    assert_eq!(st, StatusCode::CONFLICT);
    let (_, l) = b.teknisi.get(&format!("/api/lookback/{lb}")).await;
    assert_eq!(l["kantong"].as_array().unwrap().len(), 1);
    b.teknisi.ok(&format!("/api/lookback/{lb}/selesai"), json!({ "catatan": "Tidak ada donasi sebelumnya" })).await;
}

#[sqlx::test]
async fn lookback_dari_notifikasi_utd_mitra(db: PgPool) {
    let mut b = Bdrs::siapkan(&db).await;
    b.terima(&[("N-77", "prc", "A", "pos")]).await;
    let v = b
        .teknisi
        .ok(
            "/api/lookback",
            json!({ "nomor_donasi": "N-77", "alasan": "UTD: donor reaktif HBsAg pada donasi berikutnya" }),
        )
        .await;
    let (_, k) = b.teknisi.get("/api/kantong/lacak/N-77").await;
    assert_eq!(k["status"], "karantina");
    let (_, l) = b.teknisi.get(&format!("/api/lookback/{}", v["id"].as_str().unwrap())).await;
    assert_eq!(l["kantong"][0]["nomor"], "N-77");
}
