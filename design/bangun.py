#!/usr/bin/env python3
"""Membangun berkas desain FigCraft utd-rs lewat `figcraft-cli run`.

python3 design/bangun.py  ->  design/utd-rs.figcraft + design/render/*.png
Data di layar fiktif (nama pasien, nomor kantong berawalan 99).
"""
import json
import pathlib
import re
import subprocess
import sys

DIR = pathlib.Path(__file__).resolve().parent
IKON = DIR.parent / "web/node_modules/@lucide/svelte/dist/icons"

SANS = "Plus Jakarta Sans"
MONO = "IBM Plex Mono"  # nol bertitik, G jelas beda dari 6

# Token warna (kandidat app.css baru)
INK = "#0B1220"
INK2 = "#344054"
MUTED = "#5D6B82"  # 5,3:1 di atas putih
FAINT = "#98A2B3"
LINE = "#E4E7EC"
LINE2 = "#EEF0F3"
BG = "#F4F5F8"
SURF = "#FFFFFF"
SIDE = "#0D1424"
SIDE2 = "#18213A"
SIDE_TX = "#C9D0DE"
SIDE_MUTED = "#7E89A3"
BRAND = "#D0103A"
BRAND_DK = "#8E0C2A"
FOCUS = "#2E6BFF"

TONE = {  # (latar, teks, titik)
    "aman": ("#E9F9EF", "#05603A", "#12B76A"),
    "waspada": ("#FFF6E5", "#93370D", "#F79009"),
    "bahaya": ("#FEEEEC", "#B42318", "#F04438"),
    "info": ("#EAF1FF", "#1D44B8", "#2E6BFF"),
    "netral": ("#F0F2F5", "#344054", "#98A2B3"),
}
# Warna golongan mengikuti konvensi label ISBT 128: O biru, A kuning, B merah muda, AB putih.
GOL = {
    "O": ("#DCE9FF", "#163D9C"),
    "A": ("#FFF3B8", "#6B4405"),
    "B": ("#FFE1F1", "#8F1352"),
    "AB": ("#FFFFFF", "#344054"),
}

SHADOW_SM = [{"type": "dropShadow", "color": "#1018280D", "offset": [0, 1], "radius": 2, "spread": 0}]
SHADOW_MD = [
    {"type": "dropShadow", "color": "#10182814", "offset": [0, 4], "radius": 12, "spread": -2},
    {"type": "dropShadow", "color": "#1018280A", "offset": [0, 2], "radius": 4, "spread": -2},
]
SHADOW_LG = [{"type": "dropShadow", "color": "#10182833", "offset": [0, 24], "radius": 48, "spread": -12}]


ARGS = {"M": 2, "L": 2, "H": 1, "V": 1, "C": 6, "S": 4, "Q": 4, "T": 2, "A": 7, "Z": 0}


def normal_path(d: str) -> str:
    """Path SVG -> perintah absolut eksplisit (tanpa pengulangan implisit, flag busur dipisah)."""
    tok = re.findall(r"[MmLlHhVvCcSsQqTtAaZz]|-?(?:\d*\.\d+|\d+\.?)(?:[eE][-+]?\d+)?", d)
    out, i, cmd = [], 0, None
    x = y = sx = sy = 0.0

    def num():
        nonlocal i
        v = float(tok[i])
        i += 1
        return v

    while i < len(tok):
        if tok[i].isalpha():
            cmd = tok[i]
            i += 1
        elif cmd is None:
            break
        up, rel = cmd.upper(), cmd.islower()
        if up == "Z":
            out.append("Z")
            x, y = sx, sy
            continue
        if up == "A":
            rx, ry, rot = num(), num(), num()
            flags = []
            while len(flags) < 2:  # flag bisa tertulis rapat: "011"
                t = tok[i]
                flags.append(t[0])
                rest = t[1:]
                if rest:
                    tok[i] = rest
                else:
                    i += 1
            ex, ey = num(), num()
            if rel:
                ex, ey = ex + x, ey + y
            out.append(f"A{rx} {ry} {rot} {flags[0]} {flags[1]} {ex:g} {ey:g}")
            x, y = ex, ey
        elif up == "H":
            v = num()
            x = v + x if rel else v
            out.append(f"L{x:g} {y:g}")
        elif up == "V":
            v = num()
            y = v + y if rel else v
            out.append(f"L{x:g} {y:g}")
        else:
            vals = [num() for _ in range(ARGS[up])]
            if rel:
                vals = [v + (x if k % 2 == 0 else y) for k, v in enumerate(vals)]
            out.append(up + " ".join(f"{v:g}" for v in vals))
            x, y = vals[-2], vals[-1]
            if up == "M":
                sx, sy = x, y
                cmd = "l" if rel else "L"  # pasangan setelah M adalah lineto
    return " ".join(out)


def ikon_path(nama: str) -> list[str]:
    f = IKON / f"{nama}.svelte"
    if not f.exists():
        alias = re.search(r'from "\./(.+?)\.svelte"', (IKON / f"{nama}.js").read_text())
        f = IKON / f"{alias.group(1)}.svelte"
    data = json.loads(re.search(r"__iconData = (\{.*?\});", f.read_text()).group(1))
    d = []
    for tag, a in data["node"]:
        g = lambda k: float(a.get(k, 0))
        if tag == "path":
            d.append(a["d"])
        elif tag in ("circle", "ellipse"):
            cx, cy = g("cx"), g("cy")
            rx = g("r") if tag == "circle" else g("rx")
            ry = g("r") if tag == "circle" else g("ry")
            k = 0.5523  # empat kurva Bezier: busur setengah lingkaran tidak tergambar benar di FigCraft
            d.append(
                f"M{cx + rx} {cy}C{cx + rx} {cy + k * ry} {cx + k * rx} {cy + ry} {cx} {cy + ry}"
                f"C{cx - k * rx} {cy + ry} {cx - rx} {cy + k * ry} {cx - rx} {cy}"
                f"C{cx - rx} {cy - k * ry} {cx - k * rx} {cy - ry} {cx} {cy - ry}"
                f"C{cx + k * rx} {cy - ry} {cx + rx} {cy - k * ry} {cx + rx} {cy}Z"
            )
        elif tag == "rect":
            x, y, w, h, r = g("x"), g("y"), g("width"), g("height"), g("rx")
            if r:
                d.append(
                    f"M{x + r} {y}H{x + w - r}A{r} {r} 0 0 1 {x + w} {y + r}V{y + h - r}"
                    f"A{r} {r} 0 0 1 {x + w - r} {y + h}H{x + r}A{r} {r} 0 0 1 {x} {y + h - r}"
                    f"V{y + r}A{r} {r} 0 0 1 {x + r} {y}Z"
                )
            else:
                d.append(f"M{x} {y}H{x + w}V{y + h}H{x}Z")
        elif tag == "line":
            d.append(f"M{a['x1']} {a['y1']}L{a['x2']} {a['y2']}")
        elif tag in ("polyline", "polygon"):
            pts = a["points"].replace(",", " ").split()
            d.append("M" + " L".join(f"{pts[i]} {pts[i + 1]}" for i in range(0, len(pts), 2)) + ("Z" if tag == "polygon" else ""))
    # FigCraft hanya menggambar subpath pertama dari satu vektor; pecah per "M".
    # Tiap elemen dinormalkan sendiri: "m" di awal elemen relatif ke (0,0), bukan ke elemen sebelumnya.
    return [p.strip() for e in d for p in re.split(r"(?=M)", normal_path(e)) if p.strip()]


class Doc:
    def __init__(self):
        self.steps = []
        self.next_id = 2  # 1 = halaman pertama
        self.expect = {}
        self.akhir = []  # dijalankan setelah semua lapisan dibuat

    def cmd(self, command, **params):
        self.steps.append({"command": command, "params": params})

    def _new(self, command, params):
        nid = self.next_id
        self.next_id += 1
        params["select"] = False
        self.cmd(command, **params)
        self.expect[len(self.steps) - 1] = nid
        return nid

    # --- primitif ---
    def frame(self, parent=None, name="Frame", w=None, h=None, x=0, y=0, **props):
        p = {"x": x, "y": y, "width": w or 10, "height": h or 10, "name": name}
        props.setdefault("fill", "none")
        if props.get("layout"):
            props.setdefault("sizingH", "fixed" if w else "hug")
            props.setdefault("sizingV", "fixed" if h else "hug")
        if parent:
            p["parent"] = parent
        p["props"] = props
        return self._new("frame.create", p)

    def row(self, parent, gap=8, name="Baris", **props):
        props.setdefault("crossAlign", "center")
        return self.frame(parent, name, layout="horizontal", gap=gap, **props)

    def col(self, parent, gap=8, name="Kolom", **props):
        return self.frame(parent, name, layout="vertical", gap=gap, **props)

    def text(self, parent, t, size=14, weight="Regular", color=INK, mono=False, w=None, name=None, **props):
        props.update(fontFamily=MONO if mono else SANS, fontStyle=weight, fontSize=size, fill=color)
        if not mono and size >= 22:
            props.setdefault("letterSpacing", "-2%")
        if w == "fill":
            props.update(sizingH="fill", textAutoResize="height")
        p = {"text": t, "x": 0, "y": 0, "parent": parent, "props": props}
        if isinstance(w, (int, float)):
            p["width"] = w
            props["textAutoResize"] = "height"
        if name:
            p["name"] = name
        return self._new("text.create", p)

    def rect(self, parent, w, h, fill, name="Bidang", **props):
        props["fill"] = fill
        return self._new("shape.rectangle", {"x": 0, "y": 0, "width": w, "height": h, "parent": parent, "name": name, "props": props})

    def icon(self, parent, nama, size=20, color=INK, stroke=2):
        box = self.frame(parent, f"ikon/{nama}", 24, 24)
        for bagian in ikon_path(nama):
            self._new(
                "vector.create",
                {"path": bagian, "x": 0, "y": 0, "parent": box, "name": nama,
                 "props": {"fill": "none", "stroke": color, "strokeWeight": stroke, "strokeCap": "round", "strokeJoin": "round"}},
            )
        if size != 24:
            self.cmd("transform.scale", ids=[box], factor=size / 24)
        return box

    # --- komponen ---
    def chip(self, parent, label, tone="netral", dot=True, size=12):
        bg, fg, dt = TONE[tone]
        c = self.row(parent, 6, f"chip/{tone}", fill=bg, padding=[3, 10, 3, 8 if dot else 10], cornerRadius=999)
        if dot:
            self.frame(c, "titik", 6, 6, fill=dt, cornerRadius=3)
        self.text(c, label, size, "SemiBold", fg)
        return c

    def gol(self, parent, abo, rh="+", size="md"):
        bg, fg = GOL[abo]
        fs, pad, h = {"sm": (12, [2, 7], 22), "md": (14, [4, 9], 28), "lg": (24, [8, 14], 48)}[size]
        b = self.row(parent, 0, f"golongan/{abo}{rh}", fill=bg, padding=pad, cornerRadius=8 if size != "lg" else 12, mainAlign="center")
        if abo == "AB" or rh == "-":
            # Rh negatif diberi garis tepi tegas supaya tidak hanya dibedakan lewat tanda minus.
            self.cmd("node.set", ids=[b], stroke=fg if rh == "-" else "#D0D5DD", strokeWeight=1.5 if rh == "-" else 1, strokeAlign="inside")
        self.text(b, abo + ("+" if rh == "+" else "−"), fs, "ExtraBold", fg, letterSpacing="2%")
        return b

    def button(self, parent, label, kind="primary", icon=None, big=False, w=None):
        styles = {
            "primary": (INK, "#FFFFFF", None),
            "brand": (BRAND, "#FFFFFF", None),
            "secondary": (SURF, INK, "#D0D5DD"),
            "ghost": ("none", INK2, None),
            "danger": ("#D92D20", "#FFFFFF", None),
        }
        bg, fg, st = styles[kind]
        pad = [14, 22] if big else [9, 14]
        b = self.row(parent, 8, f"tombol/{kind}", fill=bg, padding=pad, cornerRadius=12 if big else 10, mainAlign="center",
                     **({"sizingH": "fill"} if w == "fill" else {}))
        if st:
            self.cmd("node.set", ids=[b], stroke=st, strokeWeight=1, strokeAlign="inside", effects=SHADOW_SM)
        if icon:
            self.icon(b, icon, 20 if big else 16, fg)
        self.text(b, label, 16 if big else 14, "SemiBold", fg)
        return b

    def field(self, parent, label, value="", placeholder="", mono=False, icon=None, hint=None, error=None, w="fill", big=False, ok=False):
        f = self.col(parent, 6, f"isian/{label}", **({"sizingH": "fill"} if w == "fill" else {}))
        if label:
            self.text(f, label, 13, "SemiBold", INK2)
        border = "#F04438" if error else ("#12B76A" if ok else "#D0D5DD")
        box = self.row(f, 10, "kotak", fill=SURF, padding=[14 if big else 10, 14], cornerRadius=12 if big else 10,
                       stroke=border, strokeWeight=2 if (error or ok) else 1, strokeAlign="inside", sizingH="fill" if w == "fill" else "fixed",
                       effects=SHADOW_SM, **({} if w == "fill" else {"width": w}))
        if w != "fill":
            self.cmd("node.set", ids=[box], width=w)
        if icon:
            self.icon(box, icon, 20 if big else 18, MUTED)
        self.text(box, value or placeholder, 18 if big else 14, "Medium" if value else "Regular", INK if value else FAINT, mono=mono and bool(value))
        if ok:
            spacer = self.frame(box, "jarak", 1, 1, sizingH="fill")
            self.icon(box, "circle-check", 20, "#12B76A")
        if error:
            r = self.row(f, 6, "galat")
            self.icon(r, "circle-alert", 14, "#B42318")
            self.text(r, error, 12, "Medium", "#B42318")
        elif hint:
            self.text(f, hint, 12, "Regular", MUTED)
        return f

    def card(self, parent, title=None, action=None, pad=20, gap=16, name="kartu", sizingH="fill", w=None, sub=None):
        c = self.col(parent, gap, name, fill=SURF, padding=pad, cornerRadius=16, stroke=LINE, strokeWeight=1,
                     strokeAlign="inside", effects=SHADOW_SM, sizingH=sizingH if not w else "fixed", **({"width": w} if w else {}))
        if w:
            self.cmd("node.set", ids=[c], width=w)
        if title:
            h = self.row(c, 8, "kepala", sizingH="fill", mainAlign="spaceBetween")
            t = self.col(h, 2, "judul")
            self.text(t, title, 16, "Bold", INK)
            if sub:
                self.text(t, sub, 13, "Regular", MUTED)
            if action:
                a = self.row(h, 4, "tautan")
                self.text(a, action, 13, "SemiBold", INK2)
                self.icon(a, "chevron-right", 16, INK2)
        return c

    def alert(self, parent, tone, title, body, action=None, icon="triangle-alert", big=False):
        bg, fg, dt = TONE[tone]
        a = self.row(parent, 14, f"peringatan/{tone}", fill=bg, padding=[14, 16] if not big else [22, 24], cornerRadius=14,
                     stroke=dt, strokeWeight=1, strokeAlign="inside", sizingH="fill", crossAlign="min")
        ic = self.row(a, 0, "ikon-latar", fill=dt, padding=8 if not big else 12, cornerRadius=10)
        self.icon(ic, icon, 18 if not big else 28, "#FFFFFF", 2.25)
        t = self.col(a, 4, "isi", sizingH="fill")
        self.text(t, title, 15 if not big else 24, "Bold", fg, w="fill")
        self.text(t, body, 14 if not big else 17, "Medium", fg, w="fill", lineHeight="150%")
        if action:
            self.button(a, action, "secondary")
        return a

    def divider(self, parent, color=LINE2):
        return self.rect(parent, 10, 1, color, "garis", sizingH="fill")

    def avatar(self, parent, inisial, size=32, bg="#2B3654", fg="#FFFFFF"):
        a = self.row(parent, 0, "avatar", fill=bg, cornerRadius=size / 2, mainAlign="center", width=size)
        self.cmd("node.set", ids=[a], sizingH="fixed", sizingV="fixed", width=size, height=size)
        self.text(a, inisial, size * 0.4, "Bold", fg)
        return a

    def logo(self, parent, dark=True, size=36):
        r = self.row(parent, 10, "logo")
        m = self.row(r, 0, "tanda", cornerRadius=size * 0.3, mainAlign="center",
                     fills=[{"type": "linearGradient", "stops": ["#F43F5E", BRAND_DK], "from": [0, 0], "to": [1, 1]}])
        self.cmd("node.set", ids=[m], sizingH="fixed", sizingV="fixed", width=size, height=size)
        self.icon(m, "droplet", size * 0.58, "#FFFFFF", 2.5)
        t = self.col(r, 0, "nama")
        self.text(t, "utd-rs", 17, "ExtraBold", "#FFFFFF" if dark else INK, letterSpacing="-1%")
        self.text(t, "Bank Darah RS", 11, "Medium", SIDE_MUTED if dark else MUTED)
        return r

    # --- tata letak layar ---
    def sidebar(self, parent, aktif):
        s = self.col(parent, 22, "bilah-samping", fill=SIDE, padding=[22, 14], sizingV="fill", width=264)
        self.cmd("node.set", ids=[s], sizingH="fixed", width=264)
        top = self.row(s, 8, "atas", sizingH="fill", padding=[0, 6])
        self.logo(top)
        grup = [
            ("", [("layout-dashboard", "Beranda")]),
            ("PELAYANAN", [("clipboard-list", "Permintaan darah"), ("bed", "Transfusi di ruang"), ("heart-pulse", "Reaksi & insiden")]),
            ("BANK DARAH", [("package", "Stok kantong"), ("truck", "Penerimaan dari UTD"), ("thermometer", "Suhu alat simpan"),
                            ("scan-search", "Lacak kantong"), ("archive-x", "Pengembalian & musnah")]),
            ("UTD", [("hand-heart", "Donor"), ("test-tubes", "Donasi & pelulusan")]),
            ("LAPORAN", [("chart-column", "Laporan & mutu"), ("printer", "Lembar darurat")]),
        ]
        for judul, menu in grup:
            g = self.col(s, 2, f"grup/{judul or 'utama'}", sizingH="fill")
            if judul:
                jr = self.row(g, 0, "judul", padding=[0, 10])
                self.text(jr, judul, 11, "Bold", SIDE_MUTED, letterSpacing="8%")
                self.frame(g, "jarak", 1, 4)
            for ik, label in menu:
                on = label == aktif
                m = self.row(g, 12, f"menu/{label}", fill=SIDE2 if on else "none", padding=[9, 10], cornerRadius=10, sizingH="fill")
                self.icon(m, ik, 18, "#FFFFFF" if on else SIDE_MUTED)
                self.text(m, label, 14, "SemiBold" if on else "Medium", "#FFFFFF" if on else SIDE_TX)
                if on:
                    self.frame(m, "jarak", 1, 1, sizingH="fill")
                    self.frame(m, "penanda", 6, 6, fill="#FB7185", cornerRadius=3)
        self.frame(s, "isi", 1, 1, sizingV="fill")
        u = self.row(s, 10, "pengguna", fill=SIDE2, padding=[10, 12], cornerRadius=12, sizingH="fill")
        self.avatar(u, "KB", 34, BRAND, "#FFFFFF")
        t = self.col(u, 1, "nama", sizingH="fill")
        self.text(t, "dr. Kepala BDRS", 13, "SemiBold", "#FFFFFF")
        self.text(t, "Kepala · Shift pagi", 12, "Medium", SIDE_MUTED)
        self.icon(u, "log-out", 18, SIDE_MUTED)
        return s

    def topbar(self, parent, crumbs, title, sub=None, actions=()):
        t = self.col(parent, 14, "kepala-halaman", sizingH="fill")
        bar = self.row(t, 12, "remah", sizingH="fill", mainAlign="spaceBetween")
        cr = self.row(bar, 6, "jalur")
        for i, c in enumerate(crumbs):
            if i:
                self.icon(cr, "chevron-right", 14, FAINT)
            self.text(cr, c, 13, "Medium", MUTED if i < len(crumbs) - 1 else INK2)
        right = self.row(bar, 10, "alat")
        self.field(right, "", placeholder="Cari pasien, no. RM, nomor kantong…  ⌘K", icon="search", w=340)
        nb = self.row(right, 0, "notifikasi", fill=SURF, padding=9, cornerRadius=10, stroke=LINE, strokeWeight=1, strokeAlign="inside")
        self.icon(nb, "bell", 18, INK2)
        h = self.row(t, 16, "judul", sizingH="fill", mainAlign="spaceBetween", crossAlign="max")
        tt = self.col(h, 4, "teks")
        self.text(tt, title, 28, "ExtraBold", INK)
        if sub:
            self.text(tt, sub, 14, "Medium", MUTED)
        if actions:
            a = self.row(h, 10, "aksi")
            for lbl, kind, ic in actions:
                self.button(a, lbl, kind, ic)
        return t

    def screen(self, name, x, y, aktif, w=1440, h=960):
        root = self.frame(None, name, w, None, x, y, fill=BG, layout="horizontal", gap=0, clip=True, minHeight=h)
        side = self.sidebar(root, aktif)
        main = self.col(root, 22, "isi", padding=[22, 32, 32, 32], sizingH="fill")
        # hug pada frame puncak hanya berlaku bila diset setelah dibuat; lalu bilah samping mengisi tingginya
        self.akhir += [("node.set", {"ids": [root], "sizingV": "hug"}), ("node.set", {"ids": [side], "sizingV": "fill"})]
        return root, main

    def table(self, parent, cols, rows, name="tabel"):
        """cols: [(judul, lebar|'fill', rata)], rows: list of list of callables(cell)."""
        t = self.col(parent, 0, name, sizingH="fill")
        hd = self.row(t, 0, "kepala", fill="#F8F9FB", padding=[10, 16], sizingH="fill", cornerRadius=10)
        for judul, lebar, rata in cols:
            c = self.row(hd, 0, f"k/{judul}", **self._lebar(lebar), mainAlign=rata)
            self.text(c, judul.upper(), 11, "Bold", MUTED, letterSpacing="6%")
        for i, r in enumerate(rows):
            sel = isinstance(r, dict)
            cells = r["cells"] if sel else r
            br = self.row(t, 0, f"baris/{i + 1}", padding=[12, 16], sizingH="fill",
                          fill="#F3F6FF" if sel else "none", cornerRadius=10 if sel else 0)
            if not sel:
                self.cmd("node.set", ids=[br], strokeBottomWeight=1, stroke=LINE2)
            else:
                self.cmd("node.set", ids=[br], stroke="#B9CBFF", strokeWeight=1, strokeAlign="inside")
            for (judul, lebar, rata), fn in zip(cols, cells):
                c = self.row(br, 6, f"s/{judul}", **self._lebar(lebar), mainAlign=rata)
                fn(c)
        return t

    @staticmethod
    def _lebar(lebar):
        return {"sizingH": "fill"} if lebar == "fill" else {"width": lebar, "sizingH": "fixed"}

    # --- eksekusi ---
    def run(self, out):
        script = DIR / ".skrip.json"
        script.write_text(json.dumps(self.steps + [{"command": c, "params": p} for c, p in self.akhir]))
        r = subprocess.run(["figcraft-cli", "run", str(script), "--out", str(out)], capture_output=True, text=True)
        lines = [json.loads(l) for l in r.stdout.splitlines() if l.startswith("{")]
        for ln in lines:
            if not ln.get("ok"):
                print(json.dumps(ln)[:600], file=sys.stderr)
                print(json.dumps(self.steps[ln["step"]])[:600], file=sys.stderr)
                sys.exit(1)
            want = self.expect.get(ln["step"])
            if want is not None and ln["result"].get("id") != want:
                sys.exit(f"id langkah {ln['step']} = {ln['result'].get('id')}, diharapkan {want}")
        if r.returncode:
            sys.exit(r.stderr)
        script.unlink()
