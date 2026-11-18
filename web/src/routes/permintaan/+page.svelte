<script lang="ts">
	import { merek } from '#lib/merek.svelte.ts';
	import { page } from '$app/state';
	import { api } from '#lib/api.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import Chip from '#lib/Chip.svelte';
	import Golongan from '#lib/Golongan.svelte';
	import KepalaHalaman from '#lib/KepalaHalaman.svelte';
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
	const aktifDarurat = (r: any) => r.jenis === 'darurat' && ['diminta', 'diproses', 'siap'].includes(r.status);
</script>

<svelte:head><title>Permintaan darah — {merek.judul}</title></svelte:head>

<KepalaHalaman
	judul="Permintaan darah"
	jalur={[['Pelayanan'], ['Permintaan darah']]}
	sub={daftar.data ? `${daftar.data.filter(aktifDarurat).length} darurat berjalan · diperbarui tiap menit` : undefined}
>
	{#snippet aksi()}
		{#if sesi.boleh('permintaan.buat')}
			<a class="btn btn--utama" href="/permintaan/baru"><Plus class="size-4" aria-hidden="true" />Buat permintaan</a>
		{/if}
	{/snippet}
</KepalaHalaman>

<nav class="mb-4 flex w-fit max-w-full flex-wrap gap-0.5 rounded-[10px] bg-halus p-1" aria-label="Saring status">
	{#each FILTER as [nilai, label] (nilai)}
		<a
			class="rounded-lg px-3 py-1.5 text-sm font-medium text-redup hover:text-teks aria-[current=page]:bg-kartu aria-[current=page]:font-semibold aria-[current=page]:text-teks aria-[current=page]:shadow-kartu"
			href={nilai ? `?status=${nilai}` : '/permintaan'}
			aria-current={status === nilai ? 'page' : undefined}>{label}</a
		>
	{/each}
</nav>

<section class="panel" aria-labelledby="j-daftar">
	<div class="mb-3 flex flex-wrap items-baseline justify-between gap-2">
		<h2 id="j-daftar">Darurat lebih dulu, lalu yang terlama</h2>
		{#if daftar.data}<p class="text-sm text-redup">{daftar.data.length} permintaan</p>{/if}
	</div>
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
							<th scope="col">Golongan</th>
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
							<tr class={aktifDarurat(r) ? 'bg-bahaya-latar' : ''}>
								<td>
									<a class="mono font-semibold hover:underline" href="/permintaan/{r.id}">{r.nomor}</a>
									{#if r.jenis === 'darurat'}<span class="mt-1 block"><Chip nada="bahaya" polos>Darurat</Chip></span>{/if}
								</td>
								<td><span class="font-semibold">{r.pasien.nama}</span><span class="mono block text-xs text-redup">{r.no_rm}</span></td>
								<td>
									{#if r.golongan_terverifikasi}<Golongan abo={r.golongan_terverifikasi.abo} rhesus={r.golongan_terverifikasi.rhesus} />
									{:else}<span class="text-xs text-redup">Belum diperiksa</span>{/if}
								</td>
								<td>{r.ruang}</td>
								<td>{r.baris.map((b: any) => `${b.jumlah} ${KOMPONEN[b.komponen] ?? b.komponen}`).join(', ')}</td>
								<td class="tabular-nums">
									{r.dialokasikan}/{r.diminta} dialokasikan
									<span class="block text-xs text-redup">{r.dikeluarkan} dikeluarkan</span>
								</td>
								<td><Chip nada={st?.[1]}>{st?.[0] ?? r.status}</Chip></td>
								<td class="whitespace-nowrap">{relatif(r.diminta_pada)}<span class="block text-xs text-redup">{waktu(r.diminta_pada)}</span></td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/snippet}
	</Keadaan>
</section>
