<!--
  Alokasi kantong: alokasi langsung (komponen tanpa uji silang) atau darurat dengan persetujuan dokter,
  daftar alokasi, pelepasan, dan pengeluaran ke petugas ruang.
-->
<script lang="ts">
	import { api } from '#lib/api.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Isian from '#lib/Isian.svelte';
	import Chip from '#lib/Chip.svelte';
	import Pindai from '#lib/Pindai.svelte';
	import { sesi } from '#lib/sesi.svelte.ts';
	import { KOMPONEN, golongan, relatif, waktu } from '#lib/format.ts';
	import type { Minta } from '#lib/pelayanan/TandaTangan.svelte';

	let {
		p,
		aktif,
		aturan,
		muat,
		minta
	}: { p: any; aktif: boolean; aturan: Record<string, string>; muat: () => Promise<void>; minta: Minta } = $props();

	const tanpaUji = $derived(p.baris.filter((b: any) => aturan[b.komponen] === 'tidak_perlu').map((b: any) => KOMPONEN[b.komponen]));
	const bisaAlokasi = $derived(aktif && sesi.boleh('uji.input') && (p.jenis === 'darurat' || tanpaUji.length > 0));

	// Alokasi langsung / darurat.
	let nomor = $state('');
	let darurat = $state(false);
	let alasanDarurat = $state('');
	const aksiAlokasi = new Aksi();

	function selesaiAlokasi() {
		nomor = '';
		darurat = false;
		return muat();
	}

	async function alokasikan(e: SubmitEvent) {
		e.preventDefault();
		if (!darurat) {
			if (await aksiAlokasi.jalankan(() => api('POST', `/permintaan/${p.id}/alokasi`, { nomor_kantong: nomor }), 'Kantong dialokasikan.'))
				await selesaiAlokasi();
			return;
		}
		minta({
			judul: 'Persetujuan dokter untuk darah darurat',
			makna: `Dokter penyetuju menyetujui pengeluaran kantong ${nomor} tanpa uji silang lengkap dengan alasan: "${alasanDarurat}".`,
			petugasLain: true,
			labelPetugas: 'Nama pengguna dokter penyetuju',
			kirim: async ({ nama_pengguna, sandi }) => {
				const r = await aksiAlokasi.jalankan(
					() =>
						api('POST', `/permintaan/${p.id}/alokasi`, {
							nomor_kantong: nomor,
							darurat: true,
							alasan_darurat: alasanDarurat,
							penyetuju: nama_pengguna,
							sandi_penyetuju: sandi
						}),
					'Kantong darurat dialokasikan dengan persetujuan dokter.'
				);
				if (r) await selesaiAlokasi();
			}
		});
	}

	// Aksi per baris alokasi: keluarkan atau lepas.
	let buka = $state<{ id: string; jenis: 'keluar' | 'lepas' | 'kembali'; pengeluaran?: string } | null>(null);
	let isi = $state('');
	const aksiBaris = new Aksi();

	async function kirimBaris(e: SubmitEvent) {
		e.preventDefault();
		if (!buka) return;
		const { id, jenis, pengeluaran } = buka;
		const r = await aksiBaris.jalankan(
			() =>
				jenis === 'keluar'
					? api('POST', `/alokasi/${id}/keluarkan`, { penerima_ruang: isi })
					: jenis === 'kembali'
						? api('POST', `/pengeluaran/${pengeluaran}/kembali`, { alasan: isi })
						: api('POST', `/alokasi/${id}/lepas`, { alasan: isi }),
			jenis === 'keluar'
				? 'Kantong dikeluarkan. Transfusi harus dimulai dalam 30 menit.'
				: jenis === 'kembali'
					? 'Kantong diterima kembali dan dikarantina; kembalikan ke UTD atau musnahkan lewat berita acara.'
					: 'Alokasi dilepas; kantong kembali tersedia.'
		);
		if (r) {
			buka = null;
			isi = '';
			await muat();
		}
	}

	function status(a: any): [string, string] {
		if (a.dilepas_pada) return ['Dilepas', 'netral'];
		if (a.transfusi) return ['Ditransfusikan', 'netral'];
		if (a.pengeluaran && a.kantong?.status !== 'dikeluarkan') return ['Kembali dari ruang', 'waspada'];
		if (a.pengeluaran) return ['Dikeluarkan', 'info'];
		return ['Menunggu dikeluarkan', 'aman'];
	}
</script>

<section class="panel flex flex-col gap-4" aria-labelledby="j-alokasi">
	<h2 id="j-alokasi">Alokasi dan pengeluaran</h2>

	{#if p.alokasi.length}
		<div class="overflow-x-auto">
			<table class="tabel">
				<thead>
					<tr>
						<th scope="col">Kantong</th><th scope="col">Golongan</th><th scope="col">Kedaluwarsa</th><th scope="col">Jalur</th>
						<th scope="col">Status</th><th scope="col" class="tanpa-cetak">Aksi</th>
					</tr>
				</thead>
				<tbody>
					{#each p.alokasi as a (a.id)}
						{@const [label, nada] = status(a)}
						{@const lewat = new Date(a.kantong.kedaluwarsa) <= new Date()}
						{@const menunggu = !a.dilepas_pada && !a.pengeluaran}
						<tr>
							<td class="mono">{a.kantong.nomor}<span class="block font-sans text-xs text-redup">{KOMPONEN[a.kantong.komponen]}</span></td>
							<td>{golongan(a.kantong.abo, a.kantong.rhesus)}</td>
							<td class={lewat ? 'font-semibold text-bahaya' : ''}>
								{waktu(a.kantong.kedaluwarsa)}<span class="block text-xs {lewat ? '' : 'text-redup'}">{lewat ? 'Kedaluwarsa' : relatif(a.kantong.kedaluwarsa)}</span>
							</td>
							<td>
								{#if a.darurat}
									<Chip nada="bahaya">Darurat</Chip><span class="block text-xs text-redup">{a.alasan_darurat}</span>
								{:else}
									{a.uji_silang_id ? 'Uji silang' : 'Tanpa uji silang (aturan komponen)'}
								{/if}
							</td>
							<td>
								<Chip {nada}>{label}</Chip>
								{#if a.dilepas_pada}
									<span class="block text-xs text-redup">{waktu(a.dilepas_pada)} · {a.alasan_lepas}</span>
								{:else if a.transfusi}
									<a class="block text-xs text-utama hover:underline" href="/transfusi/{a.transfusi.id}">Mulai {waktu(a.transfusi.mulai)}</a>
								{:else if a.pengeluaran}
									<span class="block text-xs text-redup">{waktu(a.pengeluaran.dikeluarkan_pada)} · diterima {a.pengeluaran.penerima_ruang}</span>
								{/if}
							</td>
							<td class="tanpa-cetak">
								{#if menunggu && aktif}
									<div class="flex flex-wrap gap-2">
										{#if sesi.boleh('penyerahan.kelola') && !lewat}
											<button type="button" class="btn btn--utama" onclick={() => ((buka = { id: a.id, jenis: 'keluar' }), (isi = ''))}>Keluarkan</button>
										{/if}
										{#if sesi.boleh('permintaan.proses')}
											<button type="button" class="btn" onclick={() => ((buka = { id: a.id, jenis: 'lepas' }), (isi = ''))}>Lepas</button>
										{/if}
									</div>
								{:else if a.pengeluaran && !a.transfusi && a.kantong?.status === 'dikeluarkan' && sesi.boleh('penyerahan.kelola')}
									<button
										type="button"
										class="btn"
										onclick={() => ((buka = { id: a.id, jenis: 'kembali', pengeluaran: a.pengeluaran.id }), (isi = ''))}>Terima kembali dari ruang</button
									>
								{/if}
							</td>
						</tr>
						{#if buka && buka.id === a.id}
							<tr class="tanpa-cetak">
								<td colspan="6">
									<form class="flex flex-wrap items-end gap-2" onsubmit={kirimBaris}>
										<div class="min-w-56 flex-1">
											<Isian
												label={buka.jenis === 'keluar'
													? 'Nama petugas ruang yang menerima (bukan keluarga pasien)'
													: buka.jenis === 'kembali'
														? 'Alasan kantong dikembalikan (mis. transfusi batal, pasien pulang)'
														: 'Alasan pelepasan'}
												id="baris-isi"
												galat={aksiBaris.isian.penerima_ruang ?? aksiBaris.isian.alasan}
											>
												<input id="baris-isi" class="input" bind:value={isi} required />
											</Isian>
										</div>
										<button class="btn btn--utama" disabled={aksiBaris.sibuk}>{buka.jenis === 'keluar' ? 'Keluarkan kantong' : buka.jenis === 'kembali' ? 'Terima kembali' : 'Lepas alokasi'}</button>
										<button type="button" class="btn" onclick={() => (buka = null)}>Batal</button>
									</form>
								</td>
							</tr>
						{/if}
					{/each}
				</tbody>
			</table>
		</div>
		<PesanAksi aksi={aksiBaris} />
	{:else}
		<p class="text-sm text-redup">Belum ada kantong yang dialokasikan.</p>
	{/if}

	{#if bisaAlokasi}
		<form class="tanpa-cetak flex flex-col gap-4 border-t border-garis pt-4" onsubmit={alokasikan}>
			<h3 class="font-semibold">Alokasi langsung</h3>
			<p class="text-sm text-redup">
				{#if tanpaUji.length}Komponen tanpa uji silang menurut aturan BDRS: {tanpaUji.join(', ')}; cukup golongan ABO/Rh yang kompatibel.{/if}
				{#if p.jenis === 'darurat'}Permintaan darurat: kantong dapat dikeluarkan sebelum uji silang dengan persetujuan dokter.{/if}
			</p>
			<Pindai label="Pindai nomor kantong" id="a-kantong" onpindai={(v) => (nomor = v)} />
			{#if nomor}<p class="rounded-md bg-halus p-3">Kantong <span class="mono font-semibold">{nomor}</span></p>{/if}
			{#if p.jenis === 'darurat'}
				<label class="inline-flex min-h-10 items-center gap-2 font-medium">
					<input type="checkbox" bind:checked={darurat} onchange={() => (alasanDarurat ||= p.alasan_darurat ?? '')} />
					Pengeluaran darurat tanpa uji silang (perlu persetujuan dokter)
				</label>
				{#if darurat}
					{#if !p.golongan_terverifikasi}
						<p class="catatan">Golongan pasien belum terverifikasi: hanya PRC/WB golongan O Rh− atau plasma golongan AB yang boleh dikeluarkan.</p>
					{/if}
					<Isian label="Alasan klinis pengeluaran darurat" id="a-alasan" galat={aksiAlokasi.isian.alasan_darurat}>
						<textarea id="a-alasan" class="input" rows="2" bind:value={alasanDarurat} required></textarea>
					</Isian>
				{/if}
			{/if}
			<PesanAksi aksi={aksiAlokasi} />
			<button class="btn {darurat ? 'btn--bahaya' : 'btn--utama'} self-start" disabled={aksiAlokasi.sibuk || !nomor}>
				{darurat ? 'Minta persetujuan dan alokasikan' : 'Alokasikan'}
			</button>
		</form>
	{/if}
</section>
