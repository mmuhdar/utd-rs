# Data induk pasien milik SIMRS; utd-rs hanya menyimpan salinan identitas

Permenkes 24/2022 dan PP 28/2024 menuntut RME yang terintegrasi dan tidak menduplikasi master pasien. utd-rs menyimpan salinan identitas pasien yang dibutuhkan untuk identifikasi dua faktor (no. RM, nama, tanggal lahir, jenis kelamin), dengan kunci no. RM. Salinan itu diisi lewat API oleh SIMRS, atau diketik manual saat SIMRS tidak terhubung. Golongan darah hasil pemeriksaan BDRS disimpan per pemeriksaan, bukan di data pasien, karena regulasi mewajibkan pemeriksaan ulang pada setiap permintaan.

SIMRS terhubung lewat akun layanan berperan `simrs` dengan token API. SIMRS mengirim permintaan darah, lalu membaca status dan dokumentasi transfusi untuk RME-nya.
