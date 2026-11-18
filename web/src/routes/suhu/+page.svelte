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
	import KepalaHalaman from '#lib/KepalaHalaman.svelte';

	const alat = new Sumber(() => api<Alat[]>('GET', '/master/alat'));
	const penyimpangan = new Sumber(() => api<any[]>('GET', '/penyimpangan-suhu'));
	$effect(() => {
		alat.muat();
		penyimpangan.muat();
	});

	const aktif = $derived((alat.data ?? []).filter((a) => a.aktif));
	const terbuka = $derived((penyimpangan.data ?? []).filter((p) => p.status === 'terbuka'));
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

<KepalaHalaman
	judul="Suhu alat simpan"
	sub={alat.data ? `${aktif.length} alat aktif · ${aktif.filter((a) => a.penyimpangan_id || diLuar(a)).length} menyimpang` : undefined}
/>

{#if terbuka.length}
	<div class="peringatan mb-5" role="alert">
		<Thermometer aria-hidden="true" />
		<div class="flex-1">
			<p class="font-bold">Penyimpangan suhu terbuka di {terbuka.map((p) => p.alat).join(', ')}</p>
			<p>Kantong di alat tersebut dikarantina sampai dinilai penanggung jawab.</p>
		</div>
		{#if terbuka.length === 1}<a class="btn shrink-0 text-teks" href="/suhu/penyimpangan/{terbuka[0].id}">Nilai sekarang</a>{/if}
	</div>
{/if}

<section aria-labelledby="judul-alat" class="mb-5">
	<h2 id="judul-alat" class="sr-only">Alat simpan</h2>
	<Keadaan sumber={alat} baris={3}>
		{#snippet kosong()}<p>Belum ada alat simpan. Tambahkan di Data induk.</p>{/snippet}
		{#snippet children()}
			<ul class="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
				{#each aktif as a (a.id)}
					{@const buruk = !!a.penyimpangan_id || diLuar(a)}
					<li class="panel flex flex-col gap-3 {buruk ? 'border-bahaya-titik bg-bahaya-latar' : ''}">
						<div class="flex items-start justify-between gap-2">
							<span class="flex min-w-0 items-center gap-2.5">
								<span class="grid size-9 shrink-0 place-items-center rounded-lg {buruk ? 'bg-bahaya-titik text-white' : 'bg-halus text-redup'}">
									<Thermometer class="size-[18px]" aria-hidden="true" />
								</span>
								<span class="min-w-0">
									<span class="block truncate font-bold">{a.nama}</span>
									<span class="block truncate text-xs text-redup"><span class="mono">{a.kode}</span> · {NAMA_JENIS_ALAT[a.jenis]}</span>
								</span>
							</span>
							{#if a.penyimpangan_id}
								<Chip nada="bahaya">Menyimpang</Chip>
							{:else if diLuar(a)}
								<Chip nada="bahaya">Di luar rentang</Chip>
							{:else if a.suhu_terakhir}
								<Chip nada="aman">Normal</Chip>
							{:else}
								<Chip>Belum dicatat</Chip>
							{/if}
						</div>
						<div>
							<p class="text-4xl font-extrabold tracking-tight tabular-nums {buruk ? 'text-bahaya' : ''}">
								{a.suhu_terakhir ? suhu(a.suhu_terakhir.suhu) : '—'}
							</p>
							<p class="mt-1 text-xs text-redup">
								{a.suhu_terakhir ? `${relatif(a.suhu_terakhir.waktu)} (${waktu(a.suhu_terakhir.waktu)})` : 'Belum ada pembacaan'}
							</p>
						</div>
						<div class="mt-auto flex flex-wrap items-center justify-between gap-2 border-t pt-3 text-sm {buruk ? 'border-bahaya-titik/40' : 'border-garis'}">
							<span class="text-teks-2">Rentang <span class="font-semibold tabular-nums">{suhu(a.suhu_min)} – {suhu(a.suhu_maks)}</span></span>
							<span class="text-redup"><span class="font-semibold text-teks-2 tabular-nums">{a.jumlah_kantong}</span> kantong</span>
						</div>
						{#if a.penyimpangan_id}
							<a class="btn btn--bahaya" href="/suhu/penyimpangan/{a.penyimpangan_id}">Penyimpangan terbuka — nilai kantong</a>
						{/if}
					</li>
				{/each}
			</ul>
		{/snippet}
	</Keadaan>
</section>

<div class="mb-5 grid items-start gap-5 xl:grid-cols-[1fr_360px]">
	<section class="panel min-w-0" aria-labelledby="judul-grafik">
		<div class="mb-4 flex flex-wrap items-end justify-between gap-3">
			<div>
				<h2 id="judul-grafik">Grafik suhu 7 hari</h2>
				{#if alatGrafik}<p class="text-sm text-redup">{alatGrafik.nama} · rentang {suhu(alatGrafik.suhu_min)} – {suhu(alatGrafik.suhu_maks)}</p>{/if}
			</div>
			{#if aktif.length}
				<div>
					<label class="sr-only" for="g-alat">Alat</label>
					<select id="g-alat" class="input w-auto" value={alatGrafik?.id} onchange={(e) => (pilihGrafik = e.currentTarget.value)}>
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

	{#if sesi.boleh('suhu.catat')}
		<form class="panel flex flex-col gap-4" onsubmit={catat} aria-labelledby="judul-catat">
			<div>
				<h2 id="judul-catat">Catat suhu manual</h2>
				<p class="text-sm text-redup">Untuk alat tanpa data logger atau saat logger mati.</p>
			</div>
			<Isian label="Alat simpan" id="c-alat" galat={aksi.isian.alat_id}>
				<select id="c-alat" class="input" bind:value={alat_id} required>
					<option value="" disabled>Pilih alat</option>
					{#each aktif as a (a.id)}<option value={a.id}>{a.kode} — {a.nama}</option>{/each}
				</select>
			</Isian>
			<div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-1">
				<Isian label="Suhu (°C)" id="c-suhu" galat={aksi.isian.suhu}>
					<input id="c-suhu" class="input text-base font-semibold tabular-nums" type="number" step="0.1" bind:value={nilai} required aria-invalid={!!aksi.isian.suhu} />
				</Isian>
				<Isian label="Waktu pembacaan" id="c-waktu" galat={aksi.isian.waktu}>
					<input id="c-waktu" class="input" type="datetime-local" bind:value={waktuBaca} required aria-invalid={!!aksi.isian.waktu} />
				</Isian>
			</div>
			<Isian label="Catatan" id="c-catatan">
				<input id="c-catatan" class="input" bind:value={catatan} />
			</Isian>
			{#if hasil?.menyimpang}
				<div class="peringatan" role="alert">
					<Thermometer aria-hidden="true" />
					<p>
						Suhu di luar rentang. Penyimpangan suhu dibuka dan kantong di alat ini dikarantina.
						<a class="font-semibold underline" href="/suhu/penyimpangan/{hasil.penyimpangan_id}">Nilai kantong</a>
					</p>
				</div>
			{:else}
				<PesanAksi {aksi} />
			{/if}
			<div class="flex justify-end gap-2.5 border-t border-garis pt-4">
				<button class="btn btn--utama" disabled={aksi.sibuk}>Catat</button>
			</div>
		</form>
	{/if}
</div>

<section class="panel" aria-labelledby="judul-penyimpangan">
	<div class="mb-3 flex flex-wrap items-baseline justify-between gap-2">
		<h2 id="judul-penyimpangan">Penyimpangan suhu</h2>
		{#if penyimpangan.data}<p class="text-sm text-redup">{terbuka.length} terbuka · {penyimpangan.data.length} seluruhnya</p>{/if}
	</div>
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
								<td><a class="mono font-semibold hover:underline" href="/suhu/penyimpangan/{p.id}">{p.alat}</a></td>
								<td class="whitespace-nowrap text-teks-2">{waktu(p.mulai)}</td>
								<td class="font-semibold text-bahaya tabular-nums">{suhu(p.suhu_ekstrem)}</td>
								<td>
									<span class="tabular-nums">{p.jumlah}</span>
									{#if p.belum_dinilai}<span class="mt-1 block"><Chip nada="waspada">{p.belum_dinilai} belum dinilai</Chip></span>{/if}
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
