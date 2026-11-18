# Sistem desain antarmuka

Sumber desain: `design/utd-rs.figcraft`, dibangun oleh skrip `design/layar.py`. Hasil render PNG ada di `design/render/`. Token warna ada di `web/src/app.css`.

Arahnya **modern klinis**: permukaan terang dan lega, bilah samping gelap, dan warna yang hanya dipakai untuk menyampaikan makna.

## Huruf

- **Plus Jakarta Sans** untuk semua teks. Huruf ini dirancang Tokotype, foundry asal Jakarta.
  - Judul halaman: 1,85rem ExtraBold dengan tracking rapat.
  - Judul kartu: Bold.
  - Isi: Medium/Regular.
- **IBM Plex Mono** (kelas `.mono`) untuk nomor kantong, no. RM, dan nomor dokumen. Font ini dipilih setelah dibandingkan dengan JetBrains Mono, Geist Mono, dan Fira Code: nolnya bertitik sehingga beda dari huruf O, dan huruf G tidak tertukar dengan angka 6.
- Kedua huruf dibundel lewat `@fontsource`, sehingga tetap berjalan offline di jaringan RS.

## Warna

- **Mode terang** adalah bawaan. Mode gelap mengikuti setelan sistem. Alasannya: ruang lab terang dan banyak layar yang dicetak.
- Warna **hanya lewat token**:
  - netral: `teks`, `teks-2`, `redup`, `garis`, `garis-kuat`, `halus`, `latar`, `kartu`;
  - aksi utama: `utama` (tinta gelap untuk tombol utama);
  - fokus keyboard: `fokus` (biru);
  - merek: `merek` (merah tua, hanya untuk logo);
  - bilah samping: `samping-*`.
- **Makna**: `aman`, `waspada`, `bahaya`, `info`. Masing-masing punya varian `-latar` (latar lembut) dan `-titik` (titik/ikon). Kontras teks minimal 4,5:1 di kedua mode.
- **Golongan darah** (komponen `Golongan.svelte`) memakai warna label ISBT 128: O biru, A kuning, B merah muda, AB putih.
  - Golongan Rh− diberi garis tepi tegas.
  - Teks "A+" / "O−" selalu tertulis, sehingga lencana tetap terbaca tanpa warna.

## Komponen

- **Tanpa warna saja**: status (`Chip`) selalu berisi titik dan teks. Prop `polos` menghilangkan titik, untuk penanda seperti "Darurat".
- **Peringatan klinis** (`.peringatan` untuk bahaya, `.catatan` untuk waspada):
  - ikon pertama otomatis tampil dalam kotak berwarna;
  - tidak hilang sendiri;
  - menyebut tindakan yang harus dilakukan, misalnya "JANGAN TRANSFUSIKAN…".
- **Kartu** (`.panel`): sudut 16px, garis tipis, dan bayangan halus. **Tabel** (`.tabel`): kepala kolom berlatar abu lembut dengan huruf kapital kecil.
- **Pemindai barcode** (`Pindai.svelte`):
  - isian besar dengan cincin fokus biru;
  - pemindai mengetik lalu menekan Enter, lalu isian langsung kosong dan tetap fokus untuk pindaian berikutnya;
  - label ISBT 128 diurai di `isbt.ts`, dan hasilnya selalu tampil untuk dikonfirmasi petugas.
- **Tanda tangan** (`DialogSandi.svelte`, kelas `.dialog`):
  - memakai `<dialog>` bawaan, sehingga fokus terkurung dan Esc membatalkan;
  - untuk petugas kedua, dialog meminta nama pengguna dan sandinya sendiri.
- **Tahap permintaan** (`routes/permintaan/[id]/Tahap.svelte`): enam tahap dari sampel sampai transfusi, dihitung dari data permintaan.
- **Tablet ruang**: target sentuh minimal 44px (`.btn--besar`). Hasil pindai yang baru terbaca berwarna biru (info), bukan hijau, karena pencocokan baru dilakukan server setelah petugas kedua menandatangani.
- **Cetak**: elemen layar diberi `.tanpa-cetak`. Berita acara, formulir, laporan, dan lembar darurat rapi di kertas A4.
- **Empat keadaan data** (`Keadaan.svelte`): memuat, kosong, galat, dan penuh.
