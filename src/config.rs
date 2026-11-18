//! Konfigurasi dari variabel lingkungan. Nilai bawaan aman untuk produksi (cookie Secure).

pub struct Config {
    pub database_url: String,
    pub alamat: String,
    pub dir_web: String,
    pub cookie_secure: bool,
    /// Aktifkan bila aplikasi di belakang reverse proxy, supaya alamat klien diambil dari X-Forwarded-For.
    pub di_balik_proxy: bool,
}

fn env(k: &str, bawaan: &str) -> String {
    std::env::var(k).unwrap_or_else(|_| bawaan.to_string())
}

impl Config {
    pub fn dari_env() -> anyhow::Result<Self> {
        Ok(Config {
            database_url: std::env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL belum diisi"))?,
            alamat: env("UTD_ALAMAT", "0.0.0.0:8080"),
            dir_web: env("UTD_DIR_WEB", "web/build"),
            cookie_secure: env("UTD_COOKIE_SECURE", "1") != "0",
            di_balik_proxy: env("UTD_DI_BALIK_PROXY", "0") == "1",
        })
    }

    #[cfg(test)]
    pub fn uji() -> Self {
        Config {
            database_url: String::new(),
            alamat: String::new(),
            dir_web: "web/build".into(),
            cookie_secure: false,
            di_balik_proxy: false,
        }
    }
}
