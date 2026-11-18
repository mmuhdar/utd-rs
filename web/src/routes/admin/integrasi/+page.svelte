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
	import KepalaHalaman from '#lib/KepalaHalaman.svelte';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';

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
	const KET: Record<string, string> = { antre: 'Menunggu dikirim pekerja latar', terkirim: 'Diterima SATUSEHAT', gagal: 'Perlu dikirim ulang' };

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

<KepalaHalaman judul="Integrasi SATUSEHAT" jalur={[['Pengaturan'], ['Integrasi SATUSEHAT']]} sub="Peristiwa pelulusan dan pemusnahan kantong hasil donasi dikirim ke SATUSEHAT (modul UPD).">
	{#snippet aksi()}
		{#if sesi.boleh('integrasi.kelola')}
			<button class="btn" onclick={() => outbox.muat()} disabled={outbox.memuat}><RefreshCw class="size-4" aria-hidden="true" />Muat ulang</button>
		{/if}
	{/snippet}
</KepalaHalaman>

{#if !sesi.boleh('integrasi.kelola')}
	<p class="catatan">Anda tidak berhak mengelola integrasi.</p>
{:else}
	<Keadaan sumber={outbox} apakahKosong={() => false}>
		{#snippet children(o)}
			<div class="mb-5 grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
				<section class="panel flex flex-col gap-2" aria-labelledby="judul-status">
					<div class="flex items-center justify-between gap-2">
						<h2 id="judul-status" class="text-sm font-semibold text-redup">Status koneksi</h2>
						<Chip nada={o.aktif ? 'aman' : 'netral'}>{o.aktif ? 'Aktif' : 'Tidak aktif'}</Chip>
					</div>
					<p class="text-sm text-redup">
						{#if o.aktif}
							Pekerja latar mengirim antrean ke SATUSEHAT (modul UPD) tiap 30 detik, dengan jeda yang makin panjang setelah kegagalan.
						{:else}
							Kredensial SATUSEHAT belum dikonfigurasi di server. Peristiwa kantong tetap disimpan di antrean dan akan dikirim setelah
							konfigurasi diisi (lihat docs/pemasangan.md).
						{/if}
					</p>
				</section>
				{#each Object.entries(STATUS) as [k, [label]] (k)}
					{@const n = o.ringkasan[k as Baris['status']] ?? 0}
					<dl class="panel flex flex-col gap-2">
						<dt class="text-sm font-semibold text-redup">{label}</dt>
						<dd class="text-4xl font-extrabold tracking-tight tabular-nums {k === 'gagal' && n ? 'text-bahaya' : ''}">{n}</dd>
						<dd class="text-xs font-semibold text-redup">{KET[k]}</dd>
					</dl>
				{/each}
			</div>

			<section class="panel" aria-labelledby="judul-outbox">
				<h2 id="judul-outbox">Antrean kiriman</h2>
				<p class="mb-3 text-sm text-redup">200 peristiwa terbaru. Kiriman gagal bisa dijadwalkan ulang.</p>
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
										<td class="mono font-semibold">{b.nomor_kantong}</td>
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
