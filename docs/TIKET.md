# Tiket

Urutan pengerjaan. Setiap tiket selesai bila ada ujinya dan `scripts/cargo.sh test` serta `clippy` bersih.

- [x] T00 Fondasi: akun & peran, sesi, kunci akun, wajib ganti sandi, audit berantai hash
- [ ] T01 Tanda tangan elektronik (oleh diri sendiri / oleh petugas lain) + token API akun layanan
- [ ] T02 Data induk: profil instalasi, UTD mitra, alat simpan, aturan komponen (M1)
- [ ] T03 Permintaan ke UTD + penerimaan kantong + parser label ISBT 128 (M2)
- [ ] T04 Stok FEFO, pencairan, pemindahan alat, pengembalian & pemusnahan berita acara (M2)
- [ ] T05 Log suhu + penyimpangan suhu + penilaian kantong (M2)
- [ ] T06 Pasien + permintaan darah + sampel (M3)
- [ ] T07 Pemeriksaan golongan pasien + verifikasi kedua + diskrepansi (M3)
- [ ] T08 Uji silang serasi + konfirmasi golongan kantong + alokasi + kompatibilitas/override darurat (M3)
- [ ] T09 Pengeluaran + verifikasi samping tempat tidur + KNC + transfusi + observasi (M4)
- [ ] T10 Reaksi transfusi + penelusuran + penutupan; ketertelusuran & koreksi data (M4)
- [ ] T11 Mode UTD-RS: donor, cekal, seleksi, donasi, IMLTD, pengolahan, pelulusan, look-back (M5)
- [ ] T12 Laporan: Formulir 3, mutu (C:T), perencanaan, ketersediaan (M6)
- [ ] T13 Integrasi SIMRS lewat token API + outbox & klien SATUSEHAT (M7)
- [ ] T14 Web: kerangka, masuk, navigasi per peran, layar semua modul (desain lewat ui-ux-pro-max)
- [ ] T15 Docker produksi, backup, mode darurat, dokumen pemasangan/operasional/kepatuhan/validasi (M8)
- [ ] T16 Data demo fiktif (`utdrs seed-demo`)
