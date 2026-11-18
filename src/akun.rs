//! Pengelolaan akun oleh admin. Akun tidak dihapus, hanya dinonaktifkan, supaya jejak audit tetap bisa dirujuk.

use crate::AppState;
use crate::audit;
use crate::auth::{Pengguna, hak_peran, hash_sandi, periksa_sandi};
use crate::error::{Galat, Hasil, tolak};
use axum::extract::{Path, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

pub fn rute() -> Router<AppState> {
    Router::new()
        .route("/akun", get(daftar).post(buat))
        .route("/akun/{id}", put(ubah))
        .route("/akun/{id}/reset-sandi", post(reset_sandi))
}

#[derive(Serialize, sqlx::FromRow)]
struct BarisAkun {
    id: Uuid,
    nama_pengguna: String,
    nama: String,
    nip: Option<String>,
    peran: Vec<String>,
    aktif: bool,
    terkunci: bool,
}

async fn daftar(State(s): State<AppState>, p: Pengguna) -> Hasil<Json<Vec<BarisAkun>>> {
    p.wajib("akun.kelola")?;
    let v = sqlx::query_as(
        "select id, nama_pengguna, nama, nip, peran, aktif, coalesce(terkunci_sampai > now(), false) as terkunci
         from akun order by aktif desc, nama",
    )
    .fetch_all(&s.db)
    .await?;
    Ok(Json(v))
}

#[derive(Deserialize)]
struct IsianAkun {
    nama_pengguna: Option<String>,
    nama: String,
    nip: Option<String>,
    peran: Vec<String>,
    aktif: Option<bool>,
    sandi_sementara: Option<String>,
}

fn periksa_peran(peran: &[String]) -> Hasil<()> {
    if peran.is_empty() || peran.iter().any(|r| hak_peran(r).is_empty()) {
        return Err(tolak("peran", "Pilih minimal satu peran yang dikenal."));
    }
    Ok(())
}

async fn buat(State(s): State<AppState>, p: Pengguna, Json(i): Json<IsianAkun>) -> Hasil<Json<serde_json::Value>> {
    p.wajib("akun.kelola")?;
    periksa_peran(&i.peran)?;
    let nama_pengguna = i.nama_pengguna.as_deref().unwrap_or("").trim().to_lowercase();
    let sandi = i.sandi_sementara.as_deref().unwrap_or("");
    periksa_sandi(sandi).map_err(|e| tolak("sandi_sementara", e))?;
    let mut tx = s.db.begin().await?;
    let id: Uuid = sqlx::query_scalar(
        "insert into akun (nama_pengguna, nama, nip, peran, sandi_hash) values ($1, $2, $3, $4, $5) returning id",
    )
    .bind(&nama_pengguna)
    .bind(i.nama.trim())
    .bind(i.nip.as_deref().map(str::trim).filter(|t| !t.is_empty()))
    .bind(&i.peran)
    .bind(hash_sandi(sandi)?)
    .fetch_one(&mut *tx)
    .await?;
    audit::ubah(
        &mut tx,
        &p,
        "buat",
        "akun",
        id,
        json!({ "nama_pengguna": nama_pengguna, "nama": i.nama, "peran": i.peran }),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({ "id": id })))
}

async fn ubah(
    State(s): State<AppState>,
    p: Pengguna,
    Path(id): Path<Uuid>,
    Json(i): Json<IsianAkun>,
) -> Hasil<Json<serde_json::Value>> {
    p.wajib("akun.kelola")?;
    periksa_peran(&i.peran)?;
    let aktif = i.aktif.unwrap_or(true);
    if id == p.akun_id && (!aktif || !i.peran.iter().any(|r| r == "admin")) {
        return Err(Galat::Konflik(
            "Anda tidak bisa mencabut peran admin atau menonaktifkan akun Anda sendiri.".into(),
        ));
    }
    let mut tx = s.db.begin().await?;
    let sebelum: Option<serde_json::Value> =
        sqlx::query_scalar("select jsonb_build_object('nama', nama, 'nip', nip, 'peran', peran, 'aktif', aktif) from akun where id = $1 for update")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
    let sebelum = sebelum.ok_or_else(|| Galat::TidakDitemukan("Akun tidak ditemukan.".into()))?;
    sqlx::query("update akun set nama = $2, nip = $3, peran = $4, aktif = $5 where id = $1")
        .bind(id)
        .bind(i.nama.trim())
        .bind(i.nip.as_deref().map(str::trim).filter(|t| !t.is_empty()))
        .bind(&i.peran)
        .bind(aktif)
        .execute(&mut *tx)
        .await?;
    if !aktif {
        sqlx::query("delete from sesi where akun_id = $1").bind(id).execute(&mut *tx).await?;
    }
    let sesudah = json!({ "nama": i.nama, "nip": i.nip, "peran": i.peran, "aktif": aktif });
    audit::catat(
        &mut tx,
        Some(&p),
        audit::Jejak {
            aksi: "ubah",
            objek: "akun",
            objek_id: Some(id.to_string()),
            sebelum: Some(sebelum),
            sesudah: Some(sesudah),
            ..Default::default()
        },
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({})))
}

#[derive(Deserialize)]
struct Reset {
    sandi_sementara: String,
}

/// Reset sandi oleh admin juga membuka kunci akun dan mengakhiri semua sesinya.
async fn reset_sandi(
    State(s): State<AppState>,
    p: Pengguna,
    Path(id): Path<Uuid>,
    Json(r): Json<Reset>,
) -> Hasil<Json<serde_json::Value>> {
    p.wajib("akun.kelola")?;
    periksa_sandi(&r.sandi_sementara).map_err(|e| tolak("sandi_sementara", e))?;
    let mut tx = s.db.begin().await?;
    let n = sqlx::query("update akun set sandi_hash = $2, wajib_ganti_sandi = true, gagal_masuk = 0, terkunci_sampai = null where id = $1")
        .bind(id)
        .bind(hash_sandi(&r.sandi_sementara)?)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    if n == 0 {
        return Err(Galat::TidakDitemukan("Akun tidak ditemukan.".into()));
    }
    sqlx::query("delete from sesi where akun_id = $1").bind(id).execute(&mut *tx).await?;
    audit::ubah(&mut tx, &p, "reset_sandi", "akun", id, json!({})).await?;
    tx.commit().await?;
    Ok(Json(json!({})))
}
