<!--
  Grafik suhu 7 hari satu alat simpan: garis pembacaan, garis putus-putus batas min/maks, titik di luar rentang
  ditandai merah. Tabel nilai di bawahnya adalah alternatif aksesibel dan bisa dibuka.
-->
<script lang="ts">
	import { api } from '#lib/api.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import { waktu } from '#lib/format.ts';
	import { suhu, type Alat } from './stok.ts';

	type Log = { id: number; waktu: string; suhu: number; sumber: string; petugas: string; catatan: string | null };
	let { alat }: { alat: Alat } = $props();

	const log = new Sumber(() => api<Log[]>('GET', `/suhu/${alat.id}`));
	$effect(() => {
		void alat.id;
		log.muat();
	});

	const L = 600, T = 220, KIRI = 44, KANAN = 8, ATAS = 10, BAWAH = 24;
	const HARI = 86_400_000;
	const tanggalPendek = new Intl.DateTimeFormat('id-ID', { day: '2-digit', month: 'short' });

	const g = $derived.by(() => {
		const data = (log.data ?? []).map((l) => ({ ...l, t: new Date(l.waktu).getTime(), v: Number(l.suhu) }));
		const akhir = Date.now();
		const awal = akhir - 7 * HARI;
		const nilai = [alat.suhu_min, alat.suhu_maks, ...data.map((d) => d.v)];
		const pad = Math.max(1, (alat.suhu_maks - alat.suhu_min) * 0.25);
		const lo = Math.min(...nilai) - pad;
		const hi = Math.max(...nilai) + pad;
		const x = (t: number) => KIRI + ((t - awal) / (akhir - awal)) * (L - KIRI - KANAN);
		const y = (v: number) => ATAS + ((hi - v) / (hi - lo)) * (T - ATAS - BAWAH);
		const hari = Array.from({ length: 8 }, (_, i) => awal + i * HARI);
		const titik = data.map((d) => ({ ...d, x: x(d.t), y: y(d.v), luar: d.v < alat.suhu_min || d.v > alat.suhu_maks }));
		return { data: titik, x, y, hari, garis: titik.map((d) => `${d.x},${d.y}`).join(' ') };
	});
	const luar = $derived(g.data.filter((d) => d.luar).length);
</script>

<Keadaan sumber={log} baris={4}>
	{#snippet kosong()}<p>Belum ada pembacaan suhu {alat.kode} dalam 7 hari terakhir.</p>{/snippet}
	{#snippet children(data)}
		<svg
			viewBox="0 0 {L} {T}"
			class="h-auto w-full"
			role="img"
			aria-label="Suhu {alat.kode} 7 hari terakhir: {data.length} pembacaan, {luar} di luar rentang {alat.suhu_min} sampai {alat.suhu_maks} °C."
		>
			<rect x={KIRI} width={L - KIRI - KANAN} y={g.y(alat.suhu_maks)} height={g.y(alat.suhu_min) - g.y(alat.suhu_maks)} class="fill-aman-latar" />
			{#each g.hari as h, i (h)}
				<line x1={g.x(h)} x2={g.x(h)} y1={ATAS} y2={T - BAWAH} class="stroke-garis" />
				{#if i < 7}<text x={g.x(h) + 2} y={T - 8} class="fill-redup text-[10px]">{tanggalPendek.format(h)}</text>{/if}
			{/each}
			{#each [alat.suhu_min, alat.suhu_maks] as b (b)}
				<line x1={KIRI} x2={L - KANAN} y1={g.y(b)} y2={g.y(b)} class="stroke-bahaya" stroke-dasharray="6 4" />
				<text x={KIRI - 4} y={g.y(b) + 3} text-anchor="end" class="fill-bahaya text-[10px]">{b}°</text>
			{/each}
			<polyline points={g.garis} fill="none" class="stroke-utama" stroke-width="2" stroke-linejoin="round" />
			{#each g.data as d (d.id)}
				<circle cx={d.x} cy={d.y} r={d.luar ? 4 : 2.5} class={d.luar ? 'fill-bahaya' : 'fill-utama'}>
					<title>{waktu(d.waktu)}: {suhu(d.v)}</title>
				</circle>
			{/each}
		</svg>
		<p class="mt-1 text-xs text-redup">Garis putus-putus merah: batas {suhu(alat.suhu_min)} dan {suhu(alat.suhu_maks)}. Pita hijau: rentang aman. Titik merah besar: di luar rentang.</p>
		<details class="mt-2">
			<summary class="cursor-pointer text-sm font-semibold text-teks-2 hover:underline">Tabel nilai ({data.length} pembacaan)</summary>
			<div class="mt-2 max-h-72 overflow-auto">
				<table class="tabel">
					<thead><tr><th scope="col">Waktu</th><th scope="col">Suhu</th><th scope="col">Sumber</th><th scope="col">Petugas</th></tr></thead>
					<tbody>
						{#each g.data.slice().reverse() as d (d.id)}
							<tr>
								<td class="whitespace-nowrap">{waktu(d.waktu)}</td>
								<td class="tabular-nums {d.luar ? 'font-semibold text-bahaya' : ''}">{suhu(d.v)}{d.luar ? ' (di luar rentang)' : ''}</td>
								<td>{d.sumber === 'logger' ? 'Data logger' : 'Manual'}</td>
								<td>{d.petugas}{#if d.catatan}<span class="block text-xs text-redup">{d.catatan}</span>{/if}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</details>
	{/snippet}
</Keadaan>
