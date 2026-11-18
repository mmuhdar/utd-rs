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

<h1 class="mb-4">Look-back</h1>

{#if sesi.boleh('stok.kelola')}
	<form class="panel mb-6 flex max-w-2xl flex-col gap-4" onsubmit={simpan} aria-labelledby="judul-baru">
		<h2 id="judul-baru">Catat notifikasi dari UTD</h2>
		<p class="text-sm text-redup">Kantong dari nomor donasi ini yang masih di stok langsung dikarantina; penerimanya ditampilkan untuk ditindaklanjuti.</p>
		<Isian label="Nomor donasi" id="nomor" galat={aksi.isian.nomor_donasi} bantuan="Ketik atau pindai label ISBT 128.">
			<input id="nomor" class="input mono" autocomplete="off" bind:value={nomor} required aria-invalid={!!aksi.isian.nomor_donasi} />
		</Isian>
		<Isian label="Isi notifikasi UTD" id="alasan" galat={aksi.isian.alasan} bantuan="Mis. donor reaktif HCV pada donasi berikutnya.">
			<textarea id="alasan" class="input" rows="3" bind:value={alasan} required aria-invalid={!!aksi.isian.alasan}></textarea>
		</Isian>
		<PesanAksi {aksi} />
		<button class="btn btn--utama self-start" disabled={aksi.sibuk}>Buka look-back</button>
	</form>
{/if}

<section class="panel" aria-labelledby="judul-daftar">
	<h2 id="judul-daftar" class="mb-3">Daftar look-back</h2>
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
								<td class="whitespace-nowrap"><a class="text-utama hover:underline" href="/lookback/{l.id}">{waktu(l.dibuat_pada)}</a></td>
								<td>
									{#if l.nomor_donasi}<span class="mono">{l.nomor_donasi}</span>{/if}
									{#if l.donor}<span class="block">{l.donor}</span>{/if}
								</td>
								<td>{l.alasan}</td>
								<td>{#if l.status === 'terbuka'}<Chip nada="waspada">Terbuka</Chip>{:else}<Chip nada="aman">Selesai</Chip>{/if}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/snippet}
	</Keadaan>
</section>
