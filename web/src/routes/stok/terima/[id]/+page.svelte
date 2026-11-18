<script lang="ts">
	import { page } from '$app/state';
	import { api } from '#lib/api.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import { sesi } from '#lib/sesi.svelte.ts';
	import { NAMA_KOMPONEN, golongan, waktu } from '#lib/format.ts';
	import { suhu } from '#lib/stok/stok.ts';
	import Printer from '@lucide/svelte/icons/printer';

	const r = new Sumber(() => api<any>('GET', `/penerimaan/${page.params.id}`));
	$effect(() => {
		void page.params.id;
		r.muat();
	});
	const ditolak = (k: any) => k.ditolak_saat_terima === true;
</script>

<svelte:head><title>Berita serah terima — utd-rs</title></svelte:head>

<div class="tanpa-cetak mb-4 flex flex-wrap items-center justify-between gap-2">
	<a class="text-sm text-utama hover:underline" href="/stok/terima">Kembali ke penerimaan</a>
	<button type="button" class="btn" onclick={() => window.print()}><Printer class="size-4" aria-hidden="true" />Cetak</button>
</div>

<Keadaan sumber={r}>
	{#snippet children(d)}
		{@const kantong = d.kantong ?? []}
		<article class="panel print:border-0 print:p-0">
			<p class="text-sm text-redup">{sesi.profil?.nama_rs} · Bank Darah Rumah Sakit</p>
			<h1 class="mb-4">Berita serah terima kantong darah <span class="mono">{d.nomor}</span></h1>
			<dl class="mb-4 grid gap-x-6 gap-y-2 text-sm sm:grid-cols-2">
				<div><dt class="text-redup">Diterima</dt><dd>{waktu(d.diterima_pada)}</dd></div>
				<div><dt class="text-redup">UTD mitra</dt><dd>{d.utd.nama} ({d.utd.kode})</dd></div>
				<div><dt class="text-redup">Nomor dokumen UTD</dt><dd class="mono">{d.nomor_dokumen_utd ?? '—'}</dd></div>
				<div><dt class="text-redup">Suhu selama transportasi</dt><dd>{suhu(d.suhu_min_transport)} – {suhu(d.suhu_maks_transport)}</dd></div>
				<div><dt class="text-redup">Petugas UTD</dt><dd>{d.petugas_utd}</dd></div>
				<div><dt class="text-redup">Petugas BDRS</dt><dd>{d.petugas_bdrs}</dd></div>
				{#if d.catatan}<div class="sm:col-span-2"><dt class="text-redup">Catatan</dt><dd>{d.catatan}</dd></div>{/if}
			</dl>
			<p class="mb-2 text-sm">
				{kantong.length} kantong: {kantong.filter((k: any) => !ditolak(k)).length} diterima, {kantong.filter(ditolak).length} ditolak.
			</p>
			<div class="overflow-x-auto">
				<table class="tabel">
					<thead>
						<tr>
							<th scope="col">No.</th>
							<th scope="col">Nomor kantong</th>
							<th scope="col">Komponen</th>
							<th scope="col">Golongan</th>
							<th scope="col">Volume</th>
							<th scope="col">Kedaluwarsa (label)</th>
							<th scope="col">IMLTD</th>
							<th scope="col">Keputusan</th>
						</tr>
					</thead>
					<tbody>
						{#each kantong as k, i (k.id)}
							<tr>
								<td class="tabular-nums">{i + 1}</td>
								<td class="mono">{k.nomor}</td>
								<td>{NAMA_KOMPONEN[k.komponen]}{#if k.kode_produk}<span class="block text-xs text-redup">{k.kode_produk}</span>{/if}</td>
								<td class="font-semibold whitespace-nowrap">{golongan(k.abo, k.rhesus)}</td>
								<td class="tabular-nums">{k.volume_ml ? `${k.volume_ml} mL` : '—'}</td>
								<td class="whitespace-nowrap">{waktu(k.kedaluwarsa_label)}</td>
								<td>{k.imltd_nonreaktif ? 'Non-reaktif' : 'Tidak non-reaktif'}</td>
								<td>
									{#if ditolak(k)}
										<span class="font-medium text-bahaya">Ditolak</span><span class="block text-xs">{k.alasan_status}</span>
									{:else}
										<span class="font-medium text-aman">Diterima</span>
									{/if}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
			<div class="mt-10 grid grid-cols-2 gap-8 text-center text-sm break-inside-avoid">
				<div><p>Yang menyerahkan (UTD)</p><p class="mt-16 border-t border-garis pt-1">{d.petugas_utd}</p></div>
				<div><p>Yang menerima (BDRS)</p><p class="mt-16 border-t border-garis pt-1">{d.petugas_bdrs}</p></div>
			</div>
		</article>
	{/snippet}
</Keadaan>
