//! Log suhu alat simpan dan penyimpangan suhu (Permenkes 91/2015 §3.10.1 E, §3.10.2; FR-13). Pembacaan di luar
//! rentang membuka penyimpangan dan mengarantina semua kantong di alat itu. Penanggung jawab BDRS menilai setiap
//! kantong (layak → tersedia lagi; tidak layak → tetap karantina untuk dikembalikan/dimusnahkan), lalu menutupnya.

use crate::AppState;
use crate::audit;
use crate::auth::Pengguna;
use crate::error::{Galat, Hasil, tolak};
use crate::master::{teks, wajib_teks};
use crate::stok::ubah_status;
use crate::tanda_tangan;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};
use time::OffsetDateTime;
use uuid::Uuid;

pub fn rute() -> Router<AppState> {
    Router::new()
        .route("/suhu", post(catat))
        .route("/suhu/{alat_id}", get(riwayat))
        .route("/penyimpangan-suhu", get(daftar))
        .route("/penyimpangan-suhu/{id}", get(detail))
        .route("/penyimpangan-suhu/{id}/nilai", post(nilai))
        .route("/penyimpangan-suhu/{id}/tutup", post(tutup))
}

#[derive(Deserialize)]
struct IsianSuhu {
    alat_id: Option<Uuid>,
    /// Data logger mengirim kode alat, bukan id.
    alat_kode: Option<String>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    waktu: Option<OffsetDateTime>,
    suhu: f64,
    catatan: Option<String>,
}

/// POST /api/suhu — pembacaan manual atau dari data logger (akun berperan `logger`).
async fn catat(State(s): State<AppState>, p: Pengguna, Json(i): Json<IsianSuhu>) -> Hasil<Json<Value>> {
    p.wajib("suhu.catat")?;
    let sekarang = OffsetDateTime::now_utc();
    let waktu = i.waktu.unwrap_or(sekarang);
    if waktu > sekarang + time::Duration::minutes(5) {
        return Err(tolak("waktu", "Waktu pembacaan tidak boleh di masa depan. Periksa jam alat."));
    }
    if !(-100.0..=60.0).contains(&i.suhu) {
        return Err(tolak("suhu", "Suhu di luar kemampuan ukur yang wajar; periksa sensor."));
    }
    let mut tx = s.db.begin().await?;
    let alat: Option<(Uuid, String, f64, f64)> = sqlx::query_as(
        "select id, kode, suhu_min::float8, suhu_maks::float8 from alat_simpan
         where aktif and (id = $1 or kode = $2) for update",
    )
    .bind(i.alat_id)
    .bind(teks(&i.alat_kode))
    .fetch_optional(&mut *tx)
    .await?;
    let (alat_id, kode, min, maks) = alat.ok_or_else(|| tolak("alat_id", "Alat simpan aktif tidak ditemukan."))?;
    let sumber = if p.peran.iter().any(|r| r == "logger") { "logger" } else { "manual" };
    sqlx::query("insert into log_suhu (alat_id, waktu, suhu, sumber, dicatat_oleh, catatan) values ($1, $2, $3::float8::numeric, $4, $5, $6)")
        .bind(alat_id)
        .bind(waktu)
        .bind(i.suhu)
        .bind(sumber)
        .bind(p.akun_id)
        .bind(teks(&i.catatan))
        .execute(&mut *tx)
        .await?;
    let menyimpang = i.suhu < min || i.suhu > maks;
    let mut penyimpangan: Option<Uuid> = None;
    if menyimpang {
        let terbuka: Option<(Uuid, f64)> = sqlx::query_as(
            "select id, suhu_ekstrem::float8 from penyimpangan_suhu where alat_id = $1 and status = 'terbuka'",
        )
        .bind(alat_id)
        .fetch_optional(&mut *tx)
        .await?;
        match terbuka {
            Some((id, ekstrem)) => {
                let tengah = (min + maks) / 2.0;
                if (i.suhu - tengah).abs() > (ekstrem - tengah).abs() {
                    sqlx::query("update penyimpangan_suhu set suhu_ekstrem = $2::float8::numeric where id = $1")
                        .bind(id)
                        .bind(i.suhu)
                        .execute(&mut *tx)
                        .await?;
                }
                penyimpangan = Some(id);
            }
            None => {
                let id: Uuid = sqlx::query_scalar(
                    "insert into penyimpangan_suhu (alat_id, mulai, suhu_ekstrem) values ($1, $2, $3::float8::numeric) returning id",
                )
                .bind(alat_id)
                .bind(waktu)
                .bind(i.suhu)
                .fetch_one(&mut *tx)
                .await?;
                let alasan = format!("Penyimpangan suhu alat {kode}: {:.1} °C (rentang {min:.1}–{maks:.1} °C)", i.suhu);
                let kantong: Vec<Uuid> = sqlx::query_scalar(
                    "select id from kantong where alat_id = $1 and status in ('karantina', 'tersedia', 'dialokasikan') for update",
                )
                .bind(alat_id)
                .fetch_all(&mut *tx)
                .await?;
                for k in kantong {
                    sqlx::query("insert into penilaian_kantong (penyimpangan_id, kantong_id) values ($1, $2)")
                        .bind(id)
                        .bind(k)
                        .execute(&mut *tx)
                        .await?;
                    sqlx::query("update alokasi set dilepas_pada = now(), alasan_lepas = $2 where kantong_id = $1 and dilepas_pada is null")
                        .bind(k)
                        .bind(&alasan)
                        .execute(&mut *tx)
                        .await?;
                    let st: String = sqlx::query_scalar("select status from kantong where id = $1")
                        .bind(k)
                        .fetch_one(&mut *tx)
                        .await?;
                    if st != "karantina" {
                        ubah_status(&mut tx, &p, k, &["tersedia", "dialokasikan"], "karantina", Some(&alasan)).await?;
                    }
                }
                audit::ubah(&mut tx, &p, "buka", "penyimpangan_suhu", id, json!({ "alat": kode, "suhu": i.suhu }))
                    .await?;
                penyimpangan = Some(id);
            }
        }
    }
    tx.commit().await?;
    Ok(Json(json!({ "menyimpang": menyimpang, "penyimpangan_id": penyimpangan })))
}

#[derive(Deserialize)]
struct Rentang {
    #[serde(default, with = "time::serde::rfc3339::option")]
    dari: Option<OffsetDateTime>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    sampai: Option<OffsetDateTime>,
}

/// GET /api/suhu/{alat_id} — bawaan 7 hari terakhir (setara grafik suhu 7 hari, §3.10.2).
async fn riwayat(
    State(s): State<AppState>,
    p: Pengguna,
    Path(alat): Path<Uuid>,
    Query(r): Query<Rentang>,
) -> Hasil<Json<Value>> {
    p.wajib("stok.lihat")?;
    let v: Value = sqlx::query_scalar(
        "select coalesce(jsonb_agg(to_jsonb(l) || jsonb_build_object('petugas', a.nama) order by l.waktu), '[]')
         from log_suhu l join akun a on a.id = l.dicatat_oleh
         where l.alat_id = $1 and l.waktu >= coalesce($2, now() - interval '7 days') and l.waktu <= coalesce($3, now())",
    )
    .bind(alat)
    .bind(r.dari)
    .bind(r.sampai)
    .fetch_one(&s.db)
    .await?;
    Ok(Json(v))
}

async fn daftar(State(s): State<AppState>, p: Pengguna) -> Hasil<Json<Value>> {
    p.wajib("stok.lihat")?;
    let v: Value = sqlx::query_scalar(
        "select coalesce(jsonb_agg(to_jsonb(ps) || jsonb_build_object('alat', a.kode,
                    'jumlah', (select count(*) from penilaian_kantong n where n.penyimpangan_id = ps.id),
                    'belum_dinilai', (select count(*) from penilaian_kantong n where n.penyimpangan_id = ps.id and n.keputusan is null))
                order by ps.status = 'terbuka' desc, ps.mulai desc), '[]')
         from (select * from penyimpangan_suhu order by mulai desc limit 200) ps join alat_simpan a on a.id = ps.alat_id",
    )
    .fetch_one(&s.db)
    .await?;
    Ok(Json(v))
}

async fn detail(State(s): State<AppState>, p: Pengguna, Path(id): Path<Uuid>) -> Hasil<Json<Value>> {
    p.wajib("stok.lihat")?;
    let v: Option<Value> = sqlx::query_scalar(
        "select to_jsonb(ps) || jsonb_build_object('alat', to_jsonb(a),
                'kantong', (select jsonb_agg(to_jsonb(n) || jsonb_build_object('nomor', k.nomor, 'komponen', k.komponen,
                    'abo', k.abo, 'rhesus', k.rhesus, 'status', k.status, 'kedaluwarsa', k.kedaluwarsa) order by k.nomor)
                    from penilaian_kantong n join kantong k on k.id = n.kantong_id where n.penyimpangan_id = ps.id))
         from penyimpangan_suhu ps join alat_simpan a on a.id = ps.alat_id where ps.id = $1",
    )
    .bind(id)
    .fetch_optional(&s.db)
    .await?;
    Ok(Json(v.ok_or_else(|| Galat::TidakDitemukan("Penyimpangan suhu tidak ditemukan.".into()))?))
}

#[derive(Deserialize)]
struct Penilaian {
    kantong_id: Uuid,
    keputusan: String,
    alasan: Option<String>,
}

async fn nilai(
    State(s): State<AppState>,
    p: Pengguna,
    Path(id): Path<Uuid>,
    Json(n): Json<Penilaian>,
) -> Hasil<Json<Value>> {
    p.wajib("karantina.lepas")?;
    let alasan = wajib_teks(&n.alasan, "alasan", "Dasar penilaian (lama dan besar penyimpangan, jenis komponen)")?;
    if !matches!(n.keputusan.as_str(), "layak" | "tidak_layak") {
        return Err(tolak("keputusan", "Pilih layak atau tidak layak."));
    }
    let mut tx = s.db.begin().await?;
    let r = sqlx::query(
        "update penilaian_kantong n set keputusan = $3, alasan = $4, dinilai_oleh = $5, dinilai_pada = now()
         from penyimpangan_suhu ps
         where n.penyimpangan_id = $1 and n.kantong_id = $2 and ps.id = n.penyimpangan_id and ps.status = 'terbuka'
           and n.keputusan is null",
    )
    .bind(id)
    .bind(n.kantong_id)
    .bind(&n.keputusan)
    .bind(alasan)
    .bind(p.akun_id)
    .execute(&mut *tx)
    .await?;
    if r.rows_affected() == 0 {
        return Err(Galat::Konflik(
            "Kantong ini tidak menunggu penilaian pada penyimpangan yang masih terbuka.".into(),
        ));
    }
    let keterangan = format!("Penilaian penyimpangan suhu: {alasan}");
    if n.keputusan == "layak" {
        let (imltd, asal): (bool, String) = sqlx::query_as("select imltd_nonreaktif, asal from kantong where id = $1")
            .bind(n.kantong_id)
            .fetch_one(&mut *tx)
            .await?;
        // Kantong donasi yang belum diluluskan tetap karantina sampai pelulusan.
        let belum_lulus: bool = asal == "donasi"
            && sqlx::query_scalar::<_, bool>(
                "select not exists (select 1 from tanda_tangan where objek = 'kantong' and objek_id = $1::text and makna = 'pelulusan')",
            )
            .bind(n.kantong_id)
            .fetch_one(&mut *tx)
            .await?;
        if imltd && !belum_lulus {
            ubah_status(&mut tx, &p, n.kantong_id, &["karantina"], "tersedia", Some(&keterangan)).await?;
        }
    } else {
        audit::ubah(&mut tx, &p, "nilai_tidak_layak", "kantong", n.kantong_id, json!({ "alasan": alasan })).await?;
    }
    tx.commit().await?;
    Ok(Json(json!({})))
}

#[derive(Deserialize)]
struct Penutupan {
    kesimpulan: Option<String>,
    sandi: Option<String>,
}

async fn tutup(
    State(s): State<AppState>,
    p: Pengguna,
    Path(id): Path<Uuid>,
    Json(t): Json<Penutupan>,
) -> Hasil<Json<Value>> {
    p.wajib("karantina.lepas")?;
    let kesimpulan = wajib_teks(&t.kesimpulan, "kesimpulan", "Kesimpulan dan tindakan perbaikan")?;
    let mut tx = s.db.begin().await?;
    let belum: i64 =
        sqlx::query_scalar("select count(*) from penilaian_kantong where penyimpangan_id = $1 and keputusan is null")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
    if belum > 0 {
        return Err(Galat::Konflik(format!("Masih ada {belum} kantong yang belum dinilai.")));
    }
    tanda_tangan::sendiri(
        &mut tx,
        &p,
        t.sandi.as_deref().unwrap_or(""),
        "penutupan penyimpangan suhu",
        "penyimpangan_suhu",
        id,
        &json!({ "kesimpulan": kesimpulan }),
    )
    .await?;
    let n = sqlx::query(
        "update penyimpangan_suhu set status = 'ditutup', kesimpulan = $2, ditutup_oleh = $3, ditutup_pada = now()
         where id = $1 and status = 'terbuka'",
    )
    .bind(id)
    .bind(kesimpulan)
    .bind(p.akun_id)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if n == 0 {
        return Err(Galat::Konflik("Penyimpangan tidak ditemukan atau sudah ditutup.".into()));
    }
    audit::ubah(&mut tx, &p, "tutup", "penyimpangan_suhu", id, json!({ "kesimpulan": kesimpulan })).await?;
    tx.commit().await?;
    Ok(Json(json!({})))
}
