# utd-rs

Bank darah RS / UTD-RS. Istilah di `GLOSSARY.md`, keputusan di `docs/adr/`, requirement (FR-xx/NFR-xx) di `docs/SPEK.md` dan `docs/research/regulasi-utd-rs.md`, keterlacakan uji di `docs/validasi.md`. Rujuk ID requirement di komentar dan commit.

## Lingkungan dev

Mesin dev tidak punya compiler C, jadi cargo selalu dijalankan di container:

- `scripts/dev-db.sh start` menjalankan PostgreSQL di localhost:54330.
- `scripts/cargo.sh test|clippy --all-targets|fmt` memakai image `utd-rs-dev`.
- `scripts/dev-server.sh [seed-demo]` menjalankan server di :8080.
- Web: `cd web && npm run dev` (proxy /api ke :8080), lalu `npm run check` dan `npm test`.

## Backend (Rust, `src/`)

- Satu modul per domain, masing-masing dengan `pub fn rute() -> Router<AppState>` yang di-merge di `main.rs`.
- Setiap handler memanggil `p.wajib("<hak>")` sebelum menyentuh data. Hak per peran ada di `auth::hak_peran`.
- Setiap perubahan data, dan setiap pembukaan data pasien atau donor, ditulis dengan `audit::catat` di transaksi yang sama.
- Data tidak dihapus (ADR 0003). Perubahan status kantong selalu lewat `stok::ubah_status`.
- Hasil yang diinput manual baru berlaku setelah verifikasi kedua (`pratransfusi::verifikasi`). Tanda tangan lewat `tanda_tangan::{sendiri, petugas_lain, verifikasi_kedua}`.
- Query sqlx dijalankan saat runtime (ADR 0007). SQL dinamis hanya dengan nama dari daftar tetap, dibungkus `sqlx::AssertSqlSafe`.
- Migrasi bernomor urut. Migrasi yang sudah di-commit tidak diubah.
- Uji `#[sqlx::test]` ujung ke ujung ada di `src/uji_*.rs` (alat bantu: `uji::Uji`, `uji::Bdrs`). `seed_demo_berhasil` menjaga data demo tetap sah.
- Pesan galat ditulis untuk pengguna: apa yang salah dan apa yang harus dilakukan.

## Frontend (`web/`)

SvelteKit 3 + Svelte 5 runes, SPA statis. Ikuti `docs/desain.md`:

- warna hanya lewat token;
- status selalu berupa teks + warna;
- `Keadaan` untuk data, `Aksi` + `PesanAksi` untuk formulir, `Isian` untuk field, `Pindai` untuk barcode, `DialogSandi` untuk tanda tangan, `Golongan` untuk lencana golongan darah, `Kop` di atas setiap dokumen cetak;
- import `#lib/...` dengan ekstensi;
- semua teks UI berbahasa Indonesia.
