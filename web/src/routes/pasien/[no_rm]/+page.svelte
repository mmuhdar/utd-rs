<!-- Riwayat pasien: golongan darah, permintaan, dan transfusi (ketertelusuran no. RM → semua kantong). -->
<script lang="ts">
	import { merek } from '#lib/merek.svelte.ts';
	import { page } from '$app/state';
	import { api } from '#lib/api.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import Chip from '#lib/Chip.svelte';
	import Golongan from '#lib/Golongan.svelte';
	import KepalaHalaman from '#lib/KepalaHalaman.svelte';
	import { KOMPONEN, STATUS_PERMINTAAN, tanggal, waktu, inisial } from '#lib/format.ts';
	import { JENIS_PERMINTAAN } from '#lib/pelayanan/darah.ts';

	const no_rm = $derived(page.params.no_rm ?? '');
	const detail = new Sumber(() => api<any>('GET', `/pasien/${encodeURIComponent(no_rm)}`));
	$effect(() => {
		void no_rm;
		detail.muat();
	});
</script>

<svelte:head><title>Pasien {no_rm} — {merek.judul}</title></svelte:head>

<KepalaHalaman judul={no_rm} mono jalur={[['Pelayanan'], ['Riwayat pasien']]} sub="Golongan darah, permintaan, dan transfusi untuk no. RM ini" />

<Keadaan sumber={detail} apakahKosong={() => false} baris={8}>
	{#snippet children(p)}
		{@const gol = p.golongan.find((g: any) => g.terverifikasi && !g.diskrepansi)}
		{@const reaksi = p.transfusi.filter((t: any) => t.reaksi).length}
		<div class="flex flex-col gap-5">
			<section class="panel flex flex-wrap items-center gap-x-6 gap-y-4" aria-labelledby="j-pasien">
				<div class="flex min-w-72 flex-1 items-center gap-4">
					<span class="grid size-13 shrink-0 place-items-center rounded-full bg-halus text-lg font-bold text-teks-2" aria-hidden="true">{inisial(p.nama)}</span>
					<div class="min-w-0">
						<h2 id="j-pasien" class="text-xl font-extrabold tracking-tight">{p.nama}</h2>
						<p class="mt-0.5 text-sm text-redup">
							<span class="mono font-semibold text-teks-2">{p.no_rm}</span> · {p.jenis_kelamin === 'L' ? 'Laki-laki' : 'Perempuan'} · lahir {tanggal(p.tanggal_lahir)}
							· data dari {p.sumber === 'simrs' ? 'SIMRS' : 'input manual'}
						</p>
					</div>
				</div>
				<dl class="flex flex-wrap gap-x-7 gap-y-2 text-sm">
					<div><dt class="text-xs font-semibold text-redup">Permintaan</dt><dd class="font-bold tabular-nums">{p.permintaan.length}</dd></div>
					<div><dt class="text-xs font-semibold text-redup">Transfusi</dt><dd class="font-bold tabular-nums">{p.transfusi.length}</dd></div>
					<div>
						<dt class="text-xs font-semibold text-redup">Reaksi</dt>
						<dd class="font-bold tabular-nums {reaksi ? 'text-bahaya' : ''}">{reaksi}</dd>
					</div>
				</dl>
				<div class="flex flex-col items-center gap-1">
					{#if gol}
						<Golongan abo={gol.abo} rhesus={gol.rhesus} ukuran="besar" />
						<span class="text-xs font-semibold text-aman">Terverifikasi</span>
					{:else}
						<Chip nada="waspada">Golongan belum terverifikasi</Chip>
					{/if}
				</div>
			</section>

			<div class="grid items-start gap-5 xl:grid-cols-[1fr_360px]">
				<div class="flex min-w-0 flex-col gap-5">
					<section class="panel" aria-labelledby="j-minta">
						<div class="mb-3 flex flex-wrap items-baseline justify-between gap-2">
							<h2 id="j-minta">Permintaan darah</h2>
							<p class="text-sm text-redup">{p.permintaan.length} permintaan</p>
						</div>
						{#if p.permintaan.length}
							<div class="overflow-x-auto">
								<table class="tabel">
									<thead><tr><th scope="col">Nomor</th><th scope="col">Diminta</th><th scope="col">Ruang</th><th scope="col">Jenis</th><th scope="col">Status</th></tr></thead>
									<tbody>
										{#each p.permintaan as r (r.id)}
											{@const st = STATUS_PERMINTAAN[r.status]}
											<tr>
												<td><a class="mono font-semibold hover:underline" href="/permintaan/{r.id}">{r.nomor}</a></td>
												<td class="whitespace-nowrap">{waktu(r.diminta_pada)}</td>
												<td>{r.ruang}</td>
												<td><Chip nada={JENIS_PERMINTAAN[r.jenis]?.[1]} polos>{JENIS_PERMINTAAN[r.jenis]?.[0] ?? r.jenis}</Chip></td>
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
						<div class="mb-3 flex flex-wrap items-baseline justify-between gap-2">
							<h2 id="j-trans">Transfusi</h2>
							<p class="text-sm text-redup">{p.transfusi.length} kantong</p>
						</div>
						{#if p.transfusi.length}
							<div class="overflow-x-auto">
								<table class="tabel">
									<thead><tr><th scope="col">Mulai</th><th scope="col">Kantong</th><th scope="col">Golongan</th><th scope="col">Akhir</th><th scope="col">Reaksi</th></tr></thead>
									<tbody>
										{#each p.transfusi as t (t.id)}
											<tr>
												<td class="whitespace-nowrap"><a class="font-semibold hover:underline" href="/transfusi/{t.id}">{waktu(t.mulai)}</a></td>
												<td><span class="mono font-semibold">{t.nomor_kantong}</span><span class="block text-xs text-redup">{KOMPONEN[t.komponen]}</span></td>
												<td><Golongan abo={t.abo} rhesus={t.rhesus} /></td>
												<td>
													{#if !t.selesai}<Chip nada="info">Berjalan</Chip>
													{:else if t.dihentikan}<Chip nada="bahaya">Dihentikan</Chip> <span class="block text-xs text-redup">{waktu(t.selesai)}</span>
													{:else}<Chip nada="aman">Selesai</Chip> <span class="block text-xs text-redup">{waktu(t.selesai)}</span>{/if}
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

				<section class="panel" aria-labelledby="j-gol">
					<h2 id="j-gol" class="mb-3">Riwayat pemeriksaan golongan</h2>
					{#if p.golongan.length}
						<ul class="flex flex-col">
							{#each p.golongan as g, i (i)}
								<li class="flex items-center gap-3 border-b border-garis/70 py-2.5 last:border-b-0">
									<Golongan abo={g.abo} rhesus={g.rhesus} />
									<span class="min-w-0 flex-1 text-sm text-redup">{waktu(g.waktu)}</span>
									{#if g.diskrepansi}<Chip nada="bahaya">Diskrepansi</Chip>
									{:else if g.terverifikasi}<Chip nada="aman">Terverifikasi</Chip>
									{:else}<Chip nada="waspada">Menunggu verifikasi</Chip>{/if}
								</li>
							{/each}
						</ul>
					{:else}
						<p class="text-sm text-redup">Belum pernah diperiksa golongan darah di bank darah ini.</p>
					{/if}
				</section>
			</div>
		</div>
	{/snippet}
</Keadaan>
