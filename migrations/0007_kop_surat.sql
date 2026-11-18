-- Kop surat untuk semua dokumen cetak (berita acara, formulir, laporan, lembar darurat). Satu baris per instalasi,
-- seperti profil_instalasi. Teks kosong = baris kop tidak dicetak; nama_rs kosong = memakai profil_instalasi.nama_rs.
-- Logo disimpan sebagai data URI PNG/JPEG (maks. 512 KB) supaya tetap berjalan offline tanpa penyimpanan berkas.
create table kop_surat (
    id boolean primary key default true check (id),
    pemerintah text not null default '',
    nama_rs text not null default '',
    unit text not null default 'Instalasi Bank Darah',
    alamat text not null default '',
    kontak text not null default '',
    akreditasi text not null default '',
    logo_kiri text not null default '',
    logo_kanan text not null default '',
    diubah_pada timestamptz not null default now()
);
insert into kop_surat default values;
