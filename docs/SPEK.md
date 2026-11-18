# Spesifikasi utd-rs

Turunan dari `docs/research/regulasi-utd-rs.md` §8 (FR-xx, NFR-xx). Istilah mengikuti `GLOSSARY.md`; keputusan arsitektur ada di `docs/adr/`. Setiap alur di bawah punya uji ujung ke ujung (`src/uji_*.rs`).

## Keputusan atas pertanyaan terbuka (riset §9)

| # | Pertanyaan | Keputusan |
|---|---|---|
| 1–2 | Lingkup BDRS atau UTD; RS menerima donor? | Keduanya. BDRS adalah inti; alur donor (termasuk donor pengganti) aktif pada mode UTD-RS (ADR 0001). |
| 3 | Format label UTD mitra | Setiap UTD mitra diberi format `isbt128` atau `lokal`. ISBT 128 diurai (nomor donasi, kode produk, golongan, kedaluwarsa); format lokal memakai nomor apa adanya dan isian manual. Impor data elektronik UTD belum didukung karena formatnya belum ada. |
| 4 | Uji silang TC/FFP/kriopresipitat | Dapat dikonfigurasi per komponen; bawaannya **minor** (versi lebih ketat, §5.7.3) (ADR 0004). |
| 5 | Kanal laporan ketersediaan darah nasional | Spesifikasinya belum ada. Disediakan `GET /api/laporan/ketersediaan` (JSON per komponen/golongan) supaya bisa diteruskan ke kanal apa pun begitu Kemenkes menetapkannya. |
| 6–7 | SATUSEHAT untuk BDRS; langsung atau lewat SIMRS | Hanya modul UPD, langsung dari utd-rs lewat outbox, bawaannya mati. Data transfusi pasien dikirim SIMRS (ADR 0006). |
| 8 | Permenkes pengganti 83/2014 & 91/2015 | Dipantau. Aturan yang paling mungkin berubah sudah berupa konfigurasi. |
| 9–11 | Posisi RS, PSE publik/privat, DPO/SMKI | Tidak memengaruhi kode. `docs/kepatuhan.md` memuat daftar dokumen yang harus disiapkan RS. |
| 12 | Retensi | Tidak ada penghapusan (ADR 0003). |
| 13 | PP 33/2026 | Tidak terverifikasi; tidak dijadikan dasar requirement. |
| 14 | Integrasi alat lab dan data logger suhu | Hasil lab dimasukkan manual dengan verifikasi kedua. Data logger mengirim lewat `POST /api/suhu` dengan token API akun berperan `logger`. |
| 15 | Biaya pengganti pengolahan darah (BPPD) / billing | Di luar lingkup. SIMRS membaca kantong yang dikeluarkan per pasien dari API. |

## Peran dan hak

Peran: `admin`, `dokter`, `perawat`, `teknisi` (petugas BDRS/UTD), `kepala` (penanggung jawab BDRS), `auditor`, serta dua akun layanan, `simrs` dan `logger`. Akun layanan memakai token API (header `Authorization: Bearer`), tidak memakai sesi. Hak per peran ada di `auth::hak_peran`.

Pemisahan tugas yang ditegakkan sistem:
- Verifikasi kedua (pemeriksaan golongan, uji silang, uji saring IMLTD) harus oleh akun yang berbeda dari penginput.
- Verifikasi samping tempat tidur ditandatangani dua akun yang berbeda.
- Pelulusan dan penutupan penelusuran reaksi hanya oleh `kepala`.

## Modul dan alur

### M1 — Data induk
- UTD mitra (kode, nama, format label). Alat simpan (kode, nomor seri, jenis, rentang suhu). Aturan komponen (FR-05, FR-07, FR-12; ADR 0004).
- Profil instalasi (nama RS, kode faskes, mode BDRS / UTD-RS).

### M2 — Penerimaan dan stok (FR-09, FR-10, FR-11, FR-13, FR-19, FR-20, FR-22, FR-33)
- **Permintaan ke UTD**: rutin, khusus, atau darurat; berisi baris komponen × golongan × jumlah.
- **Penerimaan**: berita serah terima berisi UTD, nomor dokumen UTD, suhu min/maks selama transportasi, petugas UTD, serta daftar kantong yang dipindai.
  - Kantong yang diterima langsung `tersedia`.
  - Kantong yang ditolak (kondisi, label, suhu, kedaluwarsa, IMLTD tidak non-reaktif) langsung dicatat sebagai pengembalian beralasan.
  - Kantong kedaluwarsa tidak bisa diterima sebagai tersedia.
- **Stok**: per komponen × golongan, diurutkan FEFO. Kantong mendekati kedaluwarsa (≤ 3 hari, TC ≤ 1 hari) ditandai. Kantong kedaluwarsa otomatis tidak bisa dialokasikan atau dikeluarkan.
- **Pencairan** FFP/kriopresipitat: kedaluwarsa dipersingkat sesuai `aturan_komponen.jam_setelah_cair`, dengan tercatat siapa dan kapan.
- **Pemindahan antar alat simpan.** Alat tujuan harus berjenis sesuai komponen.
- **Log suhu**, manual atau dari logger.
  - Pembacaan di luar rentang membuka **penyimpangan suhu**.
  - Penyimpangan hanya bisa ditutup setelah setiap kantong di alat itu dinilai `layak` atau `tidak_layak`; yang tidak layak masuk karantina.
- **Pengembalian** ke UTD dan **pemusnahan**, berupa berita acara bernomor berisi kantong + alasan terstruktur.

### M3 — Permintaan darah pasien dan uji pratransfusi (FR-01..FR-08, FR-14, FR-24)
- **Pasien**: salinan identitas dengan kunci no. RM (ADR 0002).
- **Permintaan darah**: semua isian FR-01, satu atau beberapa baris komponen. Permintaan darurat wajib beralasan. Status: `diminta` → `diproses` → `siap` → `selesai`, atau `batal`/`ditolak`.
- **Sampel**: waktu ambil, pengambil, jenis (EDTA/serum), penyimpanan (suhu ruang/dingin).
  - Diterima atau ditolak dengan alasan (lisis, bekuan, volume kurang, label tidak sesuai).
  - Batas usia: EDTA suhu ruang 24 jam, EDTA dingin 3 hari, serum/plasma dingin 7 hari. Sampel lewat batas tidak bisa dipakai untuk pemeriksaan baru.
- **Pemeriksaan golongan pasien** pada setiap sampel, mencakup cell grouping, serum grouping, dan Rh.
  - Cell dan serum grouping yang tidak konsisten ditolak.
  - Bila hasilnya berbeda dari pemeriksaan terverifikasi sebelumnya untuk no. RM yang sama, sampel ditandai diskrepansi dan wajib sampel baru.
  - Berlaku hanya setelah verifikasi kedua.
- **Uji silang serasi** per kantong × sampel, mencakup konfirmasi golongan kantong (ABO/Rh dari selang), mayor/minor/autokontrol sesuai aturan komponen, dan CCC untuk metode tabung.
  - Kesimpulan dihitung sistem. Bila golongan kantong berbeda dari label, kantong masuk karantina.
  - Setelah verifikasi kedua: serasi → kantong **dialokasikan** ke permintaan.
  - Komponen yang aturannya `tidak_perlu` dialokasikan cukup dengan kecocokan ABO/Rh.
- **Kompatibilitas ABO/Rh** diperiksa saat alokasi.
  - Darurat tanpa golongan pasien terverifikasi: hanya O Rh-neg (sel) atau AB (plasma), dengan persetujuan dokter yang tercatat.
  - Pasien Rh-neg yang menerima Rh-pos hanya lewat override darurat beralasan.
- Alokasi yang tidak dikeluarkan dalam 3×24 jam bisa dilepas kembali ke stok.
- **Rasio C:T** di laporan mutu.

### M4 — Pengeluaran, transfusi, reaksi (FR-14..FR-18, FR-34)
- **Pengeluaran**: hanya kantong teralokasi, belum kedaluwarsa, dengan uji silang terverifikasi. Mencatat jam keluar, petugas BDRS, dan petugas ruang penerima. Kantong yang belum mulai ditransfusikan 30 menit setelah keluar ditandai terlambat.
- **Verifikasi samping tempat tidur**: no. RM dari gelang dan nomor kantong dipindai. Sistem mencocokkan pasien, golongan, kedaluwarsa, dan status uji.
  - Petugas 1 adalah pengguna yang masuk; petugas 2 menandatangani dengan sandinya sendiri.
  - Bila ada ketidakcocokan, transfusi diblokir dan **KNC** dicatat otomatis.
- **Transfusi**: mulai, observasi (pra, 15 menit, berkala, selesai, pasca 4 jam rawat inap / 1 jam rawat jalan), lalu selesai atau dihentikan.
- **Reaksi transfusi**: dilaporkan dari ruang (waktu, gejala, tindakan). BDRS menelusuri (ABO/Rh ulang pasien dan kantong, uji silang ulang, kesimpulan), lalu `kepala` menutupnya dengan tanda tangan. Daftar reaksi tersedia untuk tim keselamatan pasien.
- **Ketertelusuran**: nomor kantong → asal (UTD/donor) → uji → pasien penerima → reaksi/akhir. No. RM → semua kantong yang pernah diterima.
- **Koreksi data klinis** (FR-34): koreksi beralasan, nilai lama tetap di jejak audit. Koreksi oleh penginput di atas 2×24 jam ditolak dan harus dilakukan oleh `kepala`.

### M5 — Mode UTD-RS (FR-18, FR-29..FR-32)
- **Donor**: NIK unik, identitas, kontak. **Cekal**: sementara (sampai tanggal) atau permanen.
- **Seleksi**: tanda vital, Hb, berat, kuesioner. Hasil ditolak otomatis membuat cekal.
- **Donasi**: nomor donasi unik, jenis (sukarela/pengganti untuk no. RM), volume.
- **Golongan donasi** dan **uji saring IMLTD** (HIV, HBsAg, HCV, sifilis; opsional malaria), masing-masing dengan verifikasi kedua.
  - Hasil reaktif membuat cekal sementara pada donor, mengarantina semua kantong dari donasi itu, dan memunculkan look-back atas donasi-donasi sebelumnya.
- **Pengolahan** satu donasi menjadi kantong komponen (karantina, kedaluwarsa dari `aturan_komponen.masa_simpan_jam`).
- **Pelulusan** oleh `kepala` dengan tanda tangan. Pelulusan ditolak bila IMLTD belum lengkap atau ada yang reaktif, golongan belum terverifikasi, atau donor sedang dicekal.
- Laporan UTD: jumlah donasi, hasil seleksi, IMLTD reaktif, dan komponen diolah per periode.

### M6 — Laporan (FR-21, FR-23, FR-24, FR-25)
- **Formulir 3 bulanan**:
  - A. Permintaan ke UTD.
  - B. Penerimaan dari UTD per komponen/golongan.
  - C. Pemakaian per ruang.
  - D. Reaksi transfusi.
  - E. Darah kembali ke UTD beserta alasan.
- **Mutu**: rasio C:T, kantong kedaluwarsa/dimusnahkan, permintaan tidak terpenuhi.
- **Perencanaan**: rerata pemakaian mingguan 26 minggu terakhir per komponen × golongan, untuk menghitung stok minimal (1 minggu) dan ideal (2 minggu).
- **Ketersediaan darah** (JSON) untuk kanal nasional.

### M7 — Integrasi (FR-26..FR-28)
- **Token API** untuk akun layanan. SIMRS dapat: upsert pasien, membuat permintaan darah, dan membaca permintaan beserta transfusinya.
- **Outbox SATUSEHAT** (mode UTD-RS): kantong hasil donasi yang diluluskan, dikeluarkan ke fasyankes lain, atau dimusnahkan → `BiologicallyDerivedProduct`. Klien OAuth2 client-credentials dengan cache token dan backoff.

### M8 — Operasional (NFR-07..NFR-17)
- Dockerfile multi-stage, `compose.yaml` produksi (app + PostgreSQL + backup terjadwal), dan `docs/pemasangan.md`.
- Backup harian dengan `pg_dump`, terenkripsi bila `BACKUP_PASSPHRASE` diisi, retensi lokal 30 hari, serta instruksi penyalinan off-site dan uji pemulihan.
- **Mode darurat (downtime)**: halaman cetak stok dan golongan pasien aktif, serta SPO manual di `docs/operasional.md`.
- Paket validasi: `docs/validasi.md` (URS → uji yang menutupnya).

## Antarmuka (web)

Semua layar untuk stasiun kerja BDRS dan tablet ruang. Dapat dipakai dengan pemindai barcode: isian nomor kantong dan no. RM menerima input pemindai lalu Enter. Kontras dan fokus keyboard mengikuti WCAG 2.1 AA (NFR-20). Bahasa Indonesia.
