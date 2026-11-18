// Data dan aturan tampilan bersama untuk layar bank darah (stok, penerimaan, suhu, berita acara).

export const ABO = ['A', 'B', 'AB', 'O'];
export const RHESUS: Record<string, string> = { pos: 'Rh+ (pos)', neg: 'Rh− (neg)' };

/** Pasangan golongan untuk tabel ringkasan (urutan sama dengan beranda). */
export const GOL: [string, string][] = [
	['O', 'pos'],
	['O', 'neg'],
	['A', 'pos'],
	['A', 'neg'],
	['B', 'pos'],
	['B', 'neg'],
	['AB', 'pos'],
	['AB', 'neg']
];

/** Jenis alat simpan per komponen, sama dengan bawaan `aturan_komponen.jenis_alat`. */
export const JENIS_ALAT: Record<string, string> = {
	wb: 'lemari',
	prc: 'lemari',
	tc: 'inkubator',
	ffp: 'freezer',
	cryo: 'freezer'
};

export const NAMA_JENIS_ALAT: Record<string, string> = {
	lemari: 'Lemari darah',
	freezer: 'Freezer plasma',
	inkubator: 'Inkubator trombosit'
};

/** Sama dengan `stok::ALASAN_AKHIR` di server. */
export const ALASAN_AKHIR = [
	'kelebihan stok',
	'kedaluwarsa',
	'bocor',
	'selang putus',
	'rusak',
	'tidak sesuai permintaan',
	'salah label',
	'golongan tidak sesuai label',
	'penyimpangan suhu',
	'reaktif IMLTD'
];

export type Alat = {
	id: string;
	kode: string;
	nama: string;
	nomor_seri: string | null;
	jenis: string;
	suhu_min: number;
	suhu_maks: number;
	lokasi: string | null;
	aktif: boolean;
	jumlah_kantong: number;
	penyimpangan_id: string | null;
	suhu_terakhir: { suhu: number; waktu: string; sumber: string } | null;
};

export type Utd = { id: string; kode: string; nama: string; format_label: 'isbt128' | 'lokal'; aktif: boolean };

/** Alat yang boleh menerima kantong: aktif, tanpa penyimpangan terbuka, jenisnya sesuai (plasma cair → lemari). */
export const alatUntuk = (alat: Alat[], komponen: string, dicairkan = false) =>
	alat.filter((a) => a.aktif && !a.penyimpangan_id && a.jenis === (dicairkan ? 'lemari' : JENIS_ALAT[komponen]));

export const labelAlat = (a: Alat) => `${a.kode} — ${a.nama} (${a.suhu_min}…${a.suhu_maks} °C)`;

export const suhu = (v: number | null | undefined) => (v == null ? '—' : `${Number(v).toFixed(1)} °C`);
