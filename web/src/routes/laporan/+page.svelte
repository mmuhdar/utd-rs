<script lang="ts">
	import { merek } from '#lib/merek.svelte.ts';
	import { api } from '#lib/api.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import { sesi } from '#lib/sesi.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import Chip from '#lib/Chip.svelte';
	import Tab, { tabAktif } from '#lib/utd/Tab.svelte';
	import { KOMPONEN, hariIni, waktu } from '#lib/format.ts';
	import { PARAMETER_IMLTD } from '#lib/utd/label.ts';
	import KepalaHalaman from '#lib/KepalaHalaman.svelte';
	import Golongan from '#lib/Golongan.svelte';
	import Formulir3 from './Formulir3.svelte';
	import Printer from '@lucide/svelte/icons/printer';
	import Download from '@lucide/svelte/icons/download';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';

	const daftarTab = $derived([
		{ id: 'formulir', label: 'Formulir 3 bulanan' },
		{ id: 'mutu', label: 'Mutu' },
		{ id: 'perencanaan', label: 'Perencanaan' },
		{ id: 'ketersediaan', label: 'Ketersediaan' },
		...(sesi.modeUtd ? [{ id: 'utd', label: 'Laporan UTD' }] : [])
	]);
	const tab = $derived(tabAktif(daftarTab));
	let bulan = $state(hariIni().slice(0, 7));

	type Mutu = {
		bulan: string;
		rasio_ct: { komponen: string; uji_silang: number; transfusi: number; rasio: number | null }[];
		batas_rasio_ct: number;
		kedaluwarsa: number;
		kedaluwarsa_di_stok: number;
		dimusnahkan: number;
		permintaan_tidak_terpenuhi: number;
		menit_tunggu_darurat: number | null;
		insiden: number;
		reaksi: number;
	};
	type Rencana = {
		komponen: string;
		abo: string;
		rhesus: string;
		rerata_mingguan: number;
		stok_minimal: number;
		stok_ideal: number;
		tersedia: number;
		kurang: number;
	};
	type Utd = {
		bulan: string;
		seleksi: Record<'total' | 'diterima' | 'ditolak_sementara' | 'ditolak_permanen', number>;
		donasi: Record<'total' | 'sukarela' | 'pengganti' | 'gagal', number>;
		imltd_reaktif: Record<string, number>;
		komponen_diolah: Record<string, number>;
	};

	const formulir = new Sumber(() => api<any>('GET', `/laporan/formulir3?bulan=${bulan}`));
	const mutu = new Sumber(() => api<Mutu>('GET', `/laporan/mutu?bulan=${bulan}`));
	const rencana = new Sumber(() => api<Rencana[]>('GET', '/laporan/perencanaan'));
	const tersedia = new Sumber(() => api<{ waktu: string; kode_faskes: string | null }>('GET', '/laporan/ketersediaan'));
	const utd = new Sumber(() => api<Utd>('GET', `/laporan/utd?bulan=${bulan}`));
	const SUMBER: Record<string, Sumber<any>> = { formulir, mutu, perencanaan: rencana, ketersediaan: tersedia, utd };
	const pakaiBulan = $derived(['formulir', 'mutu', 'utd'].includes(tab));

	$effect(() => {
		void bulan;
		if (/^\d{4}-\d{2}$/.test(bulan)) SUMBER[tab]?.muat();
	});

	const angka = (n: number | null | undefined, d = 0) =>
		n === null || n === undefined ? '—' : n.toLocaleString('id-ID', { maximumFractionDigits: d });

	function unduh(data: { waktu: string; kode_faskes: string | null }) {
		const url = URL.createObjectURL(new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' }));
		const a = document.createElement('a');
		a.href = url;
		a.download = `ketersediaan-darah-${data.kode_faskes ?? 'rs'}-${data.waktu.slice(0, 16).replace(/[:T]/g, '')}.json`;
		a.click();
		URL.revokeObjectURL(url);
	}
</script>

{#snippet kartu(label: string, nilai: string, nada?: string, ket?: string)}
	<div class="panel flex flex-col gap-2">
		<p class="text-sm font-semibold text-redup">{label}</p>
		<p class="text-4xl font-extrabold tracking-tight tabular-nums {nada ?? ''}">{nilai}</p>
		{#if ket}<p class="text-xs font-semibold text-redup">{ket}</p>{/if}
	</div>
{/snippet}

<svelte:head><title>Laporan — {merek.judul}</title></svelte:head>

<div class="tanpa-cetak">
	<KepalaHalaman judul="Laporan & mutu" jalur={[['Laporan'], ['Laporan & mutu']]} sub="Formulir 3 bulanan, indikator mutu, perencanaan kebutuhan, dan ketersediaan darah.">
		{#snippet aksi()}
			<div class="flex items-end gap-2">
				{#if pakaiBulan}
					<div>
						<label class="label" for="bulan">Bulan</label>
						<input id="bulan" class="input" type="month" bind:value={bulan} max={hariIni().slice(0, 7)} />
					</div>
				{/if}
				{#if tab === 'formulir'}
					<button type="button" class="btn" onclick={() => window.print()} disabled={!formulir.data}>
						<Printer class="size-4" aria-hidden="true" />Cetak
					</button>
				{/if}
			</div>
		{/snippet}
	</KepalaHalaman>
</div>

<Tab daftar={daftarTab} label="Jenis laporan" />

{#if tab === 'formulir'}
	<section class="panel print:border-0 print:p-0">
		<Keadaan sumber={formulir} apakahKosong={() => false} baris={8}>
			{#snippet children(d)}<Formulir3 {d} />{/snippet}
		</Keadaan>
	</section>
{:else if tab === 'mutu'}
	<Keadaan sumber={mutu} apakahKosong={() => false}>
		{#snippet children(m)}
			<h2 class="sr-only">Indikator mutu</h2>
			<div class="mb-5 grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
				{@render kartu('Kantong kedaluwarsa (dimusnahkan)', angka(m.kedaluwarsa), m.kedaluwarsa ? 'text-waspada' : undefined)}
				{@render kartu('Kedaluwarsa masih di stok', angka(m.kedaluwarsa_di_stok), m.kedaluwarsa_di_stok ? 'text-bahaya' : undefined, 'Saat ini; segera musnahkan')}
				{@render kartu('Kantong dimusnahkan', angka(m.dimusnahkan))}
				{@render kartu('Permintaan tidak terpenuhi', angka(m.permintaan_tidak_terpenuhi), m.permintaan_tidak_terpenuhi ? 'text-waspada' : undefined)}
				{@render kartu('Rerata tunggu darurat (menit)', angka(m.menit_tunggu_darurat, 1), undefined, 'Dari diminta sampai kantong pertama keluar')}
				{@render kartu('Insiden / KNC', angka(m.insiden), m.insiden ? 'text-waspada' : undefined)}
				{@render kartu('Reaksi transfusi', angka(m.reaksi), m.reaksi ? 'text-waspada' : undefined)}
			</div>
			<section class="panel" aria-labelledby="judul-ct">
				<h2 id="judul-ct">Rasio C:T per komponen</h2>
				<p class="mb-3 text-sm text-redup">Kantong yang diuji silang dibagi kantong yang ditransfusikan. Di atas {angka(m.batas_rasio_ct, 1)} dianggap uji silang berlebihan.</p>
				<div class="overflow-x-auto">
					<table class="tabel">
						<thead>
							<tr><th scope="col">Komponen</th><th scope="col" class="text-right">Diuji silang</th><th scope="col" class="text-right">Ditransfusikan</th><th scope="col" class="text-right">Rasio C:T</th><th scope="col">Penilaian</th></tr>
						</thead>
						<tbody>
							{#each m.rasio_ct as r (r.komponen)}
								{@const lebih = r.rasio !== null && r.rasio > m.batas_rasio_ct}
								<tr>
									<td>{KOMPONEN[r.komponen] ?? r.komponen}</td>
									<td class="text-right tabular-nums">{r.uji_silang}</td>
									<td class="text-right tabular-nums">{r.transfusi}</td>
									<td class="text-right font-semibold tabular-nums {lebih ? 'text-bahaya' : ''}">{angka(r.rasio, 2)}</td>
									<td>
										{#if r.rasio === null}
											<Chip>Tidak ada transfusi</Chip>
										{:else if lebih}
											<Chip nada="bahaya">Berlebihan (&gt; {angka(m.batas_rasio_ct, 1)})</Chip>
										{:else}
											<Chip nada="aman">Wajar</Chip>
										{/if}
									</td>
								</tr>
							{:else}
								<tr><td colspan="5" class="text-redup">Tidak ada uji silang atau transfusi pada periode ini.</td></tr>
							{/each}
						</tbody>
					</table>
				</div>
			</section>
		{/snippet}
	</Keadaan>
{:else if tab === 'perencanaan'}
	<section class="panel">
		<Keadaan sumber={rencana}>
			{#snippet kosong()}<p>Belum ada pemakaian maupun stok untuk dihitung.</p>{/snippet}
			{#snippet children(data)}
				<h2>Perencanaan kebutuhan</h2>
				<p class="mb-3 text-sm text-redup">Rerata pemakaian mingguan 26 minggu terakhir. Stok minimal = kebutuhan 1 minggu, stok ideal = 2 minggu.</p>
				<div class="overflow-x-auto">
					<table class="tabel">
						<thead>
							<tr>
								<th scope="col">Komponen</th>
								<th scope="col">Golongan</th>
								<th scope="col" class="text-right">Rerata/minggu</th>
								<th scope="col" class="text-right">Stok minimal</th>
								<th scope="col" class="text-right">Stok ideal</th>
								<th scope="col" class="text-right">Tersedia</th>
								<th scope="col">Penilaian</th>
							</tr>
						</thead>
						<tbody>
							{#each data as r (`${r.komponen}${r.abo}${r.rhesus}`)}
								<tr class={r.kurang > 0 ? 'bg-bahaya-latar' : ''}>
									<td>{KOMPONEN[r.komponen] ?? r.komponen}</td>
									<td><Golongan abo={r.abo} rhesus={r.rhesus} ukuran="kecil" /></td>
									<td class="text-right tabular-nums">{angka(r.rerata_mingguan, 1)}</td>
									<td class="text-right tabular-nums">{r.stok_minimal}</td>
									<td class="text-right tabular-nums">{r.stok_ideal}</td>
									<td class="text-right font-semibold tabular-nums">{r.tersedia}</td>
									<td>
										{#if r.kurang > 0}
											<Chip nada="bahaya">Kurang {r.kurang} dari minimal</Chip>
										{:else if r.tersedia < r.stok_ideal}
											<Chip nada="waspada">Di bawah ideal</Chip>
										{:else}
											<Chip nada="aman">Cukup</Chip>
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
{:else if tab === 'ketersediaan'}
	<section class="panel">
		<Keadaan sumber={tersedia} apakahKosong={() => false}>
			{#snippet children(data)}
				<div class="mb-3 flex flex-wrap items-start justify-between gap-3">
					<div>
						<h2>Ketersediaan darah (kanal nasional)</h2>
						<p class="text-sm text-redup">Kantong tersedia yang belum kedaluwarsa, per {waktu(data.waktu)}. Format JSON untuk diteruskan ke kanal yang ditetapkan Kemenkes.</p>
					</div>
					<div class="flex gap-2">
						<button type="button" class="btn" onclick={() => tersedia.muat()} disabled={tersedia.memuat}>
							<RefreshCw class="size-4" aria-hidden="true" />Perbarui
						</button>
						<button type="button" class="btn btn--utama" onclick={() => unduh(data)}>
							<Download class="size-4" aria-hidden="true" />Unduh .json
						</button>
					</div>
				</div>
				<pre class="mono max-h-[32rem] overflow-auto rounded-xl border border-garis bg-latar p-4 text-xs">{JSON.stringify(data, null, 2)}</pre>
			{/snippet}
		</Keadaan>
	</section>
{:else if tab === 'utd'}
	<Keadaan sumber={utd} apakahKosong={() => false}>
		{#snippet children(u)}
			<h2 class="sr-only">Kegiatan UTD</h2>
			<div class="grid gap-5 md:grid-cols-2">
				<div>
					<h3 class="mb-2 font-bold">Seleksi donor</h3>
					<div class="grid grid-cols-2 gap-4">
						{@render kartu('Total', angka(u.seleksi.total))}
						{@render kartu('Diterima', angka(u.seleksi.diterima))}
						{@render kartu('Ditolak sementara', angka(u.seleksi.ditolak_sementara))}
						{@render kartu('Ditolak permanen', angka(u.seleksi.ditolak_permanen))}
					</div>
				</div>
				<div>
					<h3 class="mb-2 font-bold">Donasi</h3>
					<div class="grid grid-cols-2 gap-4">
						{@render kartu('Total', angka(u.donasi.total))}
						{@render kartu('Sukarela', angka(u.donasi.sukarela))}
						{@render kartu('Pengganti', angka(u.donasi.pengganti))}
						{@render kartu('Gagal', angka(u.donasi.gagal))}
					</div>
				</div>
				<div class="panel">
					<h3 class="mb-3 font-bold">Uji saring IMLTD reaktif (terverifikasi)</h3>
					<table class="tabel">
						<thead><tr><th scope="col">Parameter</th><th scope="col" class="text-right">Reaktif</th></tr></thead>
						<tbody>
							{#each Object.keys(PARAMETER_IMLTD) as p (p)}
								{@const n = u.imltd_reaktif[p] ?? 0}
								<tr>
									<td>{PARAMETER_IMLTD[p]}</td>
									<td class="text-right font-semibold tabular-nums {n ? 'text-bahaya' : ''}">{n}</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
				<div class="panel">
					<h3 class="mb-3 font-bold">Komponen diolah</h3>
					<table class="tabel">
						<thead><tr><th scope="col">Komponen</th><th scope="col" class="text-right">Kantong</th></tr></thead>
						<tbody>
							{#each Object.keys(KOMPONEN) as k (k)}
								<tr><td>{KOMPONEN[k]}</td><td class="text-right tabular-nums">{u.komponen_diolah[k] ?? 0}</td></tr>
							{/each}
						</tbody>
					</table>
				</div>
			</div>
		{/snippet}
	</Keadaan>
{/if}
