# BDRS sebagai inti, UTD sebagai mode opsional dalam satu aplikasi

Sebagian besar RS hanya menjalankan BDRS dan menerima darah dari UTD mitra. Sebagian kecil juga menjalankan UTD sendiri, dan untuk itu CPOB BPOM 11/2025 berlaku penuh. Kami memutuskan membangun satu aplikasi dengan alur BDRS sebagai inti, dan alur donor (seleksi, cekal, donasi, uji saring IMLTD, pengolahan, pelulusan) yang aktif hanya bila `profil_instalasi.jenis = 'utd_rs'`. Kantong hasil donasi sendiri masuk ke stok yang sama dengan kantong dari UTD mitra, sehingga ketertelusuran donor → kantong → pasien berada di satu database.

Alur BDRS tidak menyediakan uji saring IMLTD ulang untuk kantong dari UTD mitra (Permenkes 83/2014 Pasal 44); hasil UTD hanya dicatat dari label atau dokumen.

**Considered Options**: dua aplikasi terpisah (BDRS dan UTD) ditolak, karena look-back dari donor ke pasien jadi butuh integrasi antar-aplikasi di RS yang sama.
