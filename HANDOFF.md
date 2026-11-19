# Handoff — utd-rs

Diperbarui: 2026-11-19

## Status

- **Versi 0.1 selesai** dan dirilis open source (AGPL-3.0) di https://github.com/mmuhdar/utd-rs.
  - Tiket T00–T16 selesai.
  - Desain baru (FigCraft) sudah diterapkan ke semua layar.
  - Fitur tambahan: kop surat, judul dan logo aplikasi dari kop, tema terang/gelap, kebijakan sandi.
  - Dokumentasi: README lengkap, pedoman penggunaan dan pelatihan per peran (`docs/pedoman/Pedoman-Penggunaan-utd-rs.pdf`).
  - Uji: backend 23/23, frontend 6/6.
- **Sudah terpasang di server BDRS rumah sakit.** Alamat server, akun SSH, dan detail pemasangannya sengaja tidak dicatat di repo publik ini; ada di catatan lokal pemilik proyek.
  - **Saat ini app memakai database demo `utdrs_demo`** (`UTD_DB_NAMA=utdrs_demo` di `.env`, diisi `seed-demo` 2026-11-19). Data asli di database `utdrs` tidak tersentuh dan tetap dibackup. Cara kembali ada di `docs/operasional.md`.
  - Backup berjalan setiap hari pukul 01.00 WITA, dengan retensi 30 hari.
  - Root CA bisa diunduh lewat `http://<alamat-server>/sertifikat`.

## Cara memperbarui server

Bila server tidak bisa mengakses GitHub, npm, atau crates.io, build image di mesin dev lalu kirim lewat SSH:

```sh
UTD_VERSI=0.1.0 UTD_DB_SANDI=x UTD_BACKUP_PASSPHRASE=x docker compose -p utd-rs build
docker save utd-rs:0.1.0 utd-rs-backup:latest | gzip -1 | ssh <pengguna>@<server> 'gunzip | docker load'
# salin compose.yaml / deploy/ bila berubah (tar … | ssh … 'tar xzf - -C ~/utd-rs'), lalu:
ssh <pengguna>@<server> 'cd ~/utd-rs && docker compose up -d --pull never'
```

Jangan pernah menjalankan `docker compose down -v`: volume `caddy` menyimpan root CA, dan volume `db` menyimpan data.

## Tindak lanjut (sesi berikutnya)

1. **Pengaturan awal di server** (masuk sebagai `admin`, ganti sandi sementara):
   - profil instalasi: nama RS dan mode BDRS atau UTD-RS;
   - kop surat dan logo;
   - UTD mitra dan alat simpan beserta rentang suhunya;
   - kebijakan sandi;
   - akun petugas per peran.
2. **Backup**:
   - `UTD_BACKUP_PASSPHRASE` sudah disimpan pemilik proyek di pengelola sandi (selesai 2026-11-19);
   - pindahkan `UTD_DIR_OFFSITE` ke NAS atau disk lain, karena saat ini masih di disk yang sama;
   - lakukan uji pemulihan pertama (`docs/operasional.md`).
3. **UAT di RS** memakai `docs/validasi.md`, dengan pemindai barcode dan data logger suhu yang nyata. Kepala BDRS meninjau pedoman dan menyesuaikannya dengan SPO RS.
4. **SATUSEHAT**: verifikasi URL extension FHIR `BiologicallyDerivedProduct` di sandbox (`shortcut:` di `src/integrasi.rs`).
5. **Konfirmasi hukum** (`docs/kepatuhan.md`):
   - status PSE untuk sistem on-prem;
   - berlakunya Permenkes 24/2022 setelah PP 28/2024;
   - PP 33/2026;
   - pengganti Permenkes 83/2014 dan 91/2015.
6. **Integrasi rust-simrs** (`../rust-simrs`, F-PNJ-04): `POST /api/permintaan` dan `GET /api/permintaan/{id}` lewat token berperan `simrs`.
7. **Penyempurnaan UI**:
   - galat sandi tampil di dalam `DialogSandi`;
   - pencarian global (ada di desain, belum dibuat);
   - tabel uji silang di detail permintaan masih melebar;
   - detail transfusi baru dua kolom di lebar ≥ 1280 px;
   - desain FigCraft untuk layar selain 8 layar yang ada.
8. **Data demo dev** berisi artefak pengujian: penyimpangan suhu terbuka di LD-01, KNC, dan uji silang golongan beda. Isi ulang di database kosong sebelum dipakai untuk pelatihan.

## Lingkungan dev

```sh
scripts/dev-db.sh start && scripts/cargo.sh test      # cargo jalan di container (mesin ini tanpa compiler C)
scripts/dev-server.sh seed-demo && scripts/dev-server.sh   # :8080, akun demo bersandi demo-utd-rs
cd web && npm run dev
```
