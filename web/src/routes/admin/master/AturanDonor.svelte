<script lang="ts">
	import { api } from '#lib/api.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Isian from '#lib/Isian.svelte';
	import { waktu } from '#lib/format.ts';

	// Kunci sama dengan kolom tabel aturan_donor; server menolak kunci lain.
	const ISIAN: [string, string, string][] = [
		['umur_min', 'Umur minimum', 'tahun'],
		['umur_maks', 'Umur maksimum', 'tahun'],
		['berat_min', 'Berat minimum', 'kg'],
		['hb_min', 'Hb minimum', 'g/dL'],
		['hb_maks', 'Hb maksimum', 'g/dL'],
		['sistolik_min', 'Sistolik minimum', 'mmHg'],
		['sistolik_maks', 'Sistolik maksimum', 'mmHg'],
		['diastolik_min', 'Diastolik minimum', 'mmHg'],
		['diastolik_maks', 'Diastolik maksimum', 'mmHg'],
		['nadi_min', 'Nadi minimum', 'kali/menit'],
		['nadi_maks', 'Nadi maksimum', 'kali/menit'],
		['suhu_maks', 'Suhu maksimum', '°C'],
		['hari_antar_donasi', 'Jarak minimal antar donasi', 'hari']
	];
	const DESIMAL = ['berat_min', 'hb_min', 'hb_maks', 'suhu_maks'];

	let nilai = $state<Record<string, number>>({});
	let alasan = $state('');
	const aturan = new Sumber(async () => {
		const a = await api<Record<string, number> & { diubah_pada: string }>('GET', '/aturan-donor');
		nilai = Object.fromEntries(ISIAN.map(([k]) => [k, a[k]]));
		return a;
	});
	$effect(() => {
		aturan.muat();
	});
	const aksi = new Aksi();

	async function simpan(e: SubmitEvent) {
		e.preventDefault();
		const r = await aksi.jalankan(() => api('PUT', '/aturan-donor', { nilai, alasan }), 'Ambang seleksi donor disimpan.');
		if (r) {
			alasan = '';
			aturan.muat();
		}
	}
</script>

<Keadaan sumber={aturan} apakahKosong={() => false}>
	{#snippet children(a)}
		<form class="panel flex max-w-3xl flex-col gap-4" onsubmit={simpan}>
			<div>
				<h2>Ambang seleksi donor</h2>
				<p class="text-sm text-redup">Donor di luar ambang ini tidak bisa diterima pada seleksi. Terakhir diubah {waktu(a.diubah_pada)}.</p>
			</div>
			<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
				{#each ISIAN as [k, label, satuan] (k)}
					<Isian label="{label} ({satuan})" id="ad-{k}" galat={aksi.isian[k]}>
						<input id="ad-{k}" class="input" type="number" step={DESIMAL.includes(k) ? '0.1' : '1'} bind:value={nilai[k]} required />
					</Isian>
				{/each}
			</div>
			<Isian label="Alasan perubahan" id="ad-alasan" galat={aksi.isian.alasan}>
				<textarea id="ad-alasan" class="input" rows="2" bind:value={alasan} required></textarea>
			</Isian>
			<PesanAksi {aksi} />
			<button class="btn btn--utama self-start" disabled={aksi.sibuk}>Simpan ambang</button>
		</form>
	{/snippet}
</Keadaan>
