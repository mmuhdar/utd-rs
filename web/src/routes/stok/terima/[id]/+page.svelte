<script lang="ts">
	import { merek } from '#lib/merek.svelte.ts';
	import Kop from '#lib/Kop.svelte';
	import { page } from '$app/state';
	import { api } from '#lib/api.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import { sesi } from '#lib/sesi.svelte.ts';
	import { NAMA_KOMPONEN, waktu } from '#lib/format.ts';
	import KepalaHalaman from '#lib/KepalaHalaman.svelte';
	import Golongan from '#lib/Golongan.svelte';
	import Chip from '#lib/Chip.svelte';
	import { suhu } from '#lib/stok/stok.ts';
	import Printer from '@lucide/svelte/icons/printer';

	const r = new Sumber(() => api<any>('GET', `/penerimaan/${page.params.id}`));
	$effect(() => {
		void page.params.id;
		r.muat();
	});
	const ditolak = (k: any) => k.ditolak_saat_terima === true;
</script>

<svelte:head><title>Berita serah terima — {merek.judul}</title></svelte:head>

<div class="tanpa-cetak">
	<KepalaHalaman
		judul="Berita serah terima"
		sub={r.data ? `${r.data.nomor} · ${r.data.utd.nama} · ${waktu(r.data.diterima_pada)}` : undefined}
		jalur={[['Stok kantong', '/stok'], ['Penerimaan dari UTD', '/stok/terima'], [r.data?.nomor ?? 'Berita serah terima']]}
	>
		{#snippet aksi()}
			<button type="button" class="btn btn--utama" onclick={() => window.print()}><Printer class="size-4" aria-hidden="true" />Cetak</button>
		{/snippet}
	</KepalaHalaman>
</div>

<Keadaan sumber={r}>
	{#snippet children(d)}
		{@const kantong = d.kantong ?? []}
		{@const nTolak = kantong.filter(ditolak).length}
		<section class="tanpa-cetak mb-5 grid gap-4 sm:grid-cols-2 xl:grid-cols-4" aria-label="Ringkasan">
			<div class="panel flex flex-col gap-1.5">
				<p class="text-sm font-semibold text-redup">Kantong</p>
				<p class="text-4xl font-extrabold tracking-tight tabular-nums">{kantong.length}</p>
			</div>
			<div class="panel flex flex-col gap-1.5">
				<p class="text-sm font-semibold text-redup">Diterima ke stok</p>
				<p class="text-4xl font-extrabold tracking-tight tabular-nums text-aman">{kantong.length - nTolak}</p>
			</div>
			<div class="panel flex flex-col gap-1.5 {nTolak ? 'border-bahaya-titik/60' : ''}">
				<p class="text-sm font-semibold text-redup">Ditolak</p>
				<p class="text-4xl font-extrabold tracking-tight tabular-nums {nTolak ? 'text-bahaya' : ''}">{nTolak}</p>
			</div>
			<div class="panel flex flex-col gap-1.5">
				<p class="text-sm font-semibold text-redup">Suhu transportasi</p>
				<p class="text-2xl font-extrabold tracking-tight tabular-nums">{suhu(d.suhu_min_transport)} – {suhu(d.suhu_maks_transport)}</p>
			</div>
		</section>

		<article class="panel print:border-0 print:p-0 print:shadow-none">
			<div class="mb-5"><Kop /></div>
			<h2 class="mb-5 text-xl font-extrabold tracking-tight">Berita serah terima kantong darah <span class="mono">{d.nomor}</span></h2>
			<dl class="mb-5 grid gap-x-6 gap-y-3 text-sm sm:grid-cols-2 lg:grid-cols-3 print:grid-cols-2">
				<div><dt class="judul-kecil">Diterima</dt><dd class="mt-0.5 font-medium">{waktu(d.diterima_pada)}</dd></div>
				<div><dt class="judul-kecil">UTD mitra</dt><dd class="mt-0.5 font-medium">{d.utd.nama} ({d.utd.kode})</dd></div>
				<div><dt class="judul-kecil">Nomor dokumen UTD</dt><dd class="mono mt-0.5">{d.nomor_dokumen_utd ?? '—'}</dd></div>
				<div><dt class="judul-kecil">Suhu selama transportasi</dt><dd class="mt-0.5 font-medium tabular-nums">{suhu(d.suhu_min_transport)} – {suhu(d.suhu_maks_transport)}</dd></div>
				<div><dt class="judul-kecil">Petugas UTD</dt><dd class="mt-0.5 font-medium">{d.petugas_utd}</dd></div>
				<div><dt class="judul-kecil">Petugas BDRS</dt><dd class="mt-0.5 font-medium">{d.petugas_bdrs}</dd></div>
				{#if d.catatan}<div class="sm:col-span-2 lg:col-span-3"><dt class="judul-kecil">Catatan</dt><dd class="mt-0.5">{d.catatan}</dd></div>{/if}
			</dl>
			<p class="mb-2 text-sm font-medium text-teks-2">
				{kantong.length} kantong: {kantong.length - nTolak} diterima, {nTolak} ditolak.
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
								<td class="tabular-nums text-redup">{i + 1}</td>
								<td class="mono font-semibold">{k.nomor}</td>
								<td>{NAMA_KOMPONEN[k.komponen]}{#if k.kode_produk}<span class="mono block text-xs text-redup">{k.kode_produk}</span>{/if}</td>
								<td><Golongan abo={k.abo} rhesus={k.rhesus} ukuran="kecil" /></td>
								<td class="tabular-nums">{k.volume_ml ? `${k.volume_ml} mL` : '—'}</td>
								<td class="whitespace-nowrap">{waktu(k.kedaluwarsa_label)}</td>
								<td>{k.imltd_nonreaktif ? 'Non-reaktif' : 'Tidak non-reaktif'}</td>
								<td>
									{#if ditolak(k)}
										<Chip nada="bahaya">Ditolak</Chip><span class="mt-1 block text-xs text-redup">{k.alasan_status}</span>
									{:else}
										<Chip nada="aman">Diterima</Chip>
									{/if}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
			<div class="mt-10 grid grid-cols-2 gap-8 text-center text-sm break-inside-avoid">
				<div><p class="text-redup">Yang menyerahkan (UTD)</p><p class="mt-16 border-t border-garis-kuat pt-1 font-medium">{d.petugas_utd}</p></div>
				<div><p class="text-redup">Yang menerima (BDRS)</p><p class="mt-16 border-t border-garis-kuat pt-1 font-medium">{d.petugas_bdrs}</p></div>
			</div>
		</article>
	{/snippet}
</Keadaan>
