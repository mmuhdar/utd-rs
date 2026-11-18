# Sistem desain antarmuka

Arah dasarnya diambil dari skill ui-ux-pro-max (gaya Swiss minimal, kepadatan tinggi, Fira Sans/Fira Code), lalu disesuaikan untuk konteks klinis. Tokennya ada di `web/src/app.css`.

- **Mode terang bawaan**, mode gelap mengikuti setelan sistem. Ruang lab terang dan banyak layar yang dicetak. Rekomendasi "gelap sebagai bawaan" untuk dasbor ditolak dengan alasan itu.
- **Warna hanya lewat token**: `utama` (teal medis), `bahaya`, `waspada`, `aman`, `redup`, `garis`, `halus`. Kontras teks minimal 4,5:1 di kedua mode.
- **Status tidak pernah hanya warna**: chip selalu berisi teks, dan peringatan berisi ikon serta kalimat.
- **Peringatan klinis** (`.peringatan`, `role="alert"`) tidak hilang sendiri dan menyebut tindakan yang harus dilakukan, misalnya "JANGAN TRANSFUSIKAN…".
- **Huruf**: Fira Sans 15px untuk teks; Fira Code (`.mono`) untuk nomor kantong, no. RM, dan nomor dokumen agar karakter mirip (0/O, 1/l) mudah dibedakan. Huruf dibundel lewat `@fontsource`, jadi berjalan offline di jaringan RS.
- **Pemindai barcode** (`Pindai.svelte`): pemindai mengetik lalu menekan Enter. Isian langsung kosong dan tetap fokus untuk pindaian berikutnya. Label ISBT 128 diurai di `isbt.ts`, dan hasilnya selalu tampil untuk dikonfirmasi petugas.
- **Tanda tangan** (`DialogSandi.svelte`): memakai `<dialog>` bawaan, sehingga fokus terkurung dan Esc membatalkan. Untuk petugas kedua, dialog meminta nama pengguna dan sandinya sendiri.
- **Tablet ruang**: target sentuh minimal 44px (`.btn--besar`).
- **Cetak**: elemen layar diberi `.tanpa-cetak`. Berita acara, formulir, laporan, dan lembar darurat rapi di kertas A4.
- **Empat keadaan data** (`Keadaan.svelte`): memuat, kosong, galat, dan penuh.
