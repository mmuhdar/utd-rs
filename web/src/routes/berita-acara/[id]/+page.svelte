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
	import Printer from '@lucide/svelte/icons/printer';

	const b = new Sumber(() => api<any>('GET', `/berita-acara/${page.params.id}`));
	$effect(() => {
		void page.params.id;
		b.muat();
	});
</script>

<svelte:head><title>Berita acara — {merek.judul}</title></svelte:head>

<div class="tanpa-cetak">
	<KepalaHalaman
		judul={b.data ? (b.data.jenis === 'pengembalian' ? 'Berita acara pengembalian' : 'Berita acara pemusnahan') : 'Berita acara'}
		sub={b.data ? `${b.data.nomor} · ${waktu(b.data.dibuat_pada)} · ${(b.data.kantong ?? []).length} kantong` : undefined}
		jalur={[['Pengembalian dan pemusnahan', '/berita-acara'], [b.data?.nomor ?? 'Berita acara']]}
	>
		{#snippet aksi()}
			<button type="button" class="btn btn--utama" onclick={() => window.print()}><Printer class="size-4" aria-hidden="true" />Cetak</button>
		{/snippet}
	</KepalaHalaman>
</div>

<Keadaan sumber={b}>
	{#snippet children(d)}
		{@const kembali = d.jenis === 'pengembalian'}
		{@const kantong = d.kantong ?? []}
		<article class="panel print:border-0 print:p-0 print:shadow-none">
			<div class="mb-5"><Kop /></div>
			<div class="mb-5 flex flex-wrap items-start justify-between gap-3">
				<h2 class="text-xl font-extrabold tracking-tight">
					Berita acara {kembali ? 'pengembalian kantong darah ke UTD' : 'pemusnahan kantong darah'}
					<span class="mono">{d.nomor}</span>
				</h2>
				<span class="tanpa-cetak"><Chip nada={kembali ? 'info' : 'bahaya'}>{kembali ? 'Pengembalian' : 'Pemusnahan'}</Chip></span>
			</div>
			<dl class="mb-5 grid gap-x-6 gap-y-3 text-sm sm:grid-cols-2 lg:grid-cols-4 print:grid-cols-2">
				<div><dt class="judul-kecil">Tanggal</dt><dd class="mt-0.5 font-medium">{waktu(d.dibuat_pada)}</dd></div>
				<div><dt class="judul-kecil">Petugas BDRS</dt><dd class="mt-0.5 font-medium">{d.petugas}</dd></div>
				{#if kembali}
					<div><dt class="judul-kecil">UTD tujuan</dt><dd class="mt-0.5 font-medium">{d.utd?.nama} ({d.utd?.kode})</dd></div>
				{:else}
					<div><dt class="judul-kecil">Metode pemusnahan</dt><dd class="mt-0.5 font-medium">{d.metode}</dd></div>
				{/if}
				<div><dt class="judul-kecil">Saksi</dt><dd class="mt-0.5 font-medium">{d.saksi ?? '—'}</dd></div>
				{#if d.catatan}<div class="sm:col-span-2 lg:col-span-4"><dt class="judul-kecil">Catatan</dt><dd class="mt-0.5">{d.catatan}</dd></div>{/if}
			</dl>
			<p class="mb-2 text-sm font-medium text-teks-2">Jumlah: {kantong.length} kantong.</p>
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
								<td class="tabular-nums text-redup">{i + 1}</td>
								<td class="mono font-semibold">{k.nomor}</td>
								<td>{NAMA_KOMPONEN[k.komponen]}</td>
								<td><Golongan abo={k.abo} rhesus={k.rhesus} ukuran="kecil" /></td>
								<td class="tabular-nums">{k.volume_ml ? `${k.volume_ml} mL` : '—'}</td>
								<td class="whitespace-nowrap">{waktu(k.kedaluwarsa)}</td>
								<td>{k.alasan ?? '—'}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
			<div class="mt-10 grid grid-cols-2 gap-8 text-center text-sm break-inside-avoid">
				<div><p class="text-redup">Petugas BDRS</p><p class="mt-16 border-t border-garis-kuat pt-1 font-medium">{d.petugas}</p></div>
				<div>
					<p class="text-redup">{kembali ? 'Penerima (UTD)' : 'Saksi'}</p>
					<p class="mt-16 border-t border-garis-kuat pt-1 font-medium">{kembali ? '' : (d.saksi ?? '')}&nbsp;</p>
				</div>
			</div>
		</article>
	{/snippet}
</Keadaan>
