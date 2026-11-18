// Pilihan tema per browser: terang, gelap, atau ikuti sistem (bawaan). Pemasangan awal ada di static/tema.js.
export type Pilihan = 'terang' | 'gelap' | 'sistem';
const KUNCI = 'utd-tema';
const media = window.matchMedia('(prefers-color-scheme: dark)');

function baca(): Pilihan {
	try {
		const v = localStorage.getItem(KUNCI);
		if (v === 'terang' || v === 'gelap') return v;
	} catch {
		// penyimpanan diblokir: ikuti sistem
	}
	return 'sistem';
}

class Tema {
	pilihan = $state<Pilihan>(baca());

	atur(p: Pilihan) {
		this.pilihan = p;
		try {
			if (p === 'sistem') localStorage.removeItem(KUNCI);
			else localStorage.setItem(KUNCI, p);
		} catch {
			// tetap berlaku untuk sesi ini walau tidak tersimpan
		}
		this.terapkan();
	}

	terapkan() {
		const gelap = this.pilihan === 'gelap' || (this.pilihan === 'sistem' && media.matches);
		document.documentElement.dataset.tema = gelap ? 'gelap' : 'terang';
	}
}

export const tema = new Tema();
media.addEventListener('change', () => tema.terapkan());
