-- Fondasi: profil instalasi, akun dan peran, sesi, jejak audit append-only berantai hash.

create extension if not exists pgcrypto;

-- Satu baris per instalasi.
create table profil_instalasi (
    id boolean primary key default true check (id),
    nama_rs text not null default 'Rumah Sakit (belum diisi)',
    kode_faskes text,
    -- 'bdrs': bank darah RS yang menerima darah dari UTD; 'utd_rs': UTD milik RS yang juga mengambil darah donor.
    jenis text not null default 'bdrs' check (jenis in ('bdrs', 'utd_rs')),
    diubah_pada timestamptz not null default now()
);
insert into profil_instalasi default values;

create table akun (
    id uuid primary key default gen_random_uuid(),
    nama_pengguna text not null unique check (nama_pengguna ~ '^[a-z0-9._-]{3,40}$'),
    nama text not null check (length(trim(nama)) > 0),
    nip text,
    sandi_hash text not null,
    peran text[] not null check (peran <@ array['admin', 'dokter', 'perawat', 'teknisi', 'kepala', 'auditor']
                                 and cardinality(peran) > 0),
    aktif boolean not null default true,
    gagal_masuk int not null default 0,
    terkunci_sampai timestamptz,
    wajib_ganti_sandi boolean not null default true,
    dibuat_pada timestamptz not null default now()
);

-- Cookie hanya berisi token acak; database menyimpan hash SHA-256-nya.
create table sesi (
    hash bytea primary key,
    akun_id uuid not null references akun (id),
    dibuat_pada timestamptz not null default now(),
    terakhir_pada timestamptz not null default now(),
    berakhir_pada timestamptz not null,
    alamat text,
    perangkat text
);
create index on sesi (akun_id);

-- Jejak audit. Ditulis di transaksi yang sama dengan perubahan datanya. Setiap baris menyimpan hash baris
-- sebelumnya (menurut `urutan`), sehingga perubahan atau penghapusan baris lama terdeteksi oleh audit_periksa().
create table audit (
    id bigint generated always as identity primary key,
    urutan bigint not null unique,
    waktu timestamptz not null default clock_timestamp(),
    akun_id uuid,
    aksi text not null,
    objek text not null,
    objek_id text,
    sebelum jsonb,
    sesudah jsonb,
    alasan text,
    alamat text,
    perangkat text,
    hash_sebelum bytea,
    hash bytea not null
);
create index on audit (objek, objek_id);
create index on audit (akun_id, waktu);

create function audit_hash(a audit) returns bytea language sql immutable as $$
    select digest(coalesce(a.hash_sebelum, ''::bytea) || convert_to(concat_ws(
        '|', a.urutan, extract(epoch from a.waktu)::text, a.akun_id, a.aksi, a.objek, a.objek_id,
        a.sebelum::text, a.sesudah::text, a.alasan, a.alamat, a.perangkat), 'UTF8'), 'sha256')
$$;

-- shortcut: kunci advisory menyerialkan semua penulisan audit; cukup untuk beban bank darah satu RS.
create function audit_rantai() returns trigger language plpgsql as $$
declare
    akhir audit;
begin
    perform pg_advisory_xact_lock(7710001);
    select * into akhir from audit order by urutan desc limit 1;
    new.urutan := coalesce(akhir.urutan, 0) + 1;
    new.hash_sebelum := akhir.hash;
    new.waktu := clock_timestamp();
    new.hash := audit_hash(new);
    return new;
end $$;
create trigger audit_rantai before insert on audit for each row execute function audit_rantai();

create function audit_tolak_ubah() returns trigger language plpgsql as $$
begin
    raise exception 'tabel audit bersifat append-only';
end $$;
create trigger audit_append_only before update or delete or truncate on audit
    for each statement execute function audit_tolak_ubah();

-- Urutan pertama yang rantainya putus, atau null bila utuh.
create function audit_periksa() returns bigint language sql stable as $$
    select min(a.urutan) from audit a left join audit p on p.urutan = a.urutan - 1
    where a.hash <> audit_hash(a) or a.hash_sebelum is distinct from p.hash
       or (a.urutan > 1 and p.urutan is null)
$$;
