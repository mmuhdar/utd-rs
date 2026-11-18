//! Uji alur pasien ujung ke ujung (SPEK M3–M4): permintaan → sampel → golongan → uji silang → alokasi →
//! pengeluaran → verifikasi samping tempat tidur → transfusi → reaksi.

use crate::uji::{Bdrs, SANDI, Uji};
use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;

fn sekarang(menit: i64) -> String {
    (time::OffsetDateTime::now_utc() + time::Duration::minutes(menit))
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap()
}

fn hari_ini() -> String {
    time::OffsetDateTime::now_utc().to_offset(time::macros::offset!(+7)).date().to_string()
}

fn permintaan(no_rm: &str, jenis: &str, baris: Value) -> Value {
    json!({
        "pasien": { "no_rm": no_rm, "nama": "Pasien Uji", "tanggal_lahir": "1980-05-01", "jenis_kelamin": "P" },
        "ruang": "IGD", "diagnosis": "Perdarahan", "indikasi": "Hb 6", "jenis": jenis,
        "alasan_darurat": if jenis == "darurat" { json!("syok hemoragik") } else { Value::Null },
        "hb": 6.0, "riwayat_transfusi": false, "riwayat_reaksi": false, "rencana_transfusi": hari_ini(), "baris": baris
    })
}

/// Golongan reaksi untuk ABO/Rh tertentu.
fn reaksi_golongan(abo: &str, rh: &str) -> Value {
    let p = |b: bool| if b { "pos" } else { "neg" };
    json!({ "anti_a": p(abo.contains('A')), "anti_b": p(abo.contains('B')), "sel_a": p(!abo.contains('A')),
            "sel_b": p(!abo.contains('B')), "anti_d": rh, "metode": "gel" })
}

struct Alur {
    b: Bdrs,
    dokter: Uji,
    perawat: Uji,
    perawat2: Uji,
}

impl Alur {
    async fn siapkan(db: &PgPool) -> Self {
        let b = Bdrs::siapkan(db).await;
        let dokter = Uji::sebagai(db, "dokter", &["dokter"]).await;
        let perawat = Uji::sebagai(db, "perawat", &["perawat"]).await;
        let perawat2 = Uji::sebagai(db, "perawat2", &["perawat"]).await;
        Alur { b, dokter, perawat, perawat2 }
    }

    /// Permintaan + sampel diterima + golongan diverifikasi; mengembalikan (permintaan, sampel).
    async fn siap_uji(&mut self, no_rm: &str, abo: &str, rh: &str, baris: Value) -> (String, String) {
        let pm = self.dokter.ok("/api/permintaan", permintaan(no_rm, "rutin", baris)).await;
        let pid = pm["id"].as_str().unwrap().to_string();
        assert_eq!(pm["dokter"], "Uji dokter");
        let sp = self
            .b
            .teknisi
            .ok(
                &format!("/api/permintaan/{pid}/sampel"),
                json!({ "diambil_pada": sekarang(-30), "diambil_oleh": "Ners A",
                "jenis": "edta", "penyimpanan": "suhu_ruang", "diterima": true }),
            )
            .await;
        let sid = sp["id"].as_str().unwrap().to_string();
        let g = self.b.teknisi.ok(&format!("/api/sampel/{sid}/golongan"), reaksi_golongan(abo, rh)).await;
        assert_eq!(g["abo"], abo);
        self.b
            .teknisi2
            .ok(
                &format!("/api/golongan/{}/verifikasi", g["id"].as_str().unwrap()),
                json!({ "setuju": true, "sandi": SANDI }),
            )
            .await;
        (pid, sid)
    }
}

#[sqlx::test]
async fn alur_lengkap_sampai_reaksi(db: PgPool) {
    let mut a = Alur::siapkan(&db).await;
    let k =
        a.b.terima(&[("PRC-O1", "prc", "O", "pos"), ("PRC-B1", "prc", "B", "pos"), ("PRC-A1", "prc", "A", "pos")])
            .await;

    // Formulir tidak lengkap ditolak per isian.
    let mut kurang = permintaan("RM001", "rutin", json!([{ "komponen": "prc", "jumlah": 1 }]));
    kurang["indikasi"] = json!("");
    let (st, v) = a.dokter.post("/api/permintaan", kurang).await;
    assert_eq!(st, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(v["isian"][0]["isian"], "indikasi");

    let pm =
        a.dokter.ok("/api/permintaan", permintaan("RM001", "rutin", json!([{ "komponen": "prc", "jumlah": 1 }]))).await;
    let pid = pm["id"].as_str().unwrap();
    // Sampel lewat batas usia (EDTA suhu ruang > 24 jam) ditolak.
    let (st, _) =
        a.b.teknisi
            .post(
                &format!("/api/permintaan/{pid}/sampel"),
                json!({ "diambil_pada": sekarang(-25 * 60), "diambil_oleh": "Ners A",
            "jenis": "edta", "penyimpanan": "suhu_ruang", "diterima": true }),
            )
            .await;
    assert_eq!(st, StatusCode::UNPROCESSABLE_ENTITY);
    let sp =
        a.b.teknisi
            .ok(
                &format!("/api/permintaan/{pid}/sampel"),
                json!({ "diambil_pada": sekarang(-30), "diambil_oleh": "Ners A",
            "jenis": "edta", "penyimpanan": "suhu_ruang", "diterima": true }),
            )
            .await;
    let sid = sp["id"].as_str().unwrap();

    // Cell dan serum grouping tidak konsisten ditolak.
    let mut salah = reaksi_golongan("A", "pos");
    salah["sel_a"] = json!("pos");
    let (st, _) = a.b.teknisi.post(&format!("/api/sampel/{sid}/golongan"), salah).await;
    assert_eq!(st, StatusCode::UNPROCESSABLE_ENTITY);
    let g = a.b.teknisi.ok(&format!("/api/sampel/{sid}/golongan"), reaksi_golongan("A", "pos")).await;
    let gid = g["id"].as_str().unwrap();

    // Uji silang sebelum golongan terverifikasi ditolak.
    let uji = |nomor: &str| {
        json!({ "permintaan_id": pid, "sampel_id": sid, "nomor_kantong": nomor, "metode": "gel",
        "abo_kantong": nomor.split('-').nth(1).unwrap()[..1].to_string(), "rhesus_kantong": "pos",
        "mayor": "neg", "minor": "neg", "autokontrol": "neg" })
    };
    let (st, _) = a.b.teknisi.post("/api/uji-silang", uji("PRC-O1")).await;
    assert_eq!(st, StatusCode::CONFLICT);

    // Verifikasi kedua tidak boleh oleh penginput sendiri.
    let (st, _) =
        a.b.teknisi.post(&format!("/api/golongan/{gid}/verifikasi"), json!({ "setuju": true, "sandi": SANDI })).await;
    assert_eq!(st, StatusCode::CONFLICT);
    a.b.teknisi2.ok(&format!("/api/golongan/{gid}/verifikasi"), json!({ "setuju": true, "sandi": SANDI })).await;

    // Kantong B tidak kompatibel untuk pasien A.
    let (st, _) = a.b.teknisi.post("/api/uji-silang", uji("PRC-B1")).await;
    assert_eq!(st, StatusCode::CONFLICT);
    // Uji tabung negatif tanpa CCC tidak valid.
    let mut tabung = uji("PRC-A1");
    tabung["metode"] = json!("tabung");
    let (st, _) = a.b.teknisi.post("/api/uji-silang", tabung).await;
    assert_eq!(st, StatusCode::UNPROCESSABLE_ENTITY);

    let u = a.b.teknisi.ok("/api/uji-silang", uji("PRC-A1")).await;
    assert_eq!(u["kesimpulan"], "serasi");
    // Belum terverifikasi → belum dialokasikan.
    let (_, d) = a.b.teknisi.get(&format!("/api/permintaan/{pid}")).await;
    assert_eq!(d["alokasi"].as_array().unwrap().len(), 0);
    let v =
        a.b.teknisi2
            .ok(
                &format!("/api/uji-silang/{}/verifikasi", u["id"].as_str().unwrap()),
                json!({ "setuju": true, "sandi": SANDI }),
            )
            .await;
    let alokasi = v["alokasi_id"].as_str().unwrap().to_string();
    let (_, d) = a.b.teknisi.get(&format!("/api/permintaan/{pid}")).await;
    assert_eq!(d["status"], "siap");
    // Kebutuhan sudah terpenuhi: kantong lain tidak bisa diuji untuk permintaan ini.
    let (st, _) = a.b.teknisi.post("/api/uji-silang", uji("PRC-O1")).await;
    assert_eq!(st, StatusCode::CONFLICT);

    // Pengeluaran.
    let (st, _) = a.b.teknisi.post(&format!("/api/alokasi/{alokasi}/keluarkan"), json!({ "penerima_ruang": "" })).await;
    assert_eq!(st, StatusCode::UNPROCESSABLE_ENTITY);
    a.b.teknisi.ok(&format!("/api/alokasi/{alokasi}/keluarkan"), json!({ "penerima_ruang": "Ners B (IGD)" })).await;
    let (_, aktif) = a.perawat.get("/api/pengeluaran/aktif").await;
    assert_eq!(aktif[0]["terlambat"], false);

    // Verifikasi samping tempat tidur: pasien salah → KNC dan diblokir.
    a.dokter
        .ok(
            "/api/pasien",
            json!({ "no_rm": "RM999", "nama": "Orang Lain", "tanggal_lahir": "1990-01-01", "jenis_kelamin": "L" }),
        )
        .await;
    let samping = |rm: &str, petugas2: &str| {
        json!({ "no_rm": rm, "tanggal_lahir": "1980-05-01", "nomor_kantong": "PRC-A1",
        "petugas_2": petugas2, "sandi_petugas_2": SANDI })
    };
    let (st, v) = a.perawat.post("/api/transfusi/verifikasi", samping("RM999", "perawat2")).await;
    assert_eq!(st, StatusCode::CONFLICT);
    assert!(v["pesan"].as_str().unwrap().starts_with("JANGAN TRANSFUSIKAN"));
    let (_, ins) = a.b.kepala.get("/api/insiden").await;
    assert_eq!(ins[0]["jenis"], "knc");
    // Petugas kedua tidak boleh diri sendiri.
    let (st, _) = a.perawat.post("/api/transfusi/verifikasi", samping("RM001", "perawat")).await;
    assert_eq!(st, StatusCode::UNPROCESSABLE_ENTITY);
    let t = a.perawat.ok("/api/transfusi/verifikasi", samping("RM001", "perawat2")).await;
    let tid = t["id"].as_str().unwrap();

    // Observasi dan selesai.
    for tahap in ["pra", "15_menit", "selesai"] {
        a.perawat
            .ok(
                &format!("/api/transfusi/{tid}/observasi"),
                json!({ "tahap": tahap, "suhu": 36.8, "nadi": 88, "sistolik": 110, "diastolik": 70 }),
            )
            .await;
    }
    let (st, _) =
        a.perawat.post(&format!("/api/transfusi/{tid}/observasi"), json!({ "tahap": "pasca", "suhu": 36.8 })).await;
    assert_eq!(st, StatusCode::CONFLICT, "pasca sebelum selesai");
    a.perawat.ok(&format!("/api/transfusi/{tid}/selesai"), json!({ "volume_masuk_ml": 250 })).await;

    // Reaksi → penelusuran → penutupan oleh kepala.
    let r = a
        .perawat
        .ok(
            &format!("/api/transfusi/{tid}/reaksi"),
            json!({ "gejala": ["demam", "menggigil"], "tindakan": "Parasetamol" }),
        )
        .await;
    assert_eq!(r["kategori"], "akut");
    let rid = r["id"].as_str().unwrap();
    let (st, _) = a.b.kepala.post(&format!("/api/reaksi/{rid}/tutup"), json!({ "sandi": SANDI })).await;
    assert_eq!(st, StatusCode::CONFLICT, "belum ditelusuri");
    a.b.teknisi
        .ok(
            &format!("/api/reaksi/{rid}/telusur"),
            json!({ "abo_pasien_ulang": "A", "rhesus_pasien_ulang": "pos", "abo_kantong_ulang": "A",
            "rhesus_kantong_ulang": "pos", "uji_silang_ulang": "serasi", "kesimpulan": "Reaksi demam non-hemolitik" }),
        )
        .await;
    let (st, _) = a.b.teknisi.post(&format!("/api/reaksi/{rid}/tutup"), json!({ "sandi": SANDI })).await;
    assert_eq!(st, StatusCode::FORBIDDEN);
    a.b.kepala.ok(&format!("/api/reaksi/{rid}/tutup"), json!({ "sandi": SANDI })).await;
    let (_, rr) = a.b.kepala.get(&format!("/api/reaksi/{rid}")).await;
    assert_eq!(rr["status"], "ditutup");
    assert_eq!(rr["penutup"], "Uji kepala");
    assert_eq!(rr["uji_silang"][0]["kesimpulan"], "serasi");

    // Ketertelusuran kantong → pasien.
    let (_, l) = a.b.teknisi.get("/api/kantong/lacak/PRC-A1").await;
    assert_eq!(l["status"], "ditransfusikan");
    assert_eq!(l["transfusi"]["pasien"]["no_rm"], "RM001");
    assert_eq!(l["reaksi"].as_array().unwrap().len(), 1);
    let _ = k;
}

#[sqlx::test]
async fn diskrepansi_golongan_wajib_sampel_baru(db: PgPool) {
    let mut a = Alur::siapkan(&db).await;
    a.siap_uji("RM002", "O", "pos", json!([{ "komponen": "prc", "jumlah": 1 }])).await;
    let pm =
        a.dokter.ok("/api/permintaan", permintaan("RM002", "rutin", json!([{ "komponen": "prc", "jumlah": 1 }]))).await;
    let pid = pm["id"].as_str().unwrap();
    let sp =
        a.b.teknisi
            .ok(
                &format!("/api/permintaan/{pid}/sampel"),
                json!({ "diambil_pada": sekarang(-10), "diambil_oleh": "Ners A",
            "jenis": "edta", "penyimpanan": "dingin", "diterima": true }),
            )
            .await;
    let sid = sp["id"].as_str().unwrap();
    let g = a.b.teknisi.ok(&format!("/api/sampel/{sid}/golongan"), reaksi_golongan("A", "pos")).await;
    assert_eq!(g["diskrepansi"], true);
    assert!(g["pesan"].as_str().unwrap().contains("sampel baru"));
    // Sampel diskrepansi tidak bisa dipakai untuk uji apa pun.
    let (st, _) = a.b.teknisi.post(&format!("/api/sampel/{sid}/golongan"), reaksi_golongan("O", "pos")).await;
    assert_eq!(st, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn golongan_kantong_tidak_sesuai_label_dikarantina(db: PgPool) {
    let mut a = Alur::siapkan(&db).await;
    a.b.terima(&[("PRC-O9", "prc", "O", "pos")]).await;
    let (pid, sid) = a.siap_uji("RM003", "O", "pos", json!([{ "komponen": "prc", "jumlah": 1 }])).await;
    let u =
        a.b.teknisi
            .ok(
                "/api/uji-silang",
                json!({ "permintaan_id": pid, "sampel_id": sid, "nomor_kantong": "PRC-O9", "metode": "gel",
            "abo_kantong": "A", "rhesus_kantong": "pos", "mayor": "neg", "minor": "neg", "autokontrol": "neg" }),
            )
            .await;
    assert_eq!(u["kesimpulan"], "golongan_beda");
    let (_, l) = a.b.teknisi.get("/api/kantong/lacak/PRC-O9").await;
    assert_eq!(l["status"], "karantina");
}

#[sqlx::test]
async fn pengeluaran_darurat_tanpa_golongan(db: PgPool) {
    let mut a = Alur::siapkan(&db).await;
    a.b.terima(&[("ON-1", "prc", "O", "neg"), ("AP-1", "prc", "A", "pos")]).await;
    let pm = a
        .dokter
        .ok("/api/permintaan", permintaan("RM004", "darurat", json!([{ "komponen": "prc", "jumlah": 2 }])))
        .await;
    let pid = pm["id"].as_str().unwrap();
    let darurat = |nomor: &str, penyetuju: &str| {
        json!({ "nomor_kantong": nomor, "darurat": true, "alasan_darurat": "syok, tidak sempat uji",
        "penyetuju": penyetuju, "sandi_penyetuju": SANDI })
    };
    // Tanpa jalur darurat ditolak.
    let (st, _) = a.b.teknisi.post(&format!("/api/permintaan/{pid}/alokasi"), json!({ "nomor_kantong": "ON-1" })).await;
    assert_eq!(st, StatusCode::CONFLICT);
    // Golongan belum diketahui: hanya O Rh-neg.
    let (st, _) = a.b.teknisi.post(&format!("/api/permintaan/{pid}/alokasi"), darurat("AP-1", "dokter")).await;
    assert_eq!(st, StatusCode::CONFLICT);
    // Penyetuju harus berhak menyetujui darurat (perawat tidak).
    let (st, _) = a.b.teknisi.post(&format!("/api/permintaan/{pid}/alokasi"), darurat("ON-1", "perawat")).await;
    assert_eq!(st, StatusCode::UNPROCESSABLE_ENTITY);
    let v = a.b.teknisi.ok(&format!("/api/permintaan/{pid}/alokasi"), darurat("ON-1", "dokter")).await;
    assert_eq!(v["darurat"], true);
    // Alokasi darurat bisa langsung dikeluarkan.
    a.b.teknisi
        .ok(
            &format!("/api/alokasi/{}/keluarkan", v["alokasi_id"].as_str().unwrap()),
            json!({ "penerima_ruang": "Ners IGD" }),
        )
        .await;
    let _ = (&mut a.perawat, &mut a.perawat2);
}

#[sqlx::test]
async fn ffp_minor_dan_koreksi_observasi(db: PgPool) {
    let mut a = Alur::siapkan(&db).await;
    a.b.terima(&[("FFP-A1", "ffp", "A", "pos")]).await;
    let (pid, sid) = a.siap_uji("RM005", "O", "pos", json!([{ "komponen": "ffp", "jumlah": 1 }])).await;
    // Plasma A tidak kompatibel untuk pasien O? Kebalikannya: plasma A berisi anti-B, aman untuk O.
    let u =
        a.b.teknisi
            .ok(
                "/api/uji-silang",
                json!({ "permintaan_id": pid, "sampel_id": sid, "nomor_kantong": "FFP-A1", "metode": "gel",
            "abo_kantong": "A", "rhesus_kantong": "pos", "minor": "neg" }),
            )
            .await;
    assert_eq!(u["kesimpulan"], "serasi");
    // Kepala mengubah aturan FFP menjadi tidak perlu uji silang (dengan alasan); PRC tidak boleh.
    let (st, _) =
        a.b.kepala
            .kirim(
                "PUT",
                "/api/master/aturan-komponen/prc",
                Some(json!({ "uji_silang": "minor", "masa_simpan_jam": 840, "jam_peringatan": 72, "alasan": "x" })),
            )
            .await;
    assert_eq!(st, StatusCode::UNPROCESSABLE_ENTITY);
    let (st, _) = a
        .b
        .kepala
        .kirim("PUT", "/api/master/aturan-komponen/ffp", Some(json!({ "uji_silang": "tidak_perlu", "masa_simpan_jam": 8760, "jam_setelah_cair": 6, "jam_peringatan": 720, "alasan": "Keputusan komite transfusi" })))
        .await;
    assert_eq!(st, StatusCode::OK);
}
