import { api } from './api.ts';
import { Sumber } from './sumber.svelte.ts';

/** Kebijakan kata sandi instalasi (Pengaturan → Kebijakan sandi); dimuat sekali untuk petunjuk di formulir. */
export type KebijakanSandi = { panjang_min: number; diubah_pada: string };
export const kebijakanSandi = new Sumber(() => api<KebijakanSandi>('GET', '/kebijakan-sandi'));

/** "Minimal N karakter" untuk teks bantuan; memuat kebijakan bila belum. */
export function minimalSandi(): string {
	if (!kebijakanSandi.data && !kebijakanSandi.memuat && !kebijakanSandi.galat) kebijakanSandi.muat();
	return `Minimal ${kebijakanSandi.data?.panjang_min ?? 10} karakter`;
}
