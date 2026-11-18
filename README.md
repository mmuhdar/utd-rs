# utd-rs

Sistem informasi Bank Darah Rumah Sakit (BDRS) dan Unit Transfusi Darah milik RS (UTD-RS) untuk rumah sakit di Indonesia. Dipasang on-prem dengan Docker; backend Rust (axum + PostgreSQL), antarmuka web SvelteKit.

## Cakupan

- **Bank darah**: permintaan ke UTD; penerimaan kantong dengan berita serah terima dan pemindaian label ISBT 128 atau label lokal; stok FEFO; pencairan; karantina; log suhu alat simpan; penyimpangan suhu; pengembalian dan pemusnahan dengan berita acara.
- **Pelayanan pasien**: permintaan darah (formulir Permenkes 91/2015 §5.6); sampel dengan batas usia; golongan ABO/Rh pasien pada setiap permintaan; uji silang serasi per kantong; alokasi, termasuk jalur darurat dengan persetujuan dokter; pengeluaran dengan pengingat 30 menit; verifikasi dua petugas di samping tempat tidur dengan pencatatan KNC otomatis; observasi transfusi; reaksi transfusi beserta penelusurannya.
- **Mode UTD-RS** (opsional): donor; seleksi; cekal; donasi; golongan donasi; uji saring IMLTD; pengolahan komponen; pelulusan bertanda tangan elektronik; look-back.
- **Integritas data** (CPOB Aneks 2):
  - Verifikasi kedua oleh petugas lain untuk setiap hasil yang diinput manual.
  - Tanda tangan elektronik.
  - Jejak audit append-only yang berantai hash.
  - Koreksi selalu beralasan, dan data tidak pernah dihapus.
- **Laporan**: Formulir 3 bulanan, rasio C:T dan indikator mutu, perencanaan stok, ketersediaan darah, laporan UTD.
- **Integrasi**: token API untuk SIMRS dan data logger suhu; SATUSEHAT modul UPD lewat outbox (mode UTD-RS, bawaannya mati).

Dasar regulasi ada di `docs/research/regulasi-utd-rs.md`, istilah di `GLOSSARY.md`, keputusan di `docs/adr/`, spesifikasi di `docs/SPEK.md`.

## Mencoba dengan data demo

```sh
cp deploy/env.contoh .env    # isi sandi; untuk mencoba cukup nilai apa saja
docker compose up -d --build
docker compose exec app /app/utdrs seed-demo
```

Buka `https://utd.rs.lokal` (atau domain di `.env`). Akun demo: `kepala`, `teknisi`, `teknisi2`, `dokter`, `perawat`, `perawat2`, `admin`, `auditor`; sandinya `demo-utd-rs`. Semua data demo fiktif. Jangan pakai data demo di instalasi produksi.

## Dokumen

| Berkas | Isi |
|---|---|
| `docs/pemasangan.md` | Kebutuhan server, pemasangan, akun admin pertama, TLS, pembaruan versi |
| `docs/operasional.md` | Backup dan uji pemulihan, prosedur downtime, insiden, peninjauan audit |
| `docs/kepatuhan.md` | Pemetaan regulasi → fitur, dan dokumen yang harus disiapkan RS |
| `docs/validasi.md` | Matriks keterlacakan requirement → uji (dasar paket validasi sistem komputer) |
| `docs/desain.md` | Sistem desain antarmuka |

## Pengembangan

Lihat `CLAUDE.md`. Ringkasnya: `scripts/dev-db.sh start`, `scripts/cargo.sh test`, `scripts/dev-server.sh`, lalu `cd web && npm run dev`.

Lisensi: AGPL-3.0-or-later.
