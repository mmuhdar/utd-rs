<script lang="ts">
	import { api } from '#lib/api.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Isian from '#lib/Isian.svelte';
	import Chip from '#lib/Chip.svelte';
	import { waktu } from '#lib/format.ts';

	type Jenis = 'lemari' | 'freezer' | 'inkubator';
	type Alat = {
		id: string;
		kode: string;
		nama: string;
		nomor_seri: string | null;
		jenis: Jenis;
		suhu_min: number;
		suhu_maks: number;
		lokasi: string | null;
		aktif: boolean;
		jumlah_kantong: number;
		penyimpangan_id: string | null;
		suhu_terakhir: { suhu: number; waktu: string } | null;
	};
	const JENIS: Record<Jenis, string> = { lemari: 'Lemari darah', freezer: 'Freezer plasma', inkubator: 'Inkubator trombosit' };
	const kosong = () => ({
		kode: '',
		nama: '',
		nomor_seri: '',
		jenis: 'lemari' as Jenis,
		suhu_min: null as number | null,
		suhu_maks: null as number | null,
		lokasi: '',
		aktif: true
	});

	const alat = new Sumber(() => api<Alat[]>('GET', '/master/alat'));
	$effect(() => {
		alat.muat();
	});
	let id = $state<string | null>(null);
	let isi = $state(kosong());
	const aksi = new Aksi();

	function ubah(a: Alat) {
		id = a.id;
		isi = { kode: a.kode, nama: a.nama, nomor_seri: a.nomor_seri ?? '', jenis: a.jenis, suhu_min: a.suhu_min, suhu_maks: a.suhu_maks, lokasi: a.lokasi ?? '', aktif: a.aktif };
		aksi.galat = aksi.berhasil = null;
	}
	function batal() {
		id = null;
		isi = kosong();
	}

	async function simpan(e: SubmitEvent) {
		e.preventDefault();
		const r = await aksi.jalankan(
			() => (id ? api('PUT', `/master/alat/${id}`, isi) : api('POST', '/master/alat', isi)),
			id ? 'Alat simpan diperbarui.' : 'Alat simpan ditambahkan.'
		);
		if (r) {
			batal();
			alat.muat();
		}
	}
</script>

<div class="grid items-start gap-5 xl:grid-cols-[1fr_360px]">
	<section class="panel min-w-0" aria-labelledby="judul-alat">
		<h2 id="judul-alat">Alat simpan</h2>
		<p class="mb-3 text-sm text-redup">Lemari darah, freezer plasma, dan inkubator trombosit beserta rentang suhunya.</p>
		<Keadaan sumber={alat}>
			{#snippet kosong()}<p>Belum ada alat simpan. Tambahkan lemari darah, freezer plasma, atau inkubator trombosit.</p>{/snippet}
			{#snippet children(data)}
				<div class="overflow-x-auto">
					<table class="tabel">
						<thead>
							<tr><th>Kode</th><th>Nama</th><th>Jenis</th><th>Rentang suhu</th><th>Suhu terakhir</th><th>Kantong</th><th>Status</th><th><span class="sr-only">Aksi</span></th></tr>
						</thead>
						<tbody>
							{#each data as a (a.id)}
								<tr>
									<td class="mono font-semibold">{a.kode}</td>
									<td>{a.nama}<span class="block text-xs text-redup">{[a.nomor_seri, a.lokasi].filter(Boolean).join(' · ')}</span></td>
									<td>{JENIS[a.jenis]}</td>
									<td class="tabular-nums">{a.suhu_min} s.d. {a.suhu_maks} °C</td>
									<td class="tabular-nums">
										{#if a.suhu_terakhir}{a.suhu_terakhir.suhu} °C<span class="block text-xs text-redup">{waktu(a.suhu_terakhir.waktu)}</span>{:else}—{/if}
									</td>
									<td class="tabular-nums">{a.jumlah_kantong}</td>
									<td>
										<span class="flex flex-wrap gap-1">
											<Chip nada={a.aktif ? 'aman' : 'netral'}>{a.aktif ? 'Aktif' : 'Nonaktif'}</Chip>
											{#if a.penyimpangan_id}<Chip nada="bahaya">Penyimpangan suhu</Chip>{/if}
										</span>
									</td>
									<td><button class="btn btn--teks" onclick={() => ubah(a)} aria-label="Ubah {a.nama}">Ubah</button></td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{/snippet}
		</Keadaan>
	</section>

	<form class="panel flex flex-col gap-4" onsubmit={simpan} aria-labelledby="judul-form-alat">
		<h2 id="judul-form-alat">{id ? 'Ubah alat simpan' : 'Tambah alat simpan'}</h2>
		<div class="form-grid">
			<Isian label="Kode" id="alat-kode" galat={aksi.isian.kode}>
				<input id="alat-kode" class="input mono" bind:value={isi.kode} required />
			</Isian>
			<Isian label="Nomor seri (opsional)" id="alat-seri" galat={aksi.isian.nomor_seri}>
				<input id="alat-seri" class="input mono" bind:value={isi.nomor_seri} />
			</Isian>
		</div>
		<Isian label="Nama" id="alat-nama" galat={aksi.isian.nama}>
			<input id="alat-nama" class="input" bind:value={isi.nama} required />
		</Isian>
		<Isian label="Jenis" id="alat-jenis" galat={aksi.isian.jenis} bantuan="Menentukan komponen yang boleh disimpan di alat ini.">
			<select id="alat-jenis" class="input" bind:value={isi.jenis}>
				{#each Object.entries(JENIS) as [k, l] (k)}<option value={k}>{l}</option>{/each}
			</select>
		</Isian>
		<div class="form-grid">
			<Isian label="Suhu minimum (°C)" id="alat-min" galat={aksi.isian.suhu_min}>
				<input id="alat-min" class="input" type="number" step="0.1" bind:value={isi.suhu_min} required />
			</Isian>
			<Isian label="Suhu maksimum (°C)" id="alat-maks" galat={aksi.isian.suhu_maks}>
				<input id="alat-maks" class="input" type="number" step="0.1" bind:value={isi.suhu_maks} required />
			</Isian>
		</div>
		<Isian label="Lokasi (opsional)" id="alat-lokasi" galat={aksi.isian.lokasi}>
			<input id="alat-lokasi" class="input" bind:value={isi.lokasi} />
		</Isian>
		<label class="flex items-center gap-2 text-sm"><input type="checkbox" bind:checked={isi.aktif} />Aktif (alat tidak dihapus, cukup dinonaktifkan)</label>
		<PesanAksi {aksi} />
		<div class="flex justify-end gap-2.5 border-t border-garis pt-4">
			{#if id}<button type="button" class="btn btn--teks" onclick={batal}>Batal</button>{/if}
			<button class="btn btn--utama" disabled={aksi.sibuk}>{id ? 'Simpan' : 'Tambah'}</button>
		</div>
	</form>
</div>
