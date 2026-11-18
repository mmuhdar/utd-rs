import { GalatApi, pesanGalat } from './api.ts';

/** Data yang dimuat dari server beserta keadaannya: memuat, galat, atau tersedia. */
export class Sumber<T> {
	data = $state<T | undefined>(undefined);
	galat = $state<string | null>(null);
	/** Mengulang hanya berguna untuk gangguan jaringan atau server, bukan hak akses atau data yang tidak ada. */
	bisaDiulang = $state(true);
	memuat = $state(false);
	#ambil: () => Promise<T>;

	constructor(ambil: () => Promise<T>) {
		this.#ambil = ambil;
	}

	async muat() {
		this.memuat = true;
		this.galat = null;
		try {
			this.data = await this.#ambil();
		} catch (e) {
			this.galat = pesanGalat(e);
			this.bisaDiulang = !(e instanceof GalatApi) || e.status === 0 || e.status >= 500;
		} finally {
			this.memuat = false;
		}
	}
}
