# Pemasangan on-prem

## Kebutuhan

- Server Linux 64-bit dengan Docker Engine ≥ 24 dan Docker Compose v2: 2 vCPU, 4 GB RAM, dan 50 GB disk sudah cukup untuk bank darah satu RS.
- Jam server tersinkron NTP (NFR-22). Cap waktu audit, batas 30 menit pengeluaran, dan batas usia sampel bergantung padanya.
- Lokasi backup kedua di luar ruang server, misalnya NAS di gedung lain, yang di-mount ke host (Permenkes 24/2022 Pasal 20 ayat 4–5).
- Seluruh data dan backup berada di wilayah Indonesia (PP 71/2019 Pasal 20 ayat 2).
- Internet tidak wajib. Internet hanya diperlukan untuk membangun image dan untuk SATUSEHAT.

## Langkah

1. Salin repositori ke server, misalnya `/srv/utd-rs`.
2. Siapkan konfigurasi: `cp deploy/env.contoh .env`, lalu isi `.env`.
   - `UTD_DB_SANDI` dan `UTD_BACKUP_PASSPHRASE`: acak dan panjang. Frasa backup juga disimpan di brankas terpisah, karena tanpa frasa itu backup tidak bisa dipulihkan.
   - `UTD_DOMAIN`: nama host yang dipakai klien. Daftarkan di DNS internal RS. Bila belum ada DNS internal, isi dengan alamat IP server (mis. `192.168.1.10`); klien membuka `https://<alamat-ip>`.
   - `UTD_DIR_BACKUP` dan `UTD_DIR_OFFSITE`: dua lokasi berbeda.
   - Kunci berkasnya: `chmod 600 .env`.
3. Jalankan `docker compose up -d --build`.
4. Buat admin pertama: `docker compose exec app /app/utdrs buat-admin admin "Nama Admin TI"`. Sandi sementara diketik di prompt dan wajib diganti saat masuk pertama.
5. Admin masuk ke **Pengaturan → Data induk & aturan**.
   - Isi profil RS dan pilih mode (BDRS, atau UTD-RS bila RS mengambil darah donor sendiri).
   - Daftarkan UTD mitra dan alat simpan.
   - Penanggung jawab BDRS meninjau **aturan komponen** (uji silang, masa simpan) dan **aturan donor** sesuai SPO RS (ADR 0004).
6. Buat akun pegawai sesuai peran (**Pengaturan → Akun pengguna**). Satu orang satu akun, tidak ada akun bersama.
7. Integrasi:
   - **SIMRS / data logger suhu**: buat akun berperan `simrs` atau `logger`, terbitkan token API, lalu pasang di sistem tersebut sebagai header `Authorization: Bearer <token>`. Endpoint: `POST /api/pasien`, `POST /api/permintaan`, `GET /api/permintaan/{id}`, `POST /api/suhu` (`{ "alat_kode": "LD-01", "suhu": 4.2 }`).
   - **SATUSEHAT** (mode UTD-RS): isi variabel `UTD_SATUSEHAT_*` di `.env`, mulai dari `sandbox`. Verifikasi URL extension FHIR di sandbox sebelum beralih ke production (lihat komentar `shortcut:` di `src/integrasi.rs`).

## TLS

Proxy Caddy melayani HTTPS. Karena utd-rs berjalan di jaringan lokal RS, bawaannya memakai CA internal Caddy (root CA berlaku 10 tahun). Root CA itu dipasang **sekali di setiap komputer dan tablet** supaya browser tidak menampilkan peringatan:

1. Dari perangkat klien, buka `http://<UTD_DOMAIN>/sertifikat` (HTTP biasa, tanpa "s"). Halaman ini berisi tombol unduh `utd-rs-root-ca.crt` dan langkah pemasangan untuk Windows, Firefox, Android, iPad/iPhone, dan Linux. Halaman Masuk juga menautkannya.
2. Sebelum memasang, cocokkan sidik jari SHA-256 berkasnya dengan milik server. Di server (folder `utd-rs`): `docker compose cp proxy:/data/caddy/pki/authorities/local/root.crt utd-rs-root-ca.crt && openssl x509 -in utd-rs-root-ca.crt -noout -fingerprint -sha256`. Nilai yang sama tampil di Windows (tab *Details* → *Thumbprint* SHA-256 / sidik jari) dan Android saat memasang.
3. Root CA disimpan di volume `caddy`. **Jangan hapus volume itu**: bila hilang, Caddy membuat CA baru dan semua perangkat harus memasang ulang. Ikutkan berkas root CA dalam arsip konfigurasi server.

Selain halaman sertifikat, semua akses HTTP dialihkan ke HTTPS. Bila RS punya sertifikat sendiri, ikuti petunjuk di `deploy/Caddyfile`. Port database tidak pernah dibuka ke jaringan.

## Pembaruan versi (NFR-08)

1. Baca catatan rilis. Backup manual dulu: `docker compose exec backup backup.sh`.
2. Ganti versi kode, naikkan `UTD_VERSI`, lalu jalankan `docker compose up -d --build`. Migrasi database berjalan otomatis saat aplikasi mulai, dan migrasi yang sudah ada tidak pernah diubah.
3. Uji singkat setelah instalasi, lalu catat hasilnya di log perubahan RS:
   - masuk;
   - buka stok;
   - `GET /api/audit/periksa` bernilai `utuh: true`;
   - satu alur uji di lingkungan uji.
