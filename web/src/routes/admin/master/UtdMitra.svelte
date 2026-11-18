<script lang="ts">
	import { api } from '#lib/api.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Isian from '#lib/Isian.svelte';
	import Chip from '#lib/Chip.svelte';

	type Utd = { id: string; kode: string; nama: string; alamat: string | null; format_label: 'isbt128' | 'lokal'; aktif: boolean };
	const FORMAT = { isbt128: 'ISBT 128', lokal: 'Lokal (nomor apa adanya)' };
	const kosong = () => ({ kode: '', nama: '', alamat: '', format_label: 'isbt128' as Utd['format_label'], aktif: true });

	const utd = new Sumber(() => api<Utd[]>('GET', '/master/utd'));
	$effect(() => {
		utd.muat();
	});
	let id = $state<string | null>(null);
	let isi = $state(kosong());
	const aksi = new Aksi();

	function ubah(u: Utd) {
		id = u.id;
		isi = { kode: u.kode, nama: u.nama, alamat: u.alamat ?? '', format_label: u.format_label, aktif: u.aktif };
		aksi.galat = aksi.berhasil = null;
	}
	function batal() {
		id = null;
		isi = kosong();
	}

	async function simpan(e: SubmitEvent) {
		e.preventDefault();
		const r = await aksi.jalankan(
			() => (id ? api('PUT', `/master/utd/${id}`, isi) : api('POST', '/master/utd', isi)),
			id ? 'UTD mitra diperbarui.' : 'UTD mitra ditambahkan.'
		);
		if (r) {
			batal();
			utd.muat();
		}
	}
</script>

<div class="grid gap-4 xl:grid-cols-[1fr_24rem]">
	<section class="panel min-w-0" aria-labelledby="judul-utd">
		<h2 id="judul-utd" class="mb-3">UTD mitra</h2>
		<Keadaan sumber={utd}>
			{#snippet kosong()}<p>Belum ada UTD mitra. Tambahkan UTD pemasok kantong lewat formulir.</p>{/snippet}
			{#snippet children(data)}
				<div class="overflow-x-auto">
					<table class="tabel">
						<thead><tr><th>Kode</th><th>Nama</th><th>Format label</th><th>Status</th><th><span class="sr-only">Aksi</span></th></tr></thead>
						<tbody>
							{#each data as u (u.id)}
								<tr>
									<td class="mono">{u.kode}</td>
									<td>{u.nama}{#if u.alamat}<span class="block text-xs text-redup">{u.alamat}</span>{/if}</td>
									<td>{FORMAT[u.format_label]}</td>
									<td><Chip nada={u.aktif ? 'aman' : 'netral'}>{u.aktif ? 'Aktif' : 'Nonaktif'}</Chip></td>
									<td><button class="btn btn--teks" onclick={() => ubah(u)} aria-label="Ubah {u.nama}">Ubah</button></td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{/snippet}
		</Keadaan>
	</section>

	<form class="panel flex flex-col gap-4 self-start" onsubmit={simpan} aria-labelledby="judul-form-utd">
		<h2 id="judul-form-utd">{id ? 'Ubah UTD mitra' : 'Tambah UTD mitra'}</h2>
		<Isian label="Kode" id="utd-kode" galat={aksi.isian.kode}>
			<input id="utd-kode" class="input mono" bind:value={isi.kode} required />
		</Isian>
		<Isian label="Nama" id="utd-nama" galat={aksi.isian.nama}>
			<input id="utd-nama" class="input" bind:value={isi.nama} required />
		</Isian>
		<Isian label="Alamat (opsional)" id="utd-alamat" galat={aksi.isian.alamat}>
			<input id="utd-alamat" class="input" bind:value={isi.alamat} />
		</Isian>
		<Isian
			label="Format label kantong"
			id="utd-format"
			galat={aksi.isian.format_label}
			bantuan="ISBT 128: nomor donasi, golongan, dan kedaluwarsa diurai dari barcode. Lokal: nomor dipakai apa adanya, isian lain manual."
		>
			<select id="utd-format" class="input" bind:value={isi.format_label}>
				{#each Object.entries(FORMAT) as [k, l] (k)}<option value={k}>{l}</option>{/each}
			</select>
		</Isian>
		<label class="flex items-center gap-2 text-sm"><input type="checkbox" bind:checked={isi.aktif} />Aktif (UTD tidak dihapus, cukup dinonaktifkan)</label>
		<PesanAksi {aksi} />
		<div class="flex gap-2">
			<button class="btn btn--utama" disabled={aksi.sibuk}>{id ? 'Simpan' : 'Tambah'}</button>
			{#if id}<button type="button" class="btn" onclick={batal}>Batal</button>{/if}
		</div>
	</form>
</div>
