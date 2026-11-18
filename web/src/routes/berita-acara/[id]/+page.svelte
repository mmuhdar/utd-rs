<script lang="ts">
	import Kop from '#lib/Kop.svelte';
	import { page } from '$app/state';
	import { api } from '#lib/api.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import { sesi } from '#lib/sesi.svelte.ts';
	import { NAMA_KOMPONEN, golongan, waktu } from '#lib/format.ts';
	import Printer from '@lucide/svelte/icons/printer';

	const b = new Sumber(() => api<any>('GET', `/berita-acara/${page.params.id}`));
	$effect(() => {
		void page.params.id;
		b.muat();
	});
</script>

<svelte:head><title>Berita acara — utd-rs</title></svelte:head>

<div class="tanpa-cetak mb-4 flex flex-wrap items-center justify-between gap-2">
	<a class="text-sm text-utama hover:underline" href="/berita-acara">Kembali ke daftar</a>
	<button type="button" class="btn" onclick={() => window.print()}><Printer class="size-4" aria-hidden="true" />Cetak</button>
</div>

<Keadaan sumber={b}>
	{#snippet children(d)}
		{@const kembali = d.jenis === 'pengembalian'}
		{@const kantong = d.kantong ?? []}
		<article class="panel print:border-0 print:p-0">
			<div class="mb-5"><Kop /></div>
			<h1 class="mb-4">
				Berita acara {kembali ? 'pengembalian kantong darah ke UTD' : 'pemusnahan kantong darah'}
				<span class="mono">{d.nomor}</span>
			</h1>
			<dl class="mb-4 grid gap-x-6 gap-y-2 text-sm sm:grid-cols-2">
				<div><dt class="text-redup">Tanggal</dt><dd>{waktu(d.dibuat_pada)}</dd></div>
				<div><dt class="text-redup">Petugas BDRS</dt><dd>{d.petugas}</dd></div>
				{#if kembali}
					<div><dt class="text-redup">UTD tujuan</dt><dd>{d.utd?.nama} ({d.utd?.kode})</dd></div>
				{:else}
					<div><dt class="text-redup">Metode pemusnahan</dt><dd>{d.metode}</dd></div>
				{/if}
				<div><dt class="text-redup">Saksi</dt><dd>{d.saksi ?? '—'}</dd></div>
				{#if d.catatan}<div class="sm:col-span-2"><dt class="text-redup">Catatan</dt><dd>{d.catatan}</dd></div>{/if}
			</dl>
			<p class="mb-2 text-sm">Jumlah: {kantong.length} kantong.</p>
			<div class="overflow-x-auto">
				<table class="tabel">
					<thead>
						<tr>
							<th scope="col">No.</th>
							<th scope="col">Nomor kantong</th>
							<th scope="col">Komponen</th>
							<th scope="col">Golongan</th>
							<th scope="col">Volume</th>
							<th scope="col">Kedaluwarsa</th>
							<th scope="col">Alasan</th>
						</tr>
					</thead>
					<tbody>
						{#each kantong as k, i (k.nomor)}
							<tr>
								<td class="tabular-nums">{i + 1}</td>
								<td class="mono">{k.nomor}</td>
								<td>{NAMA_KOMPONEN[k.komponen]}</td>
								<td class="font-semibold whitespace-nowrap">{golongan(k.abo, k.rhesus)}</td>
								<td class="tabular-nums">{k.volume_ml ? `${k.volume_ml} mL` : '—'}</td>
								<td class="whitespace-nowrap">{waktu(k.kedaluwarsa)}</td>
								<td>{k.alasan ?? '—'}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
			<div class="mt-10 grid grid-cols-2 gap-8 text-center text-sm break-inside-avoid">
				<div><p>Petugas BDRS</p><p class="mt-16 border-t border-garis pt-1">{d.petugas}</p></div>
				<div>
					<p>{kembali ? 'Penerima (UTD)' : 'Saksi'}</p>
					<p class="mt-16 border-t border-garis pt-1">{kembali ? '' : (d.saksi ?? '')}&nbsp;</p>
				</div>
			</div>
		</article>
	{/snippet}
</Keadaan>
