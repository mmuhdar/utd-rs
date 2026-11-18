# Tanda tangan elektronik berupa konfirmasi ulang kata sandi, terikat hash isi catatan

Aneks 2 CPOB mensyaratkan tanda tangan elektronik yang terhubung permanen ke catatan, bertanggal, dan keamanannya terkendali. Syarat ini tidak menuntut TTE tersertifikasi BSrE. Kami memakai konfirmasi ulang kata sandi petugas pada saat menandatangani. Tabel `tanda_tangan` menyimpan siapa, kapan, maknanya (mis. "verifikasi kedua uji silang", "pelulusan"), dan hash SHA-256 isi catatan saat ditandatangani. Bila catatannya kemudian dikoreksi, ketidakcocokan hash membuat tanda tangan itu tampil sebagai "tidak berlaku lagi". Verifikasi kedua mensyaratkan penanda tangan yang berbeda dari penginput.

Jejak audit append-only dan berantai hash (migrasi 0001), sehingga pemalsuan langsung di database terdeteksi oleh `audit_periksa()`.

**Considered Options**: TTE tersertifikasi (BSrE/PSrE) ditolak untuk tahap ini karena butuh layanan internet pihak ketiga di setiap tanda tangan, padahal bank darah harus tetap jalan saat internet putus. Integrasinya bisa ditambahkan untuk dokumen keluar (berita acara) bila RS memintanya.
