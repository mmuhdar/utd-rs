# Kepatuhan

Ringkasan dari `docs/research/regulasi-utd-rs.md`, yang memuat pasal dan URL lengkapnya. Beberapa interpretasi ditandai di sana dan perlu dikonfirmasi ke bagian hukum RS.

## Yang ditangani aplikasi

| Kewajiban | Fitur |
|---|---|
| PP 28/2024 Pasal 279–303; Permenkes 83/2014, 91/2015 (sepanjang tidak bertentangan) | Alur BDRS dan mode UTD-RS; aturan klinis yang tidak konsisten antarpasal dijadikan konfigurasi (ADR 0004) |
| Peraturan BPOM 11/2025 (CPOB UPD) Aneks 2 | Verifikasi kedua, tanda tangan elektronik (ADR 0005), audit berantai hash, pencegahan pelulusan kantong dari donor dicekal atau reaktif, hak akses per peran yang tercatat |
| Permenkes 24/2022 (RME) | Tidak ada penghapusan data (ADR 0003); koreksi beralasan dengan jendela 2×24 jam; backup terenkripsi di dua lokasi |
| UU 27/2022 (PDP) | Setiap pembukaan data pasien dan donor dicatat; minimisasi data pasien (salinan identitas saja, ADR 0002); data untuk notifikasi 3×24 jam tersedia di audit |
| PP 71/2019 | Data dan backup on-prem di Indonesia; HTTPS; sesi berbatas waktu; kunci akun |
| SATUSEHAT | Modul UPD lewat outbox (ADR 0006) |

## Yang harus disiapkan RS (di luar aplikasi)

1. **SPO** BDRS/UTD yang merujuk layar aplikasi, termasuk prosedur downtime (`docs/operasional.md`).
2. **Penetapan hak akses** oleh pimpinan RS, yaitu daftar pegawai dan perannya (Permenkes 24/2022 Pasal 30).
3. **Validasi sistem komputer**:
   - URS RS yang merujuk `docs/SPEK.md`;
   - analisis risiko;
   - UAT dengan matriks `docs/validasi.md`;
   - kualifikasi server (IQ/OQ);
   - laporan validasi dan prosedur kontrol perubahan.
   Wajib penuh bila RS menjalankan UPD (CPOB).
4. **Perjanjian dengan pengembang/vendor** yang memuat tanggung jawab pemeliharaan, akses jarak jauh, dan kerahasiaan (CPOB Aneks 2 butir 7–10).
5. **Pendaftaran PSE**:
   - RS pemerintah: lingkup publik;
   - RS swasta: lingkup privat (Permenkominfo 5/2020).
   Status PSE untuk sistem on-prem perlu dikonfirmasi.
6. **DPO, DPIA**, dan prosedur notifikasi kegagalan pelindungan data. Kategorisasi sistem elektronik menurut BSSN dan SMKI/ISO 27001 bila diwajibkan.
7. **Perjanjian kerja sama dengan UTD mitra**, termasuk format label kantong dan alur notifikasi look-back.
8. **Registrasi SATUSEHAT** (Organization, Location, kredensial) bila menjalankan mode UTD-RS.

## Yang perlu dipantau

- Permenkes pengganti 83/2014 dan 91/2015 (delegasi PP 28/2024 Pasal 303).
- Kanal pelaporan ketersediaan darah nasional (PP 28/2024 Pasal 299). Data sudah tersedia di `GET /api/laporan/ketersediaan`.
- Modul SATUSEHAT untuk BDRS dan transfusi pasien. Modul UPD saat ini baru tersedia di sandbox.
- PP 33/2026 (pelaksana UU PDP) belum terverifikasi.
