-- Mode UTD-RS (ADR 0001; SPEK M5): donor, cekal, seleksi, donasi, golongan donasi, uji saring IMLTD, look-back.

-- Ambang seleksi donor; dapat diubah penanggung jawab. Bawaan mengikuti kriteria umum seleksi donor
-- (Permenkes 91/2015 Bab III) dan wajib ditinjau sesuai SPO UTD setempat.
create table aturan_donor (
    id boolean primary key default true check (id),
    umur_min int not null default 17,
    umur_maks int not null default 65,
    berat_min numeric(4, 1) not null default 45,
    hb_min numeric(4, 1) not null default 12.5,
    hb_maks numeric(4, 1) not null default 17.0,
    sistolik_min int not null default 90,
    sistolik_maks int not null default 160,
    diastolik_min int not null default 60,
    diastolik_maks int not null default 100,
    nadi_min int not null default 50,
    nadi_maks int not null default 100,
    suhu_maks numeric(3, 1) not null default 37.5,
    hari_antar_donasi int not null default 60,
    diubah_pada timestamptz not null default now()
);
insert into aturan_donor default values;

create table donor (
    id uuid primary key default gen_random_uuid(),
    nik text not null unique check (nik ~ '^[0-9]{16}$'),
    nama text not null check (length(trim(nama)) > 0),
    tanggal_lahir date not null,
    jenis_kelamin text not null check (jenis_kelamin in ('L', 'P')),
    alamat text,
    telepon text,
    -- Golongan dari donasi terakhir yang terverifikasi.
    abo abo,
    rhesus rhesus,
    dibuat_pada timestamptz not null default now(),
    diperbarui_pada timestamptz not null default now()
);

create table cekal (
    id uuid primary key default gen_random_uuid(),
    donor_id uuid not null references donor (id),
    jenis text not null check (jenis in ('sementara', 'permanen')),
    mulai date not null default current_date,
    sampai date,
    alasan text not null,
    sumber text not null check (sumber in ('seleksi', 'imltd', 'manual')),
    dibuat_oleh uuid not null references akun (id),
    dibuat_pada timestamptz not null default now(),
    dicabut_pada timestamptz,
    dicabut_ttd uuid references tanda_tangan (id),
    alasan_cabut text,
    check ((jenis = 'sementara') = (sampai is not null))
);
create index on cekal (donor_id);

create table seleksi (
    id uuid primary key default gen_random_uuid(),
    donor_id uuid not null references donor (id),
    waktu timestamptz not null default now(),
    berat_kg numeric(4, 1) not null,
    hb numeric(4, 1) not null,
    sistolik int not null,
    diastolik int not null,
    nadi int not null,
    suhu numeric(3, 1) not null,
    kuesioner_lolos boolean not null,
    keputusan text not null check (keputusan in ('diterima', 'ditolak_sementara', 'ditolak_permanen')),
    alasan text,
    dicatat_oleh uuid not null references akun (id),
    check (keputusan = 'diterima' or alasan is not null)
);

create table donasi (
    id uuid primary key default gen_random_uuid(),
    nomor text not null unique check (length(trim(nomor)) > 0),
    donor_id uuid not null references donor (id),
    seleksi_id uuid not null unique references seleksi (id),
    jenis text not null check (jenis in ('sukarela', 'pengganti')),
    untuk_no_rm text references pasien (no_rm),
    waktu timestamptz not null default now(),
    volume_ml int not null check (volume_ml between 100 and 550),
    jenis_kantong text not null check (jenis_kantong in ('single', 'double', 'triple', 'quadruple')),
    status text not null default 'diambil' check (status in ('diambil', 'diolah', 'gagal')),
    catatan text,
    dicatat_oleh uuid not null references akun (id),
    check (jenis = 'sukarela' or untuk_no_rm is not null)
);
create index on donasi (donor_id, waktu);

alter table kantong add constraint kantong_donasi_fk foreign key (donasi_id) references donasi (id);
alter table kantong add constraint kantong_asal_donasi check ((asal = 'donasi') = (donasi_id is not null));

create table golongan_donasi (
    id uuid primary key default gen_random_uuid(),
    donasi_id uuid not null references donasi (id),
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
    batal_alasan text,
    check (batal_alasan is null or tanda_tangan_id is null)
);
create unique index on golongan_donasi (donasi_id) where batal_alasan is null;

create table uji_imltd (
    id uuid primary key default gen_random_uuid(),
    donasi_id uuid not null references donasi (id),
    parameter text not null check (parameter in ('hiv', 'hbsag', 'hcv', 'sifilis', 'malaria')),
    metode text not null check (metode in ('rapid', 'eia', 'chlia', 'nat')),
    hasil text not null check (hasil in ('nonreaktif', 'reaktif')),
    dicatat_oleh uuid not null references akun (id),
    dicatat_pada timestamptz not null default now(),
    tanda_tangan_id uuid references tanda_tangan (id),
    batal_alasan text,
    check (batal_alasan is null or tanda_tangan_id is null)
);
create unique index on uji_imltd (donasi_id, parameter) where batal_alasan is null;

-- Look-back: penelusuran dari donasi reaktif ke donasi-donasi sebelumnya dan penerimanya (BPOM 11/2025 butir 1.52–1.57).
create table lookback (
    id uuid primary key default gen_random_uuid(),
    donor_id uuid references donor (id),
    -- Untuk notifikasi dari UTD mitra: nomor donasi yang dilaporkan reaktif.
    nomor_donasi text,
    donasi_pemicu uuid references donasi (id),
    alasan text not null,
    status text not null default 'terbuka' check (status in ('terbuka', 'selesai')),
    catatan text,
    dibuat_oleh uuid not null references akun (id),
    dibuat_pada timestamptz not null default now(),
    ditutup_oleh uuid references akun (id),
    ditutup_pada timestamptz,
    check (donor_id is not null or nomor_donasi is not null)
);
