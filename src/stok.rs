//! Penerimaan dan stok kantong (SPEK M2): permintaan ke UTD, berita serah terima, stok FEFO, pencairan, pemindahan,
//! karantina, berita acara pengembalian/pemusnahan, dan pelacakan satu kantong dari asal sampai akhir.

use crate::AppState;
use crate::audit::{self, Jejak};
use crate::auth::Pengguna;
use crate::error::{Galat, GalatIsian, Hasil, tolak};
use crate::master::{teks, wajib_teks};
use crate::tanda_tangan;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::PgConnection;
use time::OffsetDateTime;
use uuid::Uuid;

pub fn rute() -> Router<AppState> {
    Router::new()
        .route("/permintaan-utd", get(daftar_permintaan_utd).post(buat_permintaan_utd))
        .route("/permintaan-utd/{id}/status", post(status_permintaan_utd))
        .route("/penerimaan", get(daftar_penerimaan).post(terima))
        .route("/penerimaan/{id}", get(detail_penerimaan))
        .route("/stok", get(stok))
        .route("/stok/ringkasan", get(ringkasan))
        .route("/kantong/lacak/{nomor}", get(lacak))
        .route("/kantong/{id}/cairkan", post(cairkan))
        .route("/kantong/{id}/pindah", post(pindah))
        .route("/kantong/{id}/karantina", post(karantina))
        .route("/kantong/{id}/lepas-karantina", post(lepas_karantina))
        .route("/berita-acara", get(daftar_berita_acara).post(buat_berita_acara))
        .route("/berita-acara/{id}", get(detail_berita_acara))
}

pub const KOMPONEN: [&str; 5] = ["wb", "prc", "tc", "ffp", "cryo"];
pub const ABO: [&str; 4] = ["A", "B", "AB", "O"];

/// Mengubah status kantong bila status saat ini termasuk `dari`; dicatat di audit di transaksi yang sama.
pub async fn ubah_status(
    conn: &mut PgConnection,
    p: &Pengguna,
    id: Uuid,
    dari: &[&str],
    ke: &str,
    alasan: Option<&str>,
) -> Hasil<Value> {
    ubah_status_ba(conn, p, id, dari, ke, alasan, None).await
}

/// Seperti `ubah_status`, sekaligus mengikat kantong ke berita acara (status akhir wajib punya berita acara).
async fn ubah_status_ba(
    conn: &mut PgConnection,
    p: &Pengguna,
    id: Uuid,
    dari: &[&str],
    ke: &str,
    alasan: Option<&str>,
    ba: Option<Uuid>,
) -> Hasil<Value> {
    let k: Option<(String, String)> = sqlx::query_as("select status, nomor from kantong where id = $1 for update")
        .bind(id)
        .fetch_optional(&mut *conn)
        .await?;
    let (status, nomor) = k.ok_or_else(|| Galat::TidakDitemukan("Kantong tidak ditemukan.".into()))?;
    if !dari.contains(&status.as_str()) {
        return Err(Galat::Konflik(format!("Kantong {nomor} berstatus {status}; tindakan ini tidak bisa dilakukan.")));
    }
    let v: Value = sqlx::query_scalar(
        "update kantong k set status = $2, alasan_status = $3, berita_acara_id = coalesce($4, berita_acara_id),
                alat_id = case when $4 is null then alat_id end
         where id = $1 returning to_jsonb(k)",
    )
    .bind(id)
    .bind(ke)
    .bind(alasan)
    .bind(ba)
    .fetch_one(&mut *conn)
    .await?;
    audit::catat(
        conn,
        Some(p),
        Jejak {
            aksi: "status",
            objek: "kantong",
            objek_id: Some(id.to_string()),
            sebelum: Some(json!({ "status": status })),
            sesudah: Some(json!({ "status": ke })),
            alasan,
            ..Default::default()
        },
    )
    .await?;
    Ok(v)
}

// ---------- Permintaan ke UTD ----------

#[derive(Deserialize)]
struct IsianPermintaanUtd {
    utd_id: Uuid,
    jenis: String,
    baris: Vec<BarisPermintaanUtd>,
    catatan: Option<String>,
}

#[derive(Deserialize, serde::Serialize)]
struct BarisPermintaanUtd {
    komponen: String,
    abo: String,
    rhesus: String,
    jumlah: i32,
}

async fn buat_permintaan_utd(
    State(s): State<AppState>,
    p: Pengguna,
    Json(i): Json<IsianPermintaanUtd>,
) -> Hasil<Json<Value>> {
    p.wajib("stok.kelola")?;
    if !matches!(i.jenis.as_str(), "rutin" | "khusus" | "darurat") {
        return Err(tolak("jenis", "Pilih jenis permintaan: rutin, khusus, atau darurat."));
    }
    if i.baris.is_empty() {
        return Err(tolak("baris", "Isi minimal satu baris komponen."));
    }
    for b in &i.baris {
        if !KOMPONEN.contains(&b.komponen.as_str())
            || !ABO.contains(&b.abo.as_str())
            || !matches!(b.rhesus.as_str(), "pos" | "neg")
            || b.jumlah < 1
        {
            return Err(tolak("baris", "Setiap baris perlu komponen, golongan, rhesus, dan jumlah minimal 1."));
        }
    }
    if i.jenis == "darurat" && teks(&i.catatan).is_none() {
        return Err(tolak(
            "catatan",
            "Permintaan darurat ke UTD perlu keterangan alasan dan data pasien bila darah langka.",
        ));
    }
    let mut tx = s.db.begin().await?;
    let v: Value = sqlx::query_scalar(
        "insert into permintaan_utd (utd_id, jenis, baris, catatan, dibuat_oleh) values ($1, $2, $3, $4, $5) returning to_jsonb(permintaan_utd)",
    )
    .bind(i.utd_id)
    .bind(&i.jenis)
    .bind(serde_json::to_value(&i.baris).map_err(anyhow::Error::from)?)
    .bind(teks(&i.catatan))
    .bind(p.akun_id)
    .fetch_one(&mut *tx)
    .await?;
    audit::ubah(&mut tx, &p, "buat", "permintaan_utd", v["id"].as_str().unwrap_or_default(), v.clone()).await?;
    tx.commit().await?;
    Ok(Json(v))
}

async fn daftar_permintaan_utd(State(s): State<AppState>, p: Pengguna) -> Hasil<Json<Value>> {
    p.wajib("stok.lihat")?;
    let v: Value = sqlx::query_scalar(
        "select coalesce(jsonb_agg(to_jsonb(r) || jsonb_build_object('utd', u.nama) order by r.dibuat_pada desc), '[]')
         from (select * from permintaan_utd order by dibuat_pada desc limit 200) r join utd_mitra u on u.id = r.utd_id",
    )
    .fetch_one(&s.db)
    .await?;
    Ok(Json(v))
}

#[derive(Deserialize)]
struct UbahStatus {
    status: String,
}

async fn status_permintaan_utd(
    State(s): State<AppState>,
    p: Pengguna,
    Path(id): Path<Uuid>,
    Json(u): Json<UbahStatus>,
) -> Hasil<Json<Value>> {
    p.wajib("stok.kelola")?;
    if !matches!(u.status.as_str(), "dipenuhi" | "batal") {
        return Err(tolak("status", "Status hanya bisa diubah menjadi dipenuhi atau batal."));
    }
    let mut tx = s.db.begin().await?;
    let n = sqlx::query("update permintaan_utd set status = $2 where id = $1 and status = 'dikirim'")
        .bind(id)
        .bind(&u.status)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    if n == 0 {
        return Err(Galat::Konflik("Permintaan ke UTD tidak ditemukan atau sudah ditutup.".into()));
    }
    audit::ubah(&mut tx, &p, "status", "permintaan_utd", id, json!({ "status": u.status })).await?;
    tx.commit().await?;
    Ok(Json(json!({})))
}

// ---------- Penerimaan ----------

#[derive(Deserialize)]
struct IsianPenerimaan {
    utd_id: Uuid,
    permintaan_utd_id: Option<Uuid>,
    nomor_dokumen_utd: Option<String>,
    suhu_min_transport: Option<f64>,
    suhu_maks_transport: Option<f64>,
    petugas_utd: Option<String>,
    catatan: Option<String>,
    kantong: Vec<IsianKantong>,
}

#[derive(Deserialize)]
struct IsianKantong {
    nomor: Option<String>,
    nomor_donasi: Option<String>,
    komponen: String,
    kode_produk: Option<String>,
    abo: String,
    rhesus: String,
    volume_ml: Option<i32>,
    tanggal_ambil: Option<time::Date>,
    #[serde(with = "time::serde::rfc3339")]
    kedaluwarsa: OffsetDateTime,
    imltd_nonreaktif: bool,
    diterima: bool,
    alasan_tolak: Option<String>,
    alat_id: Option<Uuid>,
}

/// POST /api/penerimaan — berita serah terima. Kantong yang ditolak langsung masuk berita acara pengembalian
/// ke UTD yang sama, dengan alasannya (Permenkes 91/2015 §5.4.2, §5.10).
async fn terima(State(s): State<AppState>, p: Pengguna, Json(i): Json<IsianPenerimaan>) -> Hasil<Json<Value>> {
    p.wajib("stok.kelola")?;
    let petugas_utd = wajib_teks(&i.petugas_utd, "petugas_utd", "Nama petugas UTD yang menyerahkan")?;
    if i.kantong.is_empty() {
        return Err(tolak("kantong", "Pindai minimal satu kantong."));
    }
    let sekarang = OffsetDateTime::now_utc();
    let mut galat = Vec::new();
    for (n, k) in i.kantong.iter().enumerate() {
        let isian = |f: &str| format!("kantong.{n}.{f}");
        let nomor = teks(&k.nomor).unwrap_or("");
        if nomor.is_empty() {
            galat.push(GalatIsian { isian: isian("nomor"), pesan: "Nomor kantong wajib diisi.".into() });
        }
        if !KOMPONEN.contains(&k.komponen.as_str()) {
            galat.push(GalatIsian { isian: isian("komponen"), pesan: "Pilih komponen.".into() });
        }
        if !ABO.contains(&k.abo.as_str()) || !matches!(k.rhesus.as_str(), "pos" | "neg") {
            galat.push(GalatIsian { isian: isian("abo"), pesan: "Isi golongan ABO dan rhesus dari label.".into() });
        }
        if k.diterima {
            if !k.imltd_nonreaktif {
                galat.push(GalatIsian {
                    isian: isian("imltd_nonreaktif"),
                    pesan: "Kantong hanya boleh diterima bila hasil uji saring IMLTD dari UTD non-reaktif.".into(),
                });
            }
            if k.kedaluwarsa <= sekarang {
                galat.push(GalatIsian {
                    isian: isian("kedaluwarsa"),
                    pesan: "Kantong sudah kedaluwarsa; tolak dan kembalikan.".into(),
                });
            }
            if k.alat_id.is_none() {
                galat.push(GalatIsian { isian: isian("alat_id"), pesan: "Pilih alat simpan tujuan.".into() });
            }
        } else if teks(&k.alasan_tolak).is_none() {
            galat.push(GalatIsian { isian: isian("alasan_tolak"), pesan: "Isi alasan penolakan kantong.".into() });
        }
    }
    if !galat.is_empty() {
        return Err(Galat::Validasi(galat));
    }
    let mut tx = s.db.begin().await?;
    let pen: Value = sqlx::query_scalar(
        "insert into penerimaan (utd_id, permintaan_utd_id, nomor_dokumen_utd, suhu_min_transport, suhu_maks_transport,
                                 petugas_utd, catatan, diterima_oleh)
         values ($1, $2, $3, $4::float8::numeric, $5::float8::numeric, $6, $7, $8) returning to_jsonb(penerimaan)",
    )
    .bind(i.utd_id)
    .bind(i.permintaan_utd_id)
    .bind(teks(&i.nomor_dokumen_utd))
    .bind(i.suhu_min_transport)
    .bind(i.suhu_maks_transport)
    .bind(petugas_utd)
    .bind(teks(&i.catatan))
    .bind(p.akun_id)
    .fetch_one(&mut *tx)
    .await?;
    let pen_id: Uuid = serde_json::from_value(pen["id"].clone()).map_err(anyhow::Error::from)?;
    let ba_tolak: Option<Uuid> = if i.kantong.iter().any(|k| !k.diterima) {
        Some(
            sqlx::query_scalar(
                "insert into berita_acara (jenis, utd_id, catatan, dibuat_oleh) values ('pengembalian', $1, $2, $3) returning id",
            )
            .bind(i.utd_id)
            .bind(format!("Ditolak saat penerimaan {}", pen["nomor"].as_str().unwrap_or_default()))
            .bind(p.akun_id)
            .fetch_one(&mut *tx)
            .await?,
        )
    } else {
        None
    };
    for k in &i.kantong {
        if let Some(alat) = k.alat_id.filter(|_| k.diterima) {
            periksa_alat(&mut tx, alat, &k.komponen).await?;
        }
        let nomor = teks(&k.nomor).unwrap_or_default();
        let kantong: Value = sqlx::query_scalar(
            "insert into kantong (nomor, nomor_donasi, komponen, kode_produk, abo, rhesus, volume_ml, tanggal_ambil,
                                  kedaluwarsa, kedaluwarsa_label, asal, penerimaan_id, imltd_nonreaktif, status, alasan_status,
                                  alat_id, berita_acara_id)
             values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9, 'utd_mitra', $10, $11, $12, $13, $14, $15) returning to_jsonb(kantong)",
        )
        .bind(nomor)
        .bind(teks(&k.nomor_donasi).unwrap_or(nomor))
        .bind(&k.komponen)
        .bind(teks(&k.kode_produk))
        .bind(&k.abo)
        .bind(&k.rhesus)
        .bind(k.volume_ml)
        .bind(k.tanggal_ambil)
        .bind(k.kedaluwarsa)
        .bind(pen_id)
        .bind(k.imltd_nonreaktif)
        .bind(if k.diterima { "tersedia" } else { "dikembalikan" })
        .bind(if k.diterima { None } else { teks(&k.alasan_tolak) })
        .bind(k.alat_id.filter(|_| k.diterima))
        .bind(if k.diterima { None } else { ba_tolak })
        .fetch_one(&mut *tx)
        .await?;
        let kid = kantong["id"].as_str().unwrap_or_default().to_string();
        audit::ubah(&mut tx, &p, "terima", "kantong", kid, kantong).await?;
    }
    audit::ubah(&mut tx, &p, "buat", "penerimaan", pen_id, pen.clone()).await?;
    tx.commit().await?;
    Ok(Json(json!({ "id": pen_id, "nomor": pen["nomor"], "berita_acara_tolak": ba_tolak })))
}

/// Alat simpan harus aktif, tanpa penyimpangan suhu terbuka, dan berjenis sesuai komponen.
async fn periksa_alat(conn: &mut PgConnection, alat: Uuid, komponen: &str) -> Hasil<()> {
    let r: Option<(String, String, bool, bool)> = sqlx::query_as(
        "select a.kode, a.jenis, a.aktif, exists (select 1 from penyimpangan_suhu p where p.alat_id = a.id and p.status = 'terbuka')
         from alat_simpan a where a.id = $1",
    )
    .bind(alat)
    .fetch_optional(&mut *conn)
    .await?;
    let (kode, jenis, aktif, menyimpang) = r.ok_or_else(|| tolak("alat_id", "Alat simpan tidak ditemukan."))?;
    let perlu: String = sqlx::query_scalar("select jenis_alat from aturan_komponen where komponen = $1")
        .bind(komponen)
        .fetch_one(&mut *conn)
        .await?;
    if !aktif {
        return Err(tolak("alat_id", format!("Alat {kode} sudah tidak aktif.")));
    }
    if menyimpang {
        return Err(tolak("alat_id", format!("Alat {kode} sedang mengalami penyimpangan suhu; pilih alat lain.")));
    }
    if jenis != perlu {
        return Err(tolak(
            "alat_id",
            format!("Komponen {} harus disimpan di {perlu}, bukan di {jenis} ({kode}).", komponen.to_uppercase()),
        ));
    }
    Ok(())
}

async fn daftar_penerimaan(State(s): State<AppState>, p: Pengguna) -> Hasil<Json<Value>> {
    p.wajib("stok.lihat")?;
    let v: Value = sqlx::query_scalar(
        "select coalesce(jsonb_agg(to_jsonb(r) || jsonb_build_object('utd', u.nama,
                    'jumlah', (select count(*) from kantong k where k.penerimaan_id = r.id),
                    'ditolak', (select count(*) from kantong k where k.penerimaan_id = r.id and k.status = 'dikembalikan'
                                and k.berita_acara_id is not null and k.alasan_status is not null))
                order by r.diterima_pada desc), '[]')
         from (select * from penerimaan order by diterima_pada desc limit 200) r join utd_mitra u on u.id = r.utd_id",
    )
    .fetch_one(&s.db)
    .await?;
    Ok(Json(v))
}

async fn detail_penerimaan(State(s): State<AppState>, p: Pengguna, Path(id): Path<Uuid>) -> Hasil<Json<Value>> {
    p.wajib("stok.lihat")?;
    let v: Option<Value> = sqlx::query_scalar(
        "select to_jsonb(r) || jsonb_build_object('utd', to_jsonb(u), 'petugas_bdrs', a.nama,
                'kantong', (select jsonb_agg(to_jsonb(k) order by k.nomor) from kantong k where k.penerimaan_id = r.id))
         from penerimaan r join utd_mitra u on u.id = r.utd_id join akun a on a.id = r.diterima_oleh where r.id = $1",
    )
    .bind(id)
    .fetch_optional(&s.db)
    .await?;
    Ok(Json(v.ok_or_else(|| Galat::TidakDitemukan("Penerimaan tidak ditemukan.".into()))?))
}

// ---------- Stok ----------

#[derive(Deserialize)]
struct FilterStok {
    komponen: Option<String>,
    abo: Option<String>,
    rhesus: Option<String>,
    status: Option<String>,
}

/// GET /api/stok — kantong yang masih di BDRS, urut FEFO (kedaluwarsa terdekat lebih dulu).
async fn stok(State(s): State<AppState>, p: Pengguna, Query(f): Query<FilterStok>) -> Hasil<Json<Value>> {
    p.wajib("stok.lihat")?;
    let v: Value = sqlx::query_scalar(
        "select coalesce(jsonb_agg(to_jsonb(k) || jsonb_build_object(
                    'alat', a.kode, 'lewat_kedaluwarsa', k.kedaluwarsa <= now(),
                    'mendekati_kedaluwarsa', k.kedaluwarsa > now() and k.kedaluwarsa <= now() + make_interval(hours => r.jam_peringatan),
                    'permintaan', (select pm.nomor from alokasi al join permintaan pm on pm.id = al.permintaan_id
                                   where al.kantong_id = k.id and al.dilepas_pada is null))
                order by k.kedaluwarsa), '[]')
         from kantong k join aturan_komponen r on r.komponen = k.komponen left join alat_simpan a on a.id = k.alat_id
         where k.status = any(coalesce(string_to_array($4, ','), array['karantina', 'tersedia', 'dialokasikan']))
           and ($1::text is null or k.komponen = $1) and ($2::text is null or k.abo = $2) and ($3::text is null or k.rhesus = $3)",
    )
    .bind(teks(&f.komponen))
    .bind(teks(&f.abo))
    .bind(teks(&f.rhesus))
    .bind(teks(&f.status))
    .fetch_one(&s.db)
    .await?;
    Ok(Json(v))
}

/// GET /api/stok/ringkasan — jumlah kantong tersedia (belum kedaluwarsa) per komponen × golongan.
pub async fn ringkasan(State(s): State<AppState>, p: Pengguna) -> Hasil<Json<Value>> {
    p.wajib("stok.lihat")?;
    let v: Value = sqlx::query_scalar(
        "select coalesce(jsonb_agg(x order by x->>'komponen', x->>'abo', x->>'rhesus'), '[]') from (
           select jsonb_build_object('komponen', k.komponen, 'abo', k.abo, 'rhesus', k.rhesus,
                    'tersedia', count(*) filter (where k.status = 'tersedia' and k.kedaluwarsa > now()),
                    'dialokasikan', count(*) filter (where k.status = 'dialokasikan'),
                    'karantina', count(*) filter (where k.status = 'karantina'),
                    'mendekati_kedaluwarsa', count(*) filter (where k.status = 'tersedia' and k.kedaluwarsa > now()
                         and k.kedaluwarsa <= now() + make_interval(hours => r.jam_peringatan)),
                    'lewat_kedaluwarsa', count(*) filter (where k.kedaluwarsa <= now())) x
           from kantong k join aturan_komponen r on r.komponen = k.komponen
           where k.status in ('karantina', 'tersedia', 'dialokasikan')
           group by k.komponen, k.abo, k.rhesus) t",
    )
    .fetch_one(&s.db)
    .await?;
    Ok(Json(v))
}

#[derive(Deserialize)]
struct Alasan {
    alasan: Option<String>,
    alat_id: Option<Uuid>,
    sandi: Option<String>,
}

/// Pencairan FFP/kriopresipitat: kedaluwarsa dipersingkat sesuai aturan komponen (Permenkes 91/2015 §5.5.2 no. 4).
async fn cairkan(
    State(s): State<AppState>,
    p: Pengguna,
    Path(id): Path<Uuid>,
    Json(a): Json<Alasan>,
) -> Hasil<Json<Value>> {
    p.wajib("stok.kelola")?;
    let mut tx = s.db.begin().await?;
    let v: Option<Value> = sqlx::query_scalar(
        "update kantong k set dicairkan_pada = now(),
                kedaluwarsa = least(k.kedaluwarsa, now() + make_interval(hours => r.jam_setelah_cair)),
                alat_id = coalesce($2, k.alat_id)
         from aturan_komponen r
         where k.id = $1 and r.komponen = k.komponen and r.jam_setelah_cair is not null and k.dicairkan_pada is null
           and k.status in ('tersedia', 'dialokasikan') and k.kedaluwarsa > now()
         returning to_jsonb(k)",
    )
    .bind(id)
    .bind(a.alat_id)
    .fetch_optional(&mut *tx)
    .await?;
    let v = v.ok_or_else(|| {
        Galat::Konflik(
            "Hanya FFP atau kriopresipitat yang tersedia, belum kedaluwarsa, dan belum dicairkan yang bisa dicairkan."
                .into(),
        )
    })?;
    audit::ubah(
        &mut tx,
        &p,
        "cairkan",
        "kantong",
        id,
        json!({ "kedaluwarsa": v["kedaluwarsa"], "dicairkan_pada": v["dicairkan_pada"] }),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(v))
}

async fn pindah(
    State(s): State<AppState>,
    p: Pengguna,
    Path(id): Path<Uuid>,
    Json(a): Json<Alasan>,
) -> Hasil<Json<Value>> {
    p.wajib("stok.kelola")?;
    let alat = a.alat_id.ok_or_else(|| tolak("alat_id", "Pilih alat simpan tujuan."))?;
    let mut tx = s.db.begin().await?;
    let k: Option<(String, Option<Uuid>, String, Option<OffsetDateTime>)> =
        sqlx::query_as("select komponen, alat_id, status, dicairkan_pada from kantong where id = $1 for update")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
    let (komponen, lama, status, dicairkan) =
        k.ok_or_else(|| Galat::TidakDitemukan("Kantong tidak ditemukan.".into()))?;
    if !matches!(status.as_str(), "karantina" | "tersedia" | "dialokasikan") {
        return Err(Galat::Konflik(format!("Kantong berstatus {status} tidak lagi di BDRS.")));
    }
    // Plasma yang sudah dicairkan disimpan di lemari 2–6 °C, bukan di freezer.
    let komponen_alat = if dicairkan.is_some() { "prc" } else { komponen.as_str() };
    periksa_alat(&mut tx, alat, komponen_alat).await?;
    sqlx::query("update kantong set alat_id = $2 where id = $1").bind(id).bind(alat).execute(&mut *tx).await?;
    audit::catat(
        &mut tx,
        Some(&p),
        Jejak {
            aksi: "pindah",
            objek: "kantong",
            objek_id: Some(id.to_string()),
            sebelum: Some(json!({ "alat_id": lama })),
            sesudah: Some(json!({ "alat_id": alat })),
            ..Default::default()
        },
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({})))
}

async fn karantina(
    State(s): State<AppState>,
    p: Pengguna,
    Path(id): Path<Uuid>,
    Json(a): Json<Alasan>,
) -> Hasil<Json<Value>> {
    p.wajib("stok.kelola")?;
    let alasan = wajib_teks(&a.alasan, "alasan", "Alasan karantina")?;
    let mut tx = s.db.begin().await?;
    // Kantong teralokasi dilepas dulu dari permintaannya.
    sqlx::query(
        "update alokasi set dilepas_pada = now(), alasan_lepas = $2 where kantong_id = $1 and dilepas_pada is null",
    )
    .bind(id)
    .bind(format!("Dikarantina: {alasan}"))
    .execute(&mut *tx)
    .await?;
    let v = ubah_status(&mut tx, &p, id, &["tersedia", "dialokasikan"], "karantina", Some(alasan)).await?;
    tx.commit().await?;
    Ok(Json(v))
}

/// Melepas karantina hanya oleh penanggung jawab BDRS, dengan tanda tangan. Kantong hasil donasi sendiri
/// keluar dari karantina hanya lewat pelulusan.
async fn lepas_karantina(
    State(s): State<AppState>,
    p: Pengguna,
    Path(id): Path<Uuid>,
    Json(a): Json<Alasan>,
) -> Hasil<Json<Value>> {
    p.wajib("karantina.lepas")?;
    let alasan = wajib_teks(&a.alasan, "alasan", "Alasan pelepasan karantina")?;
    let mut tx = s.db.begin().await?;
    let k: Option<(String, bool, OffsetDateTime)> =
        sqlx::query_as("select asal, imltd_nonreaktif, kedaluwarsa from kantong where id = $1")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
    let (asal, imltd, kedaluwarsa) = k.ok_or_else(|| Galat::TidakDitemukan("Kantong tidak ditemukan.".into()))?;
    if asal == "donasi" {
        return Err(Galat::Konflik("Kantong hasil donasi sendiri keluar dari karantina lewat pelulusan.".into()));
    }
    let pernah_keluar: bool = sqlx::query_scalar("select exists (select 1 from pengeluaran where kantong_id = $1)")
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
    if pernah_keluar {
        return Err(Galat::Konflik(
            "Kantong yang pernah dikeluarkan ke ruang tidak boleh kembali ke stok; kembalikan ke UTD atau musnahkan."
                .into(),
        ));
    }
    if !imltd || kedaluwarsa <= OffsetDateTime::now_utc() {
        return Err(Galat::Konflik(
            "Kantong reaktif IMLTD atau kedaluwarsa tidak boleh dilepas dari karantina.".into(),
        ));
    }
    tanda_tangan::sendiri(
        &mut tx,
        &p,
        a.sandi.as_deref().unwrap_or(""),
        "pelepasan karantina",
        "kantong",
        id,
        &json!({ "alasan": alasan }),
    )
    .await?;
    let v = ubah_status(&mut tx, &p, id, &["karantina"], "tersedia", Some(alasan)).await?;
    tx.commit().await?;
    Ok(Json(v))
}

// ---------- Berita acara pengembalian/pemusnahan ----------

#[derive(Deserialize)]
struct IsianBeritaAcara {
    jenis: String,
    utd_id: Option<Uuid>,
    metode: Option<String>,
    saksi: Option<String>,
    catatan: Option<String>,
    kantong: Vec<KantongAlasan>,
}

#[derive(Deserialize)]
struct KantongAlasan {
    id: Uuid,
    alasan: String,
}

pub const ALASAN_AKHIR: [&str; 10] = [
    "kelebihan stok",
    "kedaluwarsa",
    "bocor",
    "selang putus",
    "rusak",
    "tidak sesuai permintaan",
    "salah label",
    "golongan tidak sesuai label",
    "penyimpangan suhu",
    "reaktif IMLTD",
];

/// POST /api/berita-acara — pengembalian ke UTD (Permenkes 91/2015 §5.10) atau pemusnahan (PP 28/2024 Pasal 297).
async fn buat_berita_acara(
    State(s): State<AppState>,
    p: Pengguna,
    Json(i): Json<IsianBeritaAcara>,
) -> Hasil<Json<Value>> {
    p.wajib("stok.kelola")?;
    match i.jenis.as_str() {
        "pengembalian" if i.utd_id.is_none() => return Err(tolak("utd_id", "Pilih UTD tujuan pengembalian.")),
        "pemusnahan" if teks(&i.metode).is_none() => {
            return Err(tolak("metode", "Isi metode pemusnahan (mis. insinerator limbah medis)."));
        }
        "pengembalian" | "pemusnahan" => {}
        _ => return Err(tolak("jenis", "Pilih pengembalian atau pemusnahan.")),
    }
    if i.kantong.is_empty() {
        return Err(tolak("kantong", "Pilih minimal satu kantong."));
    }
    if let Some(k) = i.kantong.iter().find(|k| !ALASAN_AKHIR.contains(&k.alasan.as_str())) {
        return Err(tolak("kantong", format!("Alasan '{}' tidak dikenal.", k.alasan)));
    }
    let mut tx = s.db.begin().await?;
    let ba: Value = sqlx::query_scalar(
        "insert into berita_acara (jenis, utd_id, metode, saksi, catatan, dibuat_oleh) values ($1, $2, $3, $4, $5, $6)
         returning to_jsonb(berita_acara)",
    )
    .bind(&i.jenis)
    .bind(if i.jenis == "pengembalian" { i.utd_id } else { None })
    .bind(teks(&i.metode))
    .bind(teks(&i.saksi))
    .bind(teks(&i.catatan))
    .bind(p.akun_id)
    .fetch_one(&mut *tx)
    .await?;
    let ba_id: Uuid = serde_json::from_value(ba["id"].clone()).map_err(anyhow::Error::from)?;
    let ke = if i.jenis == "pengembalian" { "dikembalikan" } else { "dimusnahkan" };
    for k in &i.kantong {
        // Kantong yang sudah keluar ke ruang tetapi tidak jadi ditransfusikan juga diakhiri di sini.
        let ditransfusi: bool = sqlx::query_scalar("select exists (select 1 from transfusi where kantong_id = $1)")
            .bind(k.id)
            .fetch_one(&mut *tx)
            .await?;
        if ditransfusi {
            return Err(Galat::Konflik(
                "Kantong yang sudah ditransfusikan tidak bisa dikembalikan atau dimusnahkan.".into(),
            ));
        }
        ubah_status_ba(&mut tx, &p, k.id, &["karantina", "tersedia", "dikeluarkan"], ke, Some(&k.alasan), Some(ba_id))
            .await?;
    }
    audit::ubah(&mut tx, &p, "buat", "berita_acara", ba_id, ba.clone()).await?;
    tx.commit().await?;
    Ok(Json(ba))
}

async fn daftar_berita_acara(State(s): State<AppState>, p: Pengguna) -> Hasil<Json<Value>> {
    p.wajib("stok.lihat")?;
    let v: Value = sqlx::query_scalar(
        "select coalesce(jsonb_agg(to_jsonb(b) || jsonb_build_object('utd', u.nama,
                    'jumlah', (select count(*) from kantong k where k.berita_acara_id = b.id)) order by b.dibuat_pada desc), '[]')
         from (select * from berita_acara order by dibuat_pada desc limit 200) b left join utd_mitra u on u.id = b.utd_id",
    )
    .fetch_one(&s.db)
    .await?;
    Ok(Json(v))
}

async fn detail_berita_acara(State(s): State<AppState>, p: Pengguna, Path(id): Path<Uuid>) -> Hasil<Json<Value>> {
    p.wajib("stok.lihat")?;
    let v: Option<Value> = sqlx::query_scalar(
        "select to_jsonb(b) || jsonb_build_object('utd', to_jsonb(u), 'petugas', a.nama,
                'kantong', (select jsonb_agg(jsonb_build_object('nomor', k.nomor, 'komponen', k.komponen, 'abo', k.abo,
                    'rhesus', k.rhesus, 'volume_ml', k.volume_ml, 'kedaluwarsa', k.kedaluwarsa, 'alasan', k.alasan_status)
                    order by k.nomor) from kantong k where k.berita_acara_id = b.id))
         from berita_acara b left join utd_mitra u on u.id = b.utd_id join akun a on a.id = b.dibuat_oleh where b.id = $1",
    )
    .bind(id)
    .fetch_optional(&s.db)
    .await?;
    Ok(Json(v.ok_or_else(|| Galat::TidakDitemukan("Berita acara tidak ditemukan.".into()))?))
}

// ---------- Pelacakan ----------

/// GET /api/kantong/lacak/{nomor} — ketertelusuran satu kantong (FR-18): asal, uji, alokasi, pengeluaran,
/// transfusi, reaksi, penyimpangan suhu, dan akhir.
async fn lacak(State(s): State<AppState>, p: Pengguna, Path(nomor): Path<String>) -> Hasil<Json<Value>> {
    p.wajib("stok.lihat")?;
    let mut tx = s.db.begin().await?;
    let v: Option<Value> = sqlx::query_scalar(
        "select to_jsonb(k) || jsonb_build_object(
            'alat', (select to_jsonb(a) from alat_simpan a where a.id = k.alat_id),
            'penerimaan', (select to_jsonb(r) || jsonb_build_object('utd', u.nama) from penerimaan r
                           join utd_mitra u on u.id = r.utd_id where r.id = k.penerimaan_id),
            'uji_silang', (select coalesce(jsonb_agg(to_jsonb(x) || jsonb_build_object('permintaan', pm.nomor, 'no_rm', pm.no_rm,
                              'petugas', a.nama, 'terverifikasi', x.tanda_tangan_id is not null) order by x.dicatat_pada), '[]')
                           from uji_silang x join permintaan pm on pm.id = x.permintaan_id join akun a on a.id = x.dicatat_oleh
                           where x.kantong_id = k.id),
            'alokasi', (select coalesce(jsonb_agg(to_jsonb(al) || jsonb_build_object('permintaan', pm.nomor, 'no_rm', pm.no_rm)
                           order by al.dialokasikan_pada), '[]')
                        from alokasi al join permintaan pm on pm.id = al.permintaan_id where al.kantong_id = k.id),
            'pengeluaran', (select to_jsonb(pg) || jsonb_build_object('petugas', a.nama) from pengeluaran pg
                            join akun a on a.id = pg.dikeluarkan_oleh where pg.kantong_id = k.id),
            'transfusi', (select to_jsonb(t) || jsonb_build_object('pasien', to_jsonb(ps)) from transfusi t
                          join pasien ps on ps.no_rm = t.no_rm where t.kantong_id = k.id),
            'reaksi', (select coalesce(jsonb_agg(to_jsonb(r)), '[]') from reaksi r join transfusi t on t.id = r.transfusi_id
                       where t.kantong_id = k.id),
            'penyimpangan_suhu', (select coalesce(jsonb_agg(to_jsonb(n) || jsonb_build_object('alat', a.kode)), '[]')
                                  from penilaian_kantong n join penyimpangan_suhu ps on ps.id = n.penyimpangan_id
                                  join alat_simpan a on a.id = ps.alat_id where n.kantong_id = k.id),
            'berita_acara', (select to_jsonb(b) from berita_acara b where b.id = k.berita_acara_id))
         from kantong k where k.nomor = $1",
    )
    .bind(nomor.trim())
    .fetch_optional(&mut *tx)
    .await?;
    let v = v.ok_or_else(|| Galat::TidakDitemukan(format!("Kantong {} tidak ditemukan.", nomor.trim())))?;
    // Membuka data penerima darah dicatat (UU PDP Pasal 31).
    if !v["transfusi"].is_null() {
        audit::catat(
            &mut tx,
            Some(&p),
            Jejak {
                aksi: "lihat",
                objek: "kantong",
                objek_id: v["id"].as_str().map(String::from),
                ..Default::default()
            },
        )
        .await?;
    }
    tx.commit().await?;
    Ok(Json(v))
}
