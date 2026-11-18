FROM postgres:17-alpine
RUN apk add --no-cache openssl
COPY backup.sh pulihkan.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/backup.sh /usr/local/bin/pulihkan.sh \
 && echo '0 1 * * * /usr/local/bin/backup.sh >> /proc/1/fd/1 2>&1' > /etc/crontabs/root
# Backup setiap hari pukul 01.00 waktu server (TZ). Tidak ada backup saat container mulai, supaya restart
# tidak menggeser backup harian keluar dari rotasi. Backup manual: docker compose exec backup backup.sh
CMD ["crond", "-f", "-l", "8"]
