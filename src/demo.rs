//! `utdrs seed-demo`: data fiktif untuk mencoba. Semua data dibuat lewat API aplikasi sendiri, sehingga aturan
//! domain (verifikasi kedua, kompatibilitas, FEFO, batas usia sampel) berlaku dan layar tidak menampilkan keadaan
//! yang mustahil. NIK demo berawalan 99 (bukan kode provinsi mana pun); semua akun bersandi `demo-utd-rs`.

use crate::{AppState, auth, router};
use axum::body::{Body, to_bytes};
use axum::http::{Request, header};
use serde_json::{Value, json};
use time::{Duration, OffsetDateTime};
use tower::ServiceExt;

pub const SANDI_DEMO: &str = "demo-utd-rs";

struct Klien {
    app: axum::Router,
    cookie: String,
}

impl Klien {
    async fn masuk(app: &axum::Router, nama: &str) -> anyhow::Result<Self> {
        let mut k = Klien { app: app.clone(), cookie: String::new() };
        k.kirim("POST", "/api/auth/masuk", Some(json!({ "nama_pengguna": nama, "sandi": SANDI_DEMO }))).await?;
        Ok(k)
    }

    async fn kirim(&mut self, metode: &str, uri: &str, body: Option<Value>) -> anyhow::Result<Value> {
        let mut req = Request::builder().method(metode).uri(uri).header(header::COOKIE, &self.cookie);
        if body.is_some() {
            req = req.header(header::CONTENT_TYPE, "application/json");
        }
        let req = req.body(body.map(|b| Body::from(b.to_string())).unwrap_or_default())?;
        let res = self.app.clone().oneshot(req).await?;
        if let Some(c) = res.headers().get(header::SET_COOKIE) {
            self.cookie = c.to_str()?.split(';').next().unwrap_or_default().to_string();
        }
        let st = res.status();
        let v: Value = serde_json::from_slice(&to_bytes(res.into_body(), 10 << 20).await?).unwrap_or(Value::Null);
        if !st.is_success() {
            anyhow::bail!("{metode} {uri}: {st} {v}");
        }
        Ok(v)
    }

    async fn post(&mut self, uri: &str, body: Value) -> anyhow::Result<Value> {
        self.kirim("POST", uri, Some(body)).await
    }

    async fn get(&mut self, uri: &str) -> anyhow::Result<Value> {
        self.kirim("GET", uri, None).await
    }
}

fn waktu(menit: i64) -> String {
    (OffsetDateTime::now_utc() + Duration::minutes(menit))
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
}

fn id(v: &Value) -> String {
    v["id"].as_str().unwrap_or_default().to_string()
}

fn golongan(abo: &str, rh: &str) -> Value {
    let p = |b: bool| if b { "pos" } else { "neg" };
    json!({ "anti_a": p(abo.contains('A')), "anti_b": p(abo.contains('B')), "sel_a": p(!abo.contains('A')), "sel_b": p(!abo.contains('B')),
            "anti_d": rh, "metode": "gel" })
}

pub async fn jalankan(state: AppState) -> anyhow::Result<()> {
    let db = state.db.clone();
    let ada: bool =
        sqlx::query_scalar("select exists (select 1 from akun where nama_pengguna = 'kepala')").fetch_one(&db).await?;
    if ada {
        anyhow::bail!("Data demo sudah ada (akun 'kepala'). Pakai database kosong.");
    }
    let akun = [
        ("admin", "Admin TI Demo", "admin"),
        ("kepala", "dr. Kepala BDRS Demo", "kepala"),
        ("teknisi", "Teknisi Satu Demo", "teknisi"),
        ("teknisi2", "Teknisi Dua Demo", "teknisi"),
        ("dokter", "dr. Peminta Demo", "dokter"),
        ("perawat", "Ners Satu Demo", "perawat"),
        ("perawat2", "Ners Dua Demo", "perawat"),
        ("auditor", "Auditor Mutu Demo", "auditor"),
    ];
    let hash = auth::hash_sandi(SANDI_DEMO)?;
    for (u, n, r) in akun {
        sqlx::query("insert into akun (nama_pengguna, nama, peran, sandi_hash, wajib_ganti_sandi) values ($1, $2, array[$3], $4, false)")
            .bind(u)
            .bind(n)
            .bind(r)
            .bind(&hash)
            .execute(&db)
            .await?;
    }
    let app = router(state);
    let mut kepala = Klien::masuk(&app, "kepala").await?;
    let mut tek = Klien::masuk(&app, "teknisi").await?;
    let mut tek2 = Klien::masuk(&app, "teknisi2").await?;
    let mut dokter = Klien::masuk(&app, "dokter").await?;
    let mut perawat = Klien::masuk(&app, "perawat").await?;

    kepala
        .kirim(
            "PUT",
            "/api/master/profil",
            Some(json!({ "nama_rs": "RS Demo Sejahtera (fiktif)", "kode_faskes": "9900001", "jenis": "utd_rs" })),
        )
        .await?;
    let utd = kepala
        .post(
            "/api/master/utd",
            json!({ "kode": "UTD-DEMO", "nama": "UTD Kota Demo (fiktif)", "format_label": "isbt128" }),
        )
        .await?;
    let alat = |kode: &str, nama: &str, jenis: &str, min: f64, maks: f64| json!({ "kode": kode, "nama": nama, "jenis": jenis, "suhu_min": min, "suhu_maks": maks, "nomor_seri": format!("SN-{kode}"), "lokasi": "Ruang BDRS" });
    let l1 = kepala.post("/api/master/alat", alat("LD-01", "Lemari darah 1", "lemari", 2.0, 6.0)).await?;
    kepala.post("/api/master/alat", alat("LD-02", "Lemari darah 2", "lemari", 2.0, 6.0)).await?;
    let f1 = kepala.post("/api/master/alat", alat("FP-01", "Freezer plasma", "freezer", -40.0, -25.0)).await?;
    let i1 = kepala.post("/api/master/alat", alat("IT-01", "Inkubator trombosit", "inkubator", 20.0, 24.0)).await?;
    for (a, t) in [(&l1, 4.1), (&f1, -32.0), (&i1, 22.3)] {
        tek.post("/api/suhu", json!({ "alat_id": id(a), "suhu": t })).await?;
    }

    // Penerimaan dari UTD: campuran komponen dan golongan, kedaluwarsa bervariasi untuk FEFO.
    let mut kantong = Vec::new();
    let mut n = 0;
    for (komponen, gol, jumlah, hari) in [
        ("prc", ("O", "pos"), 6, 30),
        ("prc", ("A", "pos"), 5, 25),
        ("prc", ("B", "pos"), 4, 2),
        ("prc", ("AB", "pos"), 2, 28),
        ("prc", ("O", "neg"), 2, 20),
        ("wb", ("O", "pos"), 2, 21),
        ("tc", ("A", "pos"), 2, 3),
        ("tc", ("O", "pos"), 2, 1),
        ("ffp", ("AB", "pos"), 3, 300),
        ("ffp", ("O", "pos"), 3, 280),
        ("cryo", ("A", "pos"), 2, 250),
    ] {
        for _ in 0..jumlah {
            n += 1;
            let alat_id = match komponen {
                "tc" => id(&i1),
                "ffp" | "cryo" => id(&f1),
                _ => id(&l1),
            };
            kantong.push(json!({ "nomor": format!("G99992620{n:04}"), "nomor_donasi": format!("G99992620{n:04}"), "komponen": komponen,
                "abo": gol.0, "rhesus": gol.1, "volume_ml": if komponen == "tc" { 50 } else { 250 },
                "kedaluwarsa": waktu(hari * 24 * 60), "imltd_nonreaktif": true, "diterima": true, "alat_id": alat_id }));
        }
    }
    kantong.push(json!({ "nomor": "G999926209999", "komponen": "prc", "abo": "A", "rhesus": "pos", "kedaluwarsa": waktu(20 * 24 * 60),
        "imltd_nonreaktif": true, "diterima": false, "alasan_tolak": "bocor" }));
    tek.post("/api/penerimaan", json!({ "utd_id": id(&utd), "nomor_dokumen_utd": "SPD/DEMO/001", "suhu_min_transport": 3.2, "suhu_maks_transport": 7.8,
        "petugas_utd": "Petugas UTD Demo", "kantong": kantong })).await?;

    // Pasien fiktif.
    let pasien = [
        ("DEMO-0001", "Siti Aminah (fiktif)", "1975-03-12", "P", "A", "pos"),
        ("DEMO-0002", "Budi Santoso (fiktif)", "1968-11-02", "L", "O", "pos"),
        ("DEMO-0003", "Rina Wulandari (fiktif)", "1990-07-21", "P", "B", "pos"),
        ("DEMO-0004", "Agus Pratama (fiktif)", "2001-01-30", "L", "O", "neg"),
    ];
    let minta = |no_rm: &str, ruang: &str, jenis: &str, baris: Value| {
        let p = pasien.iter().find(|p| p.0 == no_rm).copied().unwrap_or(pasien[0]);
        json!({ "pasien": { "no_rm": p.0, "nama": p.1, "tanggal_lahir": p.2, "jenis_kelamin": p.3 }, "ruang": ruang,
            "diagnosis": "Anemia pada perdarahan (demo)", "indikasi": "Hb < 7 g/dL dengan gejala", "jenis": jenis,
            "alasan_darurat": if jenis == "darurat" { json!("Perdarahan aktif, syok") } else { Value::Null },
            "hb": 6.4, "riwayat_transfusi": false, "riwayat_reaksi": false,
            "rencana_transfusi": OffsetDateTime::now_utc().to_offset(time::macros::offset!(+7)).date().to_string(), "baris": baris })
    };
    let stok = tek.get("/api/stok").await?;
    let cari = |komponen: &str, abo: &str, rh: &str, lewati: usize| -> String {
        stok.as_array()
            .into_iter()
            .flatten()
            .filter(|k| k["komponen"] == komponen && k["abo"] == abo && k["rhesus"] == rh)
            .nth(lewati)
            .map(|k| k["nomor"].as_str().unwrap_or_default().to_string())
            .unwrap_or_default()
    };

    // Siapkan permintaan sampai golongan terverifikasi; mengembalikan (permintaan, sampel).
    async fn siapkan(tek: &mut Klien, tek2: &mut Klien, pid: &str, abo: &str, rh: &str) -> anyhow::Result<String> {
        let sp = tek
            .post(
                &format!("/api/permintaan/{pid}/sampel"),
                json!({ "diambil_pada": waktu(-40), "diambil_oleh": "Ners Ruang Demo",
                "jenis": "edta", "penyimpanan": "suhu_ruang", "diterima": true }),
            )
            .await?;
        let g = tek.post(&format!("/api/sampel/{}/golongan", id(&sp)), golongan(abo, rh)).await?;
        tek2.post(&format!("/api/golongan/{}/verifikasi", id(&g)), json!({ "setuju": true, "sandi": SANDI_DEMO }))
            .await?;
        Ok(id(&sp))
    }
    async fn uji(
        tek: &mut Klien,
        tek2: &mut Klien,
        pid: &str,
        sid: &str,
        nomor: &str,
        abo: &str,
        rh: &str,
    ) -> anyhow::Result<Value> {
        let u = tek
            .post(
                "/api/uji-silang",
                json!({ "permintaan_id": pid, "sampel_id": sid, "nomor_kantong": nomor, "metode": "gel",
                "abo_kantong": abo, "rhesus_kantong": rh, "mayor": "neg", "minor": "neg", "autokontrol": "neg" }),
            )
            .await?;
        tek2.post(&format!("/api/uji-silang/{}/verifikasi", id(&u)), json!({ "setuju": true, "sandi": SANDI_DEMO }))
            .await
    }

    // 1. Permintaan baru menunggu sampel.
    dokter
        .post(
            "/api/permintaan",
            minta("DEMO-0003", "Rawat Inap Melati", "rutin", json!([{ "komponen": "prc", "jumlah": 2 }])),
        )
        .await?;
    // 2. Permintaan siap diambil (sudah diuji silang).
    let p2 = dokter
        .post(
            "/api/permintaan",
            minta("DEMO-0002", "Kamar Operasi", "elektif", json!([{ "komponen": "prc", "jumlah": 2 }])),
        )
        .await?;
    let s2 = siapkan(&mut tek, &mut tek2, &id(&p2), "O", "pos").await?;
    for i in 0..2 {
        uji(&mut tek, &mut tek2, &id(&p2), &s2, &cari("prc", "O", "pos", i), "O", "pos").await?;
    }
    // 3. Transfusi selesai dengan reaksi yang sedang ditelusuri.
    let p3 = dokter
        .post("/api/permintaan", minta("DEMO-0001", "IGD", "darurat", json!([{ "komponen": "prc", "jumlah": 1 }])))
        .await?;
    let s3 = siapkan(&mut tek, &mut tek2, &id(&p3), "A", "pos").await?;
    let nomor3 = cari("prc", "A", "pos", 0);
    let a3 = uji(&mut tek, &mut tek2, &id(&p3), &s3, &nomor3, "A", "pos").await?;
    tek.post(
        &format!("/api/alokasi/{}/keluarkan", a3["alokasi_id"].as_str().unwrap_or_default()),
        json!({ "penerima_ruang": "Ners IGD Demo" }),
    )
    .await?;
    let t3 = perawat
        .post(
            "/api/transfusi/verifikasi",
            json!({ "no_rm": "DEMO-0001", "tanggal_lahir": "1975-03-12", "nomor_kantong": nomor3,
            "petugas_2": "perawat2", "sandi_petugas_2": SANDI_DEMO }),
        )
        .await?;
    for tahap in ["pra", "15_menit"] {
        perawat
            .post(
                &format!("/api/transfusi/{}/observasi", id(&t3)),
                json!({ "tahap": tahap, "suhu": 36.9, "nadi": 92, "sistolik": 112, "diastolik": 72 }),
            )
            .await?;
    }
    perawat.post(&format!("/api/transfusi/{}/observasi", id(&t3)), json!({ "tahap": "berkala", "suhu": 38.4, "nadi": 110, "sistolik": 108, "diastolik": 70, "keluhan": "Menggigil" })).await?;
    perawat
        .post(
            &format!("/api/transfusi/{}/reaksi", id(&t3)),
            json!({ "gejala": ["demam", "menggigil"], "tindakan": "Transfusi dihentikan, NaCl 0,9%, lapor DPJP" }),
        )
        .await?;
    perawat
        .post(
            &format!("/api/transfusi/{}/selesai", id(&t3)),
            json!({ "volume_masuk_ml": 120, "dihentikan": true, "catatan": "Reaksi demam" }),
        )
        .await?;
    // 4. Transfusi trombosit sedang berjalan.
    let p4 = dokter
        .post("/api/permintaan", minta("DEMO-0001", "IGD", "rutin", json!([{ "komponen": "tc", "jumlah": 1 }])))
        .await?;
    let s4 = siapkan(&mut tek, &mut tek2, &id(&p4), "A", "pos").await?;
    let nomor4 = cari("tc", "A", "pos", 0);
    let u4 = tek
        .post(
            "/api/uji-silang",
            json!({ "permintaan_id": id(&p4), "sampel_id": s4, "nomor_kantong": nomor4, "metode": "gel",
            "abo_kantong": "A", "rhesus_kantong": "pos", "minor": "neg" }),
        )
        .await?;
    let a4 = tek2
        .post(&format!("/api/uji-silang/{}/verifikasi", id(&u4)), json!({ "setuju": true, "sandi": SANDI_DEMO }))
        .await?;
    tek.post(
        &format!("/api/alokasi/{}/keluarkan", a4["alokasi_id"].as_str().unwrap_or_default()),
        json!({ "penerima_ruang": "Ners IGD Demo" }),
    )
    .await?;
    perawat
        .post(
            "/api/transfusi/verifikasi",
            json!({ "no_rm": "DEMO-0001", "tanggal_lahir": "1975-03-12", "nomor_kantong": nomor4,
            "petugas_2": "perawat2", "sandi_petugas_2": SANDI_DEMO }),
        )
        .await?;
    // 5. Permintaan darurat pasien Rh-negatif menunggu.
    dokter
        .post("/api/permintaan", minta("DEMO-0004", "IGD", "darurat", json!([{ "komponen": "prc", "jumlah": 2 }])))
        .await?;

    // Mode UTD-RS: satu donor dengan donasi yang menunggu uji saring.
    let d = tek.post("/api/donor", json!({ "nik": "9901010101900001", "nama": "Andi Donor (fiktif)", "tanggal_lahir": "1992-04-05", "jenis_kelamin": "L",
        "alamat": "Jl. Demo No. 1", "telepon": "0800000000" })).await?;
    let sel = tek
        .post(
            &format!("/api/donor/{}/seleksi", id(&d)),
            json!({ "berat_kg": 68.0, "hb": 14.2, "sistolik": 120, "diastolik": 80,
        "nadi": 76, "suhu": 36.5, "kuesioner_lolos": true, "keputusan": "diterima" }),
        )
        .await?;
    let dn = tek
        .post(
            &format!("/api/seleksi/{}/donasi", id(&sel)),
            json!({ "nomor": "G999926300001", "jenis": "sukarela", "volume_ml": 350,
        "jenis_kantong": "triple" }),
        )
        .await?;
    let gd = tek.post(&format!("/api/donasi/{}/golongan", id(&dn)), golongan("B", "pos")).await?;
    tek2.post(&format!("/api/golongan-donasi/{}/verifikasi", id(&gd)), json!({ "setuju": true, "sandi": SANDI_DEMO }))
        .await?;
    tek.post(
        &format!("/api/donasi/{}/olah", id(&dn)),
        json!({ "komponen": [
        { "komponen": "prc", "volume_ml": 200, "alat_id": id(&l1) },
        { "komponen": "ffp", "volume_ml": 150, "alat_id": id(&f1) }] }),
    )
    .await?;
    for p in ["hiv", "hbsag"] {
        let u = tek
            .post(
                &format!("/api/donasi/{}/imltd", id(&dn)),
                json!({ "parameter": p, "metode": "chlia", "hasil": "nonreaktif" }),
            )
            .await?;
        tek2.post(&format!("/api/imltd/{}/verifikasi", id(&u)), json!({ "setuju": true, "sandi": SANDI_DEMO })).await?;
    }
    println!(
        "Data demo dibuat. Akun: admin, kepala, teknisi, teknisi2, dokter, perawat, perawat2, auditor — sandi {SANDI_DEMO}."
    );
    Ok(())
}
