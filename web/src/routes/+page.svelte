<script lang="ts">
	import { api } from '#lib/api.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import Chip from '#lib/Chip.svelte';
	import { sesi } from '#lib/sesi.svelte.ts';
	import { KOMPONEN, STATUS_PERMINTAAN, golongan, relatif, waktu } from '#lib/format.ts';
	import TriangleAlert from '@lucide/svelte/icons/triangle-alert';
	import Thermometer from '@lucide/svelte/icons/thermometer';

	type Ringkas = { komponen: string; abo: string; rhesus: string; tersedia: number; mendekati_kedaluwarsa: number; lewat_kedaluwarsa: number; karantina: number };
	const stok = new Sumber(() => api<Ringkas[]>('GET', '/stok/ringkasan'));
	const permintaan = new Sumber(() => api<any[]>('GET', '/permintaan'));
	const keluar = new Sumber(() => api<any[]>('GET', '/pengeluaran/aktif'));
	const penyimpangan = new Sumber(() => api<any[]>('GET', '/penyimpangan-suhu'));

	$effect(() => {
		if (sesi.boleh('stok.lihat')) {
			stok.muat();
			penyimpangan.muat();
		}
		if (sesi.boleh('permintaan.lihat')) {
			permintaan.muat();
			keluar.muat();
		}
		// Daftar yang bergerak cepat diperbarui tiap menit (pengingat 30 menit, permintaan darurat).
		const t = setInterval(() => {
			if (sesi.boleh('permintaan.lihat')) {
				permintaan.muat();
				keluar.muat();
			}
		}, 60_000);
		return () => clearInterval(t);
	});

	const GOL = ['O pos', 'O neg', 'A pos', 'A neg', 'B pos', 'B neg', 'AB pos', 'AB neg'];
	const sel = (k: string, g: string) => stok.data?.find((r) => r.komponen === k && `${r.abo} ${r.rhesus}` === g);
	const terbuka = $derived((penyimpangan.data ?? []).filter((p) => p.status === 'terbuka'));
	const terlambat = $derived((keluar.data ?? []).filter((k) => k.terlambat));
</script>

<svelte:head><title>Beranda — utd-rs</title></svelte:head>

<h1 class="mb-4">Beranda</h1>

{#if terbuka.length}
	<div class="peringatan mb-4" role="alert">
		<Thermometer class="mt-0.5 size-5 shrink-0" aria-hidden="true" />
		<div>
			<p>Penyimpangan suhu terbuka di {terbuka.map((p) => p.alat).join(', ')}. Kantong di alat tersebut dikarantina sampai dinilai.</p>
			<a class="underline" href="/suhu">Buka penilaian</a>
		</div>
	</div>
{/if}
{#if terlambat.length}
	<div class="peringatan mb-4" role="alert">
		<TriangleAlert class="mt-0.5 size-5 shrink-0" aria-hidden="true" />
		<p>
			{terlambat.length} kantong sudah lebih dari 30 menit di luar bank darah tanpa transfusi dimulai:
			{terlambat.map((k) => `${k.nomor_kantong} (${k.ruang})`).join(', ')}.
		</p>
	</div>
{/if}

<div class="grid gap-4 xl:grid-cols-2">
	{#if sesi.boleh('stok.lihat')}
		<section class="panel xl:col-span-2" aria-labelledby="judul-stok">
			<div class="mb-3 flex items-baseline justify-between">
				<h2 id="judul-stok">Stok tersedia</h2>
				<a class="text-sm text-utama hover:underline" href="/stok">Rincian stok</a>
			</div>
			<Keadaan sumber={stok} apakahKosong={() => false}>
				{#snippet children()}
					<div class="overflow-x-auto">
						<table class="tabel">
							<caption class="sr-only">Jumlah kantong tersedia per komponen dan golongan darah</caption>
							<thead>
								<tr>
									<th scope="col">Komponen</th>
									{#each GOL as g (g)}<th scope="col" class="text-center">{golongan(...(g.split(' ') as [string, string]))}</th>{/each}
								</tr>
							</thead>
							<tbody>
								{#each Object.keys(KOMPONEN) as k (k)}
									<tr>
										<th scope="row" class="text-left text-sm font-medium normal-case">{KOMPONEN[k]}</th>
										{#each GOL as g (g)}
											{@const r = sel(k, g)}
											<td class="text-center tabular-nums">
												<span class={!r?.tersedia ? 'text-redup' : 'font-semibold'}>{r?.tersedia ?? 0}</span>
												{#if r?.mendekati_kedaluwarsa}
													<span class="block text-xs text-waspada">{r.mendekati_kedaluwarsa} segera kedaluwarsa</span>
												{/if}
												{#if r?.lewat_kedaluwarsa}
													<span class="block text-xs text-bahaya">{r.lewat_kedaluwarsa} kedaluwarsa</span>
												{/if}
											</td>
										{/each}
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/snippet}
			</Keadaan>
		</section>
	{/if}

	{#if sesi.boleh('permintaan.lihat')}
		<section class="panel" aria-labelledby="judul-permintaan">
			<div class="mb-3 flex items-baseline justify-between">
				<h2 id="judul-permintaan">Permintaan berjalan</h2>
				<a class="text-sm text-utama hover:underline" href="/permintaan">Semua</a>
			</div>
			<Keadaan sumber={permintaan}>
				{#snippet kosong()}<p>Tidak ada permintaan darah yang sedang berjalan.</p>{/snippet}
				{#snippet children(data)}
					<ul class="divide-y divide-halus">
						{#each data.slice(0, 8) as r (r.id)}
							{@const st = STATUS_PERMINTAAN[r.status]}
							<li>
								<a href="/permintaan/{r.id}" class="flex items-center justify-between gap-3 py-2 hover:bg-halus">
									<span>
										<span class="font-medium">{r.pasien.nama}</span>
										<span class="mono text-xs text-redup">{r.no_rm}</span>
										<span class="block text-xs text-redup">{r.ruang} · {r.nomor} · {relatif(r.diminta_pada)}</span>
									</span>
									<span class="flex flex-col items-end gap-1">
										{#if r.jenis === 'darurat'}<Chip nada="bahaya">Darurat</Chip>{/if}
										<Chip nada={st?.[1]}>{st?.[0] ?? r.status}</Chip>
									</span>
								</a>
							</li>
						{/each}
					</ul>
				{/snippet}
			</Keadaan>
		</section>

		<section class="panel" aria-labelledby="judul-keluar">
			<h2 id="judul-keluar" class="mb-3">Kantong di luar bank darah</h2>
			<Keadaan sumber={keluar}>
				{#snippet kosong()}<p>Tidak ada kantong yang menunggu ditransfusikan.</p>{/snippet}
				{#snippet children(data)}
					<table class="tabel">
						<thead><tr><th>Kantong</th><th>Pasien</th><th>Keluar</th></tr></thead>
						<tbody>
							{#each data as k (k.id)}
								<tr>
									<td class="mono">{k.nomor_kantong}<span class="block font-sans text-xs text-redup">{KOMPONEN[k.komponen]} {golongan(k.abo, k.rhesus)}</span></td>
									<td>{k.pasien.nama}<span class="block text-xs text-redup">{k.ruang}</span></td>
									<td>
										<Chip nada={k.terlambat ? 'bahaya' : k.menit > 20 ? 'waspada' : 'netral'}>{k.menit} menit</Chip>
										<span class="block text-xs text-redup">{waktu(k.dikeluarkan_pada)}</span>
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				{/snippet}
			</Keadaan>
		</section>
	{/if}
</div>
