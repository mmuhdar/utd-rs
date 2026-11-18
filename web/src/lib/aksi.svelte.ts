import { GalatApi, pesanGalat } from './api.ts';

/** Keadaan satu aksi (kirim formulir): sibuk, pesan galat umum, dan galat per isian dari server. */
export class Aksi {
	sibuk = $state(false);
	galat = $state<string | null>(null);
	isian = $state<Record<string, string>>({});
	berhasil = $state<string | null>(null);

	async jalankan<T>(fn: () => Promise<T>, pesanBerhasil?: string): Promise<T | undefined> {
		this.sibuk = true;
		this.galat = null;
		this.berhasil = null;
		this.isian = {};
		try {
			const v = await fn();
			this.berhasil = pesanBerhasil ?? null;
			return v;
		} catch (e) {
			this.galat = pesanGalat(e);
			if (e instanceof GalatApi) this.isian = e.perIsian;
			return undefined;
		} finally {
			this.sibuk = false;
		}
	}
}
