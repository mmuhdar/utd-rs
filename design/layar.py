#!/usr/bin/env python3
"""Layar-layar desain utd-rs. Jalankan: python3 design/layar.py [nama-layar ...]"""
import subprocess
import sys

from bangun import *  # noqa: F403

D = Doc()
GAP = 160  # jarak antar layar di kanvas


def beranda(x, y):
    root, m = D.screen("02 Beranda", x, y, "Beranda")
    D.topbar(m, ["Beranda"], "Selamat pagi, dr. Kepala", "Rabu, 18 November 2026 · Shift pagi 07.00–14.00",
             [("Lembar darurat", "secondary", "printer"), ("Buat permintaan", "primary", "plus")])

    al = D.col(m, 10, "peringatan", sizingH="fill")
    D.alert(al, "bahaya", "Penyimpangan suhu di Lemari darah 2",
            "Tercatat 7,4 °C pukul 09.12 (batas 2–6 °C). 14 kantong di alat ini otomatis dikarantina sampai dinilai.",
            "Nilai sekarang", icon="thermometer")
    D.alert(al, "waspada", "1 kantong lebih dari 30 menit di luar bank darah",
            "G9999262000214 · PRC A+ · Ruang Melati — transfusi belum dimulai sejak 08.41.", "Hubungi ruang", icon="clock")

    kpi = D.row(m, 16, "angka-utama", sizingH="fill")
    for judul, nilai, ket, ik, tone in [
        ("Kantong tersedia", "128", "+12 dari kemarin", "package", "aman"),
        ("Segera kedaluwarsa", "9", "≤ 3 hari · utamakan keluar", "hourglass", "waspada"),
        ("Permintaan berjalan", "6", "2 di antaranya cito", "clipboard-list", "info"),
        ("Karantina", "14", "Menunggu penilaian suhu", "shield-check", "bahaya"),
    ]:
        c = D.card(kpi, gap=10, pad=18, name=f"kpi/{judul}")
        h = D.row(c, 10, "atas", sizingH="fill", mainAlign="spaceBetween")
        D.text(h, judul, 13, "SemiBold", MUTED)
        bg, fg, dt = TONE[tone]
        ib = D.row(h, 0, "ikon", fill=bg, padding=7, cornerRadius=9)
        D.icon(ib, ik, 16, fg)
        D.text(c, nilai, 34, "ExtraBold", INK)
        D.text(c, ket, 12, "Medium", fg if tone != "aman" else "#05603A")

    stok = D.card(m, "Stok tersedia per golongan", "Rincian stok", sub="Semua komponen · target dari perencanaan stok bulan ini")
    grid = D.row(stok, 12, "golongan", sizingH="fill")
    data = [("O", "+", 41, 52, (24, 8, 9)), ("O", "-", 4, 10, (3, 0, 1)), ("A", "+", 29, 30, (16, 5, 8)), ("A", "-", 2, 6, (2, 0, 0)),
            ("B", "+", 33, 30, (18, 6, 9)), ("B", "-", 3, 6, (2, 0, 1)), ("AB", "+", 14, 12, (7, 2, 5)), ("AB", "-", 2, 4, (1, 0, 1))]
    for abo, rh, n, target, (prc, tc, ffp) in data:
        kurang = n < target * 0.6
        g = D.col(grid, 10, f"gol/{abo}{rh}", sizingH="fill", padding=14, cornerRadius=12, fill="#FAFBFC",
                  stroke="#FDA29B" if kurang else LINE2, strokeWeight=1, strokeAlign="inside")
        D.gol(g, abo, rh)
        r = D.row(g, 4, "jumlah", crossAlign="baseline")
        D.text(r, str(n), 26, "ExtraBold", "#B42318" if kurang else INK)
        D.text(r, "ktg", 12, "SemiBold", MUTED)
        bar = D.frame(g, "batang", 10, 6, fill=LINE, cornerRadius=3, sizingH="fill", clip=True, layout="horizontal")
        D.rect(bar, max(4, int(92 * min(1, n / target))), 6, "#F04438" if kurang else "#12B76A", "isi", cornerRadius=3)
        D.text(g, f"PRC {prc} · TC {tc} · FFP {ffp}", 11, "Medium", MUTED)

    bawah = D.row(m, 16, "bawah", sizingH="fill", crossAlign="min")
    pm = D.card(bawah, "Permintaan berjalan", "Semua permintaan", sub="Diperbarui otomatis tiap menit")
    rows = [
        ("Siti Aminah", "00-12-34-56", "IGD", "PRC 2 ktg", ("A", "+"), ("Uji silang", "info"), True, "12 mnt"),
        ("Budi Santoso", "00-98-76-11", "ICU", "TC 4 ktg", ("O", "+"), ("Menunggu sampel", "waspada"), True, "25 mnt"),
        ("Rahmat Hidayat", "00-55-21-07", "Melati 4", "PRC 1 ktg", ("B", "+"), ("Siap diambil", "aman"), False, "1 jam"),
        ("Dewi Lestari", "00-31-08-90", "Bersalin", "FFP 2 ktg", ("O", "-"), ("Uji silang", "info"), False, "2 jam"),
    ]
    cols = [("Pasien", "fill", "min"), ("Ruang", 90, "min"), ("Permintaan", 130, "min"), ("Status", 210, "min"), ("", 56, "max")]
    D.table(pm, cols, [[
        (lambda c, r=r: (lambda k: (D.text(k, r[0], 14, "SemiBold"), D.text(k, r[1], 12, "Medium", MUTED, mono=True)))(D.col(c, 2, "pasien"))),
        (lambda c, r=r: D.text(c, r[2], 13, "Medium", INK2)),
        (lambda c, r=r: (D.gol(c, *r[4], size="sm"), D.text(c, r[3], 13, "Medium", INK2))),
        (lambda c, r=r: (D.chip(c, r[5][0], r[5][1]), r[6] and D.chip(c, "Cito", "bahaya", dot=False))),
        (lambda c, r=r: D.text(c, r[7], 12, "Medium", MUTED)),
    ] for r in rows])

    kanan = D.col(bawah, 16, "kanan", width=380)
    D.cmd("node.set", ids=[kanan], sizingH="fixed", width=380)
    su = D.card(kanan, "Suhu alat simpan", "Semua alat", gap=12)
    for nama, suhu, rentang, tone, label in [
        ("Lemari darah 1", "4,1 °C", "2–6 °C", "aman", "Normal"),
        ("Lemari darah 2", "7,4 °C", "2–6 °C", "bahaya", "Menyimpang"),
        ("Freezer plasma", "−31,6 °C", "≤ −25 °C", "aman", "Normal"),
        ("Agitator trombosit", "22,4 °C", "20–24 °C", "aman", "Normal"),
    ]:
        r = D.row(su, 12, f"alat/{nama}", sizingH="fill", padding=[10, 12], cornerRadius=10,
                  fill=TONE["bahaya"][0] if tone == "bahaya" else "#FAFBFC")
        bg, fg, dt = TONE[tone]
        D.icon(r, "thermometer", 18, fg if tone == "bahaya" else MUTED)
        t = D.col(r, 1, "nama", sizingH="fill")
        D.text(t, nama, 13, "SemiBold")
        D.text(t, f"Rentang {rentang}", 12, "Medium", MUTED)
        v = D.col(r, 2, "nilai", crossAlign="max")
        D.text(v, suhu, 15, "Bold", fg if tone == "bahaya" else INK, features={"tnum": True})
        D.chip(v, label, tone, size=11)
    ex = D.card(kanan, "Segera kedaluwarsa", "Stok", gap=10)
    for no, komp, g, sisa, tone in [("G9999262000031", "TC", ("O", "+"), "Hari ini 23.59", "bahaya"),
                                    ("G9999262000088", "PRC", ("A", "+"), "2 hari lagi", "waspada"),
                                    ("G9999262000102", "PRC", ("B", "+"), "3 hari lagi", "waspada")]:
        r = D.row(ex, 10, f"kantong/{no}", sizingH="fill")
        D.gol(r, *g, size="sm")
        t = D.col(r, 1, "id", sizingH="fill")
        D.text(t, no, 13, "Medium", INK, mono=True)
        D.text(t, komp, 12, "Medium", MUTED)
        D.chip(r, sisa, tone, size=11)
    return root


def masuk(x, y):
    root = D.frame(None, "01 Masuk", 1440, 960, x, y, fill=SURF, layout="horizontal", clip=True)
    kiri = D.col(root, 0, "merek", padding=56, sizingH="fill", sizingV="fill", mainAlign="spaceBetween",
                 fills=[{"type": "linearGradient", "stops": ["#2A0612", "#7A0B25", "#C2133A"], "from": [0, 1], "to": [1, 0]}])
    D.logo(kiri, size=44)
    tengah = D.col(kiri, 20, "pesan", sizingH="fill")
    D.text(tengah, "Setiap kantong\ntertelusur, dari UTD\nsampai samping\ntempat tidur.", 52, "ExtraBold", "#FFFFFF", lineHeight="112%")
    D.text(tengah, "Sistem informasi bank darah rumah sakit sesuai Permenkes 91 Tahun 2015 — penerimaan, uji pratransfusi, pengeluaran, dan pemantauan transfusi dalam satu alur.",
           17, "Medium", "#FFD5DE", w=520, lineHeight="155%")
    fitur = D.row(kiri, 10, "fitur", wrap=True)
    for ik, t in [("shield-check", "Jejak audit berantai"), ("scan-barcode", "Pindai ISBT 128"), ("pen-line", "Tanda tangan elektronik")]:
        c = D.row(fitur, 8, f"fitur/{t}", fill="#FFFFFF1F", padding=[8, 14], cornerRadius=999, stroke="#FFFFFF33", strokeWeight=1, strokeAlign="inside")
        D.icon(c, ik, 16, "#FFFFFF")
        D.text(c, t, 13, "SemiBold", "#FFFFFF")

    kanan = D.col(root, 0, "formulir", width=620, sizingV="fill", mainAlign="center", crossAlign="center")
    D.cmd("node.set", ids=[kanan], sizingH="fixed", width=620)
    f = D.col(kanan, 22, "isi", width=400)
    D.cmd("node.set", ids=[f], sizingH="fixed", width=400)
    h = D.col(f, 6, "judul", sizingH="fill")
    D.text(h, "Masuk", 32, "ExtraBold")
    D.text(h, "RS Demo Sejahtera · Instalasi Bank Darah", 15, "Medium", MUTED)
    D.field(f, "Nama pengguna", "teknisi", icon="user", big=True)
    D.field(f, "Kata sandi", "••••••••••••", icon="lock", big=True, hint="Akun terkunci 15 menit setelah 5 kali salah sandi.")
    D.button(f, "Masuk", "primary", "log-out", big=True, w="fill")
    D.divider(f)
    r = D.row(f, 10, "info", crossAlign="min", sizingH="fill")
    D.icon(r, "shield-check", 18, MUTED)
    D.text(r, "Sesi berakhir otomatis setelah 30 menit tanpa aktivitas. Semua akses ke data pasien dicatat.", 13, "Medium", MUTED, w="fill", lineHeight="150%")
    return root


def stok(x, y):
    root, m = D.screen("03 Stok kantong", x, y, "Stok kantong")
    D.topbar(m, ["Bank darah", "Stok kantong"], "Stok kantong", "128 tersedia · 14 karantina · 6 dialokasikan",
             [("Ekspor", "secondary", "file-text"), ("Terima dari UTD", "primary", "truck")])
    isi = D.row(m, 16, "isi", sizingH="fill", crossAlign="min")
    kiri = D.col(isi, 14, "daftar", sizingH="fill")
    # Pemindai: bidang besar, selalu fokus
    sc = D.row(kiri, 14, "pemindai", fill=SURF, padding=[14, 16], cornerRadius=14, sizingH="fill",
               stroke=FOCUS, strokeWeight=2, strokeAlign="inside", effects=[{"type": "dropShadow", "color": "#2E6BFF26", "offset": [0, 0], "radius": 0, "spread": 4}])
    ib = D.row(sc, 0, "ikon", fill="#EAF1FF", padding=10, cornerRadius=10)
    D.icon(ib, "scan-barcode", 22, FOCUS)
    t = D.col(sc, 2, "teks", sizingH="fill")
    D.text(t, "Pindai label kantong atau ketik nomornya", 15, "SemiBold")
    D.text(t, "Pemindai siap · tekan Enter untuk mencari", 12, "Medium", MUTED)
    kb = D.row(sc, 4, "pintasan", fill="#F2F4F7", padding=[4, 8], cornerRadius=6)
    D.text(kb, "F2", 12, "Bold", INK2, mono=True)

    fl = D.row(kiri, 10, "saring", sizingH="fill")
    seg = D.row(fl, 2, "komponen", fill="#E9ECF1", padding=3, cornerRadius=10)
    for i, k in enumerate(["Semua", "PRC", "TC", "FFP", "WB", "Krio"]):
        b = D.row(seg, 0, f"seg/{k}", padding=[6, 12], cornerRadius=8, fill=SURF if i == 1 else "none",
                  **({"effects": SHADOW_SM} if i == 1 else {}))
        D.text(b, k, 13, "SemiBold" if i == 1 else "Medium", INK if i == 1 else MUTED)
    for lbl in ["Golongan", "Status: tersedia", "Alat"]:
        b = D.row(fl, 6, f"filter/{lbl}", fill=SURF, padding=[7, 12], cornerRadius=10, stroke="#D0D5DD", strokeWeight=1, strokeAlign="inside")
        D.text(b, lbl, 13, "Medium", INK2)
        D.icon(b, "chevron-down", 14, MUTED)

    tc = D.card(kiri, pad=8, gap=0)
    cols = [("Nomor kantong", "fill", "min"), ("Gol.", 60, "min"), ("Vol.", 64, "min"), ("Kedaluwarsa", 140, "min"), ("Lokasi", 112, "min"), ("Status", 118, "min")]
    data = [
        ("G9999262000031", "TC", ("O", "+"), "50 ml", "Hari ini 23.59", "bahaya", "Agitator 1", ("Tersedia", "aman")),
        ("G9999262000088", "PRC", ("A", "+"), "250 ml", "20 Nov · 2 hari", "waspada", "Lemari 1 / R2", ("Tersedia", "aman")),
        ("G9999262000102", "PRC", ("B", "+"), "250 ml", "21 Nov · 3 hari", "waspada", "Lemari 1 / R3", ("Dialokasikan", "info")),
        ("G9999262000117", "PRC", ("O", "-"), "250 ml", "4 Des", None, "Lemari 2 / R1", ("Karantina", "waspada")),
        ("G9999262000123", "FFP", ("AB", "+"), "200 ml", "12 Mei 2027", None, "Freezer 1 / L2", ("Tersedia", "aman")),
        ("G9999262000130", "PRC", ("A", "+"), "250 ml", "6 Des", None, "Lemari 1 / R2", ("Tersedia", "aman")),
        ("G9999262000145", "PRC", ("O", "+"), "250 ml", "8 Des", None, "Lemari 1 / R1", ("Tersedia", "aman")),
        ("G9999262000152", "TC", ("B", "+"), "50 ml", "22 Nov · 4 hari", None, "Agitator 1", ("Tersedia", "aman")),
        ("G9999262000168", "FFP", ("O", "+"), "200 ml", "2 Apr 2027", None, "Freezer 1 / L1", ("Tersedia", "aman")),
    ]
    rows = []
    for i, r in enumerate(data):
        cells = [
            (lambda c, r=r: (lambda k: (D.text(k, r[0], 14, "SemiBold", INK, mono=True), D.text(k, r[1], 12, "SemiBold", MUTED)))(D.col(c, 2, "id"))),
            (lambda c, r=r: D.gol(c, *r[2], size="sm")),
            (lambda c, r=r: D.text(c, r[3], 13, "Medium", INK2)),
            (lambda c, r=r: (D.icon(c, "hourglass", 14, TONE[r[5]][1]), D.text(c, r[4], 13, "SemiBold", TONE[r[5]][1])) if r[5] else D.text(c, r[4], 13, "Medium", INK2)),
            (lambda c, r=r: D.text(c, r[6], 13, "Medium", INK2)),
            (lambda c, r=r: D.chip(c, *r[7])),
        ]
        rows.append({"cells": cells} if i == 1 else cells)
    D.table(tc, cols, rows)
    pg = D.row(tc, 8, "halaman", sizingH="fill", mainAlign="spaceBetween", padding=[14, 16, 8, 16])
    D.text(pg, "Menampilkan 1–9 dari 148 · diurutkan kedaluwarsa terdekat (FEFO)", 13, "Medium", MUTED)
    nav = D.row(pg, 6, "nav")
    for lbl, on in [("‹", False), ("1", True), ("2", False), ("3", False), ("…", False), ("17", False), ("›", False)]:
        b = D.row(nav, 0, f"hal/{lbl}", padding=[5, 10], cornerRadius=8, fill=INK if on else "none")
        D.text(b, lbl, 13, "SemiBold", "#FFFFFF" if on else INK2)

    # Laci rincian kantong terpilih
    dr = D.card(isi, w=380, gap=18, name="laci-rincian")
    h = D.row(dr, 8, "kepala", sizingH="fill", mainAlign="spaceBetween")
    D.text(h, "Rincian kantong", 13, "Bold", MUTED, letterSpacing="4%", textCase="upper")
    D.icon(h, "x", 18, MUTED)
    idr = D.row(dr, 14, "identitas", sizingH="fill")
    D.gol(idr, "A", "+", "lg")
    t = D.col(idr, 3, "teks")
    D.text(t, "G9999262000088", 18, "Bold", INK, mono=True)
    D.text(t, "PRC · 250 ml · UTD Kota Demo", 13, "Medium", MUTED)
    D.alert(dr, "waspada", "Kedaluwarsa 20 Nov 2026, 23.59", "Utamakan untuk permintaan berikutnya yang cocok.", icon="hourglass")
    kv = D.col(dr, 10, "data", sizingH="fill")
    for k, v, mono in [("Donasi", "18 Okt 2026", False), ("Diterima", "19 Okt 2026 · BA-261019-0003", False), ("Lokasi", "Lemari darah 1 · Rak 2", False),
                       ("Uji saring IMLTD", "Non-reaktif (HIV, HBsAg, HCV, Sifilis)", False), ("Kode produk", "E0336V00", True)]:
        r = D.row(kv, 12, f"kv/{k}", sizingH="fill", crossAlign="min")
        D.text(r, k, 13, "Medium", MUTED, w=120)
        D.text(r, v, 13, "SemiBold", INK, mono=mono, w="fill")
    D.divider(dr)
    D.text(dr, "Riwayat", 14, "Bold")
    tl = D.col(dr, 0, "linimasa", sizingH="fill")
    ev = [("Diterima dari UTD Kota Demo", "19 Okt · 10.12 · Teknisi Satu", "aman"),
          ("Disimpan di Lemari darah 1", "19 Okt · 10.15 · Teknisi Satu", "netral"),
          ("Dipindah ke Rak 2", "2 Nov · 08.03 · Teknisi Dua", "netral"),
          ("Dicadangkan untuk uji silang PD-261118-0001", "18 Nov · 08.55 · Teknisi Satu", "info")]
    for i, (a, b, tone) in enumerate(ev):
        r = D.row(tl, 12, f"ev/{i}", sizingH="fill", crossAlign="min")
        dotc = D.col(r, 4, "penanda", crossAlign="center", paddingTop=4)
        D.frame(dotc, "titik", 10, 10, fill=TONE[tone][2], cornerRadius=5)
        if i < len(ev) - 1:
            D.rect(dotc, 2, 34, LINE, "garis")
        t = D.col(r, 2, "isi", sizingH="fill")
        D.text(t, a, 13, "SemiBold", INK, w="fill")
        D.text(t, b, 12, "Medium", MUTED)
    ak = D.row(dr, 8, "aksi", sizingH="fill")
    D.button(ak, "Lacak", "secondary", "scan-search", w="fill")
    D.button(ak, "Karantina", "secondary", "shield-check", w="fill")
    return root


def langkah(parent, aktif):
    st = D.row(parent, 0, "tahap", sizingH="fill", fill=SURF, padding=[14, 18], cornerRadius=14, stroke=LINE, strokeWeight=1, strokeAlign="inside")
    nama = ["Sampel", "Golongan darah", "Uji silang serasi", "Alokasi", "Pengeluaran", "Transfusi"]
    for i, n in enumerate(nama):
        done, on = i < aktif, i == aktif
        s = D.row(st, 10, f"tahap/{n}", sizingH="fill")
        b = D.row(s, 0, "nomor", fill="#12B76A" if done else (INK if on else "#F2F4F7"), cornerRadius=14, mainAlign="center")
        D.cmd("node.set", ids=[b], sizingH="fixed", sizingV="fixed", width=28, height=28)
        if done:
            D.icon(b, "check", 16, "#FFFFFF", 3)
        else:
            D.text(b, str(i + 1), 13, "Bold", "#FFFFFF" if on else MUTED)
        t = D.col(s, 1, "teks")
        D.text(t, n, 13, "Bold" if on else "SemiBold", INK if (on or done) else MUTED)
        D.text(t, ["Diterima 08.32", "A+ terverifikasi", "1 dari 2 kantong", "—", "—", "—"][i], 11, "Medium", MUTED)
        if i < len(nama) - 1:
            D.rect(s, 10, 2, "#12B76A" if done else LINE, "sambung", sizingH="fill", cornerRadius=1)
    return st


def permintaan(x, y, dialog=False):
    root, m = D.screen("05 Permintaan — tanda tangan" if dialog else "04 Permintaan — uji silang", x, y, "Permintaan darah")
    D.topbar(m, ["Pelayanan", "Permintaan darah", "PD-261118-0001"], "PD-261118-0001", None,
             [("Batalkan", "ghost", "x"), ("Cetak formulir", "secondary", "printer")])
    ps = D.row(m, 20, "pasien", fill=SURF, padding=[18, 20], cornerRadius=16, stroke=LINE, strokeWeight=1, strokeAlign="inside", sizingH="fill", effects=SHADOW_SM)
    D.avatar(ps, "SA", 52, "#FFE1F1", "#8F1352")
    t = D.col(ps, 4, "identitas", sizingH="fill")
    r = D.row(t, 10, "nama")
    D.text(r, "Siti Aminah", 20, "ExtraBold")
    D.chip(r, "Cito", "bahaya", dot=False)
    D.chip(r, "Uji silang", "info")
    r2 = D.row(t, 8, "rm")
    D.text(r2, "00-12-34-56", 13, "SemiBold", INK2, mono=True)
    D.text(r2, "P · 42 th · IGD bed 3 · dr. Peminta Demo", 13, "Medium", MUTED)
    for k, v in [("Diagnosis", "Perdarahan saluran cerna atas"), ("Hb", "6,1 g/dL"), ("Diminta", "PRC 2 kantong · 08.20")]:
        c = D.col(ps, 3, f"info/{k}")
        D.text(c, k, 12, "SemiBold", MUTED)
        D.text(c, v, 14, "Bold")
    gc = D.col(ps, 4, "golongan", crossAlign="center")
    D.gol(gc, "A", "+", "lg")
    D.text(gc, "Terverifikasi", 11, "SemiBold", "#05603A")

    langkah(m, 2)
    isi = D.row(m, 16, "isi", sizingH="fill", crossAlign="min")
    kiri = D.col(isi, 16, "kerja", sizingH="fill")
    us = D.card(kiri, "Uji silang serasi", sub="Metode gel · mayor & minor untuk PRC (aturan komponen)")
    cols = [("Kantong", "fill", "min"), ("Mayor", 110, "min"), ("Minor", 110, "min"), ("Auto-kontrol", 120, "min"), ("Kesimpulan", 150, "min")]
    hasil = lambda c, v: D.chip(c, v, "aman" if v == "Negatif" else "bahaya", dot=False)
    D.table(us, cols, [
        [lambda c: (D.gol(c, "A", "+", "sm"), D.text(c, "G9999262000088", 14, "SemiBold", INK, mono=True)),
         lambda c: hasil(c, "Negatif"), lambda c: hasil(c, "Negatif"), lambda c: hasil(c, "Negatif"),
         lambda c: (D.icon(c, "circle-check", 16, "#05603A"), D.text(c, "Serasi", 13, "Bold", "#05603A"))],
    ])
    # Formulir hasil kantong kedua
    fm = D.col(us, 14, "input", fill="#FAFBFC", padding=16, cornerRadius=12, stroke=LINE2, strokeWeight=1, strokeAlign="inside", sizingH="fill")
    D.text(fm, "Hasil kantong kedua", 14, "Bold")
    D.field(fm, "Nomor kantong", "G9999262000130", mono=True, icon="scan-barcode", ok=True, hint="A+ · PRC · kedaluwarsa 6 Des 2026")
    rr = D.row(fm, 16, "hasil", sizingH="fill")
    for k, pilihan in [("Mayor", 0), ("Minor", 0), ("Auto-kontrol", 1)]:
        c = D.col(rr, 6, f"hasil/{k}", sizingH="fill")
        D.text(c, k, 13, "SemiBold", INK2)
        seg = D.row(c, 2, "pilih", fill="#E9ECF1", padding=3, cornerRadius=10, sizingH="fill")
        for i, v in enumerate(["Negatif", "Positif"]):
            on = i == pilihan
            b = D.row(seg, 0, f"opsi/{v}", padding=[7, 10], cornerRadius=8, sizingH="fill", mainAlign="center",
                      fill=(SURF if v == "Negatif" else "#D92D20") if on else "none", **({"effects": SHADOW_SM} if on else {}))
            D.text(b, v, 13, "SemiBold", ("#FFFFFF" if v == "Positif" else INK) if on else MUTED)
    D.alert(fm, "bahaya", "Auto-kontrol positif — kantong tidak boleh dikeluarkan",
            "Kesimpulan menjadi “tidak serasi”. Lakukan pemeriksaan DAT dan pilih kantong lain, atau rujuk sampel ke UTD.", icon="circle-alert")
    ak = D.row(fm, 10, "aksi", sizingH="fill", mainAlign="max")
    D.button(ak, "Simpan sebagai draf", "ghost")
    D.button(ak, "Simpan & tanda tangani", "primary", "pen-line")

    kanan = D.col(isi, 16, "samping", width=360)
    D.cmd("node.set", ids=[kanan], sizingH="fixed", width=360)
    sp = D.card(kanan, "Sampel", gap=12)
    for k, v in [("Diambil", "18 Nov · 08.25 · Ners Satu"), ("Diterima", "08.32 · Teknisi Satu"), ("Berlaku sampai", "21 Nov · 08.25 (72 jam)")]:
        r = D.row(sp, 10, f"kv/{k}", sizingH="fill")
        D.text(r, k, 13, "Medium", MUTED, w=110)
        D.text(r, v, 13, "SemiBold", w="fill")
    D.alert(sp, "info", "Pasien ditransfusi 3 bulan terakhir", "Sampel berlaku 72 jam sejak diambil.", icon="clock")
    vk = D.card(kanan, "Verifikasi kedua", gap=12, sub="Hasil manual baru berlaku setelah diperiksa petugas lain")
    for nama, peran, tone, st in [("Teknisi Satu", "Input golongan · 08.40", "aman", "Ditandatangani"), ("Teknisi Dua", "Verifikasi golongan · 08.47", "aman", "Ditandatangani"),
                                  ("Teknisi Satu", "Input uji silang #1 · 08.58", "aman", "Ditandatangani"), ("—", "Verifikasi uji silang #1", "waspada", "Menunggu")]:
        r = D.row(vk, 10, f"ttd/{peran}", sizingH="fill")
        D.avatar(r, "".join(w[0] for w in nama.split())[:2] if nama != "—" else "?", 30, "#EEF0F3", INK2)
        t = D.col(r, 1, "teks", sizingH="fill")
        D.text(t, nama, 13, "SemiBold")
        D.text(t, peran, 12, "Medium", MUTED)
        D.chip(r, st, tone, size=11)
    D.button(vk, "Minta verifikasi Teknisi Dua", "secondary", "users", w="fill")

    if dialog:
        sc = D.frame(root, "latar-redup", 1440, 960, 0, 0, fill="#0B122099", absolute=True)
        D.cmd("node.set", ids=[sc], x=0, y=0, width=1440, height=1180)
        dg = D.col(root, 18, "dialog/tanda-tangan", fill=SURF, padding=28, cornerRadius=20, width=480, absolute=True, effects=SHADOW_LG)
        D.cmd("node.set", ids=[dg], sizingH="fixed", width=480, x=480, y=250)
        h = D.row(dg, 14, "kepala", sizingH="fill", crossAlign="min")
        ib = D.row(h, 0, "ikon", fill="#F2F4F7", padding=12, cornerRadius=12)
        D.icon(ib, "pen-line", 22, INK)
        t = D.col(h, 4, "judul", sizingH="fill")
        D.text(t, "Tanda tangan elektronik", 20, "ExtraBold")
        D.text(t, "Verifikasi kedua hasil uji silang G9999262000088", 14, "Medium", MUTED, w="fill")
        D.icon(h, "x", 20, MUTED)
        ring = D.col(dg, 8, "ringkasan", fill="#FAFBFC", padding=14, cornerRadius=12, sizingH="fill", stroke=LINE2, strokeWeight=1, strokeAlign="inside")
        for k, v in [("Pasien", "Siti Aminah · 00-12-34-56"), ("Hasil", "Mayor −, Minor −, AK − · Serasi"), ("Diinput", "Teknisi Satu · 08.58")]:
            r = D.row(ring, 10, f"kv/{k}", sizingH="fill")
            D.text(r, k, 13, "Medium", MUTED, w=80)
            D.text(r, v, 13, "SemiBold", w="fill")
        D.field(dg, "Nama pengguna petugas kedua", "teknisi2", icon="user")
        D.field(dg, "Kata sandi", "••••••", icon="lock", error="Sandi salah. Sisa 4 percobaan sebelum akun dikunci.")
        r = D.row(dg, 8, "pernyataan", crossAlign="min", sizingH="fill")
        cb = D.row(r, 0, "centang", fill=INK, cornerRadius=5, mainAlign="center")
        D.cmd("node.set", ids=[cb], sizingH="fixed", sizingV="fixed", width=18, height=18)
        D.icon(cb, "check", 12, "#FFFFFF", 3)
        D.text(r, "Saya telah memeriksa ulang hasil ini secara mandiri dan menyetujuinya.", 13, "Medium", INK2, w="fill", lineHeight="150%")
        ak = D.row(dg, 10, "aksi", sizingH="fill", mainAlign="max")
        D.button(ak, "Batal", "secondary")
        D.button(ak, "Tandatangani", "primary", "pen-line")
    return root


def samping(x, y, cocok=True):
    W, H = 1194, 834
    root = D.frame(None, "06 Samping tempat tidur — cocok" if cocok else "07 Samping tempat tidur — TIDAK COCOK", W, None, x, y,
                   fill=BG, layout="vertical", gap=18, padding=[18, 24, 24, 24], clip=True, minHeight=H)
    D.akhir.append(("node.set", {"ids": [root], "sizingV": "hug"}))
    tb = D.row(root, 12, "bilah", sizingH="fill", mainAlign="spaceBetween")
    l = D.row(tb, 12, "kiri")
    bk = D.row(l, 0, "kembali", fill=SURF, padding=12, cornerRadius=12, stroke=LINE, strokeWeight=1, strokeAlign="inside")
    D.icon(bk, "arrow-left", 22, INK)
    t = D.col(l, 1, "judul")
    D.text(t, "Verifikasi samping tempat tidur", 22, "ExtraBold")
    D.text(t, "Ruang Melati · Bed 4 · PD-261118-0004", 14, "Medium", MUTED)
    r = D.row(tb, 10, "petugas")
    for ini, nama, ok in [("NS", "Ners Satu", True), ("ND", "Ners Dua", cocok)]:
        c = D.row(r, 8, f"petugas/{nama}", fill=SURF, padding=[6, 14, 6, 6], cornerRadius=999, stroke=LINE, strokeWeight=1, strokeAlign="inside")
        D.avatar(c, ini, 32, "#E9F9EF" if ok else "#F2F4F7", "#05603A" if ok else INK2)
        D.text(c, nama, 14, "SemiBold")
        D.icon(c, "circle-check" if ok else "clock", 18, "#12B76A" if ok else FAINT)

    if not cocok:
        D.alert(root, "bahaya", "JANGAN TRANSFUSIKAN — golongan kantong tidak sesuai",
                "Kantong B+ tidak cocok dengan pasien A+. Kembalikan kantong ke bank darah sekarang. Kejadian ini otomatis dicatat sebagai Kejadian Nyaris Cedera (KNC).",
                icon="octagon-alert", big=True)

    dua = D.row(root, 18, "dua-sisi", sizingH="fill", crossAlign="min")
    for sisi, ik, judul, pindai, baris in [
        ("pasien", "user", "Gelang pasien", "00-55-21-07",
         [("Nama", "Rahmat Hidayat", True), ("No. RM", "00-55-21-07", True), ("Tgl. lahir", "14 Jun 1971", True), ("Golongan", "A+", True)]),
        ("kantong", "droplet", "Label kantong", "G9999262000214" if cocok else "G9999262000102",
         [("Untuk", "Rahmat Hidayat · 00-55-21-07", True), ("Nomor", "G9999262000214" if cocok else "G9999262000102", True),
          ("Golongan", "A+" if cocok else "B+", cocok), ("Kedaluwarsa", "6 Des 2026 · masih 18 hari", True)]),
    ]:
        c = D.col(dua, 16, f"sisi/{sisi}", fill=SURF, padding=22, cornerRadius=20, stroke=LINE, strokeWeight=1, strokeAlign="inside",
                  sizingH="fill", effects=SHADOW_SM)
        h = D.row(c, 12, "kepala", sizingH="fill")
        ib = D.row(h, 0, "ikon", fill="#F2F4F7", padding=10, cornerRadius=12)
        D.icon(ib, ik, 22, INK)
        D.text(h, judul, 18, "Bold")
        D.field(c, "", pindai, mono=True, icon="scan-barcode", big=True, ok=(sisi == "pasien" or cocok), error=None if (sisi == "pasien" or cocok) else "Label kantong tidak cocok dengan pasien")
        for k, v, ok in baris:
            r = D.row(c, 12, f"cek/{k}", sizingH="fill", padding=[12, 14], cornerRadius=12, fill="#E9F9EF" if ok else "#FEEEEC")
            D.icon(r, "circle-check" if ok else "circle-x", 22, "#12B76A" if ok else "#F04438")
            D.text(r, k, 15, "SemiBold", MUTED, w=110)
            if k == "Golongan":
                D.gol(r, v[:-1], "+" if v.endswith("+") else "-", "md")
            else:
                D.text(r, v, 16, "Bold", INK, mono=(k in ("No. RM", "Nomor")), w="fill")
    if cocok:
        tv = D.col(root, 14, "tanda-vital", fill=SURF, padding=22, cornerRadius=20, stroke=LINE, strokeWeight=1, strokeAlign="inside", sizingH="fill", effects=SHADOW_SM)
        h = D.row(tv, 12, "kepala", sizingH="fill", mainAlign="spaceBetween")
        D.text(h, "Tanda vital sebelum transfusi", 18, "Bold")
        D.text(h, "Wajib diisi · observasi berikutnya 15 menit setelah mulai", 13, "Medium", MUTED)
        r = D.row(tv, 14, "isian", sizingH="fill")
        for k, v, sat in [("Tekanan darah", "118/76", "mmHg"), ("Nadi", "88", "×/menit"), ("Suhu", "36,8", "°C"), ("Napas", "18", "×/menit")]:
            c = D.col(r, 6, f"tv/{k}", sizingH="fill")
            D.text(c, k, 14, "SemiBold", INK2)
            b = D.row(c, 8, "kotak", fill=SURF, padding=[14, 16], cornerRadius=12, stroke="#D0D5DD", strokeWeight=1, strokeAlign="inside", sizingH="fill", crossAlign="baseline")
            D.text(b, v, 24, "Bold", INK, features={"tnum": True})
            D.text(b, sat, 14, "Medium", MUTED)
    else:
        lk = D.col(root, 12, "langkah", fill=SURF, padding=22, cornerRadius=20, stroke=LINE, strokeWeight=1, strokeAlign="inside", sizingH="fill", effects=SHADOW_SM)
        D.text(lk, "Yang harus dilakukan sekarang", 18, "Bold")
        for i, t in enumerate(["Jangan pasang kantong ini. Tutup kembali dan beri tanda “ditolak di ruang”.",
                               "Bawa kantong ke bank darah dalam 30 menit bersama formulir permintaan.",
                               "Hubungi bank darah (ext. 2140) untuk kantong pengganti yang sesuai."], 1):
            r = D.row(lk, 12, f"langkah/{i}", sizingH="fill")
            n = D.row(r, 0, "nomor", fill="#FEEEEC", cornerRadius=14, mainAlign="center")
            D.cmd("node.set", ids=[n], sizingH="fixed", sizingV="fixed", width=28, height=28)
            D.text(n, str(i), 14, "Bold", "#B42318")
            D.text(r, t, 16, "Medium", INK, w="fill")
    D.frame(root, "jarak", 1, 1, sizingV="fill")
    ft = D.row(root, 14, "bawah", sizingH="fill")
    if cocok:
        D.text(ft, "Semua data cocok dan diperiksa dua perawat. Catat tanda vital awal sebelum mulai.", 15, "Medium", INK2, w="fill")
        D.button(ft, "Mulai transfusi", "primary", "syringe", big=True)
    else:
        D.text(ft, "Transfusi diblokir. Pindai ulang tidak membuka blokir — kantong harus dikembalikan.", 15, "SemiBold", "#B42318", w="fill")
        D.button(ft, "Kembalikan ke bank darah", "danger", "archive-x", big=True)
    return root


def sistem(x, y):
    root = D.frame(None, "00 Sistem desain", 1440, None, x, y, fill=SURF, layout="vertical", gap=40, padding=56, minHeight=960)
    D.cmd("node.set", ids=[root], sizingV="hug")
    h = D.col(root, 10, "judul", sizingH="fill")
    D.logo(h, dark=False, size=44)
    D.text(h, "Sistem desain utd-rs", 44, "ExtraBold")
    D.text(h, "Modern klinis: permukaan terang dan lega, bilah samping gelap, warna hanya untuk makna. Huruf Plus Jakarta Sans (Tokotype, Jakarta) untuk teks; IBM Plex Mono untuk nomor kantong, no. RM, dan nomor dokumen.",
           17, "Medium", MUTED, w=900, lineHeight="155%")

    def bagian(judul):
        s = D.col(root, 16, f"bagian/{judul}", sizingH="fill")
        D.text(s, judul, 13, "Bold", MUTED, letterSpacing="8%", textCase="upper")
        return s

    s = bagian("Warna")
    r = D.row(s, 13, "netral", sizingH="fill")
    for nama, hexv in [("Tinta", INK), ("Tinta 2", INK2), ("Redup", MUTED), ("Samar", FAINT), ("Garis", LINE), ("Latar", BG), ("Samping", SIDE), ("Merek", BRAND), ("Fokus", FOCUS)]:
        c = D.col(r, 8, f"warna/{nama}", width=134)
        D.cmd("node.set", ids=[c], sizingH="fixed", width=134)
        D.frame(c, "contoh", 134, 72, fill=hexv, cornerRadius=12, stroke=LINE, strokeWeight=1, strokeAlign="inside")
        D.text(c, nama, 13, "Bold")
        D.text(c, hexv, 12, "Medium", MUTED, mono=True)
    r = D.row(s, 14, "makna", sizingH="fill")
    for tone, label in [("aman", "Aman · tersedia, serasi"), ("waspada", "Waspada · segera kedaluwarsa"), ("bahaya", "Bahaya · tidak serasi, menyimpang"), ("info", "Info · sedang diproses"), ("netral", "Netral · selesai, arsip")]:
        bg, fg, dt = TONE[tone]
        c = D.col(r, 10, f"makna/{tone}", sizingH="fill", padding=14, cornerRadius=14, fill=bg)
        D.chip(c, tone.capitalize(), tone)
        D.text(c, label, 12, "Medium", fg, w="fill")
    r = D.row(s, 14, "golongan", sizingH="fill")
    D.text(r, "Golongan darah — warna label ISBT 128 (O biru, A kuning, B merah muda, AB putih); Rh− diberi garis tepi:", 13, "Medium", INK2)
    for abo in ["O", "A", "B", "AB"]:
        for rh in "+-":
            D.gol(r, abo, rh)

    s = bagian("Tipografi")
    for size, wt, contoh, ket in [(44, "ExtraBold", "Judul layar", "44 / ExtraBold / −2%"), (28, "ExtraBold", "Judul halaman", "28 / ExtraBold / −2%"),
                                  (20, "Bold", "Judul kartu besar", "20 / Bold"), (16, "Bold", "Judul kartu", "16 / Bold"),
                                  (14, "Medium", "Teks isi untuk tabel dan formulir di stasiun kerja lab.", "14 / Medium"),
                                  (12, "SemiBold", "Keterangan, label kecil", "12 / SemiBold")]:
        r = D.row(s, 24, f"tipe/{size}", sizingH="fill", crossAlign="baseline")
        D.text(r, ket, 12, "Medium", MUTED, mono=True, w=200)
        D.text(r, contoh, size, wt)
    r = D.row(s, 24, "tipe/mono", sizingH="fill", crossAlign="baseline")
    D.text(r, "Mono 14 / SemiBold", 12, "Medium", MUTED, mono=True, w=200)
    D.text(r, "G9999262000088 · 00-12-34-56 · PD-261118-0001 · 0O 1lI", 16, "SemiBold", INK, mono=True)

    s = bagian("Komponen")
    r = D.row(s, 12, "tombol", sizingH="fill")
    for lbl, kind, ic in [("Simpan & tanda tangani", "primary", "pen-line"), ("Ekspor", "secondary", "file-text"), ("Batal", "ghost", None),
                          ("Musnahkan", "danger", "archive-x"), ("Buat permintaan", "brand", "plus")]:
        D.button(r, lbl, kind, ic)
    D.button(r, "Mulai transfusi (tablet)", "primary", "syringe", big=True)
    r = D.row(s, 16, "isian", sizingH="fill", crossAlign="min")
    D.field(r, "Nomor kantong", placeholder="Pindai atau ketik", icon="scan-barcode")
    D.field(r, "Nomor kantong", "G9999262000130", mono=True, icon="scan-barcode", ok=True, hint="A+ · PRC · kedaluwarsa 6 Des 2026")
    D.field(r, "Kata sandi", "••••••", icon="lock", error="Sandi salah. Sisa 4 percobaan.")
    r = D.row(s, 8, "chip", sizingH="fill")
    for lbl, tone in [("Tersedia", "aman"), ("Karantina", "waspada"), ("Dialokasikan", "info"), ("Ditransfusikan", "netral"), ("Menyimpang", "bahaya")]:
        D.chip(r, lbl, tone)
    D.chip(r, "Cito", "bahaya", dot=False)
    a = D.col(s, 10, "peringatan", sizingH="fill")
    D.alert(a, "bahaya", "JANGAN TRANSFUSIKAN — golongan kantong tidak sesuai", "Peringatan klinis tidak hilang sendiri dan selalu menyebut tindakan berikutnya.", icon="octagon-alert")
    D.alert(a, "waspada", "Sampel berumur 70 jam", "Batas 72 jam. Minta sampel baru bila uji silang belum selesai sebelum 08.25.", icon="clock")
    D.alert(a, "info", "Pemindai siap", "Isian kosong dan tetap fokus setelah setiap pindaian.", icon="scan-barcode")
    return root


LAYAR = {
    "sistem": sistem,
    "masuk": masuk,
    "beranda": beranda,
    "stok": stok,
    "permintaan": permintaan,
    "tanda-tangan": lambda x, y: permintaan(x, y, dialog=True),
    "samping": samping,
    "samping-tolak": lambda x, y: samping(x, y, cocok=False),
}

if __name__ == "__main__":
    pilih = sys.argv[1:] or list(LAYAR)
    D.cmd("page.rename", name="Desain utd-rs")
    D.cmd("page.setBackground", color="#E9EBEF")
    x = 0
    for n in pilih:
        LAYAR[n](x, 0)
        x += 1440 + GAP
    out = DIR / "utd-rs.figcraft"
    D.run(out)
    (DIR / "render").mkdir(exist_ok=True)
    subprocess.run(["figcraft-cli", "export", str(out), "-o", str(DIR / "render"), "--format", "png", "--scale", "1"], check=True)
    print("ok", out)
