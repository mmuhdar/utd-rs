<script lang="ts">
	import { merek } from '#lib/merek.svelte.ts';
	import { page } from '$app/state';
	import { api } from '#lib/api.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import Chip from '#lib/Chip.svelte';
	import { KOMPONEN, waktu } from '#lib/format.ts';
	import { JENIS_INSIDEN, STATUS_REAKSI } from '#lib/pelayanan/darah.ts';

	const tab = $derived(page.url.searchParams.get('tab') === 'insiden' ? 'insiden' : 'reaksi');
	const reaksi = new Sumber(() => api<any[]>('GET', '/reaksi'));
	const insiden = new Sumber(() => api<any[]>('GET', '/insiden'));
	$effect(() => {
		(tab === 'insiden' ? insiden : reaksi).muat();
	});
</script>

<svelte:head><title>Reaksi & insiden — {merek.judul}</title></svelte:head>

<h1 class="mb-4">Reaksi transfusi dan insiden</h1>

<nav class="mb-4 flex gap-2" aria-label="Jenis laporan">
	<a class="btn {tab === 'reaksi' ? 'btn--utama' : ''}" href="/reaksi" aria-current={tab === 'reaksi' ? 'page' : undefined}>Reaksi transfusi</a>
	<a class="btn {tab === 'insiden' ? 'btn--utama' : ''}" href="?tab=insiden" aria-current={tab === 'insiden' ? 'page' : undefined}>Insiden (KNC dan lainnya)</a>
</nav>

<section class="panel">
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
									<td><a class="mono font-medium text-utama hover:underline" href="/reaksi/{r.id}">{r.nomor}</a></td>
									<td>{waktu(r.waktu_reaksi)}</td>
									<td>{r.pasien.nama}<span class="mono block text-xs text-redup">{r.pasien.no_rm}</span></td>
									<td class="mono">{r.nomor_kantong}<span class="block font-sans text-xs text-redup">{KOMPONEN[r.komponen]}</span></td>
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
									<td>{#if i.no_rm}<a class="mono text-utama hover:underline" href="/pasien/{encodeURIComponent(i.no_rm)}">{i.no_rm}</a>{:else}—{/if}</td>
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
