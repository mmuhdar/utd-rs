<!-- Riwayat pasien: golongan darah, permintaan, dan transfusi (ketertelusuran no. RM → semua kantong). -->
<script lang="ts">
	import { merek } from '#lib/merek.svelte.ts';
	import { page } from '$app/state';
	import { api } from '#lib/api.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import Chip from '#lib/Chip.svelte';
	import { KOMPONEN, STATUS_PERMINTAAN, golongan, tanggal, waktu } from '#lib/format.ts';
	import { JENIS_PERMINTAAN } from '#lib/pelayanan/darah.ts';

	const no_rm = $derived(page.params.no_rm ?? '');
	const detail = new Sumber(() => api<any>('GET', `/pasien/${encodeURIComponent(no_rm)}`));
	$effect(() => {
		void no_rm;
		detail.muat();
	});
</script>

<svelte:head><title>Pasien {no_rm} — {merek.judul}</title></svelte:head>

<Keadaan sumber={detail} apakahKosong={() => false} baris={8}>
	{#snippet children(p)}
		{@const gol = p.golongan.find((g: any) => g.terverifikasi && !g.diskrepansi)}
		<div class="mb-4">
			<h1>{p.nama} <span class="mono text-base font-normal text-redup">{p.no_rm}</span></h1>
			<p class="text-sm text-redup">
				Lahir {tanggal(p.tanggal_lahir)} · {p.jenis_kelamin === 'L' ? 'Laki-laki' : 'Perempuan'} · data dari {p.sumber === 'simrs' ? 'SIMRS' : 'input manual'}
			</p>
			<p class="mt-2">
				Golongan terverifikasi:
				{#if gol}<span class="text-lg font-semibold">{golongan(gol.abo, gol.rhesus)}</span>{:else}<Chip nada="waspada">Belum ada</Chip>{/if}
			</p>
		</div>

		<div class="flex flex-col gap-4">
			<section class="panel" aria-labelledby="j-gol">
				<h2 id="j-gol" class="mb-3">Riwayat pemeriksaan golongan</h2>
				{#if p.golongan.length}
					<div class="overflow-x-auto">
						<table class="tabel">
							<thead><tr><th scope="col">Waktu</th><th scope="col">Hasil</th><th scope="col">Status</th></tr></thead>
							<tbody>
								{#each p.golongan as g, i (i)}
									<tr>
										<td>{waktu(g.waktu)}</td>
										<td class="font-semibold">{golongan(g.abo, g.rhesus)}</td>
										<td>
											{#if g.diskrepansi}<Chip nada="bahaya">Diskrepansi</Chip>
											{:else if g.terverifikasi}<Chip nada="aman">Terverifikasi</Chip>
											{:else}<Chip nada="waspada">Menunggu verifikasi</Chip>{/if}
										</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{:else}
					<p class="text-sm text-redup">Belum pernah diperiksa golongan darah di bank darah ini.</p>
				{/if}
			</section>

			<section class="panel" aria-labelledby="j-minta">
				<h2 id="j-minta" class="mb-3">Permintaan darah</h2>
				{#if p.permintaan.length}
					<div class="overflow-x-auto">
						<table class="tabel">
							<thead><tr><th scope="col">Nomor</th><th scope="col">Diminta</th><th scope="col">Ruang</th><th scope="col">Jenis</th><th scope="col">Status</th></tr></thead>
							<tbody>
								{#each p.permintaan as r (r.id)}
									{@const st = STATUS_PERMINTAAN[r.status]}
									<tr>
										<td><a class="mono text-utama hover:underline" href="/permintaan/{r.id}">{r.nomor}</a></td>
										<td>{waktu(r.diminta_pada)}</td>
										<td>{r.ruang}</td>
										<td><Chip nada={JENIS_PERMINTAAN[r.jenis]?.[1]}>{JENIS_PERMINTAAN[r.jenis]?.[0] ?? r.jenis}</Chip></td>
										<td><Chip nada={st?.[1]}>{st?.[0] ?? r.status}</Chip></td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{:else}
					<p class="text-sm text-redup">Belum ada permintaan darah.</p>
				{/if}
			</section>

			<section class="panel" aria-labelledby="j-trans">
				<h2 id="j-trans" class="mb-3">Transfusi</h2>
				{#if p.transfusi.length}
					<div class="overflow-x-auto">
						<table class="tabel">
							<thead><tr><th scope="col">Mulai</th><th scope="col">Kantong</th><th scope="col">Komponen</th><th scope="col">Akhir</th><th scope="col">Reaksi</th></tr></thead>
							<tbody>
								{#each p.transfusi as t (t.id)}
									<tr>
										<td><a class="text-utama hover:underline" href="/transfusi/{t.id}">{waktu(t.mulai)}</a></td>
										<td class="mono">{t.nomor_kantong}</td>
										<td>{KOMPONEN[t.komponen]} {golongan(t.abo, t.rhesus)}</td>
										<td>
											{#if !t.selesai}<Chip nada="info">Berjalan</Chip>
											{:else if t.dihentikan}<Chip nada="bahaya">Dihentikan</Chip> <span class="text-xs text-redup">{waktu(t.selesai)}</span>
											{:else}<Chip nada="aman">Selesai</Chip> <span class="text-xs text-redup">{waktu(t.selesai)}</span>{/if}
										</td>
										<td>{#if t.reaksi}<Chip nada="bahaya">Ada reaksi</Chip>{:else}—{/if}</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{:else}
					<p class="text-sm text-redup">Belum pernah menerima transfusi.</p>
				{/if}
			</section>
		</div>
	{/snippet}
</Keadaan>
