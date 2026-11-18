<script lang="ts">
	import { merek } from '#lib/merek.svelte.ts';
	import { page } from '$app/state';
	import { api } from '#lib/api.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import Chip from '#lib/Chip.svelte';
	import KepalaHalaman from '#lib/KepalaHalaman.svelte';
	import { KOMPONEN, waktu } from '#lib/format.ts';
	import { JENIS_INSIDEN, STATUS_REAKSI } from '#lib/pelayanan/darah.ts';

	const tab = $derived(page.url.searchParams.get('tab') === 'insiden' ? 'insiden' : 'reaksi');
	const reaksi = new Sumber(() => api<any[]>('GET', '/reaksi'));
	const insiden = new Sumber(() => api<any[]>('GET', '/insiden'));
	const aktif = $derived(tab === 'insiden' ? insiden : reaksi);
	$effect(() => {
		aktif.muat();
	});
	const terbuka = $derived(reaksi.data?.filter((r) => r.status !== 'ditutup').length);
</script>

<svelte:head><title>Reaksi & insiden — {merek.judul}</title></svelte:head>

<KepalaHalaman
	judul="Reaksi transfusi dan insiden"
	jalur={[['Pelayanan'], ['Reaksi & insiden']]}
	sub={terbuka != null ? `${terbuka} laporan reaksi belum ditutup` : undefined}
/>

<nav class="mb-4 flex w-fit max-w-full flex-wrap gap-0.5 rounded-[10px] bg-halus p-1" aria-label="Jenis laporan">
	<a class="rounded-lg px-3 py-1.5 text-sm font-medium text-redup hover:text-teks aria-[current=page]:bg-kartu aria-[current=page]:font-semibold aria-[current=page]:text-teks aria-[current=page]:shadow-kartu" href="/reaksi" aria-current={tab === 'reaksi' ? 'page' : undefined}>Reaksi transfusi</a>
	<a class="rounded-lg px-3 py-1.5 text-sm font-medium text-redup hover:text-teks aria-[current=page]:bg-kartu aria-[current=page]:font-semibold aria-[current=page]:text-teks aria-[current=page]:shadow-kartu" href="?tab=insiden" aria-current={tab === 'insiden' ? 'page' : undefined}>Insiden (KNC dan lainnya)</a>
</nav>

<section class="panel" aria-labelledby="j-daftar">
	<div class="mb-3 flex flex-wrap items-baseline justify-between gap-2">
		<h2 id="j-daftar">{tab === 'reaksi' ? 'Yang belum ditutup di atas' : 'Terbaru di atas'}</h2>
		{#if aktif.data}<p class="text-sm text-redup">{aktif.data.length} laporan</p>{/if}
	</div>
	{#if tab === 'reaksi'}
		<Keadaan sumber={reaksi}>
			{#snippet kosong()}<p>Belum ada laporan reaksi transfusi.</p>{/snippet}
			{#snippet children(data)}
				<div class="overflow-x-auto">
					<table class="tabel">
						<caption class="sr-only">Laporan reaksi transfusi; yang belum ditutup di atas</caption>
						<thead>
							<tr>
								<th scope="col">Nomor</th><th scope="col">Waktu reaksi</th><th scope="col">Pasien</th><th scope="col">Kantong</th>
								<th scope="col">Gejala</th><th scope="col">Kategori</th><th scope="col">Status</th>
							</tr>
						</thead>
						<tbody>
							{#each data as r (r.id)}
								{@const s = STATUS_REAKSI[r.status]}
								<tr>
									<td><a class="mono font-semibold hover:underline" href="/reaksi/{r.id}">{r.nomor}</a></td>
									<td class="whitespace-nowrap">{waktu(r.waktu_reaksi)}</td>
									<td><span class="font-semibold">{r.pasien.nama}</span><span class="mono block text-xs text-redup">{r.pasien.no_rm}</span></td>
									<td><span class="mono font-semibold">{r.nomor_kantong}</span><span class="block text-xs text-redup">{KOMPONEN[r.komponen]}</span></td>
									<td>{r.gejala.join(', ')}</td>
									<td><Chip nada={r.kategori === 'akut' ? 'bahaya' : 'waspada'}>{r.kategori === 'akut' ? 'Akut' : 'Lambat'}</Chip></td>
									<td><Chip nada={s?.[1]}>{s?.[0] ?? r.status}</Chip></td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{/snippet}
		</Keadaan>
	{:else}
		<Keadaan sumber={insiden}>
			{#snippet kosong()}<p>Belum ada insiden tercatat.</p>{/snippet}
			{#snippet children(data)}
				<div class="overflow-x-auto">
					<table class="tabel">
						<caption class="sr-only">Insiden keselamatan pasien terkait darah, terbaru di atas</caption>
						<thead>
							<tr><th scope="col">Waktu</th><th scope="col">Jenis</th><th scope="col">Uraian</th><th scope="col">Pasien</th><th scope="col">Kantong</th><th scope="col">Pelapor</th></tr>
						</thead>
						<tbody>
							{#each data as i (i.id)}
								<tr>
									<td class="whitespace-nowrap">{waktu(i.waktu)}</td>
									<td><Chip nada={i.jenis === 'knc' ? 'waspada' : 'bahaya'}>{JENIS_INSIDEN[i.jenis] ?? i.jenis}</Chip></td>
									<td>{i.uraian}</td>
									<td>{#if i.no_rm}<a class="mono font-semibold hover:underline" href="/pasien/{encodeURIComponent(i.no_rm)}">{i.no_rm}</a>{:else}—{/if}</td>
									<td class="mono">{i.nomor_kantong ?? '—'}</td>
									<td>{i.pelapor}</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{/snippet}
		</Keadaan>
	{/if}
</section>
