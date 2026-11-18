<script lang="ts">
	import { merek } from '#lib/merek.svelte.ts';
	import { api } from '#lib/api.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Isian from '#lib/Isian.svelte';
	import Chip from '#lib/Chip.svelte';
	import { sesi } from '#lib/sesi.svelte.ts';
	import { keRfc3339, lokal, relatif, waktu } from '#lib/format.ts';
	import { NAMA_JENIS_ALAT, suhu, type Alat } from '#lib/stok/stok.ts';
	import GrafikSuhu from '#lib/stok/GrafikSuhu.svelte';
	import Thermometer from '@lucide/svelte/icons/thermometer';

	const alat = new Sumber(() => api<Alat[]>('GET', '/master/alat'));
	const penyimpangan = new Sumber(() => api<any[]>('GET', '/penyimpangan-suhu'));
	$effect(() => {
		alat.muat();
		penyimpangan.muat();
	});

	const aktif = $derived((alat.data ?? []).filter((a) => a.aktif));
	let pilihGrafik = $state('');
	const alatGrafik = $derived(aktif.find((a) => a.id === pilihGrafik) ?? aktif[0]);

	let alat_id = $state('');
	let nilai = $state<number | null>(null);
	let waktuBaca = $state(lokal());
	let catatan = $state('');
	let hasil = $state<{ menyimpang: boolean; penyimpangan_id: string | null } | null>(null);
	const aksi = new Aksi();

	const diLuar = (a: Alat) => a.suhu_terakhir != null && (a.suhu_terakhir.suhu < a.suhu_min || a.suhu_terakhir.suhu > a.suhu_maks);

	async function catat(e: SubmitEvent) {
		e.preventDefault();
		hasil = null;
		const r = await aksi.jalankan(
			() => api<{ menyimpang: boolean; penyimpangan_id: string | null }>('POST', '/suhu', { alat_id, suhu: nilai, waktu: keRfc3339(waktuBaca), catatan }),
			'Pembacaan suhu dicatat.'
		);
		if (r) {
			hasil = r;
			nilai = null;
			catatan = '';
			waktuBaca = lokal();
			alat.muat();
			penyimpangan.muat();
		}
	}
</script>

<svelte:head><title>Suhu alat simpan — {merek.judul}</title></svelte:head>

<h1 class="mb-4">Suhu alat simpan</h1>

<section aria-labelledby="judul-alat" class="mb-6">
	<h2 id="judul-alat" class="sr-only">Alat simpan</h2>
	<Keadaan sumber={alat} baris={3}>
		{#snippet kosong()}<p>Belum ada alat simpan. Tambahkan di Data induk.</p>{/snippet}
		{#snippet children()}
			<ul class="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
				{#each aktif as a (a.id)}
					<li class="panel {a.penyimpangan_id ? 'border-2 border-bahaya' : ''}">
						<p class="flex items-center gap-2 font-semibold"><Thermometer class="size-4" aria-hidden="true" /><span class="mono">{a.kode}</span></p>
						<p class="text-sm text-redup">{a.nama} · {NAMA_JENIS_ALAT[a.jenis]}</p>
						<p class="mt-2 text-2xl font-semibold tabular-nums {diLuar(a) ? 'text-bahaya' : ''}">
							{a.suhu_terakhir ? suhu(a.suhu_terakhir.suhu) : '—'}
						</p>
						<p class="text-xs text-redup">
							{a.suhu_terakhir ? `${relatif(a.suhu_terakhir.waktu)} (${waktu(a.suhu_terakhir.waktu)})` : 'Belum ada pembacaan'}
						</p>
						{#if diLuar(a)}<p class="text-xs font-medium text-bahaya">Di luar rentang</p>{/if}
						<p class="mt-2 text-sm">Rentang {suhu(a.suhu_min)} – {suhu(a.suhu_maks)}</p>
						<p class="text-sm">{a.jumlah_kantong} kantong</p>
						{#if a.penyimpangan_id}
							<a class="mt-2 inline-block" href="/suhu/penyimpangan/{a.penyimpangan_id}"><Chip nada="bahaya">Penyimpangan terbuka — nilai kantong</Chip></a>
						{/if}
					</li>
				{/each}
			</ul>
		{/snippet}
	</Keadaan>
</section>

<div class="mb-6 grid gap-4 xl:grid-cols-[1fr_2fr]">
	{#if sesi.boleh('suhu.catat')}
		<form class="panel flex flex-col gap-4" onsubmit={catat} aria-labelledby="judul-catat">
			<h2 id="judul-catat">Catat suhu manual</h2>
			<Isian label="Alat simpan" id="c-alat" galat={aksi.isian.alat_id}>
				<select id="c-alat" class="input" bind:value={alat_id} required>
					<option value="" disabled>Pilih alat</option>
					{#each aktif as a (a.id)}<option value={a.id}>{a.kode} — {a.nama}</option>{/each}
				</select>
			</Isian>
			<Isian label="Suhu (°C)" id="c-suhu" galat={aksi.isian.suhu}>
				<input id="c-suhu" class="input" type="number" step="0.1" bind:value={nilai} required aria-invalid={!!aksi.isian.suhu} />
			</Isian>
			<Isian label="Waktu pembacaan" id="c-waktu" galat={aksi.isian.waktu}>
				<input id="c-waktu" class="input" type="datetime-local" bind:value={waktuBaca} required aria-invalid={!!aksi.isian.waktu} />
			</Isian>
			<Isian label="Catatan" id="c-catatan">
				<input id="c-catatan" class="input" bind:value={catatan} />
			</Isian>
			{#if hasil?.menyimpang}
				<div class="peringatan" role="alert">
					<p>
						Suhu di luar rentang. Penyimpangan suhu dibuka dan kantong di alat ini dikarantina.
						<a class="underline" href="/suhu/penyimpangan/{hasil.penyimpangan_id}">Nilai kantong</a>
					</p>
				</div>
			{:else}
				<PesanAksi {aksi} />
			{/if}
			<button class="btn btn--utama self-start" disabled={aksi.sibuk}>Catat</button>
		</form>
	{/if}

	<section class="panel" aria-labelledby="judul-grafik">
		<div class="mb-3 flex flex-wrap items-end justify-between gap-2">
			<h2 id="judul-grafik">Grafik suhu 7 hari</h2>
			{#if aktif.length}
				<div>
					<label class="label" for="g-alat">Alat</label>
					<select id="g-alat" class="input" value={alatGrafik?.id} onchange={(e) => (pilihGrafik = e.currentTarget.value)}>
						{#each aktif as a (a.id)}<option value={a.id}>{a.kode} — {a.nama}</option>{/each}
					</select>
				</div>
			{/if}
		</div>
		{#if alatGrafik}
			{#key alatGrafik.id + (alatGrafik.suhu_terakhir?.waktu ?? '')}
				<GrafikSuhu alat={alatGrafik} />
			{/key}
		{:else}
			<p class="text-sm text-redup">Belum ada alat simpan aktif.</p>
		{/if}
	</section>
</div>

<section class="panel" aria-labelledby="judul-penyimpangan">
	<h2 id="judul-penyimpangan" class="mb-3">Penyimpangan suhu</h2>
	<Keadaan sumber={penyimpangan}>
		{#snippet kosong()}<p>Belum pernah ada penyimpangan suhu.</p>{/snippet}
		{#snippet children(data)}
			<div class="overflow-x-auto">
				<table class="tabel">
					<thead>
						<tr>
							<th scope="col">Alat</th>
							<th scope="col">Mulai</th>
							<th scope="col">Suhu ekstrem</th>
							<th scope="col">Kantong</th>
							<th scope="col">Status</th>
						</tr>
					</thead>
					<tbody>
						{#each data as p (p.id)}
							<tr>
								<td><a class="mono text-utama hover:underline" href="/suhu/penyimpangan/{p.id}">{p.alat}</a></td>
								<td class="whitespace-nowrap">{waktu(p.mulai)}</td>
								<td class="tabular-nums">{suhu(p.suhu_ekstrem)}</td>
								<td>
									{p.jumlah}
									{#if p.belum_dinilai}<span class="block text-xs text-waspada">{p.belum_dinilai} belum dinilai</span>{/if}
								</td>
								<td>
									{#if p.status === 'terbuka'}
										<Chip nada="bahaya">Terbuka</Chip>
									{:else}
										<Chip>Ditutup</Chip><span class="block text-xs text-redup">{waktu(p.ditutup_pada)}</span>
									{/if}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/snippet}
	</Keadaan>
</section>
