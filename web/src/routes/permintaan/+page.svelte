<script lang="ts">
	import { merek } from '#lib/merek.svelte.ts';
	import { page } from '$app/state';
	import { api } from '#lib/api.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import Chip from '#lib/Chip.svelte';
	import { sesi } from '#lib/sesi.svelte.ts';
	import { KOMPONEN, STATUS_PERMINTAAN, relatif, waktu } from '#lib/format.ts';
	import Plus from '@lucide/svelte/icons/plus';

	const FILTER: [string, string][] = [
		['', 'Berjalan'],
		['diminta', 'Menunggu sampel'],
		['diproses', 'Diproses'],
		['siap', 'Siap diambil'],
		['selesai', 'Selesai'],
		['batal,ditolak', 'Batal/ditolak']
	];
	const status = $derived(page.url.searchParams.get('status') ?? '');
	const daftar = new Sumber(() => api<any[]>('GET', `/permintaan${status ? `?status=${encodeURIComponent(status)}` : ''}`));

	$effect(() => {
		void status;
		daftar.muat();
		const t = setInterval(() => daftar.muat(), 60_000);
		return () => clearInterval(t);
	});
</script>

<svelte:head><title>Permintaan darah — {merek.judul}</title></svelte:head>

<div class="mb-4 flex flex-wrap items-center justify-between gap-3">
	<h1>Permintaan darah</h1>
	{#if sesi.boleh('permintaan.buat')}
		<a class="btn btn--utama" href="/permintaan/baru"><Plus class="size-4" aria-hidden="true" />Buat permintaan</a>
	{/if}
</div>

<nav class="mb-4 flex flex-wrap gap-2" aria-label="Saring status">
	{#each FILTER as [nilai, label] (nilai)}
		<a
			class="btn {status === nilai ? 'btn--utama' : ''}"
			href={nilai ? `?status=${nilai}` : '/permintaan'}
			aria-current={status === nilai ? 'page' : undefined}>{label}</a
		>
	{/each}
</nav>

<section class="panel">
	<Keadaan sumber={daftar}>
		{#snippet kosong()}<p>Tidak ada permintaan darah dengan status ini.</p>{/snippet}
		{#snippet children(data)}
			<div class="overflow-x-auto">
				<table class="tabel">
					<caption class="sr-only">Daftar permintaan darah; darurat ditampilkan paling atas</caption>
					<thead>
						<tr>
							<th scope="col">Nomor</th>
							<th scope="col">Pasien</th>
							<th scope="col">Ruang</th>
							<th scope="col">Komponen</th>
							<th scope="col">Kantong</th>
							<th scope="col">Status</th>
							<th scope="col">Diminta</th>
						</tr>
					</thead>
					<tbody>
						{#each data as r (r.id)}
							{@const st = STATUS_PERMINTAAN[r.status]}
							<tr class={r.jenis === 'darurat' && ['diminta', 'diproses', 'siap'].includes(r.status) ? 'bg-bahaya-latar' : ''}>
								<td>
									<a class="mono font-medium text-utama hover:underline" href="/permintaan/{r.id}">{r.nomor}</a>
									{#if r.jenis === 'darurat'}<span class="mt-1 block"><Chip nada="bahaya">Darurat</Chip></span>{/if}
								</td>
								<td>{r.pasien.nama}<span class="mono block text-xs text-redup">{r.no_rm}</span></td>
								<td>{r.ruang}</td>
								<td>{r.baris.map((b: any) => `${b.jumlah} ${KOMPONEN[b.komponen] ?? b.komponen}`).join(', ')}</td>
								<td class="tabular-nums">
									{r.dialokasikan}/{r.diminta} dialokasikan
									<span class="block text-xs text-redup">{r.dikeluarkan} dikeluarkan</span>
								</td>
								<td><Chip nada={st?.[1]}>{st?.[0] ?? r.status}</Chip></td>
								<td>{relatif(r.diminta_pada)}<span class="block text-xs text-redup">{waktu(r.diminta_pada)}</span></td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/snippet}
	</Keadaan>
</section>
