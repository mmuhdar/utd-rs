<script lang="ts">
	import { merek } from '#lib/merek.svelte.ts';
	import { goto } from '$app/navigation';
	import { api } from '#lib/api.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Isian from '#lib/Isian.svelte';
	import KepalaHalaman from '#lib/KepalaHalaman.svelte';
	import Golongan from '#lib/Golongan.svelte';
	import Pilihan from '#lib/pelayanan/Pilihan.svelte';
	import { tanggal } from '#lib/format.ts';
	import Search from '@lucide/svelte/icons/search';
	import UserPlus from '@lucide/svelte/icons/user-plus';

	type Donor = { id: string; nik: string; nama: string; tanggal_lahir: string; jenis_kelamin: string; abo: string | null; rhesus: string | null };

	let q = $state('');
	let dicari = $state('');
	const hasil = new Sumber(() => api<Donor[]>('GET', `/donor?q=${encodeURIComponent(dicari)}`));

	function cari(e: SubmitEvent) {
		e.preventDefault();
		dicari = q.trim();
		if (dicari.length >= 3) hasil.muat();
	}

	let baru = $state({ nik: '', nama: '', tanggal_lahir: '', jenis_kelamin: '', alamat: '', telepon: '' });
	const aksi = new Aksi();

	async function daftar(e: SubmitEvent) {
		e.preventDefault();
		if (!/^\d{16}$/.test(baru.nik)) {
			aksi.isian = { nik: 'NIK terdiri dari 16 angka.' };
			return;
		}
		// Pengganti `required` radio bawaan: Pilihan tidak punya prop required.
		if (!baru.jenis_kelamin) {
			aksi.isian = { jenis_kelamin: 'Pilih jenis kelamin.' };
			return;
		}
		const d = await aksi.jalankan(() => api<Donor>('POST', '/donor', baru));
		if (d) goto(`/donor/${d.id}`);
	}
</script>

<svelte:head><title>Donor — {merek.judul}</title></svelte:head>

<KepalaHalaman judul="Donor" jalur={[['UTD'], ['Donor']]} sub="Cari donor lewat NIK atau nama, lalu catat seleksi dan pengambilan darah." />

<div class="grid items-start gap-5 xl:grid-cols-2">
	<section class="panel" aria-labelledby="judul-cari">
		<h2 id="judul-cari">Cari donor</h2>
		<p class="mb-4 text-sm text-redup">Selalu cari lebih dulu supaya donor tidak terdaftar dua kali.</p>
		<form class="flex items-end gap-2" onsubmit={cari} role="search">
			<div class="flex-1">
				<label class="label" for="q">NIK atau nama</label>
				<input id="q" class="input" bind:value={q} minlength="3" autocomplete="off" placeholder="Minimal 3 karakter" />
			</div>
			<button class="btn btn--utama"><Search class="size-4" aria-hidden="true" />Cari</button>
		</form>
		<div class="mt-4">
			{#if dicari.length >= 3}
				<Keadaan sumber={hasil} baris={3}>
					{#snippet kosong()}
						<p>Tidak ada donor dengan NIK atau nama "{dicari}". Daftarkan sebagai donor baru.</p>
					{/snippet}
					{#snippet children(data)}
						<ul class="-mx-2 divide-y divide-garis">
							{#each data as d (d.id)}
								<li>
									<a href="/donor/{d.id}" class="flex items-center justify-between gap-3 rounded-lg px-2 py-2.5 hover:bg-halus">
										<span class="min-w-0">
											<span class="font-semibold">{d.nama}</span>
											<span class="block text-xs text-redup"><span class="mono">{d.nik}</span> · lahir {tanggal(d.tanggal_lahir)} · {d.jenis_kelamin}</span>
										</span>
										<Golongan abo={d.abo} rhesus={d.rhesus} />
									</a>
								</li>
							{/each}
						</ul>
					{/snippet}
				</Keadaan>
			{:else if dicari}
				<p class="text-sm text-redup">Ketik minimal 3 karakter.</p>
			{/if}
		</div>
	</section>

	<form class="panel flex flex-col gap-4" onsubmit={daftar} aria-labelledby="judul-baru">
		<div>
			<h2 id="judul-baru">Daftarkan donor baru</h2>
			<p class="text-sm text-redup">Identitas sesuai KTP.</p>
		</div>
		<p class="catatan">Cari NIK lebih dulu. Bila NIK sudah terdaftar, menyimpan formulir ini memperbarui identitas donor tersebut.</p>
		<div class="form-grid">
			<Isian label="NIK" id="nik" galat={aksi.isian.nik} bantuan="16 angka sesuai KTP.">
				<input id="nik" class="input mono" inputmode="numeric" maxlength="16" bind:value={baru.nik} required aria-invalid={!!aksi.isian.nik} />
			</Isian>
			<Isian label="Nama lengkap" id="nama" galat={aksi.isian.nama}>
				<input id="nama" class="input" bind:value={baru.nama} required aria-invalid={!!aksi.isian.nama} />
			</Isian>
			<Isian label="Tanggal lahir" id="lahir" galat={aksi.isian.tanggal_lahir}>
				<input id="lahir" class="input" type="date" bind:value={baru.tanggal_lahir} required aria-invalid={!!aksi.isian.tanggal_lahir} />
			</Isian>
			<Pilihan
				legend="Jenis kelamin"
				name="jk"
				opsi={[
					['L', 'Laki-laki'],
					['P', 'Perempuan']
				]}
				bind:value={baru.jenis_kelamin}
				galat={aksi.isian.jenis_kelamin}
			/>
			<Isian label="Alamat" id="alamat">
				<input id="alamat" class="input" bind:value={baru.alamat} />
			</Isian>
			<Isian label="Telepon" id="telepon">
				<input id="telepon" class="input" type="tel" bind:value={baru.telepon} />
			</Isian>
		</div>
		<PesanAksi {aksi} />
		<div class="flex justify-end gap-2.5 border-t border-garis pt-4">
			<button class="btn btn--utama" disabled={aksi.sibuk}><UserPlus class="size-4" aria-hidden="true" />Simpan donor</button>
		</div>
	</form>
</div>
