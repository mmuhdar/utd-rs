# Operasional

## Backup dan pemulihan (NFR-09, NFR-11)

- Layanan `backup` membuat backup terenkripsi setiap hari pukul 01.00 waktu server (`TZ`). Backup manual kapan saja: `docker compose exec backup backup.sh`.
  - Formatnya `pg_dump` yang dienkripsi AES-256 dengan kunci dari frasa `UTD_BACKUP_PASSPHRASE`.
  - Hasilnya ditulis ke `UTD_DIR_BACKUP`, lalu disalin ke `UTD_DIR_OFFSITE` beserta checksum SHA-256. Nama berkas (`utdrs-YYYYMMDD-HHMMSS`) menjadi ID backup.
  - Retensi `UTD_BACKUP_RETENSI_HARI` hari (bawaan 30) di folder lokal dan offsite. Backup lama baru dihapus setelah backup baru selesai dan checksum-nya cocok. Untuk arsip lebih panjang, salin backup dari folder offsite ke media lain secara berkala sesuai kebijakan RS.
  - Log ada di `docker compose logs backup`. Pesan "PERINGATAN: /offsite tidak tersedia" berarti salinan kedua gagal dan harus ditindaklanjuti hari itu juga.
- **Uji pemulihan** wajib dilakukan minimal setiap 3 bulan dan setelah setiap pembaruan versi:
  ```sh
  docker compose exec backup pulihkan.sh /backup/utdrs-YYYYMMDD-HHMMSS.dump.enc uji_pulih
  docker compose exec backup dropdb uji_pulih
  ```
  Keluaran yang benar: checksum `OK`, jumlah kantong masuk akal, dan `audit utuh: true`. Catat tanggal, ID backup, hasil, dan nama petugas.
- Pemulihan produksi (bencana):
  1. Hentikan `app`.
  2. Pulihkan ke database baru.
  3. Arahkan `DATABASE_URL` ke database itu, atau ganti nama database.
  4. Jalankan `app`, lalu periksa audit.

## Prosedur saat sistem tidak tersedia (NFR-12; CPOB Aneks 2 butir 46)

1. **Cetak lembar darurat secara rutin**: menu **Laporan → Lembar darurat**, minimal setiap pergantian dinas. Isinya stok tersedia per komponen dan golongan serta permintaan yang sedang berjalan. Simpan cetakan terakhir di map BDRS.
2. Saat sistem mati, pakai formulir kertas: permintaan darah, lembar kerja golongan dan uji silang, buku pengeluaran, dan formulir verifikasi samping tempat tidur. Semua formulir memakai isian yang sama dengan layar.
3. Darurat tanpa golongan pasien terverifikasi tetap mengikuti aturan yang sama: hanya PRC O Rh-negatif atau plasma AB, dengan persetujuan dokter tertulis.
4. Setelah sistem pulih, entri susulan dilakukan oleh petugas yang mengerjakannya. Kolom keterangan diisi "entri susulan downtime" beserta jam aslinya. Verifikasi kedua tetap dilakukan.
5. Simulasi prosedur ini minimal setahun sekali dan dicatat (tanggal, skenario, temuan).

## Penyimpangan suhu

Data logger mengirim suhu lewat API, atau petugas mencatat manual minimal dua kali per dinas. Bila suhu keluar rentang, sistem membuka penyimpangan dan mengarantina semua kantong di alat itu. Penanggung jawab BDRS menilai setiap kantong, lalu menutup penyimpangan dengan kesimpulan dan tindakan perbaikan. Kantong yang tidak layak dikembalikan ke UTD atau dimusnahkan dengan berita acara.

## Insiden

- **Reaksi transfusi**: ruang melapor lewat layar transfusi. BDRS menelusuri, dan penanggung jawab menutup dengan tanda tangan. Teruskan laporannya ke tim keselamatan pasien RS (Permenkes 91/2015 §5.9 h).
- **Insiden keamanan informasi** (dugaan kebocoran data, akses tidak sah):
  1. Laporkan ke TTIS/tim TI dan DPO RS.
  2. Kumpulkan bukti dari jejak audit (menu **Pengaturan → Jejak audit**; setiap pembukaan data pasien tercatat).
  3. Bila terjadi kegagalan pelindungan data pribadi, pemberitahuan tertulis wajib dikirim paling lambat 3×24 jam (UU 27/2022 Pasal 46).

## Peninjauan berkala

| Kegiatan | Frekuensi | Oleh |
|---|---|---|
| Kajian jejak audit (koreksi data, pelepasan karantina, pengeluaran darurat, pencabutan cekal), dan periksa keutuhan rantai audit | Bulanan | Penanggung jawab BDRS + auditor mutu |
| Tinjau akun aktif dan peran; nonaktifkan pegawai yang keluar | Bulanan | Admin TI |
| Uji pemulihan backup | 3 bulanan | Admin TI |
| Tinjau aturan komponen dan aturan donor terhadap regulasi terbaru (pantau pengganti Permenkes 83/2014 & 91/2015) | Tahunan | Penanggung jawab BDRS |
| Simulasi downtime | Tahunan | BDRS + TI |
