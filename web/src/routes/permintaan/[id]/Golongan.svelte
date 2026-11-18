<!-- Pemeriksaan golongan pasien per sampel: hasil, diskrepansi, verifikasi kedua, atau input baru. -->
<script lang="ts">
	import Chip from '#lib/Chip.svelte';
	import { sesi } from '#lib/sesi.svelte.ts';
	import { golongan, waktu } from '#lib/format.ts';
	import type { Minta } from '#lib/pelayanan/TandaTangan.svelte';
	import { METODE_GOLONGAN, tanda } from '#lib/pelayanan/darah.ts';
	import FormGolongan from './FormGolongan.svelte';
	import Verifikasi from './Verifikasi.svelte';
	import TriangleAlert from '@lucide/svelte/icons/triangle-alert';

	let { p, aktif, muat, minta }: { p: any; aktif: boolean; muat: () => Promise<void>; minta: Minta } = $props();
	const sampel = $derived(p.sampel.filter((s: any) => s.golongan || (s.layak && aktif)));
</script>

<section class="panel flex flex-col gap-4" aria-labelledby="j-golongan">
	<h2 id="j-golongan">Pemeriksaan golongan pasien</h2>
	{#each sampel as s (s.id)}
		{@const g = s.golongan}
		<div class="flex flex-col gap-3 rounded-md border border-garis p-3">
			<p class="text-sm text-redup">Sampel diambil {waktu(s.diambil_pada)}</p>
			{#if g}
				<div class="overflow-x-auto">
					<table class="tabel">
						<thead>
							<tr>
								<th scope="col">Anti-A</th><th scope="col">Anti-B</th><th scope="col">Anti-D</th><th scope="col">Sel A</th><th scope="col">Sel B</th>
								<th scope="col">Metode</th><th scope="col">Hasil</th><th scope="col">Petugas</th>
							</tr>
						</thead>
						<tbody>
							<tr>
								<td class="mono">{tanda(g.anti_a)}</td><td class="mono">{tanda(g.anti_b)}</td><td class="mono">{tanda(g.anti_d)}</td>
								<td class="mono">{tanda(g.sel_a)}</td><td class="mono">{tanda(g.sel_b)}</td>
								<td>{METODE_GOLONGAN[g.metode] ?? g.metode}</td>
								<td class="text-base font-semibold">{golongan(g.abo, g.rhesus)}</td>
								<td>{g.petugas}<span class="block text-xs text-redup">{waktu(g.dicatat_pada)}</span></td>
							</tr>
						</tbody>
					</table>
				</div>
				{#if g.diskrepansi}
					<div class="peringatan" role="alert">
						<TriangleAlert class="mt-0.5 size-5 shrink-0" aria-hidden="true" />
						<p>Diskrepansi: hasil {golongan(g.abo, g.rhesus)} berbeda dari golongan terverifikasi sebelumnya untuk no. RM ini. Sampel tidak dapat dipakai; minta sampel baru sebelum darah dikeluarkan.</p>
					</div>
				{/if}
				<div class="flex flex-wrap items-center gap-3">
					{#if g.terverifikasi}
						<Chip nada="aman">Terverifikasi</Chip>
					{:else}
						<Chip nada="waspada">Menunggu verifikasi kedua</Chip>
						{#if sesi.boleh('uji.verifikasi')}
							<Verifikasi
								url="/golongan/{g.id}/verifikasi"
								penginput={g.dicatat_oleh}
								apa="golongan darah pasien"
								{minta}
								onselesai={muat}
							/>
						{/if}
					{/if}
				</div>
			{:else if sesi.boleh('uji.input')}
				<FormGolongan sampel={s.id} onselesai={muat} />
			{:else}
				<p class="text-sm text-redup">Belum diperiksa.</p>
			{/if}
		</div>
	{:else}
		<p class="text-sm text-redup">Belum ada sampel layak untuk diperiksa.</p>
	{/each}
</section>
