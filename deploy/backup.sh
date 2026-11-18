#!/bin/sh
# Backup harian terenkripsi (NFR-09): pg_dump format custom → AES-256 (openssl, PBKDF2) → /backup, salinan ke
# /offsite, checksum SHA-256, retensi lokal BACKUP_RETENSI_HARI hari. Nama berkas = ID backup.
set -eu
id="utdrs-$(date +%Y%m%d-%H%M%S)"
tmp="/backup/.$id.tmp"
pg_dump -Fc | openssl enc -aes-256-cbc -pbkdf2 -iter 200000 -salt -pass env:BACKUP_PASSPHRASE > "$tmp"
mv "$tmp" "/backup/$id.dump.enc"
(cd /backup && sha256sum "$id.dump.enc" > "$id.dump.enc.sha256")
if [ -d /offsite ] && [ -w /offsite ]; then
  cp "/backup/$id.dump.enc" "/backup/$id.dump.enc.sha256" /offsite/
  (cd /offsite && sha256sum -c "$id.dump.enc.sha256" >/dev/null) || { echo "GAGAL: salinan offsite $id rusak"; exit 1; }
else
  echo "PERINGATAN: /offsite tidak tersedia; backup $id hanya tersimpan lokal"
fi
find /backup -name 'utdrs-*.dump.enc*' -mtime "+${BACKUP_RETENSI_HARI:-30}" -delete
echo "Backup $id selesai ($(du -h "/backup/$id.dump.enc" | cut -f1))"
