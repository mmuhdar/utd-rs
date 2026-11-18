-- Akun layanan (SIMRS, data logger suhu) dengan token API; tanda tangan elektronik (ADR 0005); nomor dokumen.

alter table akun drop constraint akun_peran_check;
alter table akun add constraint akun_peran_check check (
    peran <@ array['admin', 'dokter', 'perawat', 'teknisi', 'kepala', 'auditor', 'simrs', 'logger']
    and cardinality(peran) > 0);

-- Token API hanya untuk akun layanan; disimpan sebagai hash SHA-256.
create table token_api (
    hash bytea primary key,
    akun_id uuid not null references akun (id),
    dibuat_pada timestamptz not null default now(),
    dicabut_pada timestamptz,
    terakhir_dipakai timestamptz
);

-- Tanda tangan elektronik: siapa, kapan, maknanya, dan hash isi catatan saat ditandatangani.
create table tanda_tangan (
    id uuid primary key default gen_random_uuid(),
    akun_id uuid not null references akun (id),
    waktu timestamptz not null default now(),
    makna text not null,
    objek text not null,
    objek_id text not null,
    hash_isi bytea not null
);
create index on tanda_tangan (objek, objek_id);

-- Nomor dokumen: AWALAN-YYMMDD-NNNN; angka tidak dipotong bila melebihi 4 digit.
create function nomor_dok(awalan text, n bigint) returns text language sql stable as $$
    select awalan || '-' || to_char(now() at time zone 'Asia/Jakarta', 'YYMMDD') || '-'
        || case when length(n::text) >= 4 then n::text else lpad(n::text, 4, '0') end
$$;
