<script lang="ts">
	import { page } from '$app/state';
	import { api } from '#lib/api.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Isian from '#lib/Isian.svelte';
	import Chip from '#lib/Chip.svelte';
	import StatusKantong from '#lib/StatusKantong.svelte';
	import { sesi } from '#lib/sesi.svelte.ts';
	import { KOMPONEN, waktu } from '#lib/format.ts';

	const l = new Sumber(() => api<any>('GET', `/lookback/${page.params.id}`));
	$effect(() => {
		void page.params.id;
		l.muat();
	});

	let catatan = $state('');
	const aksi = new Aksi();
	async function selesai(e: SubmitEvent) {
		e.preventDefault();
		const r = await aksi.jalankan(() => api('POST', `/lookback/${page.params.id}/selesai`, { catatan }), 'Look-back ditandai selesai.');
		if (r) l.muat();
	}
</script>

<svelte:head><title>Look-back — utd-rs</title></svelte:head>

<a class="mb-4 inline-block text-sm text-utama hover:underline" href="/lookback">Kembali ke daftar look-back</a>

<Keadaan sumber={l}>
	{#snippet children(d)}
		{@const kantong = d.kantong ?? []}
		{@const ditransfusi = kantong.filter((k: any) => k.penerima).length}
		<div class="mb-4 flex flex-wrap items-center gap-3">
			<h1>Look-back {#if d.nomor_donasi}<span class="mono">{d.nomor_donasi}</span>{:else}{d.donor?.nama}{/if}</h1>
			{#if d.status === 'terbuka'}<Chip nada="waspada">Terbuka</Chip>{:else}<Chip nada="aman">Selesai</Chip>{/if}
		</div>
		<dl class="panel mb-4 grid gap-x-6 gap-y-2 text-sm sm:grid-cols-2">
			<div><dt class="text-redup">Dibuka</dt><dd>{waktu(d.dibuat_pada)}</dd></div>
			{#if d.donor}<div><dt class="text-redup">Donor</dt><dd>{d.donor.nama} <span class="mono">({d.donor.nik})</span></dd></div>{/if}
			<div class="sm:col-span-2"><dt class="text-redup">Alasan</dt><dd>{d.alasan}</dd></div>
			{#if d.status === 'selesai'}
				<div class="sm:col-span-2"><dt class="text-redup">Tindak lanjut ({waktu(d.ditutup_pada)})</dt><dd>{d.catatan}</dd></div>
			{/if}
		</dl>

		{#if ditransfusi}
			<p class="peringatan mb-4">{ditransfusi} kantong terkait sudah ditransfusikan. Beri tahu dokter penerima masing-masing.</p>
		{/if}

		<section class="panel mb-4" aria-labelledby="judul-kantong">
			<h2 id="judul-kantong" class="mb-3">Kantong terkait ({kantong.length})</h2>
			{#if !kantong.length}
				<p class="text-sm text-redup">Tidak ada kantong dengan nomor donasi ini di utd-rs.</p>
			{:else}
				<div class="overflow-x-auto">
					<table class="tabel">
						<thead>
							<tr>
								<th scope="col">Kantong</th>
								<th scope="col">Donasi</th>
								<th scope="col">Status</th>
								<th scope="col">Penerima</th>
								<th scope="col">Dokter / ruang</th>
							</tr>
						</thead>
						<tbody>
							{#each kantong as k (k.nomor)}
								<tr>
									<td>
										<a class="mono text-utama hover:underline" href="/lacak/{encodeURIComponent(k.nomor)}">{k.nomor}</a>
										<span class="block text-xs">{KOMPONEN[k.komponen]}</span>
									</td>
									<td><span class="mono">{k.donasi}</span>{#if k.waktu_donasi}<span class="block text-xs text-redup">{waktu(k.waktu_donasi)}</span>{/if}</td>
									<td><StatusKantong status={k.status} /></td>
									{#if k.penerima}
										<td>
											{k.penerima.nama}
											<span class="mono block text-xs">{k.penerima.no_rm}</span>
											<span class="block text-xs text-redup">Transfusi {waktu(k.penerima.transfusi)}</span>
										</td>
										<td>{k.penerima.dokter}<span class="block text-xs text-redup">{k.penerima.ruang}</span></td>
									{:else}
										<td colspan="2" class="text-redup">Belum ditransfusikan</td>
									{/if}
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{/if}
		</section>

		{#if d.status === 'terbuka' && sesi.boleh('stok.kelola')}
			<form class="panel flex max-w-2xl flex-col gap-4" onsubmit={selesai} aria-labelledby="judul-selesai">
				<h2 id="judul-selesai">Selesaikan look-back</h2>
				<Isian label="Tindak lanjut" id="catatan" galat={aksi.isian.catatan} bantuan="Dokter penerima yang sudah diberi tahu, kantong yang dikembalikan atau dimusnahkan.">
					<textarea id="catatan" class="input" rows="3" bind:value={catatan} required aria-invalid={!!aksi.isian.catatan}></textarea>
				</Isian>
				<PesanAksi {aksi} />
				<button class="btn btn--utama self-start" disabled={aksi.sibuk}>Tandai selesai</button>
			</form>
		{/if}
	{/snippet}
</Keadaan>
