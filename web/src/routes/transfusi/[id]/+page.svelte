<!-- Detail transfusi: observasi (tambah/koreksi), selesai atau hentikan, dan laporan reaksi. Tata letak untuk tablet. -->
<script lang="ts">
	import { merek } from '#lib/merek.svelte.ts';
	import { page } from '$app/state';
	import { api } from '#lib/api.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Isian from '#lib/Isian.svelte';
	import Chip from '#lib/Chip.svelte';
	import { sesi } from '#lib/sesi.svelte.ts';
	import { KOMPONEN, golongan, keRfc3339, lokal, relatif, waktu } from '#lib/format.ts';
	import { GEJALA, STATUS_REAKSI, TAHAP_OBSERVASI, angka } from '#lib/pelayanan/darah.ts';
	import TriangleAlert from '@lucide/svelte/icons/triangle-alert';

	const id = $derived(page.params.id);
	const detail = new Sumber(() => api<any>('GET', `/transfusi/${id}`));
	$effect(() => {
		void id;
		detail.muat();
	});
	const muat = () => detail.muat();
	const t = $derived(detail.data);
	const bolehCatat = $derived(sesi.boleh('transfusi.catat'));
	const bolehLihatReaksi = $derived(sesi.boleh('reaksi.telusur') || sesi.boleh('laporan.lihat'));

	// Observasi baru atau koreksi.
	const kosong = () => ({ tahap: '', suhu: '' as string | number, nadi: '' as string | number, sistolik: '' as string | number, diastolik: '' as string | number, napas: '' as string | number, keluhan: '', alasan: '' });
	let o = $state(kosong());
	let koreksi = $state<number | null>(null);
	const aksiObs = new Aksi();
	const tahapBoleh = $derived(t?.selesai ? ['pasca'] : ['pra', '15_menit', 'berkala', 'selesai']);
	// Saran tahap berikutnya.
	$effect(() => {
		if (!t || koreksi !== null || (o.tahap && tahapBoleh.includes(o.tahap))) return;
		const ada = (x: string) => t.observasi.some((b: any) => b.tahap === x);
		o.tahap = t.selesai ? 'pasca' : !ada('pra') ? 'pra' : !ada('15_menit') ? '15_menit' : 'berkala';
	});

	function mulaiKoreksi(b: any) {
		koreksi = b.id;
		o = { tahap: b.tahap, suhu: b.suhu ?? '', nadi: b.nadi ?? '', sistolik: b.sistolik ?? '', diastolik: b.diastolik ?? '', napas: b.napas ?? '', keluhan: b.keluhan ?? '', alasan: '' };
		document.getElementById('form-obs')?.scrollIntoView({ behavior: 'smooth' });
	}

	async function kirimObs(e: SubmitEvent) {
		e.preventDefault();
		const body = {
			tahap: o.tahap,
			suhu: angka(o.suhu),
			nadi: angka(o.nadi),
			sistolik: angka(o.sistolik),
			diastolik: angka(o.diastolik),
			napas: angka(o.napas),
			keluhan: o.keluhan || undefined,
			alasan: koreksi !== null ? o.alasan : undefined
		};
		const r = await aksiObs.jalankan(
			() => (koreksi !== null ? api('POST', `/observasi/${koreksi}/koreksi`, body) : api('POST', `/transfusi/${id}/observasi`, body)),
			koreksi !== null ? 'Koreksi tersimpan; nilai lama tetap di jejak audit.' : 'Observasi tercatat.'
		);
		if (r) {
			koreksi = null;
			o = kosong();
			await muat();
		}
	}

	// Selesai atau hentikan.
	let volume = $state<string | number>('');
	let catatan = $state('');
	const aksiSelesai = new Aksi();
	async function akhiri(dihentikan: boolean) {
		const r = await aksiSelesai.jalankan(
			() => api('POST', `/transfusi/${id}/selesai`, { volume_masuk_ml: angka(volume), dihentikan, catatan: catatan || undefined }),
			dihentikan ? 'Transfusi dihentikan.' : 'Transfusi selesai. Catat observasi pasca-transfusi.'
		);
		if (r) await muat();
	}

	// Laporan reaksi.
	let waktuReaksi = $state(lokal());
	let gejala = $state<string[]>([]);
	let tindakan = $state('');
	const aksiReaksi = new Aksi();
	async function lapor(e: SubmitEvent) {
		e.preventDefault();
		const r = await aksiReaksi.jalankan(
			() => api('POST', `/transfusi/${id}/reaksi`, { waktu_reaksi: keRfc3339(waktuReaksi), gejala, tindakan }),
			'Reaksi dilaporkan ke bank darah untuk ditelusuri.'
		);
		if (r) {
			gejala = [];
			tindakan = '';
			waktuReaksi = lokal();
			await muat();
		}
	}
</script>

<svelte:head><title>Transfusi — {merek.judul}</title></svelte:head>

<Keadaan sumber={detail} apakahKosong={() => false} baris={8}>
	{#snippet children(t)}
		{@const k = t.kantong}
		<p class="tanpa-cetak text-sm"><a class="text-utama hover:underline" href="/transfusi">Transfusi di ruang</a></p>
		<div class="mb-4 flex flex-wrap items-center gap-3">
			<h1>{t.pasien.nama} <span class="mono text-base font-normal text-redup">{t.no_rm}</span></h1>
			{#if !t.selesai}
				<Chip nada="info">Berjalan</Chip>
			{:else if t.dihentikan}
				<Chip nada="bahaya">Dihentikan</Chip>
			{:else}
				<Chip nada="aman">Selesai</Chip>
			{/if}
		</div>

		<div class="flex flex-col gap-4">
			<section class="panel" aria-labelledby="j-info">
				<h2 id="j-info" class="sr-only">Ringkasan</h2>
				<dl class="grid gap-x-6 gap-y-2 text-sm sm:grid-cols-2 lg:grid-cols-3">
					<div><dt class="text-redup">Kantong</dt><dd class="mono font-semibold">{k.nomor}</dd></div>
					<div><dt class="text-redup">Komponen dan golongan</dt><dd>{KOMPONEN[k.komponen]} {golongan(k.abo, k.rhesus)}</dd></div>
					<div><dt class="text-redup">Permintaan</dt><dd><a class="mono text-utama hover:underline" href="/permintaan/{t.permintaan.id}">{t.permintaan.nomor}</a> · {t.permintaan.ruang}</dd></div>
					<div><dt class="text-redup">Keluar dari BDRS</dt><dd>{waktu(t.dikeluarkan_pada)}</dd></div>
					<div><dt class="text-redup">Mulai</dt><dd>{waktu(t.mulai)} ({relatif(t.mulai)})</dd></div>
					<div><dt class="text-redup">Verifikasi samping tempat tidur</dt><dd>{t.petugas_1} dan {t.petugas_2}</dd></div>
					{#if t.alasan_terlambat}<div><dt class="text-redup">Alasan terlambat</dt><dd>{t.alasan_terlambat}</dd></div>{/if}
					{#if t.selesai}
						<div><dt class="text-redup">{t.dihentikan ? 'Dihentikan' : 'Selesai'}</dt><dd>{waktu(t.selesai)}</dd></div>
						<div><dt class="text-redup">Volume masuk</dt><dd>{t.volume_masuk_ml ?? '—'} mL</dd></div>
						{#if t.catatan}<div><dt class="text-redup">Catatan</dt><dd>{t.catatan}</dd></div>{/if}
					{/if}
					<div><dt class="text-redup">Indikasi / dokter</dt><dd>{t.permintaan.indikasi} · {t.permintaan.dokter}</dd></div>
				</dl>
			</section>

			{#if t.reaksi.length}
				<section class="peringatan" aria-labelledby="j-reaksi-ada">
					<TriangleAlert class="mt-0.5 size-5 shrink-0" aria-hidden="true" />
					<div>
						<h2 id="j-reaksi-ada">Reaksi transfusi dilaporkan</h2>
						<ul>
							{#each t.reaksi as r (r.id)}
								{@const s = STATUS_REAKSI[r.status]}
								<li class="mt-1 flex flex-wrap items-center gap-2">
									{#if bolehLihatReaksi}<a class="mono underline" href="/reaksi/{r.id}">{r.nomor}</a>{:else}<span class="mono">{r.nomor}</span>{/if}
									<span>{waktu(r.waktu_reaksi)} · {r.gejala.join(', ')}</span>
									<Chip nada={s?.[1]}>{s?.[0] ?? r.status}</Chip>
								</li>
							{/each}
						</ul>
					</div>
				</section>
			{/if}

			<section class="panel" aria-labelledby="j-obs">
				<h2 id="j-obs" class="mb-3 text-lg">Observasi</h2>
				{#if t.observasi.length}
					<div class="overflow-x-auto">
						<table class="tabel">
							<thead>
								<tr>
									<th scope="col">Tahap</th><th scope="col">Waktu</th><th scope="col">Suhu (°C)</th><th scope="col">Nadi</th>
									<th scope="col">TD (mmHg)</th><th scope="col">Napas</th><th scope="col">Keluhan</th><th scope="col">Petugas</th>
									{#if bolehCatat}<th scope="col" class="tanpa-cetak"><span class="sr-only">Aksi</span></th>{/if}
								</tr>
							</thead>
							<tbody>
								{#each t.observasi as b (b.id)}
									<tr class={koreksi === b.id ? 'bg-info-latar' : ''}>
										<td>{TAHAP_OBSERVASI[b.tahap] ?? b.tahap}{#if b.dikoreksi_pada}<span class="mt-1 block"><Chip nada="waspada">Dikoreksi {waktu(b.dikoreksi_pada)}</Chip></span>{/if}</td>
										<td>{waktu(b.waktu)}</td>
										<td class="tabular-nums {b.suhu >= 38 ? 'font-semibold text-bahaya' : ''}">{b.suhu ?? '—'}</td>
										<td class="tabular-nums">{b.nadi ?? '—'}</td>
										<td class="tabular-nums">{b.sistolik ?? '—'}/{b.diastolik ?? '—'}</td>
										<td class="tabular-nums">{b.napas ?? '—'}</td>
										<td>{b.keluhan ?? '—'}</td>
										<td>{b.petugas}</td>
										{#if bolehCatat}<td class="tanpa-cetak"><button type="button" class="btn btn--teks" onclick={() => mulaiKoreksi(b)}>Koreksi</button></td>{/if}
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{:else}
					<p class="text-sm text-redup">Belum ada observasi. Catat tanda vital pra-transfusi.</p>
				{/if}

				{#if bolehCatat && (!t.selesai || koreksi !== null || !t.observasi.some((b: any) => b.tahap === 'pasca'))}
					<form id="form-obs" class="tanpa-cetak mt-4 flex flex-col gap-4 border-t border-garis pt-4" onsubmit={kirimObs}>
						<h3 class="font-semibold">{koreksi !== null ? 'Koreksi observasi' : 'Catat observasi'}</h3>
						<div class="grid gap-4 sm:grid-cols-3 lg:grid-cols-6">
							<Isian label="Tahap" id="o-tahap" galat={aksiObs.isian.tahap}>
								<select id="o-tahap" class="input min-h-12 text-base" bind:value={o.tahap}>
									{#each koreksi !== null ? Object.keys(TAHAP_OBSERVASI) : tahapBoleh as x (x)}<option value={x}>{TAHAP_OBSERVASI[x]}</option>{/each}
								</select>
							</Isian>
							<Isian label="Suhu (°C)" id="o-suhu" galat={aksiObs.isian.suhu}>
								<input id="o-suhu" class="input min-h-12 text-base" type="number" inputmode="decimal" step="0.1" min="30" max="45" bind:value={o.suhu} />
							</Isian>
							<Isian label="Nadi (/menit)" id="o-nadi">
								<input id="o-nadi" class="input min-h-12 text-base" type="number" inputmode="numeric" min="20" max="250" bind:value={o.nadi} />
							</Isian>
							<Isian label="Sistolik (mmHg)" id="o-sis">
								<input id="o-sis" class="input min-h-12 text-base" type="number" inputmode="numeric" min="40" max="300" bind:value={o.sistolik} />
							</Isian>
							<Isian label="Diastolik (mmHg)" id="o-dia">
								<input id="o-dia" class="input min-h-12 text-base" type="number" inputmode="numeric" min="20" max="200" bind:value={o.diastolik} />
							</Isian>
							<Isian label="Napas (/menit)" id="o-napas">
								<input id="o-napas" class="input min-h-12 text-base" type="number" inputmode="numeric" min="4" max="80" bind:value={o.napas} />
							</Isian>
						</div>
						<Isian label="Keluhan (opsional)" id="o-keluhan">
							<input id="o-keluhan" class="input min-h-12 text-base" bind:value={o.keluhan} />
						</Isian>
						{#if koreksi !== null}
							<Isian label="Alasan koreksi" id="o-alasan" galat={aksiObs.isian.alasan} bantuan="Nilai lama tetap tersimpan di jejak audit. Lewat 2×24 jam, koreksi dilakukan penanggung jawab BDRS.">
								<input id="o-alasan" class="input min-h-12 text-base" bind:value={o.alasan} required />
							</Isian>
						{/if}
						<PesanAksi aksi={aksiObs} />
						<div class="flex flex-wrap gap-3">
							<button class="btn btn--utama btn--besar" disabled={aksiObs.sibuk}>{koreksi !== null ? 'Simpan koreksi' : 'Simpan observasi'}</button>
							{#if koreksi !== null}
								<button type="button" class="btn btn--besar" onclick={() => ((koreksi = null), (o = kosong()))}>Batal koreksi</button>
							{/if}
						</div>
					</form>
				{/if}
			</section>

			{#if bolehCatat && !t.selesai}
				<section class="panel tanpa-cetak flex flex-col gap-4" aria-labelledby="j-selesai">
					<h2 id="j-selesai" class="text-lg">Selesaikan transfusi</h2>
					<div class="form-grid">
						<Isian label="Volume masuk (mL)" id="s-volume" galat={aksiSelesai.isian.volume_masuk_ml}>
							<input id="s-volume" class="input min-h-12 text-base" type="number" inputmode="numeric" min="0" max="2000" bind:value={volume} />
						</Isian>
						<Isian label="Catatan (wajib bila dihentikan)" id="s-catatan" galat={aksiSelesai.isian.catatan}>
							<input id="s-catatan" class="input min-h-12 text-base" bind:value={catatan} />
						</Isian>
					</div>
					<PesanAksi aksi={aksiSelesai} />
					<div class="flex flex-wrap gap-3">
						<button type="button" class="btn btn--utama btn--besar" disabled={aksiSelesai.sibuk} onclick={() => akhiri(false)}>Transfusi selesai</button>
						<button type="button" class="btn btn--bahaya btn--besar" disabled={aksiSelesai.sibuk} onclick={() => akhiri(true)}>Hentikan transfusi</button>
					</div>
				</section>
			{/if}

			{#if sesi.boleh('reaksi.lapor')}
				<form class="panel tanpa-cetak flex flex-col gap-4" aria-labelledby="j-lapor" onsubmit={lapor}>
					<h2 id="j-lapor" class="text-lg">Laporkan reaksi transfusi</h2>
					<p class="text-sm text-redup">Bila ada tanda reaksi: hentikan transfusi, pertahankan jalur infus, lapor DPJP, lalu catat di sini. Kirim kantong dan sampel darah pasien ke bank darah.</p>
					<Isian label="Waktu reaksi" id="r-waktu" galat={aksiReaksi.isian.waktu_reaksi}>
						<input id="r-waktu" class="input min-h-12 text-base sm:max-w-xs" type="datetime-local" bind:value={waktuReaksi} required />
					</Isian>
					<fieldset aria-describedby={aksiReaksi.isian.gejala ? 'r-gejala-galat' : undefined}>
						<legend class="label">Gejala (pilih semua yang ada)</legend>
						<div class="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
							{#each GEJALA as g (g)}
								<label class="flex min-h-12 cursor-pointer items-center gap-3 rounded-md border border-garis px-3 has-checked:border-bahaya has-checked:bg-bahaya-latar">
									<input type="checkbox" class="size-5" value={g} bind:group={gejala} />{g}
								</label>
							{/each}
						</div>
						{#if aksiReaksi.isian.gejala}<p class="mt-1 text-sm text-bahaya" id="r-gejala-galat" role="alert">{aksiReaksi.isian.gejala}</p>{/if}
					</fieldset>
					<Isian label="Tindakan yang sudah dilakukan" id="r-tindakan" galat={aksiReaksi.isian.tindakan}>
						<textarea id="r-tindakan" class="input text-base" rows="2" bind:value={tindakan} required></textarea>
					</Isian>
					<PesanAksi aksi={aksiReaksi} />
					<button class="btn btn--bahaya btn--besar self-start" disabled={aksiReaksi.sibuk || !gejala.length}>Laporkan reaksi</button>
				</form>
			{/if}
		</div>
	{/snippet}
</Keadaan>
