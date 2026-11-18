import { api } from './api.ts';
import { Sumber } from './sumber.svelte.ts';

/** Kebijakan kata sandi instalasi (Pengaturan → Kebijakan sandi); dimuat sekali untuk petunjuk di formulir. */
export type KebijakanSandi = { panjang_min: number; diubah_pada: string };
export const kebijakanSandi = new Sumber(() => api<KebijakanSandi>('GET', '/kebijakan-sandi'));

/** Panggil di dalam $effect halaman yang memakai minimalSandi() (tidak boleh saat render). */
export function muatKebijakanSandi() {
	if (!kebijakanSandi.data && !kebijakanSandi.memuat) kebijakanSandi.muat();
}

/** "Minimal N karakter" untuk teks bantuan. */
export const minimalSandi = () => `Minimal ${kebijakanSandi.data?.panjang_min ?? 10} karakter`;
