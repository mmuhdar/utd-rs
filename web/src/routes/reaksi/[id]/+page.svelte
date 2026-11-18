<!-- Detail reaksi transfusi untuk penelusuran BDRS: data lengkap, form telusur, penutupan bertanda tangan. Dapat dicetak. -->
<script lang="ts">
	import Kop from '#lib/Kop.svelte';
	import { page } from '$app/state';
	import { api } from '#lib/api.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Isian from '#lib/Isian.svelte';
	import Chip from '#lib/Chip.svelte';
	import { sesi } from '#lib/sesi.svelte.ts';
	import { KOMPONEN, golongan, tanggal, waktu } from '#lib/format.ts';
	import TandaTangan from '#lib/pelayanan/TandaTangan.svelte';
	import Pilihan from '#lib/pelayanan/Pilihan.svelte';
	import { ABO, KESIMPULAN, METODE_UJI_SILANG, RH, STATUS_REAKSI, TAHAP_OBSERVASI, UJI_SILANG_ULANG, angka, tanda } from '#lib/pelayanan/darah.ts';
	import Printer from '@lucide/svelte/icons/printer';

	const id = $derived(page.params.id);
	const detail = new Sumber(() => api<any>('GET', `/reaksi/${id}`));
	$effect(() => {
		void id;
		detail.muat();
	});
	let ttd: TandaTangan | undefined = $state();

	// Form telusur, diisi dari hasil penelusuran sebelumnya bila ada.
	const f = $state({ sisa_volume_ml: '' as string | number, abo_pasien_ulang: '', rhesus_pasien_ulang: '', abo_kantong_ulang: '', rhesus_kantong_ulang: '', uji_silang_ulang: '', kesimpulan: '' });
	let terisi = $state<string | null>(null);
	$effect(() => {
		const r = detail.data;
		if (!r || terisi === r.id) return;
		terisi = r.id;
		for (const k of Object.keys(f) as (keyof typeof f)[]) f[k] = r[k] ?? '';
	});
	const aksi = new Aksi();
	async function telusur(e: SubmitEvent) {
		e.preventDefault();
		const r = await aksi.jalankan(
			() =>
				api('POST', `/reaksi/${id}/telusur`, {
					...Object.fromEntries(Object.entries(f).map(([k, v]) => [k, v === '' ? undefined : v])),
					sisa_volume_ml: angka(f.sisa_volume_ml),
					uji_silang_ulang: f.uji_silang_ulang
				}),
			'Hasil penelusuran tersimpan.'
		);
		if (r) await detail.muat();
	}
	const aksiTutup = new Aksi();
	function tutup() {
		ttd?.minta({
			judul: 'Tutup penelusuran reaksi',
			makna: 'Sebagai penanggung jawab BDRS, Anda memvalidasi hasil penelusuran dan menutup laporan ini. Laporan yang ditutup tidak dapat diubah.',
			kirim: async ({ sandi }) => {
				if (await aksiTutup.jalankan(() => api('POST', `/reaksi/${id}/tutup`, { sandi }), 'Penelusuran ditutup.')) await detail.muat();
			}
		});
	}
	const opsiAbo = [['', 'Tidak diperiksa'], ...ABO.map((a) => [a, a])] as [string, string][];
	const opsiRh = [['', 'Tidak diperiksa'], ...RH] as [string, string][];
</script>

<svelte:head><title>{detail.data?.nomor ?? 'Reaksi transfusi'} — utd-rs</title></svelte:head>

<TandaTangan bind:this={ttd} />

<Keadaan sumber={detail} apakahKosong={() => false} baris={10}>
	{#snippet children(r)}
		{@const t = r.transfusi}
		{@const k = t.kantong}
		{@const s = STATUS_REAKSI[r.status]}
		<div class="mb-3"><Kop cetakSaja /></div>
		<p class="mb-2 hidden text-sm print:block">Laporan dan penelusuran reaksi transfusi · dicetak {waktu(new Date().toISOString())}</p>
		<p class="tanpa-cetak text-sm"><a class="text-utama hover:underline" href="/reaksi">Reaksi & insiden</a></p>
		<div class="mb-4 flex flex-wrap items-start justify-between gap-3">
			<div>
				<h1 class="mono">{r.nomor}</h1>
				<div class="mt-1 flex flex-wrap gap-2">
					<Chip nada={s?.[1]}>{s?.[0] ?? r.status}</Chip>
					<Chip nada={r.kategori === 'akut' ? 'bahaya' : 'waspada'}>{r.kategori === 'akut' ? 'Akut (≤ 24 jam)' : 'Lambat (> 24 jam)'}</Chip>
				</div>
			</div>
			<div class="tanpa-cetak flex flex-wrap gap-2">
				<button type="button" class="btn" onclick={() => window.print()}><Printer class="size-4" aria-hidden="true" />Cetak</button>
				{#if sesi.boleh('reaksi.tutup') && r.status === 'ditelusuri'}
					<button type="button" class="btn btn--utama" onclick={tutup} disabled={aksiTutup.sibuk}>Tutup penelusuran</button>
				{/if}
			</div>
		</div>
		<div class="mb-4"><PesanAksi aksi={aksiTutup} /></div>

		<div class="flex flex-col gap-4">
			<div class="grid gap-4 lg:grid-cols-2">
				<section class="panel" aria-labelledby="j-lapor">
					<h2 id="j-lapor" class="mb-3">Laporan dari ruang</h2>
					<dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-sm">
						<dt class="text-redup">Waktu reaksi</dt><dd>{waktu(r.waktu_reaksi)}</dd>
						<dt class="text-redup">Gejala</dt><dd class="font-semibold">{r.gejala.join(', ')}</dd>
						<dt class="text-redup">Tindakan</dt><dd>{r.tindakan}</dd>
						<dt class="text-redup">Pelapor</dt><dd>{r.pelapor} · {waktu(r.dilaporkan_pada)}</dd>
					</dl>
				</section>
				<section class="panel" aria-labelledby="j-pasien">
					<h2 id="j-pasien" class="mb-3">Pasien dan transfusi</h2>
					<dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-sm">
						<dt class="text-redup">Pasien</dt>
						<dd>{t.pasien.nama} · <a class="mono text-utama hover:underline" href="/pasien/{encodeURIComponent(t.no_rm)}">{t.no_rm}</a> · lahir {tanggal(t.pasien.tanggal_lahir)}</dd>
						<dt class="text-redup">Permintaan</dt>
						<dd><a class="mono text-utama hover:underline" href="/permintaan/{t.permintaan.id}">{t.permintaan.nomor}</a> · {t.permintaan.ruang} · {t.permintaan.dokter}</dd>
						<dt class="text-redup">Indikasi</dt><dd>{t.permintaan.indikasi}</dd>
						<dt class="text-redup">Transfusi</dt>
						<dd>
							<a class="text-utama hover:underline" href="/transfusi/{t.id}">mulai {waktu(t.mulai)}</a>
							{#if t.selesai}· {t.dihentikan ? 'dihentikan' : 'selesai'} {waktu(t.selesai)} · {t.volume_masuk_ml ?? '—'} mL masuk{/if}
						</dd>
						<dt class="text-redup">Petugas</dt><dd>{t.petugas_1} dan {t.petugas_2}</dd>
					</dl>
				</section>
			</div>

			<section class="panel" aria-labelledby="j-kantong">
				<h2 id="j-kantong" class="mb-3">Kantong</h2>
				<dl class="grid gap-x-6 gap-y-2 text-sm sm:grid-cols-2 lg:grid-cols-4">
					<div><dt class="text-redup">Nomor kantong</dt><dd class="mono font-semibold">{k.nomor}</dd></div>
					<div><dt class="text-redup">Komponen, golongan</dt><dd>{KOMPONEN[k.komponen]} {golongan(k.abo, k.rhesus)} · {k.volume_ml ?? '—'} mL</dd></div>
					<div><dt class="text-redup">Asal</dt><dd>{k.asal === 'utd_mitra' ? 'UTD mitra' : 'Donasi sendiri'}{k.nomor_donasi ? ` · donasi ${k.nomor_donasi}` : ''}</dd></div>
					<div><dt class="text-redup">Kedaluwarsa</dt><dd>{waktu(k.kedaluwarsa)}</dd></div>
				</dl>
			</section>

			<section class="panel" aria-labelledby="j-obs">
				<h2 id="j-obs" class="mb-3">Observasi</h2>
				{#if t.observasi.length}
					<div class="overflow-x-auto">
						<table class="tabel">
							<thead><tr><th scope="col">Tahap</th><th scope="col">Waktu</th><th scope="col">Suhu</th><th scope="col">Nadi</th><th scope="col">TD</th><th scope="col">Napas</th><th scope="col">Keluhan</th></tr></thead>
							<tbody>
								{#each t.observasi as b (b.id)}
									<tr>
										<td>{TAHAP_OBSERVASI[b.tahap] ?? b.tahap}{#if b.dikoreksi_pada}<span class="mt-1 block"><Chip nada="waspada">Dikoreksi</Chip></span>{/if}</td>
										<td>{waktu(b.waktu)}</td><td>{b.suhu ?? '—'}</td><td>{b.nadi ?? '—'}</td>
										<td>{b.sistolik ?? '—'}/{b.diastolik ?? '—'}</td><td>{b.napas ?? '—'}</td><td>{b.keluhan ?? '—'}</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{:else}
					<p class="text-sm text-redup">Tidak ada observasi tercatat.</p>
				{/if}
			</section>

			<section class="panel" aria-labelledby="j-arsip">
				<h2 id="j-arsip" class="mb-3">Arsip uji silang kantong</h2>
				{#if r.uji_silang.length}
					<div class="overflow-x-auto">
						<table class="tabel">
							<thead><tr><th scope="col">Waktu</th><th scope="col">Metode</th><th scope="col">Golongan selang</th><th scope="col">Mayor / minor / auto</th><th scope="col">Kesimpulan</th><th scope="col">Petugas</th></tr></thead>
							<tbody>
								{#each r.uji_silang as u (u.id)}
									<tr>
										<td>{waktu(u.dicatat_pada)}</td>
										<td>{METODE_UJI_SILANG[u.metode] ?? u.metode}</td>
										<td>{golongan(u.abo_kantong, u.rhesus_kantong)}</td>
										<td class="mono">{tanda(u.mayor)} / {tanda(u.minor)} / {tanda(u.autokontrol)}</td>
										<td><Chip nada={KESIMPULAN[u.kesimpulan]?.[1]}>{KESIMPULAN[u.kesimpulan]?.[0] ?? u.kesimpulan}</Chip></td>
										<td>{u.petugas}{u.tanda_tangan_id ? '' : ' (belum diverifikasi)'}</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{:else}
					<p class="text-sm text-redup">Kantong ini dikeluarkan tanpa uji silang (aturan komponen atau darurat).</p>
				{/if}
			</section>

			<section class="panel flex flex-col gap-4" aria-labelledby="j-telusur">
				<h2 id="j-telusur">Penelusuran BDRS</h2>
				{#if r.ditelusuri_pada}
					<dl class="grid gap-x-6 gap-y-2 text-sm sm:grid-cols-2 lg:grid-cols-3">
						<div><dt class="text-redup">Golongan pasien ulang</dt><dd>{golongan(r.abo_pasien_ulang, r.rhesus_pasien_ulang)}</dd></div>
						<div><dt class="text-redup">Golongan kantong ulang</dt><dd>{golongan(r.abo_kantong_ulang, r.rhesus_kantong_ulang)}</dd></div>
						<div><dt class="text-redup">Uji silang ulang</dt><dd>{UJI_SILANG_ULANG[r.uji_silang_ulang] ?? '—'}</dd></div>
						<div><dt class="text-redup">Sisa volume kantong</dt><dd>{r.sisa_volume_ml ?? '—'} mL</dd></div>
						<div class="sm:col-span-2"><dt class="text-redup">Kesimpulan</dt><dd class="font-semibold">{r.kesimpulan}</dd></div>
						<div><dt class="text-redup">Ditelusuri</dt><dd>{waktu(r.ditelusuri_pada)}</dd></div>
						{#if r.penutup}<div><dt class="text-redup">Ditutup oleh</dt><dd>{r.penutup} (tanda tangan elektronik)</dd></div>{/if}
					</dl>
				{:else}
					<p class="text-sm text-redup">Belum ditelusuri.</p>
				{/if}

				{#if sesi.boleh('reaksi.telusur') && r.status !== 'ditutup'}
					<form class="tanpa-cetak flex flex-col gap-4 border-t border-garis pt-4" onsubmit={telusur}>
						<h3 class="font-semibold">{r.ditelusuri_pada ? 'Perbarui hasil penelusuran' : 'Catat hasil penelusuran'}</h3>
						<div class="grid gap-4 sm:grid-cols-2">
							<Pilihan legend="ABO pasien (periksa ulang)" name="t-abo-p" opsi={opsiAbo} bind:value={f.abo_pasien_ulang} galat={aksi.isian.abo_pasien_ulang} />
							<Pilihan legend="Rh pasien (periksa ulang)" name="t-rh-p" opsi={opsiRh} bind:value={f.rhesus_pasien_ulang} />
							<Pilihan legend="ABO kantong (periksa ulang)" name="t-abo-k" opsi={opsiAbo} bind:value={f.abo_kantong_ulang} />
							<Pilihan legend="Rh kantong (periksa ulang)" name="t-rh-k" opsi={opsiRh} bind:value={f.rhesus_kantong_ulang} />
							<Pilihan legend="Uji silang ulang" name="t-uji" opsi={Object.entries(UJI_SILANG_ULANG)} bind:value={f.uji_silang_ulang} galat={aksi.isian.uji_silang_ulang} />
							<Isian label="Sisa volume kantong (mL)" id="t-sisa">
								<input id="t-sisa" class="input" type="number" min="0" bind:value={f.sisa_volume_ml} />
							</Isian>
						</div>
						<Isian label="Kesimpulan penelusuran" id="t-kesimpulan" galat={aksi.isian.kesimpulan}>
							<textarea id="t-kesimpulan" class="input" rows="3" bind:value={f.kesimpulan} required></textarea>
						</Isian>
						<PesanAksi {aksi} />
						<button class="btn btn--utama self-start" disabled={aksi.sibuk}>Simpan penelusuran</button>
					</form>
				{/if}
			</section>
			<p class="mt-8 hidden text-sm print:block">Pemeriksa: ____________________ &nbsp;&nbsp; Penanggung jawab BDRS: ____________________</p>
		</div>
	{/snippet}
</Keadaan>
