#!/bin/sh
# Pemulihan backup ke database tujuan (uji pemulihan berkala, NFR-09; prosedur di docs/operasional.md).
# Pakai: pulihkan.sh <berkas.dump.enc> <nama_database_tujuan>
# --disable-triggers: baris audit dipulihkan apa adanya (trigger rantai tidak menulis ulang waktu/hash), sehingga
# audit_periksa() memeriksa rantai asli dari backup. Butuh pengguna superuser (POSTGRES_USER image resmi).
# Database tujuan dibuat baru; JANGAN memulihkan langsung ke database produksi yang sedang dipakai.
set -eu
berkas="$1"; tujuan="$2"
(cd "$(dirname "$berkas")" && sha256sum -c "$(basename "$berkas").sha256")
createdb "$tujuan"
openssl enc -d -aes-256-cbc -pbkdf2 -iter 200000 -pass env:BACKUP_PASSPHRASE -in "$berkas" | pg_restore --no-owner --disable-triggers -d "$tujuan"
psql -d "$tujuan" -Atc "select 'audit terakhir: ' || max(waktu) from audit; select 'kantong: ' || count(*) from kantong; select 'audit utuh: ' || (audit_periksa() is null)"
