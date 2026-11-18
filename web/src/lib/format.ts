// Format tampilan dan label domain (GLOSSARY.md). Waktu ditampilkan dalam zona waktu browser.

export const KOMPONEN: Record<string, string> = {
	wb: 'WB',
	prc: 'PRC',
	tc: 'TC',
	ffp: 'FFP',
	cryo: 'Kriopresipitat'
};

export const NAMA_KOMPONEN: Record<string, string> = {
	wb: 'Darah lengkap (WB)',
	prc: 'Sel darah merah pekat (PRC)',
	tc: 'Trombosit pekat (TC)',
	ffp: 'Plasma segar beku (FFP)',
	cryo: 'Kriopresipitat'
};

export const golongan = (abo?: string | null, rh?: string | null) =>
	abo ? `${abo} ${rh === 'neg' ? 'Rh−' : rh === 'pos' ? 'Rh+' : ''}`.trim() : '—';

const tgl = new Intl.DateTimeFormat('id-ID', { day: '2-digit', month: 'short', year: 'numeric' });
const tglJam = new Intl.DateTimeFormat('id-ID', {
	day: '2-digit',
	month: 'short',
	year: 'numeric',
	hour: '2-digit',
	minute: '2-digit'
});
const jam = new Intl.DateTimeFormat('id-ID', { hour: '2-digit', minute: '2-digit' });

export const tanggal = (v?: string | null) => (v ? tgl.format(new Date(v)) : '—');
export const waktu = (v?: string | null) => (v ? tglJam.format(new Date(v)) : '—');
export const pukul = (v?: string | null) => (v ? jam.format(new Date(v)) : '—');

/** "dalam 3 hari", "2 jam lalu" — untuk kedaluwarsa dan antrean. */
export function relatif(v?: string | null): string {
	if (!v) return '—';
	const menit = Math.round((new Date(v).getTime() - Date.now()) / 60000);
	const abs = Math.abs(menit);
	const teks =
		abs < 60 ? `${abs} menit` : abs < 48 * 60 ? `${Math.round(abs / 60)} jam` : `${Math.round(abs / 1440)} hari`;
	return menit >= 0 ? `dalam ${teks}` : `${teks} lalu`;
}

/** Nilai untuk <input type="datetime-local"> dari Date (waktu lokal). */
export function lokal(d = new Date()): string {
	const p = (n: number) => String(n).padStart(2, '0');
	return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}T${p(d.getHours())}:${p(d.getMinutes())}`;
}

/** Nilai datetime-local → RFC 3339 dengan offset browser. */
export const keRfc3339 = (v: string) => new Date(v).toISOString();

export const hariIni = () => lokal().slice(0, 10);

export const STATUS_KANTONG: Record<string, [string, string]> = {
	karantina: ['Karantina', 'waspada'],
	tersedia: ['Tersedia', 'aman'],
	dialokasikan: ['Dialokasikan', 'info'],
	dikeluarkan: ['Dikeluarkan', 'info'],
	ditransfusikan: ['Ditransfusikan', 'netral'],
	dikembalikan: ['Dikembalikan', 'netral'],
	dimusnahkan: ['Dimusnahkan', 'netral']
};

export const STATUS_PERMINTAAN: Record<string, [string, string]> = {
	diminta: ['Menunggu sampel', 'waspada'],
	diproses: ['Diproses', 'info'],
	siap: ['Siap diambil', 'aman'],
	selesai: ['Selesai', 'netral'],
	batal: ['Batal', 'netral'],
	ditolak: ['Ditolak', 'bahaya']
};

/** Dua huruf awal nama untuk avatar (gelar seperti "dr." dilewati). */
export const inisial = (nama: string) =>
	nama
		.split(/\s+/)
		.filter((k) => /^[A-Za-z]/.test(k) && !k.endsWith('.'))
		.slice(0, 2)
		.map((k) => k[0].toUpperCase())
		.join('');
