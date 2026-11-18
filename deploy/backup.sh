#!/bin/sh
# Backup harian terenkripsi (NFR-09), dijalankan crond setiap pukul 01.00 waktu server (TZ):
# pg_dump format custom → AES-256 (openssl, PBKDF2) → /backup, salinan ke /offsite, checksum SHA-256.
# Rotasi: setiap folder hanya menyimpan BACKUP_SIMPAN backup terbaru (bawaan 3); begitu backup baru
# selesai dan terverifikasi, backup paling lama dihapus. Nama berkas = ID backup (urut waktu).
set -eu
simpan="${BACKUP_SIMPAN:-3}"
case "$simpan" in ''|*[!0-9]*|0) echo "GAGAL: BACKUP_SIMPAN harus angka ≥ 1 (sekarang: '$simpan')"; exit 1 ;; esac

id="utdrs-$(date +%Y%m%d-%H%M%S)"
tmp="/backup/.$id.tmp"
trap 'rm -f "$tmp"' EXIT
pg_dump -Fc | openssl enc -aes-256-cbc -pbkdf2 -iter 200000 -salt -pass env:BACKUP_PASSPHRASE > "$tmp"
mv "$tmp" "/backup/$id.dump.enc"
(cd /backup && sha256sum "$id.dump.enc" > "$id.dump.enc.sha256")

# Hapus backup terlama sampai tersisa $simpan. Dipanggil hanya setelah backup baru utuh.
rotasi() {
  ls -1 "$1"/utdrs-*.dump.enc 2>/dev/null | sort -r | tail -n "+$((simpan + 1))" | while read -r lama; do
    rm -f "$lama" "$lama.sha256"
    echo "Backup lama dihapus: $lama"
  done
}

if [ -d /offsite ] && [ -w /offsite ]; then
  cp "/backup/$id.dump.enc" "/backup/$id.dump.enc.sha256" /offsite/
  (cd /offsite && sha256sum -c "$id.dump.enc.sha256" >/dev/null) || { echo "GAGAL: salinan offsite $id rusak"; exit 1; }
  rotasi /offsite
else
  echo "PERINGATAN: /offsite tidak tersedia; backup $id hanya tersimpan lokal"
fi
rotasi /backup
echo "Backup $id selesai ($(du -h "/backup/$id.dump.enc" | cut -f1)); disimpan $simpan backup terbaru"
