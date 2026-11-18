import { api } from './api.ts';

export type Profil = {
	akun_id: string;
	nama: string;
	nama_pengguna: string;
	peran: string[];
	hak: string[];
	wajib_ganti_sandi: boolean;
	jenis_instalasi: 'bdrs' | 'utd_rs';
	nama_rs: string;
};

class Sesi {
	profil = $state<Profil | null>(null);
	siap = $state(false);

	async muat() {
		try {
			this.profil = await api<Profil>('GET', '/auth/saya');
		} catch {
			this.profil = null;
		} finally {
			this.siap = true;
		}
	}

	boleh(hak: string) {
		return !!this.profil?.hak.includes(hak);
	}

	get modeUtd() {
		return this.profil?.jenis_instalasi === 'utd_rs';
	}

	async keluar() {
		try {
			await api('POST', '/auth/keluar', {});
		} finally {
			this.profil = null;
		}
	}
}

export const sesi = new Sesi();
