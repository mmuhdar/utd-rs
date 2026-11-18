# utd-rs

Pelayanan darah di rumah sakit: dari kantong darah yang diterima (dari UTD mitra atau dari donasi sendiri) sampai darah itu ditransfusikan ke pasien, dimusnahkan, atau dikembalikan, dengan ketertelusuran dua arah.

## Unit dan pihak

**BDRS (Bank Darah Rumah Sakit)**:
Unit RS yang menyimpan darah dari UTD, melakukan uji pratransfusi, dan menyerahkan darah ke ruang perawatan. Tidak mengulang uji saring IMLTD.
_Avoid_: blood bank, depo darah

**UTD / UPD (Unit Transfusi Darah / Unit Pengelola Darah)**:
Unit yang mengambil darah donor, menguji saring, mengolah komponen, dan meluluskannya. "UPD" adalah istilah Peraturan BPOM 11/2025; keduanya merujuk hal yang sama.

**UTD mitra**:
UTD di luar RS yang memasok kantong ke BDRS berdasarkan perjanjian kerja sama.
_Avoid_: PMI (PMI adalah salah satu penyelenggara UTD, bukan nama peran)

**Mode UTD-RS**:
Instalasi yang selain menjalankan BDRS juga menjalankan UTD sendiri (menerima donor). Lawannya mode BDRS saja.

**Penanggung jawab BDRS**:
Dokter atau petugas yang berwenang memverifikasi hasil, menetapkan aturan pelayanan darah, meluluskan komponen, dan menutup penelusuran reaksi.
_Avoid_: kepala lab, supervisor

**Ruang**:
Unit perawatan RS yang meminta dan memberikan darah ke pasien (IGD, rawat inap, kamar operasi, dan sebagainya).
_Avoid_: bangsal, poli

## Darah

**Kantong**:
Satu unit komponen darah dalam satu wadah berlabel, dengan nomor kantong yang unik.
_Avoid_: unit, bag, labu

**Nomor kantong**:
Nomor identifikasi donasi unik yang tercetak pada label kantong (ISBT 128 atau format UTD mitra), ditambah kode komponen bila satu donasi menghasilkan beberapa kantong.
_Avoid_: nomor donor, nomor stok

**Komponen**:
Jenis isi kantong: darah lengkap (WB), sel darah merah pekat (PRC), trombosit pekat (TC), plasma segar beku (FFP), atau kriopresipitat.
_Avoid_: produk darah, sediaan

**Golongan darah**:
Kombinasi golongan ABO dan Rhesus (D) seseorang atau sebuah kantong.

**Kedaluwarsa**:
Batas waktu kantong boleh ditransfusikan. Ditetapkan oleh UTD pada label, dan dipersingkat oleh BDRS bila kantong dicairkan atau dibuka.

**Status kantong**:
Posisi kantong dalam alurnya: karantina, tersedia, dialokasikan, dikeluarkan, ditransfusikan, dikembalikan, atau dimusnahkan.

**Karantina**:
Status kantong yang belum boleh dipakai: hasil uji saring belum lengkap, konfirmasi golongan belum dilakukan, atau sedang diselidiki.

**Alat simpan**:
Lemari darah, freezer plasma, atau inkubator trombosit yang punya nomor seri dan rentang suhu yang diizinkan.
_Avoid_: kulkas

**Penyimpangan suhu**:
Kejadian suhu alat simpan keluar dari rentang yang diizinkan. Setiap kantong di dalamnya wajib dinilai ulang statusnya.

## Alur pasien

**Pasien**:
Penerima darah yang diidentifikasi dengan nomor rekam medis RS. Data induknya milik SIMRS.

**Permintaan darah**:
Formulir dari dokter peminta untuk satu pasien yang berisi komponen, jumlah kantong, indikasi, dan jenis permintaan (elektif, rutin, atau darurat).
_Avoid_: order darah, bon darah

**Sampel pratransfusi**:
Darah pasien yang dikirim bersama permintaan darah. Punya waktu pengambilan dan batas usia pakai.

**Pemeriksaan golongan pasien**:
Penentuan ABO (cell dan serum grouping) dan Rh pasien. Diulang pada setiap permintaan darah.

**Konfirmasi golongan kantong**:
Pemeriksaan ulang ABO dan Rh kantong dari potongan selangnya sebelum kantong dipakai.
_Avoid_: cek ulang donor

**Uji silang serasi**:
Pengujian satu kantong terhadap satu sampel pratransfusi (mayor, minor, autokontrol) yang menghasilkan kesimpulan serasi, hanya PRC, atau tidak serasi.
_Avoid_: crossmatch (boleh di UI sebagai keterangan), cocok silang

**Verifikasi kedua**:
Pemeriksaan ulang oleh petugas lain, yang menandatanganinya sendiri, atas data kritis yang dimasukkan manual.
_Avoid_: double check, approval

**Alokasi**:
Pengikatan kantong yang sudah serasi ke satu permintaan darah, sehingga tidak bisa dipakai pasien lain.
_Avoid_: booking, reservasi

**Pengeluaran**:
Penyerahan kantong dari BDRS ke petugas ruang. Waktu keluarnya dicatat, dan kantong harus mulai ditransfusikan dalam 30 menit.
_Avoid_: distribusi (distribusi adalah UTD → RS), issue

**Verifikasi samping tempat tidur**:
Pencocokan identitas pasien dan kantong oleh dua petugas tepat sebelum transfusi dimulai.
_Avoid_: bedside check

**Transfusi**:
Pemberian satu kantong ke satu pasien, dari mulai sampai selesai atau dihentikan, beserta observasinya.

**Reaksi transfusi**:
Kejadian tidak diinginkan pada pasien yang dikaitkan dengan transfusi. Akut bila terjadi dalam 24 jam, lambat bila setelahnya.

**Penelusuran reaksi**:
Investigasi BDRS atas reaksi transfusi: memeriksa ulang kantong dan sampel, lalu menyimpulkan. Ditutup oleh penanggung jawab BDRS.

**Kejadian nyaris cedera (KNC)**:
Ketidaksesuaian yang tertangkap sebelum darah masuk ke pasien, misalnya identitas tidak cocok saat verifikasi samping tempat tidur.

**Look-back**:
Penelusuran mundur dari donor atau kantong bermasalah ke semua pasien penerimanya, dan sebaliknya.

## Akhir hidup kantong

**Pengembalian**:
Penyerahan kembali kantong ke UTD mitra dengan alasan terstruktur.
_Avoid_: retur

**Pemusnahan**:
Pembuangan kantong yang tidak layak sebagai limbah medis, dengan berita acara.

**Rasio C:T**:
Jumlah kantong yang diuji silang serasi dibagi jumlah kantong yang ditransfusikan. Di atas 2,5 dianggap uji silang berlebihan.

## Donor (mode UTD-RS)

**Donor**:
Orang yang menyumbangkan darah, diidentifikasi dengan NIK.
_Avoid_: pendonor (boleh di UI), penyumbang

**Donor pengganti**:
Donor yang menyumbang untuk pasien tertentu, biasanya keluarga pasien.

**Seleksi donor**:
Penilaian kelayakan donor sebelum pengambilan darah, yang berakhir dengan diterima, ditolak sementara, atau ditolak permanen.

**Cekal**:
Larangan donor menyumbang, sementara atau permanen. Komponen dari donor yang dicekal tidak dapat diluluskan.

**Donasi**:
Satu kali pengambilan darah dari satu donor, dengan nomor donasi unik.
_Avoid_: aftap (boleh di UI sebagai keterangan), flebotomi

**Uji saring IMLTD**:
Pemeriksaan infeksi menular lewat transfusi darah pada setiap donasi: minimal HIV, HBsAg, HCV, dan sifilis.
_Avoid_: skrining infeksi

**Pengolahan**:
Pemisahan satu donasi menjadi beberapa kantong komponen.

**Pelulusan**:
Keputusan penanggung jawab, dengan tanda tangan elektronik, bahwa kantong hasil donasi sendiri boleh masuk stok tersedia.
_Avoid_: release, approval

**Tanda tangan elektronik**:
Konfirmasi ulang identitas petugas (kata sandi) yang terikat permanen pada satu catatan beserta maknanya, tanggal, dan waktunya.
_Avoid_: TTD, paraf
