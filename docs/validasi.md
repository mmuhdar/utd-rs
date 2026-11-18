# Matriks keterlacakan requirement → uji

Dasar paket validasi sistem komputer (CPOB Aneks 2 butir 12–21; Permenkes 91/2015 §2.20).

- **Uji otomatis**: berkas `src/uji*.rs` dan `src/*::tests`, dijalankan dengan `scripts/cargo.sh test`. Setiap uji ujung ke ujung memakai database baru yang sudah dimigrasi.
- **Uji frontend**: `cd web && npm test` (parser label) dan `npm run check`.
- **Pelengkap**: uji pengguna (UAT) di RS memakai skenario di kolom kanan, dengan data demo (`utdrs seed-demo`), dan hasilnya dicatat di formulir UAT RS.

| Requirement | Uji otomatis | Skenario UAT tambahan |
|---|---|---|
| FR-01 formulir permintaan lengkap, darurat beralasan | `alur_lengkap_sampai_reaksi` (isian kosong ditolak per isian), `simrs_lewat_token_api` | Dokter mengisi dari tablet; formulir tercetak lengkap |
| FR-02 penerimaan sampel, batas usia | `alur_lengkap_sampai_reaksi` (EDTA > 24 jam ditolak) | Sampel lisis ditolak dengan alasan |
| FR-03 golongan pasien tiap permintaan, diskrepansi | `diskrepansi_golongan_wajib_sampel_baru`, `tafsir_cell_dan_serum_grouping` | — |
| FR-04 konfirmasi golongan kantong | `golongan_kantong_tidak_sesuai_label_dikarantina` | Kantong dikarantina lalu dikembalikan ke UTD |
| FR-05 kompatibilitas ABO/Rh, override darurat | `aturan_kompatibilitas`, `pengeluaran_darurat_tanpa_golongan` | — |
| FR-06 uji silang, CCC, kesimpulan | `alur_lengkap_sampai_reaksi`, `kesimpulan_uji_silang` | Lembar kerja tercetak |
| FR-07 aturan uji silang per komponen | `ffp_uji_minor_dan_aturan_komponen` | — |
| FR-08 penyimpanan sampel 7 hari | — (prosedur fisik) | SPO rak sampel |
| FR-09/FR-10 penerimaan, label ISBT 128/lokal | `penerimaan_menolak_dan_mengembalikan`, `web/src/lib/isbt.test.ts` | Uji pemindai barcode di lokasi (kinerja pemindai, butir 21) |
| FR-11 stok FEFO, kedaluwarsa diblokir | `penerimaan_menolak_dan_mengembalikan`, `alur_lengkap_sampai_reaksi` | — |
| FR-12 pencairan memperpendek kedaluwarsa | `pencairan_mempersingkat_kedaluwarsa` | — |
| FR-13 log suhu, penyimpangan, penilaian | `penyimpangan_suhu_mengarantina_lalu_dinilai` | Uji data logger nyata |
| FR-14 pengeluaran, 30 menit | `alur_lengkap_sampai_reaksi` | Penanda > 30 menit di beranda |
| FR-15 verifikasi dua petugas, KNC | `alur_lengkap_sampai_reaksi` | Tablet + pemindai di ruang |
| FR-16 dokumentasi transfusi dan observasi | `alur_lengkap_sampai_reaksi` | — |
| FR-17 reaksi, penelusuran, penutupan | `alur_lengkap_sampai_reaksi` | Laporan diteruskan ke tim keselamatan pasien |
| FR-18 ketertelusuran, look-back | `alur_lengkap_sampai_reaksi` (lacak), `reaktif_mencekal_donor_dan_membuka_lookback`, `lookback_dari_notifikasi_utd_mitra` | — |
| FR-19/FR-20 pengembalian, pemusnahan | `penerimaan_menolak_dan_mengembalikan`, `penyimpangan_suhu_mengarantina_lalu_dinilai` | Berita acara tercetak |
| FR-21/FR-23/FR-24/FR-25 laporan | `alur_lengkap_sampai_reaksi` (Formulir 3, mutu, perencanaan, ketersediaan) | Bandingkan dengan rekap manual satu bulan |
| FR-22 permintaan ke UTD | — | Buat dan tutup permintaan rutin |
| FR-26 integrasi SIMRS | `simrs_lewat_token_api` | Uji dengan SIMRS RS |
| FR-27/FR-28 SATUSEHAT | `donor_sampai_pelulusan_dan_satusehat` (server tiruan), `payload_produk` | Kirim ke sandbox Kemenkes |
| FR-29–FR-31 donor, cekal, IMLTD, pelulusan | `donor_sampai_pelulusan_dan_satusehat`, `reaktif_mencekal_donor_dan_membuka_lookback` | — |
| FR-33 BDRS tidak mengulang IMLTD | Desain: tidak ada endpoint IMLTD untuk kantong UTD mitra | — |
| FR-34 koreksi beralasan, jendela 2×24 jam | `alur_lengkap_sampai_reaksi` (koreksi observasi) | — |
| NFR-01/02 RBAC, autentikasi, kunci akun | `masuk_kunci_dan_ganti_sandi`, `simrs_lewat_token_api`, uji 403 di semua berkas | — |
| NFR-03/05 audit append-only, verifikasi kedua | `audit_berantai_dan_tidak_bisa_diubah`, `alur_lengkap_sampai_reaksi` (penginput tidak boleh memverifikasi) | Kajian audit bulanan |
| NFR-06 tanda tangan elektronik | `penyimpangan_suhu_…` (sandi salah ditolak), `alur_lengkap_…` (penutupan reaksi) | — |
| NFR-09/11 backup terenkripsi, pemulihan, audit tetap utuh | `deploy/pulihkan.sh` (diuji manual 2026-11-18: checksum OK, audit utuh) | Uji pemulihan 3 bulanan |
| NFR-12 downtime | — | Simulasi tahunan (docs/operasional.md) |
| CSRF/keamanan HTTP | `mutasi_wajib_json` | Pemindaian kerentanan oleh TI RS |
| Data demo konsisten dengan aturan | `seed_demo_berhasil` | — |
