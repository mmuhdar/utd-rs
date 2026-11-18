//! Perintah admin: `utdrs buat-admin <nama_pengguna> "<Nama Lengkap>"` (sandi dari UTD_SANDI atau stdin),
//! `utdrs seed-demo` (data fiktif untuk mencoba). Migrasi selalu dijalankan sebelum perintah apa pun.

use crate::AppState;
use crate::auth::{hash_sandi, periksa_sandi};

pub async fn jalankan(state: AppState, args: &[String]) -> anyhow::Result<()> {
    let db = &state.db;
    match args[0].as_str() {
        "seed-demo" => crate::demo::jalankan(state.clone()).await?,
        "migrate" => println!("Migrasi selesai."),
        "buat-admin" => {
            let (Some(nama_pengguna), Some(nama)) = (args.get(1), args.get(2)) else {
                anyhow::bail!("pakai: utdrs buat-admin <nama_pengguna> \"<Nama Lengkap>\"");
            };
            let sandi = match std::env::var("UTD_SANDI") {
                Ok(s) => s,
                Err(_) => {
                    eprintln!("Kata sandi sementara (minimal 10 karakter):");
                    let mut s = String::new();
                    std::io::stdin().read_line(&mut s)?;
                    s.trim_end_matches(['\r', '\n']).to_string()
                }
            };
            periksa_sandi(&sandi).map_err(anyhow::Error::msg)?;
            sqlx::query(
                "insert into akun (nama_pengguna, nama, peran, sandi_hash) values ($1, $2, array['admin'], $3)",
            )
            .bind(nama_pengguna)
            .bind(nama)
            .bind(hash_sandi(&sandi)?)
            .execute(db)
            .await?;
            println!("Akun admin {nama_pengguna} dibuat. Sandi wajib diganti saat masuk pertama.");
        }
        lain => anyhow::bail!("perintah tidak dikenal: {lain}. Pilihan: serve, migrate, buat-admin, seed-demo"),
    }
    Ok(())
}
