import { api } from './api.ts';
import { Sumber } from './sumber.svelte.ts';

/** Kop surat dokumen cetak (Pengaturan → Kop surat). */
export type Kop = {
	pemerintah: string;
	nama_rs: string;
	unit: string;
	alamat: string;
	kontak: string;
	akreditasi: string;
	logo_kiri: string;
	logo_kanan: string;
	nama_rs_profil?: string;
};

/** Satu salinan untuk semua halaman; dimuat sekali, diperbarui setelah disimpan di halaman pengaturan. */
export const kopSurat = new Sumber(() => api<Kop>('GET', '/master/kop'));
