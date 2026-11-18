//! Jejak audit. Selalu ditulis di transaksi yang sama dengan perubahan datanya, sehingga perubahan tanpa jejak
//! tidak mungkin tersimpan. Tabelnya append-only dan berantai hash (migrasi 0001).

use crate::AppState;
use crate::auth::Pengguna;
use crate::error::Hasil;
use axum::Json;
use axum::extract::{Query, State};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgConnection;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Default)]
pub struct Jejak<'a> {
    pub aksi: &'a str,
    pub objek: &'a str,
    pub objek_id: Option<String>,
    pub sebelum: Option<Value>,
    pub sesudah: Option<Value>,
    pub alasan: Option<&'a str>,
    /// Dipakai bila belum ada pengguna (mis. percobaan masuk yang gagal).
    pub alamat: Option<String>,
    pub perangkat: Option<String>,
}

pub async fn catat(conn: &mut PgConnection, p: Option<&Pengguna>, j: Jejak<'_>) -> Result<(), sqlx::Error> {
    sqlx::query(
        "insert into audit (akun_id, aksi, objek, objek_id, sebelum, sesudah, alasan, alamat, perangkat)
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(p.map(|p| p.akun_id))
    .bind(j.aksi)
    .bind(j.objek)
    .bind(j.objek_id)
    .bind(j.sebelum)
    .bind(j.sesudah)
    .bind(j.alasan)
    .bind(p.and_then(|p| p.alamat.clone()).or(j.alamat))
    .bind(p.and_then(|p| p.perangkat.clone()).or(j.perangkat))
    .execute(conn)
    .await?;
    Ok(())
}

/// Singkatan untuk aksi oleh pengguna dengan objek dan isi sesudahnya.
pub async fn ubah(
    conn: &mut PgConnection,
    p: &Pengguna,
    aksi: &str,
    objek: &str,
    id: impl ToString,
    sesudah: Value,
) -> Result<(), sqlx::Error> {
    catat(
        conn,
        Some(p),
        Jejak { aksi, objek, objek_id: Some(id.to_string()), sesudah: Some(sesudah), ..Default::default() },
    )
    .await
}

#[derive(Deserialize)]
pub struct FilterAudit {
    objek: Option<String>,
    objek_id: Option<String>,
    akun_id: Option<Uuid>,
    sebelum_urutan: Option<i64>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct BarisAudit {
    urutan: i64,
    #[serde(with = "time::serde::rfc3339")]
    waktu: OffsetDateTime,
    akun_id: Option<Uuid>,
    nama: Option<String>,
    aksi: String,
    objek: String,
    objek_id: Option<String>,
    sebelum: Option<Value>,
    sesudah: Option<Value>,
    alasan: Option<String>,
    alamat: Option<String>,
}

/// GET /api/audit — terbaru lebih dulu, 100 per halaman.
pub async fn daftar(
    State(s): State<AppState>,
    p: Pengguna,
    Query(f): Query<FilterAudit>,
) -> Hasil<Json<Vec<BarisAudit>>> {
    p.wajib("audit.lihat")?;
    let baris = sqlx::query_as(
        "select a.urutan, a.waktu, a.akun_id, k.nama, a.aksi, a.objek, a.objek_id, a.sebelum, a.sesudah, a.alasan, a.alamat
         from audit a left join akun k on k.id = a.akun_id
         where ($1::text is null or a.objek = $1) and ($2::text is null or a.objek_id = $2)
           and ($3::uuid is null or a.akun_id = $3) and ($4::bigint is null or a.urutan < $4)
         order by a.urutan desc limit 100",
    )
    .bind(f.objek)
    .bind(f.objek_id)
    .bind(f.akun_id)
    .bind(f.sebelum_urutan)
    .fetch_all(&s.db)
    .await?;
    Ok(Json(baris))
}

/// GET /api/audit/periksa — memeriksa keutuhan rantai hash seluruh jejak audit.
pub async fn periksa(State(s): State<AppState>, p: Pengguna) -> Hasil<Json<Value>> {
    p.wajib("audit.lihat")?;
    let putus: Option<i64> = sqlx::query_scalar("select audit_periksa()").fetch_one(&s.db).await?;
    let jumlah: i64 = sqlx::query_scalar("select count(*) from audit").fetch_one(&s.db).await?;
    Ok(Json(serde_json::json!({ "utuh": putus.is_none(), "putus_di_urutan": putus, "jumlah": jumlah })))
}
