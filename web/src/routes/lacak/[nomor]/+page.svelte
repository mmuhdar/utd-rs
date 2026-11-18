<script lang="ts">
	import { merek } from '#lib/merek.svelte.ts';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { api } from '#lib/api.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import Pindai from '#lib/Pindai.svelte';
	import StatusKantong from '#lib/StatusKantong.svelte';
	import { NAMA_KOMPONEN, waktu } from '#lib/format.ts';
	import KepalaHalaman from '#lib/KepalaHalaman.svelte';
	import Golongan from '#lib/Golongan.svelte';
	import Hourglass from '@lucide/svelte/icons/hourglass';
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

	// Kelas ditulis utuh supaya terbaca pemindai Tailwind.
	const TITIK: Record<string, string> = { bahaya: 'bg-bahaya-titik', waspada: 'bg-waspada-titik', aman: 'bg-aman-titik' };
</script>

<svelte:head><title>Lacak {page.params.nomor} — {merek.judul}</title></svelte:head>

<KepalaHalaman judul="Lacak kantong" jalur={[['Lacak kantong', '/lacak'], [page.params.nomor ?? '']]}>
	{#snippet aksi()}
		<div class="w-full sm:w-80">
			<Pindai label="Nomor kantong lain" id="lacak" onpindai={(v) => goto(`/lacak/${encodeURIComponent(v)}`)} />
		</div>
	{/snippet}
</KepalaHalaman>

<Keadaan sumber={k}>
	{#snippet children(d)}
		{@const peristiwa = linimasa(d)}
		{@const lewat = new Date(d.kedaluwarsa).getTime() <= Date.now()}
		<div class="grid items-start gap-5 xl:grid-cols-[420px_1fr]">
			<section class="panel flex flex-col gap-5" aria-labelledby="judul-kantong">
				<p class="judul-kecil">Rincian kantong</p>
				<div class="flex items-center gap-4">
					<Golongan abo={d.abo} rhesus={d.rhesus} ukuran="besar" />
					<div class="min-w-0">
						<h2 id="judul-kantong" class="mono text-xl font-semibold break-all">{d.nomor}</h2>
						<p class="text-sm font-medium text-redup">
							{NAMA_KOMPONEN[d.komponen]}{d.volume_ml ? ` · ${d.volume_ml} mL` : ''} · {d.penerimaan ? d.penerimaan.utd : d.asal === 'donasi' ? 'Donasi sendiri' : '—'}
						</p>
						<p class="mt-1.5"><StatusKantong status={d.status} /></p>
					</div>
				</div>

				{#if lewat && ['tersedia', 'dialokasikan', 'karantina'].includes(d.status)}
					<div class="peringatan" role="alert">
						<Hourglass aria-hidden="true" />
						<div>
							<p class="font-bold">Kedaluwarsa {waktu(d.kedaluwarsa)}</p>
							<p class="text-sm">JANGAN DIKELUARKAN. Pisahkan kantong ini dan buat berita acara pemusnahan.</p>
						</div>
					</div>
				{/if}

				<dl class="grid grid-cols-[minmax(7rem,auto)_1fr] gap-x-4 gap-y-3 text-sm">
					<dt class="text-redup">Kedaluwarsa</dt>
					<dd class="font-medium"><Kedaluwarsa nilai={d.kedaluwarsa} /></dd>
					{#if d.kedaluwarsa_label !== d.kedaluwarsa}
						<dt class="text-redup">Kedaluwarsa label</dt>
						<dd class="font-medium">{waktu(d.kedaluwarsa_label)}</dd>
					{/if}
					<dt class="text-redup">Alat simpan</dt>
					<dd class="font-medium">{#if d.alat}<span class="mono">{d.alat.kode}</span> · {d.alat.nama}{:else}—{/if}</dd>
					<dt class="text-redup">Nomor donasi</dt>
					<dd class="mono font-semibold">{d.nomor_donasi}</dd>
					<dt class="text-redup">Uji saring IMLTD</dt>
					<dd class="font-medium {d.imltd_nonreaktif ? '' : 'text-bahaya'}">{d.imltd_nonreaktif ? 'Non-reaktif' : 'Belum non-reaktif / reaktif'}</dd>
					{#if d.penerimaan}
						<dt class="text-redup">Diterima</dt>
						<dd class="font-medium">
							{waktu(d.penerimaan.diterima_pada)} ·
							<a class="mono hover:underline" href="/stok/terima/{d.penerimaan.id}">{d.penerimaan.nomor}</a>
						</dd>
					{/if}
					{#if d.alasan_status}
						<dt class="text-redup">Keterangan status</dt>
						<dd>{d.alasan_status}</dd>
					{/if}
				</dl>
			</section>

			<section class="panel min-w-0" aria-labelledby="judul-linimasa">
				<div class="mb-4 flex flex-wrap items-baseline justify-between gap-2">
					<h2 id="judul-linimasa">Riwayat</h2>
					<p class="text-sm text-redup">{peristiwa.length} peristiwa · asal sampai akhir</p>
				</div>
				<ol class="flex flex-col">
					{#each peristiwa as p, i (i)}
						<li class="relative flex gap-3.5 pb-5 last:pb-0">
							{#if i < peristiwa.length - 1}<span class="absolute top-4 bottom-0 left-[5px] w-0.5 bg-garis" aria-hidden="true"></span>{/if}
							<span class="relative mt-1.5 size-3 shrink-0 rounded-full ring-4 ring-kartu {p.nada ? TITIK[p.nada] : 'bg-garis-kuat'}" aria-hidden="true"></span>
							<div class="min-w-0">
								<p class="font-semibold {p.nada === 'bahaya' ? 'text-bahaya' : p.nada === 'waspada' ? 'text-waspada' : ''}">
									{#if p.href}<a class="hover:underline" href={p.href}>{p.judul}</a>{:else}{p.judul}{/if}
								</p>
								<p class="text-xs font-medium text-redup">{p.waktu ? waktu(p.waktu) : 'Belum bertanggal'}</p>
								{#each p.rincian.filter(Boolean) as r, j (j)}<p class="mt-0.5 text-sm text-teks-2">{r}</p>{/each}
							</div>
						</li>
					{/each}
				</ol>
			</section>
		</div>
	{/snippet}
</Keadaan>
