<!--
  Uji silang serasi: saran kantong kompatibel (FEFO), pindai kantong, konfirmasi golongan dari selang,
  mayor/minor/autokontrol sesuai aturan komponen, CCC untuk tabung; lalu daftar hasil dan verifikasi kedua.
-->
<script lang="ts">
	import { api } from '#lib/api.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Isian from '#lib/Isian.svelte';
	import Chip from '#lib/Chip.svelte';
	import Pindai from '#lib/Pindai.svelte';
	import { sesi } from '#lib/sesi.svelte.ts';
	import { KOMPONEN, golongan, relatif, waktu } from '#lib/format.ts';
	import type { Minta } from '#lib/pelayanan/TandaTangan.svelte';
	import Pilihan from '#lib/pelayanan/Pilihan.svelte';
	import { ABO, KESIMPULAN, METODE_UJI_SILANG, POS_NEG, RH, kesimpulan, kompatibel, tanda } from '#lib/pelayanan/darah.ts';
	import Verifikasi from './Verifikasi.svelte';

	let {
		p,
		aktif,
		aturan,
		muat,
		minta
	}: { p: any; aktif: boolean; aturan: Record<string, string>; muat: () => Promise<void>; minta: Minta } = $props();

	const gol = $derived(p.golongan_terverifikasi);
	const sampelLayak = $derived(p.sampel.filter((s: any) => s.layak));
	const bisaInput = $derived(aktif && sesi.boleh('uji.input'));

	// Komponen yang masih dibutuhkan dan memerlukan uji silang.
	const butuh = $derived(
		p.baris
			.map((b: any) => ({
				komponen: b.komponen as string,
				sisa: b.jumlah - p.alokasi.filter((a: any) => !a.dilepas_pada && a.kantong.komponen === b.komponen).length
			}))
			.filter((b: any) => b.sisa > 0 && aturan[b.komponen] && aturan[b.komponen] !== 'tidak_perlu')
	);
	const sudahDiuji = $derived(new Set(p.uji_silang.map((u: any) => u.kantong_id)));
	const saran = new Sumber(async () => {
		const hasil: Record<string, any[]> = {};
		for (const b of butuh) hasil[b.komponen] = await api<any[]>('GET', `/stok?komponen=${b.komponen}&status=tersedia`);
		return hasil;
	});
	$effect(() => {
		if (bisaInput && gol && sesi.boleh('stok.lihat') && butuh.length) saran.muat();
	});
	const cocok = (k: any) => !k.lewat_kedaluwarsa && !sudahDiuji.has(k.id) && kompatibel(k.komponen, gol.abo, gol.rhesus, k.abo, k.rhesus);

	const kosong = () => ({ nomor: '', abo_kantong: '', rhesus_kantong: '', metode: 'gel', mayor: '', minor: '', autokontrol: '', ccc_valid: false });
	let f = $state(kosong());
	let sampelId = $state('');
	$effect(() => {
		if (!sampelLayak.some((s: any) => s.id === sampelId)) sampelId = sampelLayak[0]?.id ?? '';
	});
	const kantong = $derived(f.nomor ? Object.values(saran.data ?? {}).flat().find((k) => k.nomor === f.nomor) : undefined);
	const atur = $derived(kantong ? aturan[kantong.komponen] : 'mayor_minor');
	const opsional: [string, string][] = [...POS_NEG, ['', 'Tidak diperiksa']];
	const beda = $derived(kantong && f.abo_kantong && f.rhesus_kantong && (f.abo_kantong !== kantong.abo || f.rhesus_kantong !== kantong.rhesus));
	const pratinjau = $derived(kantong && !beda && f.minor ? kesimpulan(kantong.komponen, f.mayor || undefined, f.minor, f.autokontrol || undefined) : null);

	const aksi = new Aksi();
	let pesan = $state<string | null>(null);

	async function kirim(e: SubmitEvent) {
		e.preventDefault();
		const r = await aksi.jalankan(
			() =>
				api('POST', '/uji-silang', {
					permintaan_id: p.id,
					sampel_id: sampelId,
					nomor_kantong: f.nomor,
					metode: f.metode,
					abo_kantong: f.abo_kantong,
					rhesus_kantong: f.rhesus_kantong,
					mayor: f.mayor || undefined,
					minor: f.minor || undefined,
					autokontrol: f.autokontrol || undefined,
					ccc_valid: f.metode === 'tabung' ? f.ccc_valid : undefined
				}),
			'Hasil uji silang tersimpan; menunggu verifikasi kedua.'
		);
		if (r) {
			f = kosong();
			await muat();
		}
	}
</script>

<section class="panel flex flex-col gap-4" aria-labelledby="j-uji">
	<h2 id="j-uji">Uji silang serasi <span class="text-sm font-normal text-redup">(crossmatch)</span></h2>

	{#if bisaInput}
		{#if !gol}
			<p class="catatan tanpa-cetak">Golongan darah pasien belum terverifikasi. Periksa dan verifikasi golongan pasien lebih dulu.</p>
		{:else if !sampelLayak.length}
			<p class="catatan tanpa-cetak">Tidak ada sampel yang layak pakai. Terima sampel baru lebih dulu.</p>
		{:else if !butuh.length}
			<p class="text-sm text-redup tanpa-cetak">Tidak ada komponen yang masih memerlukan uji silang untuk permintaan ini.</p>
		{:else}
			{#if sesi.boleh('stok.lihat')}
				<div class="tanpa-cetak flex flex-col gap-3">
					<h3 class="font-semibold">Saran kantong kompatibel untuk {golongan(gol.abo, gol.rhesus)} (kedaluwarsa terdekat lebih dulu)</h3>
					{#if saran.galat}
						<p class="text-sm text-bahaya">{saran.galat}</p>
					{:else if !saran.data}
						<p class="text-sm text-redup">Memuat stok…</p>
					{:else}
						{#each butuh as b (b.komponen)}
							{@const daftar = (saran.data[b.komponen] ?? []).filter(cocok).slice(0, 5)}
							<div>
								<p class="text-sm font-medium">{KOMPONEN[b.komponen]}: masih perlu {b.sisa} kantong</p>
								{#if daftar.length}
									<ul class="mt-1 flex flex-wrap gap-2">
										{#each daftar as k (k.id)}
											<li>
												<button
													type="button"
													class="btn {f.nomor === k.nomor ? 'btn--utama' : ''}"
													onclick={() => (f.nomor = k.nomor)}
													aria-label="Pilih kantong {k.nomor}, {golongan(k.abo, k.rhesus)}, kedaluwarsa {relatif(k.kedaluwarsa)}"
												>
													<span class="mono">{k.nomor}</span>
													<span>{golongan(k.abo, k.rhesus)}</span>
													<span class="text-xs {k.mendekati_kedaluwarsa ? 'text-waspada' : 'text-redup'}">{relatif(k.kedaluwarsa)}</span>
												</button>
											</li>
										{/each}
									</ul>
								{:else}
									<p class="text-sm text-bahaya">Tidak ada kantong {KOMPONEN[b.komponen]} kompatibel yang tersedia. Hubungi UTD mitra.</p>
								{/if}
							</div>
						{/each}
					{/if}
				</div>
			{/if}

			<form class="tanpa-cetak flex flex-col gap-4 border-t border-garis pt-4" onsubmit={kirim}>
				<h3 class="font-semibold">Catat uji silang</h3>
				<div class="form-grid">
					<Pindai label="Pindai nomor kantong" id="u-kantong" onpindai={(v) => (f.nomor = v)} />
					{#if sampelLayak.length > 1}
						<Isian label="Sampel" id="u-sampel" galat={aksi.isian.sampel_id}>
							<select id="u-sampel" class="input" bind:value={sampelId}>
								{#each sampelLayak as s (s.id)}<option value={s.id}>Diambil {waktu(s.diambil_pada)}</option>{/each}
							</select>
						</Isian>
					{/if}
				</div>
				{#if f.nomor}
					<p class="rounded-md bg-halus p-3" aria-live="polite">
						Kantong <span class="mono font-semibold">{f.nomor}</span>
						{#if kantong}
							— {KOMPONEN[kantong.komponen]}, label {golongan(kantong.abo, kantong.rhesus)}, kedaluwarsa {waktu(kantong.kedaluwarsa)}
						{:else}
							<span class="block text-sm text-redup">Tidak ada di daftar saran; server memeriksa status, kedaluwarsa, dan kompatibilitasnya.</span>
						{/if}
					</p>
					<div class="grid gap-4 sm:grid-cols-2">
						<Pilihan legend="Konfirmasi ABO kantong dari selang" name="u-abo" opsi={ABO.map((a) => [a, a] as [string, string])} bind:value={f.abo_kantong} galat={aksi.isian.abo_kantong} />
						<Pilihan legend="Konfirmasi Rh kantong dari selang" name="u-rh" opsi={RH} bind:value={f.rhesus_kantong} />
					</div>
					{#if beda}
						<p class="peringatan" role="alert">Golongan dari selang berbeda dengan label kantong. Bila disimpan, kantong dikarantina dan dikembalikan ke UTD.</p>
					{/if}
					<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
						<Pilihan legend="Mayor{atur === 'mayor_minor' ? '' : ' (opsional)'}" name="u-mayor" opsi={atur === 'mayor_minor' ? POS_NEG : opsional} bind:value={f.mayor} galat={aksi.isian.mayor} />
						<Pilihan legend="Minor" name="u-minor" opsi={POS_NEG} bind:value={f.minor} galat={aksi.isian.minor} />
						<Pilihan legend="Autokontrol{atur === 'mayor_minor' ? '' : ' (opsional)'}" name="u-auto" opsi={atur === 'mayor_minor' ? POS_NEG : opsional} bind:value={f.autokontrol} />
						<Isian label="Metode" id="u-metode" galat={aksi.isian.metode}>
							<select id="u-metode" class="input" bind:value={f.metode}>
								{#each Object.entries(METODE_UJI_SILANG) as [k, l] (k)}<option value={k}>{l}</option>{/each}
							</select>
						</Isian>
					</div>
					{#if f.metode === 'tabung'}
						<div>
							<label class="inline-flex min-h-10 items-center gap-2">
								<input type="checkbox" bind:checked={f.ccc_valid} />
								Coombs Control Cell (CCC) positif: hasil negatif tabung valid
							</label>
							{#if aksi.isian.ccc_valid}<p class="text-sm text-bahaya" role="alert">{aksi.isian.ccc_valid}</p>{/if}
						</div>
					{/if}
					{#if pratinjau}
						<p class="text-sm">Kesimpulan dihitung sistem: <Chip nada={KESIMPULAN[pratinjau][1]}>{KESIMPULAN[pratinjau][0]}</Chip></p>
					{/if}
				{/if}
				<PesanAksi {aksi} />
				<button class="btn btn--utama self-start" disabled={aksi.sibuk || !f.nomor}>Simpan hasil uji silang</button>
			</form>
		{/if}
	{/if}

	{#if pesan}<p class="catatan" role="status">{pesan}</p>{/if}
	{#if p.uji_silang.length}
		<div class="overflow-x-auto">
			<table class="tabel">
				<thead>
					<tr>
						<th scope="col">Kantong</th><th scope="col">Label / selang</th><th scope="col">Metode</th>
						<th scope="col">Mayor</th><th scope="col">Minor</th><th scope="col">Auto</th><th scope="col">CCC</th>
						<th scope="col">Kesimpulan</th><th scope="col">Petugas</th><th scope="col">Verifikasi kedua</th>
					</tr>
				</thead>
				<tbody>
					{#each p.uji_silang as u (u.id)}
						{@const k = KESIMPULAN[u.kesimpulan]}
						<tr>
							<td class="mono">{u.nomor_kantong}<span class="block font-sans text-xs text-redup">{KOMPONEN[u.komponen]}</span></td>
							<td>{golongan(u.abo_label, u.rhesus_label)} / {golongan(u.abo_kantong, u.rhesus_kantong)}</td>
							<td>{METODE_UJI_SILANG[u.metode] ?? u.metode}</td>
							<td class="mono">{tanda(u.mayor)}</td><td class="mono">{tanda(u.minor)}</td><td class="mono">{tanda(u.autokontrol)}</td>
							<td>{u.ccc_valid === null ? '—' : u.ccc_valid ? 'Valid' : 'Tidak'}</td>
							<td><Chip nada={k?.[1]}>{k?.[0] ?? u.kesimpulan}</Chip></td>
							<td>{u.petugas}<span class="block text-xs text-redup">{waktu(u.dicatat_pada)}</span></td>
							<td>
								{#if u.terverifikasi}
									<Chip nada="aman">Terverifikasi</Chip>
								{:else if sesi.boleh('uji.verifikasi')}
									<Verifikasi
										url="/uji-silang/{u.id}/verifikasi"
										penginput={u.dicatat_oleh}
										apa="uji silang kantong {u.nomor_kantong}"
										{minta}
										onselesai={(r) => {
											pesan = r.pesan ?? (r.alokasi_id ? `Kantong ${u.nomor_kantong} dialokasikan ke permintaan ini.` : null);
											muat();
										}}
									/>
								{:else}
									<Chip nada="waspada">Menunggu</Chip>
								{/if}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{:else}
		<p class="text-sm text-redup">Belum ada uji silang.</p>
	{/if}
</section>
