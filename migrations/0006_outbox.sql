-- Outbox integrasi SATUSEHAT (ADR 0006): ditulis di transaksi yang sama dengan peristiwanya, dikirim oleh pekerja latar.
create table outbox (
    id bigint generated always as identity primary key,
    tujuan text not null default 'satusehat' check (tujuan in ('satusehat')),
    peristiwa text not null check (peristiwa in ('lulus', 'musnah')),
    kantong_id uuid not null references kantong (id),
    payload jsonb not null,
    status text not null default 'antre' check (status in ('antre', 'terkirim', 'gagal')),
    percobaan int not null default 0,
    galat text,
    berikut_pada timestamptz not null default now(),
    dibuat_pada timestamptz not null default now(),
    terkirim_pada timestamptz,
    respons jsonb
);
create index on outbox (status, berikut_pada);
-- Id resource BiologicallyDerivedProduct di SATUSEHAT, untuk pembaruan status (distribusi/pemusnahan).
alter table kantong add column satusehat_id text;
