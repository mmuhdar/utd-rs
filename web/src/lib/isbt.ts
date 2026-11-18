// Pengurai label ISBT 128 untuk pemindaian kantong (SPEK keputusan 3). Satu pindaian = satu struktur data.
// Hasilnya ditampilkan untuk dikonfirmasi petugas; server memvalidasi ulang isiannya.

export type Pindaian =
	| { jenis: 'din'; nomor_donasi: string; nomor: string }
	| { jenis: 'golongan'; abo: string; rhesus: 'pos' | 'neg' }
	| { jenis: 'produk'; kode_produk: string }
	| { jenis: 'kedaluwarsa'; waktu: Date }
	| { jenis: 'lain'; teks: string };

// Kode golongan ABO/RhD (struktur data 002), kode umum tanpa keterangan tambahan.
const GOLONGAN: Record<string, [string, 'pos' | 'neg']> = {
	'95': ['O', 'neg'],
	'51': ['O', 'pos'],
	'06': ['A', 'neg'],
	'62': ['A', 'pos'],
	'17': ['B', 'neg'],
	'73': ['B', 'pos'],
	'28': ['AB', 'neg'],
	'84': ['AB', 'pos']
};

// shortcut: digit abad (c) diabaikan dan tahun dianggap 20yy; perbarui sebelum tahun 2100.
function julian(cyyjjj: string, jam = 23, menit = 59): Date | null {
	const yy = Number(cyyjjj.slice(1, 3));
	const jjj = Number(cyyjjj.slice(3, 6));
	if (!Number.isInteger(yy) || !Number.isInteger(jjj) || jjj < 1 || jjj > 366) return null;
	return new Date(2000 + yy, 0, jjj, jam, menit);
}

export function urai(mentah: string): Pindaian {
	const t = mentah.trim();
	if (t.startsWith('=%') && t.length >= 4) {
		const g = GOLONGAN[t.slice(2, 4)];
		if (g) return { jenis: 'golongan', abo: g[0], rhesus: g[1] };
	}
	if (t.startsWith('=<') && t.length >= 7) return { jenis: 'produk', kode_produk: t.slice(2, 10) };
	if (t.startsWith('=>') && /^=>\d{6}$/.test(t)) {
		const w = julian(t.slice(2));
		if (w) return { jenis: 'kedaluwarsa', waktu: w };
	}
	if (t.startsWith('&>') && /^&>\d{10}$/.test(t)) {
		const w = julian(t.slice(2, 8), Number(t.slice(8, 10)), Number(t.slice(10, 12)));
		if (w) return { jenis: 'kedaluwarsa', waktu: w };
	}
	// Nomor donasi: '=' + kode fasilitas (5) + tahun (2) + urut (6) + flag (2).
	const din = /^=([A-NP-Z1-9][A-Z0-9]{4}\d{8})([A-Z0-9]{2})?$/.exec(t);
	if (din) return { jenis: 'din', nomor_donasi: din[1], nomor: din[1] };
	return { jenis: 'lain', teks: t };
}
