# Handoff — utd-rs

Diperbarui: 2026-11-18 (sesi otonom; semua keputusan terbuka diambil sendiri sesuai arahan pemilik proyek)

## Status: versi 0.1 lengkap, siap UAT

Semua tiket di `docs/TIKET.md` (T00–T16) selesai dan di-commit di `main`.

- **Riset regulasi**: `docs/research/regulasi-utd-rs.md`. Isinya 34 FR, 22 NFR, dan 15 pertanyaan terbuka.
- **Keputusan atas pertanyaan terbuka**: tabel di awal `docs/SPEK.md`. Keputusan yang sulit dibalik dicatat di `docs/adr/0001–0007`.
- **Glosarium**: `GLOSSARY.md`. Konvensi kode untuk agen: `CLAUDE.md`.
- **Backend Rust** (axum + PostgreSQL), 20 uji ujung ke ujung `#[sqlx::test]`, clippy bersih. Cakupannya:
  - alur BDRS: penerimaan, stok, suhu, permintaan, golongan, uji silang, alokasi/darurat, pengeluaran, verifikasi samping tempat tidur, observasi, reaksi, berita acara;
  - mode UTD-RS: donor, cekal, seleksi, IMLTD, pelulusan, look-back;
  - laporan;
  - token API untuk SIMRS dan logger;
  - outbox SATUSEHAT.
- **Frontend SvelteKit**: ±35 layar. `npm run check` bersih, `npm test` (6 uji) lulus. Semua rute disapu di browser tanpa galat runtime. Alur sampel → golongan dicoba manual lewat UI. Lembar darurat diperiksa dalam mode cetak dan layar transfusi pada ukuran tablet.
- **Docker produksi** (`compose.yaml`: app, PostgreSQL 17, Caddy TLS, backup terenkripsi): diuji penuh. App sehat di balik HTTPS, admin pertama berhasil dibuat, backup tersalin ke offsite. Pemulihan backup juga diuji, dan rantai audit tetap utuh.
- **Dokumen**: README, `docs/pemasangan.md`, `docs/operasional.md`, `docs/kepatuhan.md`, `docs/validasi.md`, `docs/desain.md`.

## Cara melanjutkan

```sh
scripts/dev-db.sh start && scripts/cargo.sh test      # cargo jalan di container (mesin ini tanpa compiler C)
scripts/dev-server.sh seed-demo && scripts/dev-server.sh   # :8080, akun demo bersandi demo-utd-rs
cd web && npm run dev
```

## Tindak lanjut yang disarankan

1. **UAT di RS** memakai `docs/validasi.md`, dan uji dengan pemindai barcode serta data logger suhu yang nyata.
2. **SATUSEHAT**: verifikasi URL extension FHIR `BiologicallyDerivedProduct` di sandbox. Docs publik tidak memuatnya, jadi URL saat ini ditandai `shortcut:` di `src/integrasi.rs`.
3. **Penyempurnaan UI** dari laporan agen layar:
   - galat sandi tampil di dalam `DialogSandi`, dan dialog tetap terbuka;
   - ganti `confirm()` bawaan browser dengan dialog sendiri (mode instalasi, cabut token);
   - form ubah identitas donor;
   - endpoint daftar tetap (gejala reaksi, alasan tolak sampel), karena saat ini disalin di `web/src/lib/pelayanan/darah.ts`;
   - daftar `objek` untuk filter audit;
   - tampilan payload outbox;
   - laporan triwulanan UTD (saat ini per bulan);
   - saran kantong O Rh− / AB untuk alokasi darurat.
4. **Konfirmasi hukum** (lihat `docs/kepatuhan.md`):
   - status PSE untuk sistem on-prem;
   - berlakunya Permenkes 24/2022 setelah PP 28/2024;
   - PP 33/2026;
   - pantau Permenkes pengganti 83/2014 dan 91/2015.
5. **Integrasi dengan rustSIMRS** (`../rust-simrs`, F-PNJ-04): SIMRS memanggil `POST /api/permintaan` dan membaca `GET /api/permintaan/{id}` lewat token API berperan `simrs`.
