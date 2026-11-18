#!/bin/sh
# Backup harian terenkripsi (NFR-09), dijalankan crond setiap pukul 01.00 waktu server (TZ):
# pg_dump format custom → AES-256 (openssl, PBKDF2) → /backup, salinan ke /offsite, checksum SHA-256.
# Retensi: backup yang lebih tua dari BACKUP_RETENSI_HARI hari (bawaan 30) dihapus dari kedua folder,
# hanya setelah backup baru selesai dan terverifikasi. Nama berkas = ID backup.
set -eu
retensi="${BACKUP_RETENSI_HARI:-30}"
case "$retensi" in ''|*[!0-9]*|0) echo "GAGAL: BACKUP_RETENSI_HARI harus angka ≥ 1 (sekarang: '$retensi')"; exit 1 ;; esac

id="utdrs-$(date +%Y%m%d-%H%M%S)"
tmp="/backup/.$id.tmp"
trap 'rm -f "$tmp"' EXIT
pg_dump -Fc | openssl enc -aes-256-cbc -pbkdf2 -iter 200000 -salt -pass env:BACKUP_PASSPHRASE > "$tmp"
mv "$tmp" "/backup/$id.dump.enc"
(cd /backup && sha256sum "$id.dump.enc" > "$id.dump.enc.sha256")

# Hapus backup yang lebih tua dari $retensi hari. Dipanggil hanya setelah backup baru utuh.
bersihkan() {
  find "$1" -maxdepth 1 -name 'utdrs-*.dump.enc*' -mtime "+$retensi" -print -delete | sed 's/^/Backup lama dihapus: /'
}

if [ -d /offsite ] && [ -w /offsite ]; then
  cp "/backup/$id.dump.enc" "/backup/$id.dump.enc.sha256" /offsite/
  (cd /offsite && sha256sum -c "$id.dump.enc.sha256" >/dev/null) || { echo "GAGAL: salinan offsite $id rusak"; exit 1; }
  bersihkan /offsite
else
  echo "PERINGATAN: /offsite tidak tersedia; backup $id hanya tersimpan lokal"
fi
bersihkan /backup
echo "Backup $id selesai ($(du -h "/backup/$id.dump.enc" | cut -f1)); retensi $retensi hari"
