// Klien API. Galat dari server sudah berbahasa Indonesia dan menjelaskan apa yang harus dilakukan.

export type GalatIsian = { isian: string; pesan: string };

export class GalatApi extends Error {
	constructor(
		public status: number,
		pesan: string,
		public isian: GalatIsian[] = [],
		public data: unknown = null
	) {
		super(pesan);
	}

	/** Galat per isian sebagai peta kode → pesan, untuk ditampilkan di bawah field. */
	get perIsian(): Record<string, string> {
		return Object.fromEntries(this.isian.map((g) => [g.isian, g.pesan]));
	}
}

export async function api<T>(metode: string, path: string, body?: unknown): Promise<T> {
	let res: Response;
	try {
		res = await fetch(`/api${path}`, {
			method: metode,
			headers: body === undefined ? {} : { 'Content-Type': 'application/json' },
			body: body === undefined ? undefined : JSON.stringify(body),
			credentials: 'same-origin'
		});
	} catch {
		throw new GalatApi(0, 'Tidak bisa menghubungi server. Periksa jaringan, lalu coba lagi.');
	}
	if (res.status === 204) return undefined as T;
	const teks = await res.text();
	let data: any = null;
	try {
		data = teks ? JSON.parse(teks) : null;
	} catch {
		data = teks;
	}
	if (!res.ok) {
		if (res.status === 401 && path !== '/auth/masuk') {
			window.dispatchEvent(new CustomEvent('sesi-berakhir'));
		}
		throw new GalatApi(res.status, data?.pesan ?? `Server membalas ${res.status}.`, data?.isian ?? [], data);
	}
	return data as T;
}

export const pesanGalat = (e: unknown) => (e instanceof Error ? e.message : String(e));
