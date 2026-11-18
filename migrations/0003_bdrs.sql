-- Alur BDRS (docs/SPEK.md M1–M4): data induk, kantong dan stok, suhu, permintaan darah, uji pratransfusi,
-- pengeluaran, transfusi, reaksi. Data tidak dihapus (ADR 0003); status diubah dengan jejak audit.

create domain komponen as text check (value in ('wb', 'prc', 'tc', 'ffp', 'cryo'));
create domain abo as text check (value in ('A', 'B', 'AB', 'O'));
create domain rhesus as text check (value in ('pos', 'neg'));
-- Hasil reaksi aglutinasi.
create domain hasil_uji as text check (value in ('pos', 'neg'));

-- Aturan per komponen (ADR 0004). Bawaan mengikuti versi regulasi yang lebih ketat.
create table aturan_komponen (
    komponen komponen primary key,
    nama text not null,
    -- Sel darah merah: kecocokan ABO sel; plasma: kecocokan ABO plasma (kebalikannya).
    berisi_sel_merah boolean not null,
    -- 'mayor_minor': mayor+minor+autokontrol; 'minor': minor saja; 'tidak_perlu': cukup kecocokan ABO/Rh.
    uji_silang text not null check (uji_silang in ('mayor_minor', 'minor', 'tidak_perlu')),
    jenis_alat text not null check (jenis_alat in ('lemari', 'freezer', 'inkubator')),
    -- Masa simpan kantong hasil pengolahan sendiri (mode UTD-RS); kantong dari UTD memakai tanggal di label.
    masa_simpan_jam int not null check (masa_simpan_jam > 0),
    -- Masa pakai setelah dicairkan (FFP/kriopresipitat), null bila tidak dicairkan.
    jam_setelah_cair int check (jam_setelah_cair > 0),
    -- Batas "mendekati kedaluwarsa" untuk peringatan stok.
    jam_peringatan int not null default 72,
    diubah_pada timestamptz not null default now()
);
insert into aturan_komponen values
    ('wb', 'Darah lengkap (WB)', true, 'mayor_minor', 'lemari', 35 * 24, null, 72, now()),
    ('prc', 'Sel darah merah pekat (PRC)', true, 'mayor_minor', 'lemari', 35 * 24, null, 72, now()),
    ('tc', 'Trombosit pekat (TC)', false, 'minor', 'inkubator', 5 * 24, null, 24, now()),
    ('ffp', 'Plasma segar beku (FFP)', false, 'minor', 'freezer', 365 * 24, 6, 720, now()),
    ('cryo', 'Kriopresipitat', false, 'minor', 'freezer', 365 * 24, 6, 720, now());

create table utd_mitra (
    id uuid primary key default gen_random_uuid(),
    kode text not null unique check (length(trim(kode)) > 0),
    nama text not null check (length(trim(nama)) > 0),
    alamat text,
    format_label text not null default 'lokal' check (format_label in ('isbt128', 'lokal')),
    aktif boolean not null default true
);

create table alat_simpan (
    id uuid primary key default gen_random_uuid(),
    kode text not null unique check (length(trim(kode)) > 0),
    nama text not null,
    nomor_seri text,
    jenis text not null check (jenis in ('lemari', 'freezer', 'inkubator')),
    suhu_min numeric(5, 1) not null,
    suhu_maks numeric(5, 1) not null check (suhu_maks > suhu_min),
    lokasi text,
    aktif boolean not null default true
);

create table pasien (
    no_rm text primary key check (length(trim(no_rm)) > 0),
    nama text not null check (length(trim(nama)) > 0),
    tanggal_lahir date not null,
    jenis_kelamin text not null check (jenis_kelamin in ('L', 'P')),
    sumber text not null default 'manual' check (sumber in ('manual', 'simrs')),
    diperbarui_pada timestamptz not null default now()
);

-- Permintaan darah BDRS ke UTD mitra.
create sequence permintaan_utd_seq;
create table permintaan_utd (
    id uuid primary key default gen_random_uuid(),
    nomor text not null unique default nomor_dok('PU', nextval('permintaan_utd_seq')),
    utd_id uuid not null references utd_mitra (id),
    jenis text not null check (jenis in ('rutin', 'khusus', 'darurat')),
    -- [{komponen, abo, rhesus, jumlah}]
    baris jsonb not null check (jsonb_typeof(baris) = 'array' and jsonb_array_length(baris) > 0),
    catatan text,
    status text not null default 'dikirim' check (status in ('dikirim', 'dipenuhi', 'batal')),
    dibuat_oleh uuid not null references akun (id),
    dibuat_pada timestamptz not null default now()
);

-- Berita serah terima penerimaan kantong dari UTD.
create sequence penerimaan_seq;
create table penerimaan (
    id uuid primary key default gen_random_uuid(),
    nomor text not null unique default nomor_dok('TR', nextval('penerimaan_seq')),
    utd_id uuid not null references utd_mitra (id),
    permintaan_utd_id uuid references permintaan_utd (id),
    nomor_dokumen_utd text,
    suhu_min_transport numeric(5, 1),
    suhu_maks_transport numeric(5, 1),
    petugas_utd text not null check (length(trim(petugas_utd)) > 0),
    catatan text,
    diterima_oleh uuid not null references akun (id),
    diterima_pada timestamptz not null default now()
);

-- Berita acara pengembalian ke UTD atau pemusnahan.
create sequence berita_acara_seq;
create table berita_acara (
    id uuid primary key default gen_random_uuid(),
    nomor text not null unique default nomor_dok('BA', nextval('berita_acara_seq')),
    jenis text not null check (jenis in ('pengembalian', 'pemusnahan')),
    utd_id uuid references utd_mitra (id),
    metode text,
    saksi text,
    catatan text,
    dibuat_oleh uuid not null references akun (id),
    dibuat_pada timestamptz not null default now(),
    check ((jenis = 'pengembalian') = (utd_id is not null))
);

create table kantong (
    id uuid primary key default gen_random_uuid(),
    -- Nomor donasi (+ kode komponen bila satu donasi menjadi beberapa kantong); unik selamanya.
    nomor text not null unique check (length(trim(nomor)) > 0),
    nomor_donasi text not null,
    komponen komponen not null,
    kode_produk text,
    abo abo not null,
    rhesus rhesus not null,
    volume_ml int check (volume_ml > 0),
    tanggal_ambil date,
    kedaluwarsa timestamptz not null,
    kedaluwarsa_label timestamptz not null,
    asal text not null check (asal in ('utd_mitra', 'donasi')),
    penerimaan_id uuid references penerimaan (id),
    -- Diisi oleh migrasi mode UTD-RS (foreign key ditambahkan di sana).
    donasi_id uuid,
    -- Hasil uji saring IMLTD UTD (dari label/dokumen): kantong hanya tersedia bila non-reaktif.
    imltd_nonreaktif boolean not null,
    status text not null check (status in
        ('karantina', 'tersedia', 'dialokasikan', 'dikeluarkan', 'ditransfusikan', 'dikembalikan', 'dimusnahkan')),
    alasan_status text,
    alat_id uuid references alat_simpan (id),
    dicairkan_pada timestamptz,
    berita_acara_id uuid references berita_acara (id),
    dibuat_pada timestamptz not null default now(),
    check ((asal = 'utd_mitra') = (penerimaan_id is not null)),
    check (status not in ('tersedia', 'dialokasikan', 'dikeluarkan') or imltd_nonreaktif),
    check ((status in ('dikembalikan', 'dimusnahkan')) = (berita_acara_id is not null))
);
create index on kantong (status, komponen, abo, rhesus, kedaluwarsa);
create index on kantong (nomor_donasi);

create table log_suhu (
    id bigint generated always as identity primary key,
    alat_id uuid not null references alat_simpan (id),
    waktu timestamptz not null,
    suhu numeric(5, 1) not null,
    sumber text not null check (sumber in ('manual', 'logger')),
    dicatat_oleh uuid not null references akun (id),
    catatan text
);
create index on log_suhu (alat_id, waktu);

create table penyimpangan_suhu (
    id uuid primary key default gen_random_uuid(),
    alat_id uuid not null references alat_simpan (id),
    mulai timestamptz not null,
    suhu_ekstrem numeric(5, 1) not null,
    status text not null default 'terbuka' check (status in ('terbuka', 'ditutup')),
    kesimpulan text,
    ditutup_oleh uuid references akun (id),
    ditutup_pada timestamptz
);
-- Satu penyimpangan terbuka per alat.
create unique index on penyimpangan_suhu (alat_id) where status = 'terbuka';

create table penilaian_kantong (
    penyimpangan_id uuid not null references penyimpangan_suhu (id),
    kantong_id uuid not null references kantong (id),
    keputusan text check (keputusan in ('layak', 'tidak_layak')),
    alasan text,
    dinilai_oleh uuid references akun (id),
    dinilai_pada timestamptz,
    primary key (penyimpangan_id, kantong_id)
);

create sequence permintaan_seq;
create table permintaan (
    id uuid primary key default gen_random_uuid(),
    nomor text not null unique default nomor_dok('PD', nextval('permintaan_seq')),
    no_rm text not null references pasien (no_rm),
    ruang text not null check (length(trim(ruang)) > 0),
    dokter text not null check (length(trim(dokter)) > 0),
    dokter_akun_id uuid references akun (id),
    diagnosis text not null check (length(trim(diagnosis)) > 0),
    indikasi text not null check (length(trim(indikasi)) > 0),
    jenis text not null check (jenis in ('elektif', 'rutin', 'darurat')),
    alasan_darurat text,
    hb numeric(4, 1),
    trombosit int,
    abo_klinis abo,
    rhesus_klinis rhesus,
    riwayat_transfusi boolean not null,
    riwayat_reaksi boolean not null,
    rencana_transfusi date not null,
    -- [{komponen, jumlah, volume_ml}]
    baris jsonb not null check (jsonb_typeof(baris) = 'array' and jsonb_array_length(baris) > 0),
    status text not null default 'diminta'
        check (status in ('diminta', 'diproses', 'siap', 'selesai', 'batal', 'ditolak')),
    alasan_status text,
    sumber text not null default 'utd_rs' check (sumber in ('utd_rs', 'simrs')),
    rujukan_simrs text,
    diminta_oleh uuid not null references akun (id),
    diminta_pada timestamptz not null default now(),
    check (jenis <> 'darurat' or length(trim(coalesce(alasan_darurat, ''))) > 0)
);
create index on permintaan (status, diminta_pada);
create index on permintaan (no_rm);

create table sampel (
    id uuid primary key default gen_random_uuid(),
    permintaan_id uuid not null references permintaan (id),
    diambil_pada timestamptz not null,
    diambil_oleh text not null check (length(trim(diambil_oleh)) > 0),
    jenis text not null check (jenis in ('edta', 'serum')),
    penyimpanan text not null check (penyimpanan in ('suhu_ruang', 'dingin')),
    -- Batas pakai (Permenkes 91/2015 §5.6.2 no. 4), dihitung saat sampel diterima.
    batas_pakai timestamptz not null,
    status text not null check (status in ('diterima', 'ditolak', 'diskrepansi')),
    alasan_tolak text,
    diterima_oleh uuid not null references akun (id),
    diterima_pada timestamptz not null default now(),
    check (jenis = 'edta' or penyimpanan = 'dingin'),
    check (status = 'diterima' or alasan_tolak is not null)
);

create table pemeriksaan_golongan (
    id uuid primary key default gen_random_uuid(),
    sampel_id uuid not null references sampel (id),
    no_rm text not null references pasien (no_rm),
    -- Reaksi sel pasien dengan anti-A/anti-B dan serum pasien dengan sel A/B: 'pos' atau 'neg'.
    anti_a hasil_uji not null,
    anti_b hasil_uji not null,
    sel_a hasil_uji not null,
    sel_b hasil_uji not null,
    anti_d hasil_uji not null,
    abo abo not null,
    rhesus rhesus not null,
    metode text not null check (metode in ('tabung', 'gel', 'bioplate')),
    dicatat_oleh uuid not null references akun (id),
    dicatat_pada timestamptz not null default now(),
    tanda_tangan_id uuid references tanda_tangan (id),
    diskrepansi boolean not null default false,
    -- Ditolak verifikator kedua; hasil boleh diinput ulang.
    batal_alasan text,
    check (batal_alasan is null or tanda_tangan_id is null)
);
create index on pemeriksaan_golongan (no_rm, dicatat_pada);
create unique index on pemeriksaan_golongan (sampel_id) where batal_alasan is null;

create table uji_silang (
    id uuid primary key default gen_random_uuid(),
    permintaan_id uuid not null references permintaan (id),
    sampel_id uuid not null references sampel (id),
    kantong_id uuid not null references kantong (id),
    metode text not null check (metode in ('tabung', 'gel', 'immediate_spin')),
    -- Konfirmasi golongan kantong dari selang.
    abo_kantong abo not null,
    rhesus_kantong rhesus not null,
    mayor hasil_uji,
    minor hasil_uji,
    autokontrol hasil_uji,
    -- Coombs control cell untuk uji tabung negatif: harus positif.
    ccc_valid boolean,
    kesimpulan text not null check (kesimpulan in ('serasi', 'hanya_prc', 'tidak_serasi', 'golongan_beda')),
    dicatat_oleh uuid not null references akun (id),
    dicatat_pada timestamptz not null default now(),
    tanda_tangan_id uuid references tanda_tangan (id),
    batal_alasan text,
    check (batal_alasan is null or tanda_tangan_id is null)
);
create unique index on uji_silang (sampel_id, kantong_id) where batal_alasan is null;
create index on uji_silang (kantong_id);
create index on uji_silang (permintaan_id);

create table alokasi (
    id uuid primary key default gen_random_uuid(),
    permintaan_id uuid not null references permintaan (id),
    kantong_id uuid not null references kantong (id),
    uji_silang_id uuid references uji_silang (id),
    -- Override darurat: golongan pasien belum terverifikasi atau Rh tidak sesuai; disetujui dokter.
    darurat boolean not null default false,
    alasan_darurat text,
    disetujui_ttd uuid references tanda_tangan (id),
    dialokasikan_oleh uuid not null references akun (id),
    dialokasikan_pada timestamptz not null default now(),
    dilepas_pada timestamptz,
    alasan_lepas text,
    check (not darurat or disetujui_ttd is not null)
);
-- Satu alokasi aktif per kantong.
create unique index on alokasi (kantong_id) where dilepas_pada is null;

create table pengeluaran (
    id uuid primary key default gen_random_uuid(),
    alokasi_id uuid not null unique references alokasi (id),
    kantong_id uuid not null unique references kantong (id),
    dikeluarkan_oleh uuid not null references akun (id),
    dikeluarkan_pada timestamptz not null default now(),
    penerima_ruang text not null check (length(trim(penerima_ruang)) > 0)
);

create table insiden (
    id uuid primary key default gen_random_uuid(),
    jenis text not null check (jenis in ('knc', 'ktc', 'ktd', 'kpc')),
    waktu timestamptz not null default now(),
    uraian text not null,
    no_rm text references pasien (no_rm),
    kantong_id uuid references kantong (id),
    dilaporkan_oleh uuid not null references akun (id)
);

create table transfusi (
    id uuid primary key default gen_random_uuid(),
    pengeluaran_id uuid not null unique references pengeluaran (id),
    kantong_id uuid not null unique references kantong (id),
    no_rm text not null references pasien (no_rm),
    petugas_1 uuid not null references akun (id),
    ttd_petugas_2 uuid not null references tanda_tangan (id),
    rawat_jalan boolean not null default false,
    mulai timestamptz not null default now(),
    selesai timestamptz,
    dihentikan boolean not null default false,
    volume_masuk_ml int,
    catatan text
);

create table observasi (
    id bigint generated always as identity primary key,
    transfusi_id uuid not null references transfusi (id),
    tahap text not null check (tahap in ('pra', '15_menit', 'berkala', 'selesai', 'pasca')),
    waktu timestamptz not null default now(),
    suhu numeric(4, 1),
    nadi int,
    sistolik int,
    diastolik int,
    napas int,
    keluhan text,
    dicatat_oleh uuid not null references akun (id)
);

create sequence reaksi_seq;
create table reaksi (
    id uuid primary key default gen_random_uuid(),
    nomor text not null unique default nomor_dok('RT', nextval('reaksi_seq')),
    transfusi_id uuid not null references transfusi (id),
    waktu_reaksi timestamptz not null,
    gejala text[] not null check (cardinality(gejala) > 0),
    tindakan text not null,
    kategori text not null check (kategori in ('akut', 'lambat')),
    dilaporkan_oleh uuid not null references akun (id),
    dilaporkan_pada timestamptz not null default now(),
    status text not null default 'dilaporkan' check (status in ('dilaporkan', 'ditelusuri', 'ditutup')),
    -- Penelusuran BDRS (Permenkes 91/2015 §5.9).
    sisa_volume_ml int,
    abo_pasien_ulang abo,
    rhesus_pasien_ulang rhesus,
    abo_kantong_ulang abo,
    rhesus_kantong_ulang rhesus,
    uji_silang_ulang text check (uji_silang_ulang in ('serasi', 'tidak_serasi', 'tidak_dilakukan')),
    kesimpulan text,
    ditelusuri_oleh uuid references akun (id),
    ditelusuri_pada timestamptz,
    ttd_penutupan uuid references tanda_tangan (id)
);
