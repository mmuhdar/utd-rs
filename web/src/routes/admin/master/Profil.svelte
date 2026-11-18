<script lang="ts">
	import { api } from '#lib/api.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Isian from '#lib/Isian.svelte';
	import { sesi } from '#lib/sesi.svelte.ts';
	import { waktu } from '#lib/format.ts';

	type Profil = { nama_rs: string; kode_faskes: string | null; jenis: 'bdrs' | 'utd_rs'; diubah_pada: string };
	let isi = $state({ nama_rs: '', kode_faskes: '', jenis: 'bdrs' as 'bdrs' | 'utd_rs' });
	const profil = new Sumber(async () => {
		const p = await api<Profil>('GET', '/master/profil');
		isi = { nama_rs: p.nama_rs, kode_faskes: p.kode_faskes ?? '', jenis: p.jenis };
		return p;
	});
	$effect(() => {
		profil.muat();
	});
	const aksi = new Aksi();

	async function simpan(e: SubmitEvent) {
		e.preventDefault();
		if (
			profil.data &&
			isi.jenis !== profil.data.jenis &&
			!confirm(
				isi.jenis === 'utd_rs'
					? 'Ubah ke mode UTD-RS? Menu donor, donasi, dan pelulusan akan aktif, dan kantong hasil donasi masuk antrean SATUSEHAT.'
					: 'Ubah ke mode BDRS saja? Pencatatan donor, donasi, dan pelulusan akan dinonaktifkan. Data yang sudah ada tetap tersimpan.'
			)
		)
			return;
		const r = await aksi.jalankan(() => api('PUT', '/master/profil', isi), 'Profil instalasi disimpan.');
		if (r) {
			profil.muat();
			sesi.muat(); // nama RS dan menu UTD di navigasi mengikuti profil
		}
	}
</script>

<Keadaan sumber={profil} apakahKosong={() => false} baris={4}>
	{#snippet children(p)}
		<form class="panel flex max-w-2xl flex-col gap-4" onsubmit={simpan}>
			<h2>Profil instalasi</h2>
			<div class="form-grid">
				<Isian label="Nama rumah sakit" id="p-nama" galat={aksi.isian.nama_rs} bantuan="Tampil di kop laporan dan menu.">
					<input id="p-nama" class="input" bind:value={isi.nama_rs} required />
				</Isian>
				<Isian label="Kode faskes" id="p-kode" galat={aksi.isian.kode_faskes} bantuan="Kode fasilitas kesehatan Kemenkes, untuk laporan nasional.">
					<input id="p-kode" class="input mono" bind:value={isi.kode_faskes} />
				</Isian>
			</div>
			<fieldset>
				<legend class="label">Mode instalasi</legend>
				<div class="flex flex-col gap-2">
					<label class="flex items-start gap-2 rounded-md border border-garis p-3 text-sm">
						<input type="radio" name="jenis" value="bdrs" bind:group={isi.jenis} class="mt-1" />
						<span>
							<span class="font-medium">BDRS saja</span>
							<span class="block text-redup">Bank darah RS menerima kantong dari UTD mitra, menguji pratransfusi, dan menyerahkan ke ruang. Tidak menerima donor.</span>
						</span>
					</label>
					<label class="flex items-start gap-2 rounded-md border border-garis p-3 text-sm">
						<input type="radio" name="jenis" value="utd_rs" bind:group={isi.jenis} class="mt-1" />
						<span>
							<span class="font-medium">UTD-RS</span>
							<span class="block text-redup">
								Selain BDRS, RS menjalankan UTD sendiri: menu donor, donasi, uji saring IMLTD, pengolahan, dan pelulusan aktif; laporan UTD tersedia;
								kantong hasil donasi yang diluluskan atau dimusnahkan masuk antrean SATUSEHAT.
							</span>
						</span>
					</label>
				</div>
				{#if aksi.isian.jenis}<p class="mt-1 text-sm text-bahaya" role="alert">{aksi.isian.jenis}</p>{/if}
			</fieldset>
			<p class="text-xs text-redup">Terakhir diubah {waktu(p.diubah_pada)}.</p>
			<PesanAksi {aksi} />
			<button class="btn btn--utama self-start" disabled={aksi.sibuk}>Simpan profil</button>
		</form>
	{/snippet}
</Keadaan>
