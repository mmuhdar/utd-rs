// Tahap alur permintaan (SPEK M3–M5), dihitung dari detail permintaan. Dipakai penunjuk tahap dan kartu yang dilipat.
import { golongan, pukul } from '#lib/format.ts';

export function daftarTahap(p: any, aturan: Record<string, string>) {
	const total = p.baris.reduce((n: number, b: any) => n + b.jumlah, 0);
	const totalUji = p.baris.filter((b: any) => aturan[b.komponen] !== 'tidak_perlu').reduce((n: number, b: any) => n + b.jumlah, 0);
	const layak = p.sampel.find((s: any) => s.layak);
	const gol = p.golongan_terverifikasi;
	const serasi = p.uji_silang.filter((u: any) => u.terverifikasi && ['serasi', 'hanya_prc'].includes(u.kesimpulan)).length;
	const alok = p.alokasi.filter((a: any) => !a.dilepas_pada);
	const keluar = alok.filter((a: any) => a.pengeluaran).length;
	const transfusi = alok.filter((a: any) => a.transfusi).length;
	return [
		{ nama: 'Sampel', ke: 'bagian-sampel', selesai: !!layak, ket: layak ? `Diterima ${pukul(layak.diterima_pada)}` : p.sampel.length ? 'Tidak layak' : 'Belum diterima' },
		{ nama: 'Golongan darah', ke: 'bagian-golongan', selesai: !!gol, ket: gol ? `${golongan(gol.abo, gol.rhesus)} terverifikasi` : 'Belum terverifikasi' },
		{
			nama: 'Uji silang serasi',
			ke: 'bagian-uji',
			selesai: serasi >= totalUji,
			ket: totalUji ? `${serasi} dari ${totalUji} serasi` : 'Tidak diperlukan'
		},
		{ nama: 'Alokasi', ke: 'bagian-alokasi', selesai: alok.length >= total, ket: `${alok.length} dari ${total} kantong` },
		{ nama: 'Pengeluaran', ke: 'bagian-alokasi', selesai: keluar >= total, ket: `${keluar} dari ${total} kantong` },
		{ nama: 'Transfusi', ke: 'bagian-alokasi', selesai: transfusi >= total, ket: `${transfusi} dari ${total} kantong` }
	];
	}

/** Indeks tahap yang sedang dikerjakan, atau -1 bila permintaan sudah tidak berjalan. */
export function tahapSekarang(p: any, aturan: Record<string, string>) {
	if (!['diminta', 'diproses', 'siap'].includes(p.status)) return -1;
	return daftarTahap(p, aturan).findIndex((t) => !t.selesai);
}
