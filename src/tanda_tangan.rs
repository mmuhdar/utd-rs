//! Tanda tangan elektronik (ADR 0005): konfirmasi ulang kata sandi, terikat ke hash isi catatan.

use crate::auth::{Pengguna, sandi_cocok};
use crate::error::{Galat, Hasil, tolak};
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::PgConnection;
use uuid::Uuid;

pub fn hash_isi(isi: &Value) -> Vec<u8> {
    // serde_json mengurutkan kunci objek (tanpa fitur preserve_order), jadi hash stabil untuk isi yang sama.
    Sha256::digest(isi.to_string().as_bytes()).to_vec()
}

async fn simpan(conn: &mut PgConnection, akun: Uuid, makna: &str, objek: &str, id: &str, isi: &Value) -> Hasil<Uuid> {
    Ok(sqlx::query_scalar(
        "insert into tanda_tangan (akun_id, makna, objek, objek_id, hash_isi) values ($1, $2, $3, $4, $5) returning id",
    )
    .bind(akun)
    .bind(makna)
    .bind(objek)
    .bind(id)
    .bind(hash_isi(isi))
    .fetch_one(conn)
    .await?)
}

/// Pengguna yang sedang masuk menandatangani dengan sandinya sendiri.
pub async fn sendiri(
    conn: &mut PgConnection,
    p: &Pengguna,
    sandi: &str,
    makna: &str,
    objek: &str,
    id: impl ToString,
    isi: &Value,
) -> Hasil<Uuid> {
    let hash: String =
        sqlx::query_scalar("select sandi_hash from akun where id = $1").bind(p.akun_id).fetch_one(&mut *conn).await?;
    if !sandi_cocok(&hash, sandi) {
        return Err(tolak("sandi", "Kata sandi salah. Tanda tangan memerlukan kata sandi Anda sendiri."));
    }
    simpan(conn, p.akun_id, makna, objek, &id.to_string(), isi).await
}

/// Petugas kedua (berbeda dari pengguna yang masuk) menandatangani di perangkat yang sama dengan sandinya sendiri.
/// Mengembalikan akun petugas kedua.
#[allow(clippy::too_many_arguments)]
pub async fn petugas_lain(
    conn: &mut PgConnection,
    p: &Pengguna,
    nama_pengguna: &str,
    sandi: &str,
    hak: &str,
    makna: &str,
    objek: &str,
    id: impl ToString,
    isi: &Value,
) -> Hasil<Uuid> {
    let akun: Option<(Uuid, String, Vec<String>)> =
        sqlx::query_as("select id, sandi_hash, peran from akun where nama_pengguna = $1 and aktif")
            .bind(nama_pengguna.trim().to_lowercase())
            .fetch_optional(&mut *conn)
            .await?;
    let Some((akun_id, _, peran)) = akun.filter(|(_, h, _)| sandi_cocok(h, sandi)) else {
        return Err(tolak("petugas_2", "Nama pengguna atau kata sandi petugas kedua salah."));
    };
    if akun_id == p.akun_id {
        return Err(tolak("petugas_2", "Petugas kedua harus orang lain, bukan Anda sendiri."));
    }
    if !peran.iter().any(|r| crate::auth::hak_peran(r).contains(&hak)) {
        return Err(tolak("petugas_2", format!("Petugas kedua tidak memiliki hak '{hak}'.")));
    }
    simpan(conn, akun_id, makna, objek, &id.to_string(), isi).await
}

/// Verifikasi kedua oleh pengguna yang masuk: harus berbeda dari penginput.
#[allow(clippy::too_many_arguments)]
pub async fn verifikasi_kedua(
    conn: &mut PgConnection,
    p: &Pengguna,
    penginput: Uuid,
    sandi: &str,
    makna: &str,
    objek: &str,
    id: impl ToString,
    isi: &Value,
) -> Hasil<Uuid> {
    if penginput == p.akun_id {
        return Err(Galat::Konflik("Verifikasi kedua harus oleh petugas lain, bukan penginput hasil.".into()));
    }
    sendiri(conn, p, sandi, makna, objek, id, isi).await
}
