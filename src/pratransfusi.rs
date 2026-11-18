//! Uji pratransfusi dan alokasi (SPEK M3; FR-03..FR-07, Permenkes 91/2015 §5.7). Setiap hasil yang diinput manual
//! baru berlaku setelah verifikasi kedua oleh petugas lain (CPOB Aneks 2 butir 24, 27).

use crate::AppState;
use crate::audit::{self, Jejak};
use crate::auth::Pengguna;
use crate::error::{Galat, Hasil, tolak};
use crate::master::{teks, wajib_teks};
use crate::permintaan::golongan_terverifikasi;
use crate::stok::ubah_status;
use crate::tanda_tangan;
use axum::extract::{Path, State};
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::PgConnection;
use time::OffsetDateTime;
use uuid::Uuid;

pub fn rute() -> Router<AppState> {
    Router::new()
        .route("/sampel/{id}/golongan", post(periksa_golongan))
        .route("/golongan/{id}/verifikasi", post(verifikasi_golongan))
        .route("/uji-silang", post(uji_silang))
        .route("/uji-silang/{id}/verifikasi", post(verifikasi_uji_silang))
        .route("/permintaan/{id}/alokasi", post(alokasi_langsung))
        .route("/alokasi/{id}/lepas", post(lepas))
}

/// Kompatibilitas ABO/Rh kantong terhadap pasien. Komponen sel (WB, PRC): antigen sel donor harus dimiliki
/// penerima; WB harus identik. Komponen plasma (TC, FFP, kriopresipitat): antibodi plasma donor tidak boleh mengenai
/// sel penerima. Rh: pasien Rh-neg hanya menerima Rh-neg untuk WB/PRC/TC; plasma tidak memperhatikan Rh.
pub fn kompatibel(komponen: &str, abo_pasien: &str, rh_pasien: &str, abo_kantong: &str, rh_kantong: &str) -> bool {
    let antigen = |g: &str| (g.contains('A'), g.contains('B'));
    let (da, db) = antigen(abo_kantong);
    let (pa, pb) = antigen(abo_pasien);
    let abo = match komponen {
        "wb" => abo_kantong == abo_pasien,
        "prc" => (!da || pa) && (!db || pb),
        _ => (!pa || da) && (!pb || db),
    };
    let rh = matches!(komponen, "ffp" | "cryo") || rh_pasien == "pos" || rh_kantong == "neg";
    abo && rh
}

/// ABO dari cell grouping, dan apakah serum grouping konsisten dengannya.
pub fn tafsir_abo(anti_a: &str, anti_b: &str, sel_a: &str, sel_b: &str) -> (&'static str, bool) {
    let abo = match (anti_a == "pos", anti_b == "pos") {
        (true, true) => "AB",
        (true, false) => "A",
        (false, true) => "B",
        (false, false) => "O",
    };
    // Serum mengandung antibodi terhadap antigen yang tidak dimiliki sel.
    let cocok = (sel_a == "pos") == !abo.contains('A') && (sel_b == "pos") == !abo.contains('B');
    (abo, cocok)
}

/// Kesimpulan uji silang (§5.7.3 B no. 2 d).
pub fn kesimpulan(komponen: &str, mayor: Option<&str>, minor: Option<&str>, auto: Option<&str>) -> &'static str {
    let pos = |h: Option<&str>| h == Some("pos");
    if pos(mayor) {
        "tidak_serasi"
    } else if pos(minor) || pos(auto) {
        if komponen == "prc" { "hanya_prc" } else { "tidak_serasi" }
    } else {
        "serasi"
    }
}

fn valid_hasil(h: &str) -> bool {
    matches!(h, "pos" | "neg")
}

fn hasil(h: &Option<String>) -> Option<&str> {
    h.as_deref().filter(|h| valid_hasil(h))
}

// ---------- Golongan pasien ----------

#[derive(Deserialize)]
struct IsianGolongan {
    anti_a: String,
    anti_b: String,
    sel_a: String,
    sel_b: String,
    anti_d: String,
    metode: String,
}

/// Sampel yang masih layak pakai untuk pemeriksaan baru; mengembalikan (permintaan_id, no_rm).
async fn sampel_layak(conn: &mut PgConnection, sampel: Uuid) -> Hasil<(Uuid, String)> {
    let r: Option<(Uuid, String, String, OffsetDateTime, String)> = sqlx::query_as(
        "select s.permintaan_id, r.no_rm, s.status, s.batas_pakai, r.status from sampel s join permintaan r on r.id = s.permintaan_id
         where s.id = $1",
    )
    .bind(sampel)
    .fetch_optional(&mut *conn)
    .await?;
    let (permintaan, no_rm, status, batas, st_permintaan) =
        r.ok_or_else(|| Galat::TidakDitemukan("Sampel tidak ditemukan.".into()))?;
    if status != "diterima" {
        return Err(Galat::Konflik(format!("Sampel berstatus {status}; minta sampel baru.")));
    }
    if batas <= OffsetDateTime::now_utc() {
        return Err(Galat::Konflik("Sampel sudah melewati batas usia; minta sampel baru dari ruang.".into()));
    }
    if !matches!(st_permintaan.as_str(), "diminta" | "diproses" | "siap") {
        return Err(Galat::Konflik(format!("Permintaan berstatus {st_permintaan}.")));
    }
    Ok((permintaan, no_rm))
}

/// POST /api/sampel/{id}/golongan — ABO (cell & serum grouping) dan Rh pasien, pada setiap permintaan (§5.7.3 A).
async fn periksa_golongan(
    State(s): State<AppState>,
    p: Pengguna,
    Path(sampel): Path<Uuid>,
    Json(i): Json<IsianGolongan>,
) -> Hasil<Json<Value>> {
    p.wajib("uji.input")?;
    if ![&i.anti_a, &i.anti_b, &i.sel_a, &i.sel_b, &i.anti_d].iter().all(|h| valid_hasil(h)) {
        return Err(tolak("anti_a", "Isi semua hasil reaksi dengan positif atau negatif."));
    }
    if !matches!(i.metode.as_str(), "tabung" | "gel" | "bioplate") {
        return Err(tolak("metode", "Pilih metode: tabung, gel, atau bioplate."));
    }
    let (abo, cocok) = tafsir_abo(&i.anti_a, &i.anti_b, &i.sel_a, &i.sel_b);
    if !cocok {
        return Err(tolak(
            "sel_a",
            format!(
                "Cell grouping menunjukkan {abo} tetapi serum grouping tidak sesuai. Ulangi pemeriksaan; bila tetap, rujuk sampel ke UTD (§5.13)."
            ),
        ));
    }
    let rh = if i.anti_d == "pos" { "pos" } else { "neg" };
    let mut tx = s.db.begin().await?;
    let (_, no_rm) = sampel_layak(&mut tx, sampel).await?;
    let sebelumnya = golongan_terverifikasi(&mut tx, &no_rm).await?;
    let diskrepansi = sebelumnya.as_ref().is_some_and(|(a, r)| a != abo || r != rh);
    let v: Value = sqlx::query_scalar(
        "insert into pemeriksaan_golongan (sampel_id, no_rm, anti_a, anti_b, sel_a, sel_b, anti_d, abo, rhesus, metode, dicatat_oleh, diskrepansi)
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) returning to_jsonb(pemeriksaan_golongan)",
    )
    .bind(sampel)
    .bind(&no_rm)
    .bind(&i.anti_a)
    .bind(&i.anti_b)
    .bind(&i.sel_a)
    .bind(&i.sel_b)
    .bind(&i.anti_d)
    .bind(abo)
    .bind(rh)
    .bind(&i.metode)
    .bind(p.akun_id)
    .bind(diskrepansi)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| match Galat::from(e) {
        Galat::Konflik(_) => Galat::Konflik("Sampel ini sudah punya hasil golongan. Bila salah, minta verifikator menolaknya dulu.".into()),
        g => g,
    })?;
    if diskrepansi {
        sqlx::query("update sampel set status = 'diskrepansi', alasan_tolak = $2 where id = $1")
            .bind(sampel)
            .bind(format!("Golongan {abo}{rh} berbeda dari riwayat terverifikasi"))
            .execute(&mut *tx)
            .await?;
    }
    audit::ubah(&mut tx, &p, "input", "pemeriksaan_golongan", v["id"].as_str().unwrap_or_default(), v.clone()).await?;
    tx.commit().await?;
    let mut v = v;
    if let Some((a, r)) = sebelumnya.filter(|_| diskrepansi) {
        v["pesan"] = json!(format!(
            "Hasil berbeda dari golongan terverifikasi sebelumnya ({a} {r}). Sampel ditandai diskrepansi; minta sampel baru sebelum darah dikeluarkan."
        ));
    }
    Ok(Json(v))
}

#[derive(Deserialize)]
struct Verifikasi {
    setuju: bool,
    sandi: Option<String>,
    alasan: Option<String>,
}

/// Verifikasi kedua: setuju → ditandatangani; tidak setuju → hasil dibatalkan dan boleh diinput ulang.
async fn verifikasi(
    conn: &mut PgConnection,
    p: &Pengguna,
    tabel: &str,
    makna: &str,
    id: Uuid,
    v: &Verifikasi,
) -> Hasil<(bool, Value)> {
    p.wajib("uji.verifikasi")?;
    // Nama tabel hanya dari daftar tetap ini, tidak pernah dari masukan pengguna.
    let tabel = match tabel {
        "pemeriksaan_golongan" => "pemeriksaan_golongan",
        "uji_silang" => "uji_silang",
        "uji_imltd" => "uji_imltd",
        "golongan_donasi" => "golongan_donasi",
        _ => return Err(Galat::Internal(anyhow::anyhow!("tabel verifikasi tidak dikenal: {tabel}"))),
    };
    let r: Option<(Uuid, Option<Uuid>, Option<String>, Value)> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "select dicatat_oleh, tanda_tangan_id, batal_alasan, to_jsonb(t) - 'tanda_tangan_id' - 'batal_alasan' from {tabel} t
         where id = $1 for update"
    )))
    .bind(id)
    .fetch_optional(&mut *conn)
    .await?;
    let (penginput, ttd, batal, isi) = r.ok_or_else(|| Galat::TidakDitemukan("Hasil tidak ditemukan.".into()))?;
    if ttd.is_some() || batal.is_some() {
        return Err(Galat::Konflik("Hasil ini sudah diverifikasi atau dibatalkan.".into()));
    }
    let sandi = v.sandi.as_deref().unwrap_or("");
    if v.setuju {
        let ttd = tanda_tangan::verifikasi_kedua(conn, p, penginput, sandi, makna, tabel, id, &isi).await?;
        sqlx::query(sqlx::AssertSqlSafe(format!("update {tabel} set tanda_tangan_id = $2 where id = $1")))
            .bind(id)
            .bind(ttd)
            .execute(&mut *conn)
            .await?;
        audit::ubah(conn, p, "verifikasi", tabel, id, json!({ "tanda_tangan_id": ttd })).await?;
    } else {
        let alasan = wajib_teks(&v.alasan, "alasan", "Alasan penolakan hasil")?;
        tanda_tangan::verifikasi_kedua(conn, p, penginput, sandi, &format!("penolakan {makna}"), tabel, id, &isi)
            .await?;
        sqlx::query(sqlx::AssertSqlSafe(format!("update {tabel} set batal_alasan = $2 where id = $1")))
            .bind(id)
            .bind(alasan)
            .execute(&mut *conn)
            .await?;
        audit::catat(
            conn,
            Some(p),
            Jejak {
                aksi: "tolak_verifikasi",
                objek: tabel,
                objek_id: Some(id.to_string()),
                alasan: Some(alasan),
                ..Default::default()
            },
        )
        .await?;
    }
    Ok((v.setuju, isi))
}

async fn verifikasi_golongan(
    State(s): State<AppState>,
    p: Pengguna,
    Path(id): Path<Uuid>,
    Json(v): Json<Verifikasi>,
) -> Hasil<Json<Value>> {
    let mut tx = s.db.begin().await?;
    let (setuju, isi) =
        verifikasi(&mut tx, &p, "pemeriksaan_golongan", "verifikasi kedua golongan darah pasien", id, &v).await?;
    // Bila hasil diskrepansi ditolak verifikator, sampel boleh diperiksa ulang.
    if !setuju && isi["diskrepansi"] == true {
        sqlx::query("update sampel set status = 'diterima', alasan_tolak = null where id = $1::text::uuid")
            .bind(isi["sampel_id"].as_str().unwrap_or_default())
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(Json(json!({ "setuju": setuju })))
}

// ---------- Uji silang ----------

#[derive(Deserialize)]
struct IsianUjiSilang {
    permintaan_id: Uuid,
    sampel_id: Uuid,
    kantong_id: Option<Uuid>,
    nomor_kantong: Option<String>,
    metode: String,
    abo_kantong: String,
    rhesus_kantong: String,
    mayor: Option<String>,
    minor: Option<String>,
    autokontrol: Option<String>,
    ccc_valid: Option<bool>,
}

struct Kantong {
    id: Uuid,
    nomor: String,
    komponen: String,
    abo: String,
    rhesus: String,
    status: String,
    kedaluwarsa: OffsetDateTime,
}

async fn ambil_kantong(conn: &mut PgConnection, id: Option<Uuid>, nomor: Option<&str>) -> Hasil<Kantong> {
    let r: Option<(Uuid, String, String, String, String, String, OffsetDateTime)> = sqlx::query_as(
        "select id, nomor, komponen::text, abo::text, rhesus::text, status, kedaluwarsa from kantong
         where id = $1 or nomor = $2 for update",
    )
    .bind(id)
    .bind(nomor)
    .fetch_optional(conn)
    .await?;
    let (id, nomor, komponen, abo, rhesus, status, kedaluwarsa) =
        r.ok_or_else(|| tolak("nomor_kantong", "Kantong tidak ditemukan. Pindai ulang label kantong."))?;
    Ok(Kantong { id, nomor, komponen, abo, rhesus, status, kedaluwarsa })
}

/// Jumlah kantong komponen ini yang masih dibutuhkan permintaan (diminta − alokasi aktif/dikeluarkan).
async fn kebutuhan(conn: &mut PgConnection, permintaan: Uuid, komponen: &str) -> Hasil<i64> {
    Ok(sqlx::query_scalar(
        "select coalesce((select sum((b->>'jumlah')::int) from permintaan r, jsonb_array_elements(r.baris) b
                          where r.id = $1 and b->>'komponen' = $2), 0)
              - (select count(*) from alokasi a join kantong k on k.id = a.kantong_id
                 where a.permintaan_id = $1 and k.komponen = $2 and a.dilepas_pada is null)",
    )
    .bind(permintaan)
    .bind(komponen)
    .fetch_one(conn)
    .await?)
}

/// POST /api/uji-silang — konfirmasi golongan kantong dari selang dan uji silang serasi per kantong (§5.7.3).
async fn uji_silang(State(s): State<AppState>, p: Pengguna, Json(i): Json<IsianUjiSilang>) -> Hasil<Json<Value>> {
    p.wajib("uji.input")?;
    if !matches!(i.metode.as_str(), "tabung" | "gel" | "immediate_spin") {
        return Err(tolak("metode", "Pilih metode: tabung, gel, atau immediate spin."));
    }
    if !crate::stok::ABO.contains(&i.abo_kantong.as_str()) || !valid_hasil(&i.rhesus_kantong) {
        return Err(tolak("abo_kantong", "Isi hasil konfirmasi golongan ABO dan Rh kantong dari selang."));
    }
    let mut tx = s.db.begin().await?;
    let (permintaan, no_rm) = sampel_layak(&mut tx, i.sampel_id).await?;
    if permintaan != i.permintaan_id {
        return Err(tolak("sampel_id", "Sampel ini milik permintaan lain."));
    }
    let k = ambil_kantong(&mut tx, i.kantong_id, teks(&i.nomor_kantong)).await?;
    if k.status != "tersedia" {
        return Err(Galat::Konflik(format!(
            "Kantong {} berstatus {}; pilih kantong yang tersedia.",
            k.nomor, k.status
        )));
    }
    if k.kedaluwarsa <= OffsetDateTime::now_utc() {
        return Err(Galat::Konflik(format!("Kantong {} sudah kedaluwarsa.", k.nomor)));
    }
    if kebutuhan(&mut tx, permintaan, &k.komponen).await? <= 0 {
        return Err(Galat::Konflik(format!(
            "Permintaan ini tidak (lagi) membutuhkan komponen {}.",
            k.komponen.to_uppercase()
        )));
    }
    let aturan: String = sqlx::query_scalar("select uji_silang from aturan_komponen where komponen = $1")
        .bind(&k.komponen)
        .fetch_one(&mut *tx)
        .await?;
    match aturan.as_str() {
        "tidak_perlu" => {
            return Err(Galat::Konflik(format!(
                "Aturan BDRS: {} tidak memerlukan uji silang. Alokasikan langsung setelah golongan pasien terverifikasi.",
                k.komponen.to_uppercase()
            )));
        }
        "mayor_minor" if hasil(&i.mayor).is_none() || hasil(&i.minor).is_none() || hasil(&i.autokontrol).is_none() => {
            return Err(tolak("mayor", "Isi hasil mayor, minor, dan autokontrol."));
        }
        "minor" if hasil(&i.minor).is_none() => return Err(tolak("minor", "Isi hasil uji silang minor.")),
        _ => {}
    }
    let golongan = golongan_terverifikasi(&mut tx, &no_rm).await?;
    let (abo_p, rh_p) = golongan.ok_or_else(|| {
        Galat::Konflik(
            "Golongan darah pasien belum terverifikasi. Periksa dan verifikasi golongan pasien lebih dulu.".into(),
        )
    })?;
    let golongan_beda = i.abo_kantong != k.abo || i.rhesus_kantong != k.rhesus;
    if !golongan_beda && !kompatibel(&k.komponen, &abo_p, &rh_p, &k.abo, &k.rhesus) {
        return Err(Galat::Konflik(format!(
            "Kantong {} ({}{}) tidak kompatibel dengan golongan pasien {abo_p} {rh_p}.",
            k.nomor, k.abo, k.rhesus
        )));
    }
    let semua_negatif = [&i.mayor, &i.minor, &i.autokontrol].iter().all(|h| h.as_deref() != Some("pos"));
    if !golongan_beda && i.metode == "tabung" && semua_negatif && i.ccc_valid != Some(true) {
        return Err(tolak(
            "ccc_valid",
            "Uji tabung negatif harus dikonfirmasi Coombs Control Cell. Bila CCC tidak positif, uji tidak valid; ulangi.",
        ));
    }
    let hasil_uji = if golongan_beda {
        "golongan_beda"
    } else {
        kesimpulan(&k.komponen, i.mayor.as_deref(), i.minor.as_deref(), i.autokontrol.as_deref())
    };
    let v: Value = sqlx::query_scalar(
        "insert into uji_silang (permintaan_id, sampel_id, kantong_id, metode, abo_kantong, rhesus_kantong, mayor, minor, autokontrol,
                                 ccc_valid, kesimpulan, dicatat_oleh)
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) returning to_jsonb(uji_silang)",
    )
    .bind(permintaan)
    .bind(i.sampel_id)
    .bind(k.id)
    .bind(&i.metode)
    .bind(&i.abo_kantong)
    .bind(&i.rhesus_kantong)
    .bind(hasil(&i.mayor))
    .bind(hasil(&i.minor))
    .bind(hasil(&i.autokontrol))
    .bind(i.ccc_valid)
    .bind(hasil_uji)
    .bind(p.akun_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| match Galat::from(e) {
        Galat::Konflik(_) => Galat::Konflik("Kantong ini sudah diuji dengan sampel yang sama.".into()),
        g => g,
    })?;
    // Golongan kantong tidak sesuai label: karantina lalu dikembalikan ke UTD (§5.7.3, FR-04).
    if golongan_beda {
        let alasan = format!(
            "Golongan dari selang {}{} tidak sesuai label {}{}",
            i.abo_kantong, i.rhesus_kantong, k.abo, k.rhesus
        );
        ubah_status(&mut tx, &p, k.id, &["tersedia"], "karantina", Some(&alasan)).await?;
    }
    audit::ubah(&mut tx, &p, "input", "uji_silang", v["id"].as_str().unwrap_or_default(), v.clone()).await?;
    tx.commit().await?;
    Ok(Json(v))
}

/// Mengikat kantong ke permintaan dan memperbarui status permintaan bila semua kebutuhan sudah terpenuhi.
#[allow(clippy::too_many_arguments)]
async fn alokasikan(
    conn: &mut PgConnection,
    p: &Pengguna,
    permintaan: Uuid,
    kantong: Uuid,
    uji: Option<Uuid>,
    darurat: bool,
    alasan_darurat: Option<&str>,
    ttd: Option<Uuid>,
) -> Hasil<Uuid> {
    ubah_status(conn, p, kantong, &["tersedia"], "dialokasikan", None).await?;
    let id: Uuid = sqlx::query_scalar(
        "insert into alokasi (permintaan_id, kantong_id, uji_silang_id, darurat, alasan_darurat, disetujui_ttd, dialokasikan_oleh)
         values ($1, $2, $3, $4, $5, $6, $7) returning id",
    )
    .bind(permintaan)
    .bind(kantong)
    .bind(uji)
    .bind(darurat)
    .bind(alasan_darurat)
    .bind(ttd)
    .bind(p.akun_id)
    .fetch_one(&mut *conn)
    .await?;
    audit::ubah(conn, p, "alokasi", "kantong", kantong, json!({ "permintaan_id": permintaan, "darurat": darurat }))
        .await?;
    let kurang: i64 = sqlx::query_scalar(
        "select count(*) from permintaan r, jsonb_array_elements(r.baris) b where r.id = $1 and (b->>'jumlah')::int >
           (select count(*) from alokasi a join kantong k on k.id = a.kantong_id
            where a.permintaan_id = r.id and a.dilepas_pada is null and k.komponen = b->>'komponen')",
    )
    .bind(permintaan)
    .fetch_one(&mut *conn)
    .await?;
    sqlx::query("update permintaan set status = case when $2 then 'siap' else 'diproses' end where id = $1 and status in ('diminta', 'diproses', 'siap')")
        .bind(permintaan)
        .bind(kurang == 0)
        .execute(&mut *conn)
        .await?;
    Ok(id)
}

async fn verifikasi_uji_silang(
    State(s): State<AppState>,
    p: Pengguna,
    Path(id): Path<Uuid>,
    Json(v): Json<Verifikasi>,
) -> Hasil<Json<Value>> {
    let mut tx = s.db.begin().await?;
    let (setuju, isi) = verifikasi(&mut tx, &p, "uji_silang", "verifikasi kedua uji silang serasi", id, &v).await?;
    let mut alokasi = None;
    let mut pesan = None;
    if setuju && matches!(isi["kesimpulan"].as_str(), Some("serasi" | "hanya_prc")) {
        let permintaan: Uuid = serde_json::from_value(isi["permintaan_id"].clone()).map_err(anyhow::Error::from)?;
        let kantong: Uuid = serde_json::from_value(isi["kantong_id"].clone()).map_err(anyhow::Error::from)?;
        let k = ambil_kantong(&mut tx, Some(kantong), None).await?;
        if k.status != "tersedia" || k.kedaluwarsa <= OffsetDateTime::now_utc() {
            pesan =
                Some(format!("Kantong {} kini berstatus {} atau kedaluwarsa; tidak dialokasikan.", k.nomor, k.status));
        } else if kebutuhan(&mut tx, permintaan, &k.komponen).await? <= 0 {
            pesan = Some("Kebutuhan permintaan sudah terpenuhi; kantong tetap tersedia.".to_string());
        } else {
            alokasi = Some(alokasikan(&mut tx, &p, permintaan, kantong, Some(id), false, None, None).await?);
        }
    }
    tx.commit().await?;
    Ok(Json(json!({ "setuju": setuju, "alokasi_id": alokasi, "pesan": pesan })))
}

// ---------- Alokasi langsung dan darurat ----------

#[derive(Deserialize)]
struct IsianAlokasi {
    kantong_id: Option<Uuid>,
    nomor_kantong: Option<String>,
    darurat: Option<bool>,
    alasan_darurat: Option<String>,
    /// Dokter atau penanggung jawab yang menyetujui pengeluaran darurat, dengan sandinya sendiri.
    penyetuju: Option<String>,
    sandi_penyetuju: Option<String>,
}

/// POST /api/permintaan/{id}/alokasi — untuk komponen yang aturannya tidak memerlukan uji silang, atau pengeluaran
/// darurat tanpa uji silang/golongan terverifikasi. Darurat wajib disetujui dokter (§5.7.3 A no. 2) dan hanya
/// boleh O Rh-neg (sel) atau AB (plasma) bila golongan pasien belum terverifikasi.
async fn alokasi_langsung(
    State(s): State<AppState>,
    p: Pengguna,
    Path(permintaan): Path<Uuid>,
    Json(i): Json<IsianAlokasi>,
) -> Hasil<Json<Value>> {
    p.wajib("uji.input")?;
    let darurat = i.darurat.unwrap_or(false);
    let mut tx = s.db.begin().await?;
    let r: Option<(String, String, String)> =
        sqlx::query_as("select no_rm, jenis, status from permintaan where id = $1 for update")
            .bind(permintaan)
            .fetch_optional(&mut *tx)
            .await?;
    let (no_rm, jenis, status) = r.ok_or_else(|| Galat::TidakDitemukan("Permintaan darah tidak ditemukan.".into()))?;
    if !matches!(status.as_str(), "diminta" | "diproses" | "siap") {
        return Err(Galat::Konflik(format!("Permintaan berstatus {status}.")));
    }
    let k = ambil_kantong(&mut tx, i.kantong_id, teks(&i.nomor_kantong)).await?;
    if k.status != "tersedia" || k.kedaluwarsa <= OffsetDateTime::now_utc() {
        return Err(Galat::Konflik(format!("Kantong {} berstatus {} atau sudah kedaluwarsa.", k.nomor, k.status)));
    }
    if kebutuhan(&mut tx, permintaan, &k.komponen).await? <= 0 {
        return Err(Galat::Konflik(format!(
            "Permintaan ini tidak (lagi) membutuhkan komponen {}.",
            k.komponen.to_uppercase()
        )));
    }
    let aturan: String = sqlx::query_scalar("select uji_silang from aturan_komponen where komponen = $1")
        .bind(&k.komponen)
        .fetch_one(&mut *tx)
        .await?;
    let golongan = golongan_terverifikasi(&mut tx, &no_rm).await?;
    let cocok = golongan.as_ref().is_some_and(|(a, r)| kompatibel(&k.komponen, a, r, &k.abo, &k.rhesus));
    let butuh_darurat = aturan != "tidak_perlu" || !cocok;
    if !butuh_darurat {
        if darurat {
            return Err(tolak("darurat", "Kantong ini bisa dialokasikan biasa; tidak perlu jalur darurat."));
        }
        let id = alokasikan(&mut tx, &p, permintaan, k.id, None, false, None, None).await?;
        tx.commit().await?;
        return Ok(Json(json!({ "alokasi_id": id })));
    }
    if !darurat {
        return Err(Galat::Konflik(match golongan {
            None => "Golongan darah pasien belum terverifikasi.".into(),
            Some(_) if !cocok => format!("Kantong {}{} tidak kompatibel dengan golongan pasien.", k.abo, k.rhesus),
            Some(_) => {
                format!("Komponen {} memerlukan uji silang serasi sebelum dialokasikan.", k.komponen.to_uppercase())
            }
        }));
    }
    if jenis != "darurat" {
        return Err(Galat::Konflik("Pengeluaran tanpa uji silang hanya untuk permintaan berjenis darurat.".into()));
    }
    let alasan = wajib_teks(&i.alasan_darurat, "alasan_darurat", "Alasan klinis pengeluaran darurat")?;
    match &golongan {
        None => {
            let universal = match k.komponen.as_str() {
                "ffp" | "cryo" => k.abo == "AB",
                "tc" => k.abo == "AB" && k.rhesus == "neg",
                _ => k.abo == "O" && k.rhesus == "neg",
            };
            if !universal {
                return Err(Galat::Konflik(
                    "Golongan pasien belum terverifikasi: hanya PRC/WB golongan O Rh-negatif atau plasma golongan AB yang boleh dikeluarkan.".into(),
                ));
            }
        }
        Some((a, _)) => {
            // ABO harus tetap kompatibel; yang boleh dilonggarkan hanya Rh (pasien Rh-neg tanpa stok Rh-neg).
            if !kompatibel(&k.komponen, a, "pos", &k.abo, &k.rhesus) {
                return Err(Galat::Konflik(
                    "Golongan ABO kantong tidak kompatibel dengan pasien; jalur darurat tidak mengizinkannya.".into(),
                ));
            }
        }
    }
    let penyetuju = wajib_teks(&i.penyetuju, "penyetuju", "Nama pengguna dokter penyetuju")?;
    let isi = json!({ "permintaan_id": permintaan, "kantong": k.nomor, "alasan": alasan });
    let ttd = tanda_tangan::petugas_lain(
        &mut tx,
        &p,
        penyetuju,
        i.sandi_penyetuju.as_deref().unwrap_or(""),
        "darurat.setujui",
        "persetujuan pengeluaran darah darurat",
        "permintaan",
        permintaan,
        &isi,
    )
    .await?;
    let id = alokasikan(&mut tx, &p, permintaan, k.id, None, true, Some(alasan), Some(ttd)).await?;
    tx.commit().await?;
    Ok(Json(json!({ "alokasi_id": id, "darurat": true })))
}

/// Melepas alokasi yang belum dikeluarkan (mis. transfusi ditunda > 3×24 jam, §5.6.1 no. 5).
async fn lepas(
    State(s): State<AppState>,
    p: Pengguna,
    Path(id): Path<Uuid>,
    Json(a): Json<crate::permintaan::Alasan>,
) -> Hasil<Json<Value>> {
    p.wajib("permintaan.proses")?;
    let alasan = wajib_teks(&a.alasan, "alasan", "Alasan pelepasan")?;
    let mut tx = s.db.begin().await?;
    let r: Option<(Uuid, Uuid)> = sqlx::query_as(
        "update alokasi a set dilepas_pada = now(), alasan_lepas = $2 where id = $1 and dilepas_pada is null
           and not exists (select 1 from pengeluaran g where g.alokasi_id = a.id)
         returning kantong_id, permintaan_id",
    )
    .bind(id)
    .bind(alasan)
    .fetch_optional(&mut *tx)
    .await?;
    let (kantong, permintaan) = r.ok_or_else(|| {
        Galat::Konflik("Alokasi tidak ditemukan, sudah dilepas, atau kantong sudah dikeluarkan.".into())
    })?;
    ubah_status(&mut tx, &p, kantong, &["dialokasikan"], "tersedia", Some(alasan)).await?;
    sqlx::query("update permintaan set status = 'diproses' where id = $1 and status = 'siap'")
        .bind(permintaan)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Json(json!({})))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aturan_kompatibilitas() {
        // PRC: O untuk semua, AB hanya untuk AB.
        for p in ["A", "B", "AB", "O"] {
            assert!(kompatibel("prc", p, "pos", "O", "pos"));
        }
        assert!(!kompatibel("prc", "A", "pos", "AB", "pos"));
        assert!(kompatibel("prc", "AB", "pos", "A", "pos"));
        assert!(!kompatibel("prc", "O", "pos", "A", "pos"));
        // WB harus identik.
        assert!(!kompatibel("wb", "A", "pos", "O", "pos"));
        assert!(kompatibel("wb", "A", "pos", "A", "neg"));
        // Plasma: AB untuk semua, O hanya untuk O.
        for p in ["A", "B", "AB", "O"] {
            assert!(kompatibel("ffp", p, "neg", "AB", "pos"));
        }
        assert!(!kompatibel("ffp", "A", "pos", "O", "pos"));
        assert!(kompatibel("ffp", "O", "pos", "A", "pos"));
        // Rh: pasien Rh-neg tidak menerima PRC/TC Rh-pos; plasma tidak peduli Rh.
        assert!(!kompatibel("prc", "O", "neg", "O", "pos"));
        assert!(!kompatibel("tc", "A", "neg", "A", "pos"));
        assert!(kompatibel("ffp", "A", "neg", "A", "pos"));
        assert!(kompatibel("prc", "O", "pos", "O", "neg"));
    }

    #[test]
    fn tafsir_cell_dan_serum_grouping() {
        assert_eq!(tafsir_abo("pos", "neg", "neg", "pos"), ("A", true));
        assert_eq!(tafsir_abo("neg", "pos", "pos", "neg"), ("B", true));
        assert_eq!(tafsir_abo("pos", "pos", "neg", "neg"), ("AB", true));
        assert_eq!(tafsir_abo("neg", "neg", "pos", "pos"), ("O", true));
        assert_eq!(tafsir_abo("pos", "neg", "pos", "pos"), ("A", false));
    }

    #[test]
    fn kesimpulan_uji_silang() {
        assert_eq!(kesimpulan("prc", Some("neg"), Some("neg"), Some("neg")), "serasi");
        assert_eq!(kesimpulan("prc", Some("pos"), Some("neg"), Some("neg")), "tidak_serasi");
        assert_eq!(kesimpulan("prc", Some("neg"), Some("pos"), Some("neg")), "hanya_prc");
        assert_eq!(kesimpulan("wb", Some("neg"), Some("neg"), Some("pos")), "tidak_serasi");
        assert_eq!(kesimpulan("ffp", None, Some("neg"), None), "serasi");
    }
}
