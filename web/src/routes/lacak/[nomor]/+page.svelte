<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { api } from '#lib/api.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import Pindai from '#lib/Pindai.svelte';
	import StatusKantong from '#lib/StatusKantong.svelte';
	import { NAMA_KOMPONEN, golongan, waktu } from '#lib/format.ts';
	import Kedaluwarsa from '#lib/stok/Kedaluwarsa.svelte';

	const k = new Sumber(() => api<any>('GET', `/kantong/lacak/${encodeURIComponent(page.params.nomor ?? '')}`));
	$effect(() => {
		void page.params.nomor;
		k.muat();
	});

	type Peristiwa = { waktu: string | null; judul: string; rincian: string[]; href?: string; nada?: 'bahaya' | 'waspada' | 'aman' };
	const UJI: Record<string, string> = {
		serasi: 'Serasi',
		hanya_prc: 'Hanya PRC',
		tidak_serasi: 'Tidak serasi',
		golongan_beda: 'Golongan kantong berbeda dari label'
	};

	function linimasa(d: any): Peristiwa[] {
		const e: Peristiwa[] = [];
		const r = d.penerimaan;
		if (r) {
			e.push({
				waktu: r.diterima_pada,
				judul: `Diterima dari ${r.utd}`,
				rincian: [`Berita serah terima ${r.nomor}`, `Dokumen UTD ${r.nomor_dokumen_utd ?? '—'}`, `Petugas UTD ${r.petugas_utd}`],
				href: `/stok/terima/${r.id}`
			});
		} else if (d.asal === 'donasi') {
			e.push({ waktu: d.dibuat_pada, judul: 'Diolah dari donasi sendiri', rincian: [`Nomor donasi ${d.nomor_donasi}`] });
		}
		if (d.dicairkan_pada) e.push({ waktu: d.dicairkan_pada, judul: 'Dicairkan', rincian: [`Kedaluwarsa menjadi ${waktu(d.kedaluwarsa)}`] });
		for (const u of d.uji_silang) {
			e.push({
				waktu: u.dicatat_pada,
				judul: `Uji silang: ${UJI[u.kesimpulan] ?? u.kesimpulan}`,
				rincian: [
					`Permintaan ${u.permintaan}, no. RM ${u.no_rm}`,
					`Petugas ${u.petugas}; ${u.terverifikasi ? 'terverifikasi' : 'belum diverifikasi'}`,
					...(u.batal_alasan ? [`Dibatalkan: ${u.batal_alasan}`] : [])
				],
				href: `/permintaan/${u.permintaan_id}`,
				nada: u.kesimpulan === 'serasi' ? 'aman' : u.kesimpulan === 'hanya_prc' ? 'waspada' : 'bahaya'
			});
		}
		for (const a of d.alokasi) {
			e.push({
				waktu: a.dialokasikan_pada,
				judul: `Dialokasikan ke ${a.permintaan}${a.darurat ? ' (darurat)' : ''}`,
				rincian: [`No. RM ${a.no_rm}`, ...(a.alasan_darurat ? [`Alasan darurat: ${a.alasan_darurat}`] : [])],
				href: `/permintaan/${a.permintaan_id}`,
				nada: a.darurat ? 'waspada' : undefined
			});
			if (a.dilepas_pada) e.push({ waktu: a.dilepas_pada, judul: `Alokasi ${a.permintaan} dilepas`, rincian: [a.alasan_lepas ?? ''] });
		}
		const g = d.pengeluaran;
		if (g) e.push({ waktu: g.dikeluarkan_pada, judul: `Dikeluarkan ke ${g.penerima_ruang}`, rincian: [`Petugas BDRS ${g.petugas}`] });
		const t = d.transfusi;
		if (t) {
			e.push({
				waktu: t.mulai,
				judul: `Ditransfusikan ke ${t.pasien.nama}`,
				rincian: [
					`No. RM ${t.pasien.no_rm}`,
					t.selesai ? `${t.dihentikan ? 'Dihentikan' : 'Selesai'} ${waktu(t.selesai)}` : 'Transfusi berjalan',
					...(t.volume_masuk_ml ? [`Volume masuk ${t.volume_masuk_ml} mL`] : [])
				],
				nada: t.dihentikan ? 'waspada' : undefined
			});
		}
		for (const x of d.reaksi) {
			e.push({
				waktu: x.waktu_reaksi,
				judul: `Reaksi transfusi ${x.nomor} (${x.kategori})`,
				rincian: [`Gejala: ${x.gejala.join(', ')}`, `Status ${x.status}`, ...(x.kesimpulan ? [`Kesimpulan: ${x.kesimpulan}`] : [])],
				nada: 'bahaya'
			});
		}
		for (const p of d.penyimpangan_suhu) {
			e.push({
				waktu: p.dinilai_pada,
				judul: `Penyimpangan suhu ${p.alat}: ${p.keputusan === 'layak' ? 'dinilai layak' : p.keputusan === 'tidak_layak' ? 'dinilai tidak layak' : 'belum dinilai'}`,
				rincian: p.alasan ? [p.alasan] : [],
				href: `/suhu/penyimpangan/${p.penyimpangan_id}`,
				nada: p.keputusan === 'layak' ? undefined : 'bahaya'
			});
		}
		const b = d.berita_acara;
		if (b) {
			e.push({
				waktu: b.dibuat_pada,
				judul: `${b.jenis === 'pemusnahan' ? 'Dimusnahkan' : 'Dikembalikan ke UTD'} — ${b.nomor}`,
				rincian: [d.alasan_status ?? ''],
				href: `/berita-acara/${b.id}`
			});
		}
		// Penilaian yang belum bertanggal ditaruh di akhir.
		return e.sort((x, y) => (x.waktu ? new Date(x.waktu).getTime() : Infinity) - (y.waktu ? new Date(y.waktu).getTime() : Infinity));
	}

	const WARNA: Record<string, string> = { bahaya: 'bg-bahaya', waspada: 'bg-waspada', aman: 'bg-aman' };
</script>

<svelte:head><title>Lacak {page.params.nomor} — utd-rs</title></svelte:head>

<div class="mb-4 flex flex-wrap items-end justify-between gap-4">
	<h1>Lacak kantong</h1>
	<div class="w-full max-w-sm">
		<Pindai label="Nomor kantong lain" id="lacak" onpindai={(v) => goto(`/lacak/${encodeURIComponent(v)}`)} />
	</div>
</div>

<Keadaan sumber={k}>
	{#snippet children(d)}
		<section class="panel mb-4" aria-labelledby="judul-kantong">
			<div class="mb-3 flex flex-wrap items-center gap-3">
				<h2 id="judul-kantong" class="mono text-lg">{d.nomor}</h2>
				<StatusKantong status={d.status} />
			</div>
			<dl class="grid gap-x-6 gap-y-2 text-sm sm:grid-cols-2 lg:grid-cols-4">
				<div><dt class="text-redup">Komponen</dt><dd>{NAMA_KOMPONEN[d.komponen]}{d.volume_ml ? `, ${d.volume_ml} mL` : ''}</dd></div>
				<div><dt class="text-redup">Golongan (label)</dt><dd class="font-semibold">{golongan(d.abo, d.rhesus)}</dd></div>
				<div><dt class="text-redup">Kedaluwarsa</dt><dd><Kedaluwarsa nilai={d.kedaluwarsa} /></dd></div>
				<div><dt class="text-redup">Alat simpan</dt><dd class="mono">{d.alat ? `${d.alat.kode} — ${d.alat.nama}` : '—'}</dd></div>
				<div><dt class="text-redup">Nomor donasi</dt><dd class="mono">{d.nomor_donasi}</dd></div>
				<div><dt class="text-redup">IMLTD</dt><dd>{d.imltd_nonreaktif ? 'Non-reaktif' : 'Belum non-reaktif / reaktif'}</dd></div>
				{#if d.kedaluwarsa_label !== d.kedaluwarsa}
					<div><dt class="text-redup">Kedaluwarsa label</dt><dd>{waktu(d.kedaluwarsa_label)}</dd></div>
				{/if}
				{#if d.alasan_status}<div class="sm:col-span-2"><dt class="text-redup">Keterangan status</dt><dd>{d.alasan_status}</dd></div>{/if}
			</dl>
		</section>

		<section class="panel" aria-labelledby="judul-linimasa">
			<h2 id="judul-linimasa" class="mb-3">Linimasa</h2>
			<ol class="relative ml-2 border-l-2 border-garis">
				{#each linimasa(d) as p, i (i)}
					<li class="mb-4 ml-5">
						<span class="absolute -left-[7px] mt-1.5 size-3 rounded-full {p.nada ? WARNA[p.nada] : 'bg-utama'}" aria-hidden="true"></span>
						<p class="text-xs text-redup">{p.waktu ? waktu(p.waktu) : 'Belum bertanggal'}</p>
						<p class="font-medium {p.nada === 'bahaya' ? 'text-bahaya' : p.nada === 'waspada' ? 'text-waspada' : ''}">
							{#if p.href}<a class="hover:underline" href={p.href}>{p.judul}</a>{:else}{p.judul}{/if}
						</p>
						{#each p.rincian.filter(Boolean) as r, j (j)}<p class="text-sm text-redup">{r}</p>{/each}
					</li>
				{/each}
			</ol>
		</section>
	{/snippet}
</Keadaan>
