<script lang="ts">
	import { merek } from '#lib/merek.svelte.ts';
	import { goto } from '$app/navigation';
	import { api } from '#lib/api.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Isian from '#lib/Isian.svelte';
	import Chip from '#lib/Chip.svelte';
	import { sesi } from '#lib/sesi.svelte.ts';
	import { waktu } from '#lib/format.ts';
	import { urai } from '#lib/isbt.ts';
	import KepalaHalaman from '#lib/KepalaHalaman.svelte';

	const daftar = new Sumber(() => api<any[]>('GET', '/lookback'));
	$effect(() => {
		daftar.muat();
	});

	let nomor = $state('');
	let alasan = $state('');
	const aksi = new Aksi();

	async function simpan(e: SubmitEvent) {
		e.preventDefault();
		// Label ISBT 128 yang dipindai diubah menjadi nomor donasi, sama dengan yang disimpan saat penerimaan.
		const p = urai(nomor);
		const nomor_donasi = p.jenis === 'din' ? p.nomor_donasi : nomor.trim();
		const r = await aksi.jalankan(() => api<{ id: string }>('POST', '/lookback', { nomor_donasi, alasan }));
		if (r) goto(`/lookback/${r.id}`);
	}
</script>

<svelte:head><title>Look-back — {merek.judul}</title></svelte:head>

<KepalaHalaman
	judul="Look-back"
	sub={daftar.data ? `${daftar.data.filter((l) => l.status === 'terbuka').length} terbuka · ${daftar.data.length} seluruhnya` : 'Penelusuran penerima darah dari donasi yang bermasalah'}
/>

<div class="grid items-start gap-5 xl:grid-cols-[1fr_380px]">
	{#if sesi.boleh('stok.kelola')}
		<form class="panel flex flex-col gap-4 xl:col-start-2 xl:row-start-1" onsubmit={simpan} aria-labelledby="judul-baru">
			<div>
				<h2 id="judul-baru">Catat notifikasi dari UTD</h2>
				<p class="mt-1 text-sm text-redup">Kantong dari nomor donasi ini yang masih di stok langsung dikarantina; penerimanya ditampilkan untuk ditindaklanjuti.</p>
			</div>
			<Isian label="Nomor donasi" id="nomor" galat={aksi.isian.nomor_donasi} bantuan="Ketik atau pindai label ISBT 128.">
				<input id="nomor" class="input mono min-h-12 rounded-xl text-base" autocomplete="off" bind:value={nomor} required aria-invalid={!!aksi.isian.nomor_donasi} />
			</Isian>
			<Isian label="Isi notifikasi UTD" id="alasan" galat={aksi.isian.alasan} bantuan="Mis. donor reaktif HCV pada donasi berikutnya.">
				<textarea id="alasan" class="input" rows="3" bind:value={alasan} required aria-invalid={!!aksi.isian.alasan}></textarea>
			</Isian>
			<PesanAksi {aksi} />
			<div class="flex justify-end gap-2.5 border-t border-garis pt-4">
				<button class="btn btn--utama" disabled={aksi.sibuk}>Buka look-back</button>
			</div>
		</form>
	{/if}

	<section class="panel min-w-0 xl:col-start-1 xl:row-start-1" aria-labelledby="judul-daftar">
		<div class="mb-3 flex flex-wrap items-baseline justify-between gap-2">
			<h2 id="judul-daftar">Daftar look-back</h2>
			{#if daftar.data}<p class="text-sm text-redup">{daftar.data.length} look-back</p>{/if}
		</div>
		<Keadaan sumber={daftar}>
			{#snippet kosong()}<p>Belum ada look-back.</p>{/snippet}
			{#snippet children(data)}
				<div class="overflow-x-auto">
					<table class="tabel">
						<thead>
							<tr>
								<th scope="col">Dibuka</th>
								<th scope="col">Donasi / donor</th>
								<th scope="col">Alasan</th>
								<th scope="col">Status</th>
							</tr>
						</thead>
						<tbody>
							{#each data as l (l.id)}
								<tr>
									<td class="whitespace-nowrap"><a class="font-semibold hover:underline" href="/lookback/{l.id}">{waktu(l.dibuat_pada)}</a></td>
									<td>
										{#if l.nomor_donasi}<span class="mono font-semibold">{l.nomor_donasi}</span>{/if}
										{#if l.donor}<span class="block text-teks-2">{l.donor}</span>{/if}
									</td>
									<td class="max-w-80 text-teks-2">{l.alasan}</td>
									<td>{#if l.status === 'terbuka'}<Chip nada="waspada">Terbuka</Chip>{:else}<Chip nada="aman">Selesai</Chip>{/if}</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{/snippet}
		</Keadaan>
	</section>
</div>
