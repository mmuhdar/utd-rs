<script lang="ts">
	import { merek } from '#lib/merek.svelte.ts';
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
	import KepalaHalaman from '#lib/KepalaHalaman.svelte';
	import TriangleAlert from '@lucide/svelte/icons/triangle-alert';

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

<svelte:head><title>Look-back — {merek.judul}</title></svelte:head>

<KepalaHalaman
	judul={l.data ? `Look-back ${l.data.nomor_donasi ?? l.data.donor?.nama ?? ''}` : 'Look-back'}
	sub={l.data ? `Dibuka ${waktu(l.data.dibuat_pada)}` : undefined}
	jalur={[['Look-back', '/lookback'], [l.data?.nomor_donasi ?? l.data?.donor?.nama ?? 'Rincian']]}
>
	{#if l.data}
		<p class="mt-2">{#if l.data.status === 'terbuka'}<Chip nada="waspada">Terbuka</Chip>{:else}<Chip nada="aman">Selesai</Chip>{/if}</p>
	{/if}
</KepalaHalaman>

<Keadaan sumber={l}>
	{#snippet children(d)}
		{@const kantong = d.kantong ?? []}
		{@const ditransfusi = kantong.filter((k: any) => k.penerima).length}
		{#if ditransfusi}
			<div class="peringatan mb-5" role="alert">
				<TriangleAlert aria-hidden="true" />
				<p>{ditransfusi} kantong terkait sudah ditransfusikan. Beri tahu dokter penerima masing-masing.</p>
			</div>
		{/if}

		<section class="mb-5 grid gap-4 sm:grid-cols-3" aria-label="Ringkasan">
			<div class="panel flex flex-col gap-1.5">
				<p class="text-sm font-semibold text-redup">Kantong terkait</p>
				<p class="text-4xl font-extrabold tracking-tight tabular-nums">{kantong.length}</p>
			</div>
			<div class="panel flex flex-col gap-1.5 {ditransfusi ? 'border-bahaya-titik' : ''}">
				<p class="text-sm font-semibold text-redup">Sudah ditransfusikan</p>
				<p class="text-4xl font-extrabold tracking-tight tabular-nums {ditransfusi ? 'text-bahaya' : ''}">{ditransfusi}</p>
			</div>
			<div class="panel flex flex-col gap-1.5">
				<p class="text-sm font-semibold text-redup">Belum ditransfusikan</p>
				<p class="text-4xl font-extrabold tracking-tight tabular-nums">{kantong.length - ditransfusi}</p>
			</div>
		</section>

		<div class="grid items-start gap-5 xl:grid-cols-[1fr_360px]">
			<section class="panel min-w-0" aria-labelledby="judul-kantong">
				<div class="mb-3 flex flex-wrap items-baseline justify-between gap-2">
					<h2 id="judul-kantong">Kantong terkait</h2>
					<p class="text-sm text-redup">{kantong.length} kantong</p>
				</div>
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
											<a class="mono font-semibold hover:underline" href="/lacak/{encodeURIComponent(k.nomor)}">{k.nomor}</a>
											<span class="block text-xs text-redup">{KOMPONEN[k.komponen]}</span>
										</td>
										<td><span class="mono">{k.donasi}</span>{#if k.waktu_donasi}<span class="block text-xs text-redup">{waktu(k.waktu_donasi)}</span>{/if}</td>
										<td><StatusKantong status={k.status} /></td>
										{#if k.penerima}
											<td>
												<span class="font-semibold">{k.penerima.nama}</span>
												<span class="mono block text-xs text-redup">{k.penerima.no_rm}</span>
												<span class="block text-xs text-bahaya">Transfusi {waktu(k.penerima.transfusi)}</span>
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

			<aside class="flex flex-col gap-5">
				<section class="panel" aria-labelledby="judul-notifikasi">
					<h2 id="judul-notifikasi" class="judul-kecil mb-3">Notifikasi</h2>
					<dl class="flex flex-col gap-3 text-sm">
						<div><dt class="text-redup">Dibuka</dt><dd class="font-medium">{waktu(d.dibuat_pada)}</dd></div>
						{#if d.nomor_donasi}<div><dt class="text-redup">Nomor donasi</dt><dd class="mono font-semibold">{d.nomor_donasi}</dd></div>{/if}
						{#if d.donor}<div><dt class="text-redup">Donor</dt><dd class="font-medium">{d.donor.nama} <span class="mono text-redup">({d.donor.nik})</span></dd></div>{/if}
						<div><dt class="text-redup">Alasan</dt><dd>{d.alasan}</dd></div>
						{#if d.status === 'selesai'}
							<div class="border-t border-garis pt-3"><dt class="text-redup">Tindak lanjut ({waktu(d.ditutup_pada)})</dt><dd>{d.catatan}</dd></div>
						{/if}
					</dl>
				</section>

				{#if d.status === 'terbuka' && sesi.boleh('stok.kelola')}
					<form class="panel flex flex-col gap-4" onsubmit={selesai} aria-labelledby="judul-selesai">
						<h2 id="judul-selesai">Selesaikan look-back</h2>
						<Isian label="Tindak lanjut" id="catatan" galat={aksi.isian.catatan} bantuan="Dokter penerima yang sudah diberi tahu, kantong yang dikembalikan atau dimusnahkan.">
							<textarea id="catatan" class="input" rows="4" bind:value={catatan} required aria-invalid={!!aksi.isian.catatan}></textarea>
						</Isian>
						<PesanAksi {aksi} />
						<div class="flex justify-end gap-2.5 border-t border-garis pt-4">
							<button class="btn btn--utama" disabled={aksi.sibuk}>Tandai selesai</button>
						</div>
					</form>
				{/if}
			</aside>
		</div>
	{/snippet}
</Keadaan>
