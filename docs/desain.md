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

- **Tema** bisa dipilih: terang, gelap, atau ikuti sistem (bawaan). Pilihan disimpan per browser di `localStorage` (`utd-tema`).
  - Pengguna mengganti tema lewat tombol di kartu pengguna bilah samping, halaman Akun saya, atau pojok halaman Masuk.
  - `static/tema.js` memasang `data-tema` di `<html>` sebelum halaman digambar, jadi layar tidak berkedip. `lib/tema.svelte.ts` mengikuti perubahan setelan sistem.
  - Token gelap hanya berlaku di layar (`@media screen`). Dokumen cetak selalu terang.
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
- **Kop surat** (`Kop.svelte`, diatur di Pengaturan → Kop surat, pola sama dengan lis-rebuild):
  - logo kiri, identitas RS di tengah (pemerintah daerah, nama RS, unit, alamat, kontak, akreditasi), logo kanan, dan garis ganda di bawah;
  - baris yang kosong tidak dicetak;
  - dipasang di berita acara, serah terima, Formulir 3, dan lembar darurat; di formulir permintaan dan laporan reaksi hanya tampil saat dicetak (`cetakSaja`).
- **Judul dan logo aplikasi** (`Merek.svelte`, `lib/merek.svelte.ts`) mengikuti kop surat, seperti di lis-rebuild:
  - judul = nama RS di kop (kosong → nama di profil instalasi), subjudul = unit;
  - logo = logo kanan kop (kosong → logo kiri; keduanya kosong → tanda tetes darah bawaan);
  - dipakai di bilah samping, halaman Masuk, judul tab browser (`<Halaman> — <nama RS>`), dan favicon;
  - sumbernya endpoint publik `GET /api/auth/info` dan `GET /api/auth/logo`, karena halaman Masuk belum punya sesi.
- **Cetak**: elemen layar diberi `.tanpa-cetak`. Berita acara, formulir, laporan, dan lembar darurat rapi di kertas A4.
- **Empat keadaan data** (`Keadaan.svelte`): memuat, kosong, galat, dan penuh.
