<!-- Formulir laporan BDRS (Permenkes 83/2014 Pasal 51; Permenkes 91/2015 §7.2.2), rapi saat dicetak di A4. -->
<script lang="ts">
	import { KOMPONEN, golongan, tanggal, waktu } from '#lib/format.ts';

	type Gol = { komponen: string; abo: string; rhesus: string };
	type F3 = {
		bulan: string;
		profil: { nama_rs: string; kode_faskes: string | null };
		a_permintaan_ke_utd: { nomor: string; tanggal: string; utd: string; jenis: string; status: string; baris: (Gol & { jumlah: number })[] }[];
		b_penerimaan_dari_utd: (Gol & { diterima: number; ditolak: number })[];
		c_pemakaian_per_ruang: { ruang: string; permintaan: number; kantong_diminta: number; ditransfusikan: Record<string, number> }[];
		d_reaksi_transfusi: {
			waktu: string;
			nama: string;
			usia: number;
			jenis_kelamin: string;
			diagnosis: string | null;
			riwayat_transfusi: boolean | null;
			komponen: string;
			nomor_kantong: string;
			gejala: string[] | null;
			kategori: string | null;
			status: string;
			kesimpulan: string | null;
		}[];
		e_kembali_ke_utd: (Gol & { berita_acara: string; tanggal: string; utd: string; nomor_kantong: string; alasan: string | null })[];
		pemusnahan: { berita_acara: string; tanggal: string; nomor_kantong: string; komponen: string; alasan: string | null }[];
		persediaan_saat_ini: (Gol & { jumlah: number })[];
	};

	let { d }: { d: F3 } = $props();

	const namaBulan = (b: string) => new Intl.DateTimeFormat('id-ID', { month: 'long', year: 'numeric' }).format(new Date(`${b}-01T00:00:00`));
	const GOL = ['O pos', 'O neg', 'A pos', 'A neg', 'B pos', 'B neg', 'AB pos', 'AB neg'];
	const stok = (k: string, g: string) => d.persediaan_saat_ini.find((r) => r.komponen === k && `${r.abo} ${r.rhesus}` === g)?.jumlah ?? 0;
	const KOMP = Object.keys(KOMPONEN);
</script>

{#snippet kosong(n: number, teks: string)}
	<tr><td colspan={n} class="text-redup">{teks}</td></tr>
{/snippet}

<article class="formulir flex flex-col gap-5">
	<header class="border-b-2 border-teks pb-3 text-center">
		<p class="text-lg font-semibold uppercase">{d.profil.nama_rs}</p>
		<p class="text-sm">Kode faskes: <span class="mono">{d.profil.kode_faskes ?? '—'}</span></p>
		<h2 class="mt-2 text-base">Laporan Kegiatan Bank Darah Rumah Sakit</h2>
		<p class="text-sm">Periode {namaBulan(d.bulan)}</p>
	</header>

	<section aria-labelledby="f3-a">
		<h3 id="f3-a" class="mb-2 font-semibold">A. Permintaan darah ke UTD</h3>
		<div class="overflow-x-auto">
			<table class="tabel">
				<thead><tr><th scope="col">Nomor</th><th scope="col">Tanggal</th><th scope="col">UTD</th><th scope="col">Jenis</th><th scope="col">Rincian</th><th scope="col">Status</th></tr></thead>
				<tbody>
					{#each d.a_permintaan_ke_utd as r (r.nomor)}
						<tr>
							<td class="mono">{r.nomor}</td>
							<td>{tanggal(r.tanggal)}</td>
							<td>{r.utd}</td>
							<td class="capitalize">{r.jenis}</td>
							<td>{r.baris.map((b) => `${KOMPONEN[b.komponen] ?? b.komponen} ${golongan(b.abo, b.rhesus)} × ${b.jumlah}`).join('; ')}</td>
							<td class="capitalize">{r.status}</td>
						</tr>
					{:else}
						{@render kosong(6, 'Tidak ada permintaan ke UTD pada periode ini.')}
					{/each}
				</tbody>
			</table>
		</div>
	</section>

	<section aria-labelledby="f3-b">
		<h3 id="f3-b" class="mb-2 font-semibold">B. Penerimaan darah dari UTD</h3>
		<div class="overflow-x-auto">
			<table class="tabel">
				<thead><tr><th scope="col">Komponen</th><th scope="col">Golongan</th><th scope="col" class="text-right">Diterima</th><th scope="col" class="text-right">Ditolak</th></tr></thead>
				<tbody>
					{#each d.b_penerimaan_dari_utd as r (`${r.komponen}${r.abo}${r.rhesus}`)}
						<tr>
							<td>{KOMPONEN[r.komponen] ?? r.komponen}</td>
							<td>{golongan(r.abo, r.rhesus)}</td>
							<td class="text-right tabular-nums">{r.diterima}</td>
							<td class="text-right tabular-nums">{r.ditolak}</td>
						</tr>
					{:else}
						{@render kosong(4, 'Tidak ada penerimaan pada periode ini.')}
					{/each}
				</tbody>
				{#if d.b_penerimaan_dari_utd.length}
					<tfoot>
						<tr class="font-semibold">
							<td colspan="2" class="px-3 py-2">Jumlah</td>
							<td class="px-3 py-2 text-right tabular-nums">{d.b_penerimaan_dari_utd.reduce((s, r) => s + r.diterima, 0)}</td>
							<td class="px-3 py-2 text-right tabular-nums">{d.b_penerimaan_dari_utd.reduce((s, r) => s + r.ditolak, 0)}</td>
						</tr>
					</tfoot>
				{/if}
			</table>
		</div>
	</section>

	<section aria-labelledby="f3-c">
		<h3 id="f3-c" class="mb-2 font-semibold">C. Pemakaian darah per ruang</h3>
		<div class="overflow-x-auto">
			<table class="tabel">
				<thead>
					<tr>
						<th scope="col">Ruang</th>
						<th scope="col" class="text-right">Permintaan</th>
						<th scope="col" class="text-right">Kantong diminta</th>
						{#each KOMP as k (k)}<th scope="col" class="text-right">{KOMPONEN[k]}</th>{/each}
						<th scope="col" class="text-right">Ditransfusikan</th>
					</tr>
				</thead>
				<tbody>
					{#each d.c_pemakaian_per_ruang as r (r.ruang)}
						<tr>
							<td>{r.ruang}</td>
							<td class="text-right tabular-nums">{r.permintaan}</td>
							<td class="text-right tabular-nums">{r.kantong_diminta}</td>
							{#each KOMP as k (k)}<td class="text-right tabular-nums">{r.ditransfusikan[k] ?? 0}</td>{/each}
							<td class="text-right font-semibold tabular-nums">{Object.values(r.ditransfusikan).reduce((s, n) => s + n, 0)}</td>
						</tr>
					{:else}
						{@render kosong(KOMP.length + 4, 'Tidak ada permintaan darah pada periode ini.')}
					{/each}
				</tbody>
			</table>
		</div>
	</section>

	<section aria-labelledby="f3-d">
		<h3 id="f3-d" class="mb-2 font-semibold">D. Reaksi transfusi</h3>
		<div class="overflow-x-auto">
			<table class="tabel">
				<thead>
					<tr>
						<th scope="col">Waktu</th>
						<th scope="col">Pasien</th>
						<th scope="col">Diagnosis</th>
						<th scope="col">Kantong</th>
						<th scope="col">Gejala</th>
						<th scope="col">Kategori</th>
						<th scope="col">Kesimpulan</th>
					</tr>
				</thead>
				<tbody>
					{#each d.d_reaksi_transfusi as r, i (i)}
						<tr>
							<td class="whitespace-nowrap">{waktu(r.waktu)}</td>
							<td>{r.nama}<span class="block text-xs text-redup">{r.usia} th · {r.jenis_kelamin}{r.riwayat_transfusi ? ' · pernah ditransfusi' : ''}</span></td>
							<td>{r.diagnosis ?? '—'}</td>
							<td><span class="mono">{r.nomor_kantong}</span><span class="block text-xs text-redup">{KOMPONEN[r.komponen] ?? r.komponen}</span></td>
							<td>{r.gejala?.join(', ') ?? '—'}</td>
							<td class="capitalize">{r.kategori ?? '—'}</td>
							<td>{r.kesimpulan ?? `Belum ditutup (${r.status})`}</td>
						</tr>
					{:else}
						{@render kosong(7, 'Tidak ada reaksi transfusi pada periode ini.')}
					{/each}
				</tbody>
			</table>
		</div>
	</section>

	<section aria-labelledby="f3-e">
		<h3 id="f3-e" class="mb-2 font-semibold">E. Darah dikembalikan ke UTD</h3>
		<div class="overflow-x-auto">
			<table class="tabel">
				<thead>
					<tr><th scope="col">Berita acara</th><th scope="col">Tanggal</th><th scope="col">UTD</th><th scope="col">Kantong</th><th scope="col">Komponen</th><th scope="col">Alasan</th></tr>
				</thead>
				<tbody>
					{#each d.e_kembali_ke_utd as r (r.nomor_kantong)}
						<tr>
							<td class="mono">{r.berita_acara}</td>
							<td>{tanggal(r.tanggal)}</td>
							<td>{r.utd}</td>
							<td class="mono">{r.nomor_kantong}</td>
							<td>{KOMPONEN[r.komponen] ?? r.komponen} {golongan(r.abo, r.rhesus)}</td>
							<td>{r.alasan ?? '—'}</td>
						</tr>
					{:else}
						{@render kosong(6, 'Tidak ada pengembalian pada periode ini.')}
					{/each}
				</tbody>
			</table>
		</div>
	</section>

	<section aria-labelledby="f3-musnah">
		<h3 id="f3-musnah" class="mb-2 font-semibold">Pemusnahan</h3>
		<div class="overflow-x-auto">
			<table class="tabel">
				<thead><tr><th scope="col">Berita acara</th><th scope="col">Tanggal</th><th scope="col">Kantong</th><th scope="col">Komponen</th><th scope="col">Alasan</th></tr></thead>
				<tbody>
					{#each d.pemusnahan as r (r.nomor_kantong)}
						<tr>
							<td class="mono">{r.berita_acara}</td>
							<td>{tanggal(r.tanggal)}</td>
							<td class="mono">{r.nomor_kantong}</td>
							<td>{KOMPONEN[r.komponen] ?? r.komponen}</td>
							<td>{r.alasan ?? '—'}</td>
						</tr>
					{:else}
						{@render kosong(5, 'Tidak ada pemusnahan pada periode ini.')}
					{/each}
				</tbody>
			</table>
		</div>
	</section>

	<section aria-labelledby="f3-stok">
		<h3 id="f3-stok" class="mb-2 font-semibold">Persediaan saat laporan dibuat</h3>
		<p class="mb-2 text-xs text-redup">Kantong tersedia dan dialokasikan yang belum kedaluwarsa, per {waktu(new Date().toISOString())}.</p>
		<div class="overflow-x-auto">
			<table class="tabel">
				<thead>
					<tr>
						<th scope="col">Komponen</th>
						{#each GOL as g (g)}<th scope="col" class="text-center">{golongan(...(g.split(' ') as [string, string]))}</th>{/each}
						<th scope="col" class="text-right">Jumlah</th>
					</tr>
				</thead>
				<tbody>
					{#each KOMP as k (k)}
						<tr>
							<th scope="row" class="text-left text-sm font-medium normal-case">{KOMPONEN[k]}</th>
							{#each GOL as g (g)}<td class="text-center tabular-nums">{stok(k, g)}</td>{/each}
							<td class="text-right font-semibold tabular-nums">{GOL.reduce((s, g) => s + stok(k, g), 0)}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	</section>

	<footer class="mt-6 hidden grid-cols-2 gap-8 text-center text-sm print:grid">
		<div><p>Petugas BDRS</p><p class="mt-16 border-t border-teks pt-1">Nama dan NIP</p></div>
		<div><p>Penanggung jawab BDRS</p><p class="mt-16 border-t border-teks pt-1">Nama dan NIP</p></div>
	</footer>
</article>

<style>
	@media print {
		@page {
			size: A4;
			margin: 15mm;
		}
		.formulir {
			font-size: 10pt;
		}
		.formulir :global(table) {
			break-inside: auto;
		}
		.formulir :global(tr) {
			break-inside: avoid;
		}
		.formulir h3 {
			break-after: avoid;
		}
		.formulir :global(th),
		.formulir :global(td) {
			border: 1px solid currentColor;
			padding: 2px 4px;
		}
	}
</style>
