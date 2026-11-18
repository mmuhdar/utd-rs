<script lang="ts">
	import { api } from '#lib/api.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Isian from '#lib/Isian.svelte';
	import { waktu } from '#lib/format.ts';

	type Aturan = {
		komponen: string;
		nama: string;
		berisi_sel_merah: boolean;
		uji_silang: string;
		jenis_alat: string;
		masa_simpan_jam: number;
		jam_setelah_cair: number | null;
		jam_peringatan: number;
		diubah_pada: string;
	};
	const UJI: Record<string, string> = {
		mayor_minor: 'Mayor + minor + autokontrol',
		minor: 'Minor saja',
		tidak_perlu: 'Tidak perlu (cukup kecocokan ABO/Rh)'
	};
	const durasi = (jam: number | null) => (jam == null ? '—' : jam % 24 === 0 && jam >= 48 ? `${jam / 24} hari` : `${jam} jam`);

	const aturan = new Sumber(() => api<Aturan[]>('GET', '/master/aturan-komponen'));
	$effect(() => {
		aturan.muat();
	});
	let pilih = $state<Aturan | null>(null);
	let isi = $state({ uji_silang: '', masa_simpan_jam: 0, jam_setelah_cair: null as number | null, jam_peringatan: 0, alasan: '' });
	const aksi = new Aksi();

	function ubah(a: Aturan) {
		pilih = a;
		isi = { uji_silang: a.uji_silang, masa_simpan_jam: a.masa_simpan_jam, jam_setelah_cair: a.jam_setelah_cair, jam_peringatan: a.jam_peringatan, alasan: '' };
		aksi.galat = aksi.berhasil = null;
	}

	async function simpan(e: SubmitEvent) {
		e.preventDefault();
		if (!pilih) return;
		// Isian kosong pada input number menjadi null; jam_setelah_cair memang boleh kosong.
		const body = { ...isi, jam_setelah_cair: isi.jam_setelah_cair || null };
		const r = await aksi.jalankan(() => api('PUT', `/master/aturan-komponen/${pilih!.komponen}`, body), `Aturan ${pilih.nama} disimpan.`);
		if (r) {
			pilih = null;
			aturan.muat();
		}
	}
</script>

<p class="catatan mb-4">Aturan klinis hanya diubah penanggung jawab BDRS dengan alasan tertulis. Nilai lama tetap tercatat di jejak audit.</p>

<div class="grid gap-4 xl:grid-cols-[1fr_24rem]">
	<section class="panel min-w-0" aria-labelledby="judul-aturan">
		<h2 id="judul-aturan" class="mb-3">Aturan per komponen</h2>
		<Keadaan sumber={aturan}>
			{#snippet children(data)}
				<div class="overflow-x-auto">
					<table class="tabel">
						<thead>
							<tr><th>Komponen</th><th>Uji silang</th><th>Masa simpan</th><th>Setelah dicairkan</th><th>Peringatan kedaluwarsa</th><th><span class="sr-only">Aksi</span></th></tr>
						</thead>
						<tbody>
							{#each data as a (a.komponen)}
								<tr>
									<td>{a.nama}<span class="block text-xs text-redup">Diubah {waktu(a.diubah_pada)}</span></td>
									<td>{UJI[a.uji_silang] ?? a.uji_silang}</td>
									<td class="tabular-nums">{durasi(a.masa_simpan_jam)}</td>
									<td class="tabular-nums">{durasi(a.jam_setelah_cair)}</td>
									<td class="tabular-nums">{durasi(a.jam_peringatan)} sebelumnya</td>
									<td><button class="btn btn--teks" onclick={() => ubah(a)} aria-label="Ubah aturan {a.nama}">Ubah</button></td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{/snippet}
		</Keadaan>
	</section>

	{#if pilih}
		<form class="panel flex flex-col gap-4 self-start" onsubmit={simpan} aria-labelledby="judul-form-aturan">
			<h2 id="judul-form-aturan">Ubah aturan {pilih.nama}</h2>
			<Isian
				label="Uji silang"
				id="at-uji"
				galat={aksi.isian.uji_silang}
				bantuan={pilih.berisi_sel_merah ? 'Komponen berisi sel darah merah wajib mayor dan minor (Permenkes 91/2015 §5.7.3 B).' : undefined}
			>
				<select id="at-uji" class="input" bind:value={isi.uji_silang}>
					{#each Object.entries(UJI) as [k, l] (k)}
						<option value={k} disabled={pilih.berisi_sel_merah && k !== 'mayor_minor'}>{l}</option>
					{/each}
				</select>
			</Isian>
			<Isian label="Masa simpan (jam)" id="at-masa" galat={aksi.isian.masa_simpan_jam} bantuan="Untuk kantong hasil pengolahan sendiri; kantong dari UTD memakai tanggal di label.">
				<input id="at-masa" class="input" type="number" min="1" step="1" bind:value={isi.masa_simpan_jam} required />
			</Isian>
			<Isian label="Masa pakai setelah dicairkan (jam, opsional)" id="at-cair" galat={aksi.isian.jam_setelah_cair} bantuan="Kosongkan untuk komponen yang tidak dicairkan.">
				<input id="at-cair" class="input" type="number" min="1" step="1" bind:value={isi.jam_setelah_cair} />
			</Isian>
			<Isian label="Tandai mendekati kedaluwarsa (jam sebelumnya)" id="at-peringatan" galat={aksi.isian.jam_peringatan}>
				<input id="at-peringatan" class="input" type="number" min="1" step="1" bind:value={isi.jam_peringatan} required />
			</Isian>
			<Isian label="Alasan perubahan" id="at-alasan" galat={aksi.isian.alasan}>
				<textarea id="at-alasan" class="input" rows="2" bind:value={isi.alasan} required></textarea>
			</Isian>
			<PesanAksi {aksi} />
			<div class="flex gap-2">
				<button class="btn btn--utama" disabled={aksi.sibuk}>Simpan aturan</button>
				<button type="button" class="btn" onclick={() => (pilih = null)}>Batal</button>
			</div>
		</form>
	{/if}
</div>
