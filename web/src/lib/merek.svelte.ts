import { api } from './api.ts';

/** Identitas aplikasi dari kop surat (Pengaturan → Kop surat), seperti lis-rebuild: judul = nama RS, logo = logo kop. */
type Info = { nama_rs: string; unit: string; ada_logo: boolean; versi: number };

class Merek {
	info = $state<Info | null>(null);
	/** Logo gagal dimuat: pakai ikon bawaan. */
	rusak = $state(false);

	async muat() {
		try {
			this.info = await api<Info>('GET', '/auth/info');
			this.rusak = false;
		} catch {
			// tetap memakai judul dan ikon bawaan
		}
	}
	get judul() {
		return this.info?.nama_rs || 'utd-rs';
	}
	get unit() {
		return this.info?.unit || 'Bank Darah Rumah Sakit';
	}
	/** `versi` berubah saat kop disimpan, jadi browser mengambil logo baru. */
	get logo() {
		return this.info?.ada_logo && !this.rusak ? `/api/auth/logo?v=${this.info.versi}` : null;
	}
}

export const merek = new Merek();
