FROM postgres:17-alpine
RUN apk add --no-cache openssl
COPY backup.sh pulihkan.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/backup.sh /usr/local/bin/pulihkan.sh \
 && echo '0 1 * * * /usr/local/bin/backup.sh >> /proc/1/fd/1 2>&1' > /etc/crontabs/root
# Backup pertama saat container mulai, lalu setiap pukul 01.00.
CMD ["sh", "-c", "/usr/local/bin/backup.sh; exec crond -f -l 8"]
