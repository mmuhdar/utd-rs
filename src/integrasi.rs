//! Integrasi SATUSEHAT modul UPD (ADR 0006; FR-27, FR-28). Peristiwa kantong hasil donasi sendiri (lulus, musnah)
//! masuk outbox di transaksi yang sama; pekerja latar mengirimnya sebagai `BiologicallyDerivedProduct` dengan
//! OAuth2 client credentials, cache token, dan backoff. Tanpa konfigurasi SATUSEHAT, antrean hanya tersimpan.

use crate::AppState;
use crate::auth::Pengguna;
use crate::config::SatuSehat;
use crate::error::{Galat, Hasil};
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{Value, json};
use sqlx::{PgConnection, PgPool};
use std::time::{Duration, Instant};
use uuid::Uuid;

pub fn rute() -> Router<AppState> {
    Router::new().route("/integrasi/outbox", get(daftar)).route("/integrasi/outbox/{id}/ulang", post(ulang))
}

const SISTEM_KANTONG: &str = "http://sys-ids.kemkes.go.id/biologicallyderivedproduct/blood-bag-number";
// shortcut: URL extension belum dipublikasikan di halaman docs SATUSEHAT (hanya di Postman collection modul UPD);
// pola ini mengikuti extension Kemenkes lain. Verifikasi di sandbox sebelum production.
const EXT: &str = "https://fhir.kemkes.go.id/r4/StructureDefinition/";

/// productCode per komponen (SATUSEHAT Lampiran Terminologi UPD, Lampiran 5).
fn kode_produk(komponen: &str) -> Value {
    let (sistem, kode, tampil) = match komponen {
        "wb" => ("http://snomed.info/sct", "12289007", "Whole blood product"),
        "prc" => ("http://snomed.info/sct", "431069006", "Packed red blood cells"),
        "tc" => ("http://terminology.kemkes.go.id", "TK000614", "Thrombocyte Concentrate (TC)"),
        "ffp" => ("http://snomed.info/sct", "346447007", "Fresh frozen plasma"),
        _ => ("http://snomed.info/sct", "256401009", "Cryoprecipitate"),
    };
    json!({ "coding": [{ "system": sistem, "code": kode, "display": tampil }] })
}

/// Alasan pemusnahan (Lampiran 6 dan SNOMED pada halaman resource).
fn alasan_musnah(alasan: &str) -> Value {
    let (sistem, kode, tampil) = match alasan {
        "reaktif IMLTD" => (
            "http://terminology.kemkes.go.id",
            "TK000772",
            "Hasil Reaktif Pemeriksaan Infeksi Menular Lewat Transfusi Darah (IMLTD)",
        ),
        "penyimpangan suhu" => {
            ("http://terminology.kemkes.go.id", "TK000774", "Penyimpanan Produk Darah yang Tidak Sesuai")
        }
        "bocor" | "rusak" | "selang putus" => {
            ("http://terminology.kemkes.go.id", "TK000771", "Kerusakan Fisik Produk Darah")
        }
        "kedaluwarsa" => ("http://snomed.info/sct", "117073009", "Blood product unit expired"),
        _ => ("http://snomed.info/sct", "74964007", "Other"),
    };
    json!({ "coding": [{ "system": sistem, "code": kode, "display": tampil }] })
}

/// Resource BiologicallyDerivedProduct dari baris kantong (beserta `waktu_donasi`). `collection.source` diisi saat
/// kirim, setelah IHS number donor diketahui.
pub fn produk_fhir(k: &Value, peristiwa: &str) -> Value {
    let komponen = k["komponen"].as_str().unwrap_or_default();
    let tanggal = |v: &Value| v.as_str().map(|s| s.get(..10).unwrap_or(s).to_string());
    let mut ext = vec![
        json!({ "url": format!("{EXT}ExpirationDate"), "valueDateTime": k["kedaluwarsa"] }),
        json!({ "url": format!("{EXT}Quantity"), "valueQuantity": { "value": k["volume_ml"], "unit": "mL", "system": "http://unitsofmeasure.org", "code": "mL" } }),
    ];
    if peristiwa == "musnah" {
        ext.push(json!({ "url": format!("{EXT}StatusDetail"), "valueCodeableConcept": { "coding": [{ "system": "http://snomed.info/sct", "code": "117070007", "display": "Blood product unit destroyed" }] } }));
        ext.push(json!({ "url": format!("{EXT}DestructionDate"), "valueDateTime": k["waktu_akhir"] }));
        ext.push(json!({ "url": format!("{EXT}DestructionReason"), "valueCodeableConcept": alasan_musnah(k["alasan_status"].as_str().unwrap_or_default()) }));
    }
    json!({
        "resourceType": "BiologicallyDerivedProduct",
        "identifier": [{ "system": SISTEM_KANTONG, "value": k["nomor"] }],
        "status": if peristiwa == "lulus" { "available" } else { "unavailable" },
        "productCategory": "fluid",
        "productCode": kode_produk(komponen),
        "collection": { "collectedDateTime": tanggal(&k["waktu_donasi"]) },
        "processing": [{ "timeDateTime": tanggal(&k["dibuat_pada"]) }],
        "extension": ext,
    })
}

/// Memasukkan peristiwa kantong hasil donasi sendiri ke outbox (di transaksi pemanggil).
pub async fn antre_produk(conn: &mut PgConnection, kantong: Uuid, peristiwa: &str) -> Result<(), sqlx::Error> {
    let k: Option<Value> = sqlx::query_scalar(
        "select to_jsonb(k) || jsonb_build_object('waktu_donasi', n.waktu, 'nik_donor', d.nik, 'waktu_akhir', now())
         from kantong k join donasi n on n.id = k.donasi_id join donor d on d.id = n.donor_id where k.id = $1",
    )
    .bind(kantong)
    .fetch_optional(&mut *conn)
    .await?;
    let Some(k) = k else { return Ok(()) };
    let payload = json!({ "resource": produk_fhir(&k, peristiwa), "nik_donor": k["nik_donor"] });
    sqlx::query("insert into outbox (peristiwa, kantong_id, payload) values ($1, $2, $3)")
        .bind(peristiwa)
        .bind(kantong)
        .bind(payload)
        .execute(conn)
        .await?;
    Ok(())
}

async fn daftar(State(s): State<AppState>, p: Pengguna) -> Hasil<Json<Value>> {
    p.wajib("integrasi.kelola")?;
    let v: Value = sqlx::query_scalar(
        "select jsonb_build_object('aktif', $1::bool,
            'ringkasan', (select coalesce(jsonb_object_agg(status, n), '{}') from (select status, count(*) n from outbox group by status) x),
            'baris', (select coalesce(jsonb_agg(to_jsonb(o) - 'payload' || jsonb_build_object('nomor_kantong', k.nomor) order by o.id desc), '[]')
                      from (select * from outbox order by id desc limit 200) o join kantong k on k.id = o.kantong_id))",
    )
    .bind(s.cfg.satusehat.is_some())
    .fetch_one(&s.db)
    .await?;
    Ok(Json(v))
}

async fn ulang(State(s): State<AppState>, p: Pengguna, Path(id): Path<i64>) -> Hasil<Json<Value>> {
    p.wajib("integrasi.kelola")?;
    let mut tx = s.db.begin().await?;
    let n = sqlx::query(
        "update outbox set status = 'antre', percobaan = 0, berikut_pada = now() where id = $1 and status = 'gagal'",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if n == 0 {
        return Err(Galat::Konflik("Hanya kiriman yang gagal yang bisa diulang.".into()));
    }
    crate::audit::ubah(&mut tx, &p, "ulang", "outbox", id, json!({})).await?;
    tx.commit().await?;
    Ok(Json(json!({})))
}

// ---------- Pengirim ----------

pub struct Pengirim {
    cfg: SatuSehat,
    http: reqwest::Client,
    token: Option<(String, Instant)>,
    /// Setelah autentikasi gagal, SATUSEHAT membatasi 1 permintaan token per menit.
    jeda_auth: Option<Instant>,
}

const MAKS_PERCOBAAN: i32 = 10;

impl Pengirim {
    pub fn baru(cfg: SatuSehat) -> Self {
        let http = reqwest::Client::builder().timeout(Duration::from_secs(30)).build().expect("klien HTTP");
        Pengirim { cfg, http, token: None, jeda_auth: None }
    }

    async fn token(&mut self) -> anyhow::Result<String> {
        if let Some((t, sampai)) = &self.token
            && Instant::now() < *sampai
        {
            return Ok(t.clone());
        }
        if self.jeda_auth.is_some_and(|j| Instant::now() < j) {
            anyhow::bail!("menunggu jeda setelah autentikasi gagal");
        }
        let res = self
            .http
            .post(format!("{}/accesstoken", self.cfg.auth_url))
            .query(&[("grant_type", "client_credentials")])
            .form(&[("client_id", self.cfg.client_id.as_str()), ("client_secret", self.cfg.client_secret.as_str())])
            .send()
            .await?;
        if !res.status().is_success() {
            self.jeda_auth = Some(Instant::now() + Duration::from_secs(61));
            anyhow::bail!("autentikasi SATUSEHAT ditolak: {}", res.status());
        }
        let v: Value = res.json().await?;
        let token =
            v["access_token"].as_str().ok_or_else(|| anyhow::anyhow!("respons token tanpa access_token"))?.to_string();
        let detik: u64 =
            v["expires_in"].as_str().and_then(|s| s.parse().ok()).or(v["expires_in"].as_u64()).unwrap_or(3599);
        // Diperbarui 5 menit sebelum kedaluwarsa.
        self.token = Some((token.clone(), Instant::now() + Duration::from_secs(detik.saturating_sub(300).max(60))));
        Ok(token)
    }

    async fn ihs_donor(&mut self, nik: &str) -> anyhow::Result<String> {
        let token = self.token().await?;
        let v: Value = self
            .http
            .get(format!("{}/Patient", self.cfg.fhir_url))
            .query(&[("identifier", format!("https://fhir.kemkes.go.id/id/nik|{nik}"))])
            .bearer_auth(token)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        v["entry"][0]["resource"]["id"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| anyhow::anyhow!("donor dengan NIK ini belum terdaftar di Master Patient Index SATUSEHAT"))
    }

    /// Mengirim satu baris outbox; Ok(Some(id resource)) bila terkirim, Ok(None) bila harus menunggu.
    async fn kirim(
        &mut self,
        db: &PgPool,
        id: i64,
        peristiwa: &str,
        kantong: Uuid,
        payload: &Value,
    ) -> anyhow::Result<Option<String>> {
        let mut res = payload["resource"].clone();
        let ihs = self.ihs_donor(payload["nik_donor"].as_str().unwrap_or_default()).await?;
        res["collection"]["source"] = json!({ "reference": format!("Patient/{ihs}") });
        let token = self.token().await?;
        let req = if peristiwa == "lulus" {
            self.http.post(format!("{}/BiologicallyDerivedProduct", self.cfg.fhir_url))
        } else {
            let sid: Option<String> = sqlx::query_scalar("select satusehat_id from kantong where id = $1")
                .bind(kantong)
                .fetch_one(db)
                .await?;
            let Some(sid) = sid else {
                tracing::info!(id, "pemusnahan menunggu kantong terkirim lebih dulu");
                return Ok(None);
            };
            res["id"] = json!(sid);
            self.http.put(format!("{}/BiologicallyDerivedProduct/{sid}", self.cfg.fhir_url))
        };
        let r = req.bearer_auth(token).json(&res).send().await?;
        let status = r.status();
        let body: Value = r.json().await.unwrap_or(Value::Null);
        if !status.is_success() {
            anyhow::bail!("SATUSEHAT membalas {status}: {}", body.to_string().chars().take(500).collect::<String>());
        }
        Ok(Some(body["id"].as_str().unwrap_or_default().to_string()))
    }

    /// Memproses kiriman yang jatuh tempo. Dipanggil berkala oleh `jalankan`, dan langsung oleh uji.
    pub async fn proses(&mut self, db: &PgPool) -> anyhow::Result<usize> {
        let baris: Vec<(i64, String, Uuid, Value, i32)> = sqlx::query_as(
            "select id, peristiwa, kantong_id, payload, percobaan from outbox
             where status = 'antre' and berikut_pada <= now() order by id limit 20",
        )
        .fetch_all(db)
        .await?;
        let mut terkirim = 0;
        for (id, peristiwa, kantong, payload, percobaan) in baris {
            match self.kirim(db, id, &peristiwa, kantong, &payload).await {
                Ok(Some(sid)) => {
                    let mut tx = db.begin().await?;
                    sqlx::query("update outbox set status = 'terkirim', terkirim_pada = now(), galat = null, respons = $2 where id = $1")
                        .bind(id)
                        .bind(json!({ "id": sid }))
                        .execute(&mut *tx)
                        .await?;
                    if peristiwa == "lulus" && !sid.is_empty() {
                        sqlx::query("update kantong set satusehat_id = $2 where id = $1")
                            .bind(kantong)
                            .bind(&sid)
                            .execute(&mut *tx)
                            .await?;
                    }
                    tx.commit().await?;
                    terkirim += 1;
                }
                Ok(None) => {
                    sqlx::query("update outbox set berikut_pada = now() + interval '5 minutes' where id = $1")
                        .bind(id)
                        .execute(db)
                        .await?;
                }
                Err(e) => {
                    let n = percobaan + 1;
                    tracing::warn!(id, galat = %e, "kirim SATUSEHAT gagal");
                    sqlx::query(
                        "update outbox set percobaan = $2, galat = $3,
                                status = case when $2 >= $4 then 'gagal' else 'antre' end,
                                berikut_pada = now() + least(interval '6 hours', make_interval(mins => power(2, $2)::int))
                         where id = $1",
                    )
                    .bind(id)
                    .bind(n)
                    .bind(e.to_string())
                    .bind(MAKS_PERCOBAAN)
                    .execute(db)
                    .await?;
                }
            }
        }
        Ok(terkirim)
    }
}

/// Pekerja latar: memeriksa outbox tiap 30 detik.
pub async fn jalankan(db: PgPool, cfg: SatuSehat) {
    let mut p = Pengirim::baru(cfg);
    loop {
        if let Err(e) = p.proses(&db).await {
            tracing::error!(galat = %e, "pekerja outbox SATUSEHAT");
        }
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_produk() {
        let k = json!({ "nomor": "D001-PRC", "komponen": "prc", "kedaluwarsa": "2026-12-20T00:00:00Z", "volume_ml": 250,
                        "waktu_donasi": "2026-11-15T08:00:00Z", "dibuat_pada": "2026-11-15T10:00:00Z",
                        "alasan_status": "penyimpangan suhu", "waktu_akhir": "2026-11-18T00:00:00Z" });
        let r = produk_fhir(&k, "lulus");
        assert_eq!(r["status"], "available");
        assert_eq!(r["identifier"][0]["system"], SISTEM_KANTONG);
        assert_eq!(r["productCode"]["coding"][0]["code"], "431069006");
        assert_eq!(r["collection"]["collectedDateTime"], "2026-11-15");
        let r = produk_fhir(&k, "musnah");
        assert_eq!(r["status"], "unavailable");
        let ext = r["extension"].as_array().unwrap();
        assert!(ext.iter().any(|e| e["valueCodeableConcept"]["coding"][0]["code"] == "TK000774"));
        assert!(ext.iter().any(|e| e["valueCodeableConcept"]["coding"][0]["code"] == "117070007"));
    }
}
