// Label domain UTD (GLOSSARY.md bagian Donor) yang dipakai halaman donor, donasi, dan laporan.
import { golongan } from '../format.ts';

export const PARAMETER_IMLTD: Record<string, string> = {
	hiv: 'HIV',
	hbsag: 'HBsAg',
	hcv: 'HCV',
	sifilis: 'Sifilis',
	malaria: 'Malaria'
};
export const IMLTD_WAJIB = ['hiv', 'hbsag', 'hcv', 'sifilis'];

export const METODE_IMLTD: Record<string, string> = { rapid: 'Rapid test', eia: 'EIA/ELISA', chlia: 'ChLIA', nat: 'NAT' };

export const STATUS_DONASI: Record<string, [string, string]> = {
	diambil: ['Menunggu pengolahan', 'waspada'],
	diolah: ['Diolah', 'info'],
	gagal: ['Gagal', 'netral']
};

/** Golongan gabungan dari server ("Bpos", "ABneg") ke teks tampilan. */
export const golonganGabung = (g?: string | null) => {
	const m = g ? /^(A|B|AB|O)(pos|neg)$/.exec(g) : null;
	return m ? golongan(m[1], m[2]) : '—';
};
