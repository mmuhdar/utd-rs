<script lang="ts">
	import { merek } from '#lib/merek.svelte.ts';
	import { api } from '#lib/api.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Chip from '#lib/Chip.svelte';
	import { sesi } from '#lib/sesi.svelte.ts';
	import { relatif, waktu } from '#lib/format.ts';

	type Baris = {
		id: number;
		peristiwa: 'lulus' | 'musnah';
		nomor_kantong: string;
		status: 'antre' | 'terkirim' | 'gagal';
		percobaan: number;
		galat: string | null;
		berikut_pada: string;
		dibuat_pada: string;
		terkirim_pada: string | null;
	};
	type Outbox = { aktif: boolean; ringkasan: Partial<Record<Baris['status'], number>>; baris: Baris[] };

	const STATUS: Record<string, [string, string]> = { antre: ['Antre', 'waspada'], terkirim: ['Terkirim', 'aman'], gagal: ['Gagal', 'bahaya'] };
	const PERISTIWA: Record<string, string> = { lulus: 'Pelulusan', musnah: 'Pemusnahan' };

	const outbox = new Sumber(() => api<Outbox>('GET', '/integrasi/outbox'));
	$effect(() => {
		if (sesi.boleh('integrasi.kelola')) outbox.muat();
	});
	const aksi = new Aksi();
	async function ulang(b: Baris) {
		const r = await aksi.jalankan(() => api('POST', `/integrasi/outbox/${b.id}/ulang`, {}), `Kiriman kantong ${b.nomor_kantong} dijadwalkan ulang.`);
		if (r) outbox.muat();
	}
</script>

<svelte:head><title>Integrasi SATUSEHAT — {merek.judul}</title></svelte:head>

<h1 class="mb-4">Integrasi SATUSEHAT</h1>

{#if !sesi.boleh('integrasi.kelola')}
	<p class="catatan">Anda tidak berhak mengelola integrasi.</p>
{:else}
	<Keadaan sumber={outbox} apakahKosong={() => false}>
		{#snippet children(o)}
			<div class="mb-4 grid gap-4 md:grid-cols-[1fr_2fr]">
				<section class="panel" aria-labelledby="judul-status">
					<h2 id="judul-status" class="mb-2">Status koneksi</h2>
					<Chip nada={o.aktif ? 'aman' : 'netral'}>{o.aktif ? 'Aktif' : 'Tidak aktif'}</Chip>
					<p class="mt-2 text-sm text-redup">
						{#if o.aktif}
							Pekerja latar mengirim antrean ke SATUSEHAT (modul UPD) tiap 30 detik, dengan jeda yang makin panjang setelah kegagalan.
						{:else}
							Kredensial SATUSEHAT belum dikonfigurasi di server. Peristiwa kantong tetap disimpan di antrean dan akan dikirim setelah
							konfigurasi diisi (lihat docs/pemasangan.md).
						{/if}
					</p>
				</section>
				<section class="panel" aria-labelledby="judul-ringkas">
					<h2 id="judul-ringkas" class="mb-2">Ringkasan antrean</h2>
					<dl class="grid grid-cols-3 gap-3">
						{#each Object.entries(STATUS) as [k, [label]] (k)}
							<div class="rounded-md border border-garis p-3">
								<dt class="text-xs font-semibold tracking-wide text-redup uppercase">{label}</dt>
								<dd class="text-2xl font-semibold tabular-nums {k === 'gagal' && o.ringkasan.gagal ? 'text-bahaya' : ''}">
									{o.ringkasan[k as Baris['status']] ?? 0}
								</dd>
							</div>
						{/each}
					</dl>
				</section>
			</div>

			<section class="panel" aria-labelledby="judul-outbox">
				<div class="mb-3 flex items-baseline justify-between">
					<h2 id="judul-outbox">Antrean kiriman (200 terbaru)</h2>
					<button class="btn btn--teks" onclick={() => outbox.muat()} disabled={outbox.memuat}>Muat ulang</button>
				</div>
				<div class="mb-3"><PesanAksi {aksi} /></div>
				{#if !o.baris.length}
					<p class="keadaan">Belum ada peristiwa. Kantong hasil donasi yang diluluskan atau dimusnahkan akan masuk antrean ini.</p>
				{:else}
					<div class="overflow-x-auto">
						<table class="tabel">
							<thead>
								<tr><th>ID</th><th>Peristiwa</th><th>Kantong</th><th>Status</th><th>Percobaan</th><th>Galat terakhir</th><th>Berikutnya</th><th>Terkirim</th><th><span class="sr-only">Aksi</span></th></tr>
							</thead>
							<tbody>
								{#each o.baris as b (b.id)}
									{@const st = STATUS[b.status] ?? [b.status, 'netral']}
									<tr>
										<td class="mono tabular-nums">{b.id}</td>
										<td>{PERISTIWA[b.peristiwa] ?? b.peristiwa}<span class="block text-xs text-redup">{waktu(b.dibuat_pada)}</span></td>
										<td class="mono">{b.nomor_kantong}</td>
										<td><Chip nada={st[1]}>{st[0]}</Chip></td>
										<td class="tabular-nums">{b.percobaan}</td>
										<td class="max-w-xs text-xs break-words {b.galat ? 'text-bahaya' : 'text-redup'}">{b.galat ?? '—'}</td>
										<td class="whitespace-nowrap">{b.status === 'antre' ? relatif(b.berikut_pada) : '—'}</td>
										<td class="whitespace-nowrap">{waktu(b.terkirim_pada)}</td>
										<td>
											{#if b.status === 'gagal'}
												<button class="btn" onclick={() => ulang(b)} disabled={aksi.sibuk} aria-label="Kirim ulang kantong {b.nomor_kantong}">Kirim ulang</button>
											{/if}
										</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/if}
			</section>
		{/snippet}
	</Keadaan>
{/if}
