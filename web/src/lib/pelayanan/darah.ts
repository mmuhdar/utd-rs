// Label dan aturan pelayanan pasien. Daftar tetap dan aturan di sini adalah cermin backend (sumber kebenarannya
// di Rust; server tetap memvalidasi ulang). Tampilan di sini hanya untuk memandu petugas sebelum mengirim.

/** Cermin `permintaan::ALASAN_TOLAK_SAMPEL` (belum ada endpoint yang mengeluarkannya). */
export const ALASAN_TOLAK_SAMPEL = ['lisis', 'bekuan', 'volume kurang', 'label tidak sesuai', 'tanpa label'];

/** Cermin `transfusi::GEJALA`. */
export const GEJALA = [
	'demam',
	'menggigil',
	'urtikaria/gatal',
	'sesak napas',
	'hipotensi',
	'takikardia',
	'nyeri dada',
	'nyeri pinggang',
	'urin gelap/hemoglobinuria',
	'mual/muntah',
	'syok',
	'perdarahan tidak terkontrol',
	'ikterus',
	'lainnya'
];

export const ABO = ['A', 'B', 'AB', 'O'];
export const POS_NEG: [string, string][] = [
	['pos', 'Positif (+)'],
	['neg', 'Negatif (−)']
];
export const RH: [string, string][] = [
	['pos', 'Rh+'],
	['neg', 'Rh−']
];
export const YA_TIDAK: [boolean, string][] = [
	[true, 'Ya'],
	[false, 'Tidak']
];

export const JENIS_PERMINTAAN: Record<string, [string, string]> = {
	elektif: ['Elektif', 'netral'],
	rutin: ['Rutin', 'netral'],
	darurat: ['Darurat', 'bahaya']
};

export const STATUS_SAMPEL: Record<string, [string, string]> = {
	diterima: ['Diterima', 'aman'],
	ditolak: ['Ditolak', 'bahaya'],
	diskrepansi: ['Diskrepansi', 'bahaya']
};

export const JENIS_SAMPEL: Record<string, string> = { edta: 'EDTA', serum: 'Serum/plasma' };
export const PENYIMPANAN: Record<string, string> = { suhu_ruang: 'Suhu ruang', dingin: 'Dingin (4 °C)' };

/** Cermin `permintaan::batas_usia`, dalam jam. */
export function batasUsiaJam(jenis: string, penyimpanan: string): number | null {
	if (jenis === 'edta') return penyimpanan === 'suhu_ruang' ? 24 : 72;
	return penyimpanan === 'dingin' ? 168 : null;
}

export const METODE_GOLONGAN: Record<string, string> = { tabung: 'Tabung', gel: 'Gel', bioplate: 'Bioplate' };
export const METODE_UJI_SILANG: Record<string, string> = { tabung: 'Tabung', gel: 'Gel', immediate_spin: 'Immediate spin' };

export const KESIMPULAN: Record<string, [string, string]> = {
	serasi: ['Serasi', 'aman'],
	hanya_prc: ['Hanya PRC', 'waspada'],
	tidak_serasi: ['Tidak serasi', 'bahaya'],
	golongan_beda: ['Golongan kantong tidak sesuai label', 'bahaya']
};

export const TAHAP_OBSERVASI: Record<string, string> = {
	pra: 'Pra-transfusi',
	'15_menit': '15 menit',
	berkala: 'Berkala',
	selesai: 'Selesai',
	pasca: 'Pasca-transfusi'
};

export const STATUS_REAKSI: Record<string, [string, string]> = {
	dilaporkan: ['Menunggu penelusuran', 'waspada'],
	ditelusuri: ['Ditelusuri', 'info'],
	ditutup: ['Ditutup', 'netral']
};

export const UJI_SILANG_ULANG: Record<string, string> = {
	serasi: 'Serasi',
	tidak_serasi: 'Tidak serasi',
	tidak_dilakukan: 'Tidak dilakukan'
};

export const JENIS_INSIDEN: Record<string, string> = {
	knc: 'Kejadian nyaris cedera (KNC)',
	ktc: 'Kejadian tidak cedera (KTC)',
	ktd: 'Kejadian tidak diharapkan (KTD)',
	kpc: 'Kondisi potensial cedera (KPC)'
};

export const tanda = (h?: string | null) => (h === 'pos' ? '+' : h === 'neg' ? '−' : '—');

/** Cermin `pratransfusi::tafsir_abo`: ABO dari cell grouping dan apakah serum grouping konsisten. */
export function tafsirAbo(antiA: string, antiB: string, selA: string, selB: string): [string, boolean] {
	const abo = antiA === 'pos' ? (antiB === 'pos' ? 'AB' : 'A') : antiB === 'pos' ? 'B' : 'O';
	const cocok = (selA === 'pos') === !abo.includes('A') && (selB === 'pos') === !abo.includes('B');
	return [abo, cocok];
}

/** Cermin `pratransfusi::kompatibel`. */
export function kompatibel(komponen: string, aboP: string, rhP: string, aboK: string, rhK: string): boolean {
	const [da, db] = [aboK.includes('A'), aboK.includes('B')];
	const [pa, pb] = [aboP.includes('A'), aboP.includes('B')];
	const abo =
		komponen === 'wb' ? aboK === aboP : komponen === 'prc' ? (!da || pa) && (!db || pb) : (!pa || da) && (!pb || db);
	const rh = komponen === 'ffp' || komponen === 'cryo' || rhP === 'pos' || rhK === 'neg';
	return abo && rh;
}

/** Cermin `pratransfusi::kesimpulan`. */
export function kesimpulan(komponen: string, mayor?: string, minor?: string, auto?: string): string {
	if (mayor === 'pos') return 'tidak_serasi';
	if (minor === 'pos' || auto === 'pos') return komponen === 'prc' ? 'hanya_prc' : 'tidak_serasi';
	return 'serasi';
}

/** Angka dari isian `<input type="number">` (string kosong atau null → undefined). */
export const angka = (v: unknown) => (v === '' || v === null || v === undefined ? undefined : Number(v));
