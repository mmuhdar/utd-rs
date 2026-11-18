//! Konfigurasi dari variabel lingkungan. Nilai bawaan aman untuk produksi (cookie Secure).

pub struct Config {
    pub database_url: String,
    pub alamat: String,
    pub dir_web: String,
    pub cookie_secure: bool,
    /// Aktifkan bila aplikasi di belakang reverse proxy, supaya alamat klien diambil dari X-Forwarded-For.
    pub di_balik_proxy: bool,
    /// Terisi hanya bila UTD_SATUSEHAT_ENV dan kredensialnya diisi (ADR 0006).
    pub satusehat: Option<SatuSehat>,
}

#[derive(Clone)]
pub struct SatuSehat {
    pub auth_url: String,
    pub fhir_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub org_id: String,
}

fn satusehat() -> anyhow::Result<Option<SatuSehat>> {
    let (auth, fhir) = match std::env::var("UTD_SATUSEHAT_ENV").unwrap_or_default().as_str() {
        "" => return Ok(None),
        "sandbox" => (
            "https://api-satusehat-stg.dto.kemkes.go.id/oauth2/v1",
            "https://api-satusehat-stg.dto.kemkes.go.id/fhir-r4/v1",
        ),
        "production" => {
            ("https://api-satusehat.kemkes.go.id/oauth2/v1", "https://api-satusehat.kemkes.go.id/fhir-r4/v1")
        }
        lain => anyhow::bail!("UTD_SATUSEHAT_ENV harus sandbox atau production, bukan {lain}"),
    };
    let wajib = |k: &str| std::env::var(k).map_err(|_| anyhow::anyhow!("{k} wajib diisi bila UTD_SATUSEHAT_ENV diisi"));
    Ok(Some(SatuSehat {
        auth_url: env("UTD_SATUSEHAT_AUTH_URL", auth),
        fhir_url: env("UTD_SATUSEHAT_FHIR_URL", fhir),
        client_id: wajib("UTD_SATUSEHAT_CLIENT_ID")?,
        client_secret: wajib("UTD_SATUSEHAT_CLIENT_SECRET")?,
        org_id: wajib("UTD_SATUSEHAT_ORG_ID")?,
    }))
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
            satusehat: satusehat()?,
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
            satusehat: None,
        }
    }
}
