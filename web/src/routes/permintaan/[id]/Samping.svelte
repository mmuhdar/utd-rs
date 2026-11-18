<!-- Kolom kanan detail permintaan (desain 04): sampel ringkas, daftar verifikasi kedua, dan rincian permintaan. -->
<script lang="ts">
	import Chip from '#lib/Chip.svelte';
	import { NAMA_KOMPONEN, golongan, relatif, tanggal, waktu, inisial } from '#lib/format.ts';
	import { JENIS_SAMPEL, PENYIMPANAN } from '#lib/pelayanan/darah.ts';
	let { p }: { p: any } = $props();

	const layak = $derived(p.sampel.find((s: any) => s.layak));
	// Setiap hasil manual menunggu verifikasi petugas kedua sebelum berlaku (CLAUDE.md, pratransfusi::verifikasi).
	const verifikasi = $derived([
		...p.sampel
			.filter((s: any) => s.golongan)
			.map((s: any) => ({
				apa: `Golongan pasien · ${golongan(s.golongan.abo, s.golongan.rhesus)}`,
				petugas: s.golongan.petugas,
				waktu: s.golongan.dicatat_pada,
				ok: s.golongan.terverifikasi
			})),
		...p.uji_silang.map((u: any) => ({ apa: `Uji silang ${u.nomor_kantong}`, petugas: u.petugas, waktu: u.dicatat_pada, ok: u.terverifikasi }))
	]);
</script>

<aside class="flex flex-col gap-5">
	<section class="panel flex flex-col gap-3" aria-labelledby="s-sampel">
		<h2 id="s-sampel">Sampel</h2>
		{#if layak}
			{@const lewat = new Date(layak.batas_pakai) <= new Date()}
			<dl class="grid grid-cols-[8.5rem_1fr] gap-x-3 gap-y-1.5 text-sm">
				<dt class="text-redup">Jenis</dt><dd class="font-semibold">{JENIS_SAMPEL[layak.jenis] ?? layak.jenis} · {PENYIMPANAN[layak.penyimpanan] ?? layak.penyimpanan}</dd>
				<dt class="text-redup">Diambil</dt><dd class="font-semibold">{waktu(layak.diambil_pada)} · {layak.diambil_oleh}</dd>
				<dt class="text-redup">Diterima</dt><dd class="font-semibold">{waktu(layak.diterima_pada)}</dd>
				<dt class="text-redup">Berlaku sampai</dt>
				<dd class="font-semibold {lewat ? 'text-bahaya' : ''}">{waktu(layak.batas_pakai)}<span class="block text-xs font-medium text-redup">{relatif(layak.batas_pakai)}</span></dd>
			</dl>
		{:else}
			<p class="text-sm text-redup">Belum ada sampel layak pakai.</p>
		{/if}
		{#if p.riwayat_transfusi}
			<p class="rounded-xl bg-info-latar px-3 py-2 text-sm font-medium text-info">Pasien pernah ditransfusi: sampel uji silang harus baru (batas usia lebih pendek).</p>
		{/if}
	</section>

	<section class="panel flex flex-col gap-3" aria-labelledby="s-verif">
		<div>
			<h2 id="s-verif">Verifikasi kedua</h2>
			<p class="text-sm text-redup">Hasil manual baru berlaku setelah diperiksa petugas lain.</p>
		</div>
		{#if verifikasi.length}
			<ul class="flex flex-col gap-3">
				{#each verifikasi as v, i (i)}
					<li class="flex items-center gap-2.5">
						<span class="grid size-8 shrink-0 place-items-center rounded-full bg-halus text-xs font-bold text-teks-2" aria-hidden="true">{inisial(v.petugas)}</span>
						<span class="min-w-0 flex-1">
							<span class="block truncate text-sm font-semibold">{v.apa}</span>
							<span class="block truncate text-xs text-redup">Input {v.petugas} · {waktu(v.waktu)}</span>
						</span>
						<Chip nada={v.ok ? 'aman' : 'waspada'}>{v.ok ? 'Terverifikasi' : 'Menunggu'}</Chip>
					</li>
				{/each}
			</ul>
		{:else}
			<p class="text-sm text-redup">Belum ada hasil yang perlu diverifikasi.</p>
		{/if}
	</section>

	<section class="panel flex flex-col gap-3" aria-labelledby="s-rincian">
		<h2 id="s-rincian">Rincian permintaan</h2>
		<dl class="grid grid-cols-[8.5rem_1fr] gap-x-3 gap-y-1.5 text-sm">
			<dt class="text-redup">Indikasi</dt><dd>{p.indikasi}</dd>
			<dt class="text-redup">Komponen</dt>
			<dd>
				<ul>
					{#each p.baris as b, i (i)}
						{@const n = p.alokasi.filter((a: any) => !a.dilepas_pada && a.kantong.komponen === b.komponen).length}
						<li>
							{b.jumlah} {NAMA_KOMPONEN[b.komponen] ?? b.komponen}{b.volume_ml ? ` (${b.volume_ml} mL)` : ''}
							<span class="text-redup">· {n} dialokasikan</span>
						</li>
					{/each}
				</ul>
			</dd>
			<dt class="text-redup">Golongan klinis</dt><dd>{golongan(p.abo_klinis, p.rhesus_klinis)}</dd>
			<dt class="text-redup">Trombosit</dt><dd>{p.trombosit?.toLocaleString('id-ID') ?? '—'} /µL</dd>
			<dt class="text-redup">Riwayat transfusi</dt><dd>{p.riwayat_transfusi ? 'Ya' : 'Tidak'}</dd>
			<dt class="text-redup">Riwayat reaksi</dt><dd class={p.riwayat_reaksi ? 'font-semibold text-bahaya' : ''}>{p.riwayat_reaksi ? 'Ya' : 'Tidak'}</dd>
			<dt class="text-redup">Diminta</dt><dd>{waktu(p.diminta_pada)}</dd>
			<dt class="text-redup">Rencana</dt><dd>{tanggal(p.rencana_transfusi)}</dd>
			{#if p.rujukan_simrs}<dt class="text-redup">Rujukan SIMRS</dt><dd class="mono">{p.rujukan_simrs}</dd>{/if}
		</dl>
	</section>
</aside>
