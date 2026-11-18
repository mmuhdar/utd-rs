-- Kebijakan kata sandi yang diatur admin (Pengaturan → Kebijakan sandi). Satu baris per instalasi.
-- Bawaan 10 karakter; batas bawah 4 supaya RS yang memakai PIN pendek tetap bisa, dengan peringatan di layar.
create table kebijakan_sandi (
    id boolean primary key default true check (id),
    panjang_min int not null default 10 check (panjang_min between 4 and 64),
    diubah_pada timestamptz not null default now()
);
insert into kebijakan_sandi default values;
