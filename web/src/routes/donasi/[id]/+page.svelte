<script lang="ts">
	import { merek } from '#lib/merek.svelte.ts';
	import { page } from '$app/state';
	import { api } from '#lib/api.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import { sesi } from '#lib/sesi.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Isian from '#lib/Isian.svelte';
	import Chip from '#lib/Chip.svelte';
	import StatusKantong from '#lib/StatusKantong.svelte';
	import TandaTangan from '#lib/utd/TandaTangan.svelte';
	import KepalaHalaman from '#lib/KepalaHalaman.svelte';
	import GolonganLencana from '#lib/Golongan.svelte';
	import Pilihan from '#lib/pelayanan/Pilihan.svelte';
	import Tahap from '#lib/utd/Tahap.svelte';
	import { KOMPONEN, NAMA_KOMPONEN, golongan, relatif, waktu, inisial } from '#lib/format.ts';
	import { IMLTD_WAJIB, METODE_IMLTD, PARAMETER_IMLTD, STATUS_DONASI } from '#lib/utd/label.ts';
	import TriangleAlert from '@lucide/svelte/icons/triangle-alert';
	import CircleAlert from '@lucide/svelte/icons/circle-alert';
	import BadgeCheck from '@lucide/svelte/icons/badge-check';

	type Hasil = { id: string; dicatat_oleh: string; dicatat_pada: string; petugas: string; terverifikasi: boolean };
	type Golongan = Hasil & { anti_a: string; anti_b: string; sel_a: string; sel_b: string; anti_d: string; abo: string; rhesus: string; metode: string };
	type Imltd = Hasil & { parameter: string; metode: string; hasil: 'reaktif' | 'nonreaktif' };
	type Kantong = {
		id: string;
		nomor: string;
		komponen: string;
		abo: string;
		rhesus: string;
		volume_ml: number;
		kedaluwarsa: string;
		status: string;
		alasan_status: string | null;
		lulus: boolean;
	};
	type Donasi = {
		id: string;
		nomor: string;
		waktu: string;
		jenis: string;
		untuk_no_rm: string | null;
		volume_ml: number;
		jenis_kantong: string;
		status: string;
		catatan: string | null;
		donor: { id: string; nama: string; nik: string };
		lookback_id: string | null;
		golongan: Golongan | null;
		imltd: Imltd[];
		kantong: Kantong[];
	};
	type Alat = { id: string; kode: string; nama: string; jenis: string; aktif: boolean };
	type AturanKomponen = { komponen: string; jenis_alat: string };

	const id = $derived(page.params.id ?? '');
	const donasi = new Sumber(() => api<Donasi>('GET', `/donasi/${id}`));
	const alat = new Sumber(() => api<Alat[]>('GET', '/master/alat'));
	const aturan = new Sumber(() => api<AturanKomponen[]>('GET', '/master/aturan-komponen'));
	$effect(() => {
		if (id) donasi.muat();
	});
	// Alat simpan dan aturan komponen hanya diperlukan form pengolahan.
	$effect(() => {
		if (donasi.data?.status === 'diambil' && !alat.data) {
			alat.muat();
			aturan.muat();
		}
	});

	let tt: TandaTangan | undefined = $state();
	const saya = (h: Hasil) => h.dicatat_oleh === sesi.profil?.akun_id;
	const REAKSI = [
		['anti_a', 'Anti-A'],
		['anti_b', 'Anti-B'],
		['sel_a', 'Sel A'],
		['sel_b', 'Sel B'],
		['anti_d', 'Anti-D']
	] as const;
	const METODE_GOL: Record<string, string> = { tabung: 'Tabung', gel: 'Gel', bioplate: 'Bioplate' };
	const POS_NEG: [string, string][] = [
		['neg', 'Negatif'],
		['pos', 'Positif']
	];

	// Penunjuk tahap: seleksi → pengambilan → golongan → IMLTD → pengolahan → pelulusan.
	const tahap = $derived.by(() => {
		const d = donasi.data;
		if (!d) return [];
		const wajib = IMLTD_WAJIB.filter((p) => d.imltd.some((u) => u.parameter === p && u.terverifikasi)).length;
		const reaktif = d.imltd.some((u) => u.hasil === 'reaktif');
		const lulus = d.kantong.filter((k) => k.lulus).length;
		const g = d.golongan;
		return [
			{ nama: 'Seleksi', selesai: true, ket: 'Diterima' },
			{ nama: 'Pengambilan', selesai: true, ket: `${d.volume_ml} mL` },
			{ nama: 'Golongan', selesai: !!g?.terverifikasi, ket: g ? (g.terverifikasi ? `${golongan(g.abo, g.rhesus)} terverifikasi` : 'Menunggu verifikasi') : 'Belum diperiksa' },
			{ nama: 'Uji saring IMLTD', selesai: wajib === IMLTD_WAJIB.length && !reaktif, ket: reaktif ? 'Ada hasil reaktif' : `${wajib} dari ${IMLTD_WAJIB.length} wajib terverifikasi` },
			{ nama: 'Pengolahan', selesai: d.status === 'diolah', ket: d.status === 'gagal' ? 'Donasi gagal' : d.kantong.length ? `${d.kantong.length} kantong` : 'Belum diolah' },
			{ nama: 'Pelulusan', selesai: d.kantong.length > 0 && lulus === d.kantong.length, ket: d.kantong.length ? `${lulus} dari ${d.kantong.length} kantong` : '—' }
		];
	});
	const sekarang = $derived(donasi.data?.status === 'gagal' ? -1 : tahap.findIndex((t) => !t.selesai));

	// ---------- Verifikasi kedua (golongan dan IMLTD) ----------
	const aksiVerif = new Aksi();
	let tolak = $state<{ jalur: string; alasan: string } | null>(null);
	let lookbackId = $state<string | null>(null);

	function verifikasi(jalur: string, judul: string, setuju: boolean, alasan?: string) {
		const makna = setuju
			? 'Anda menyatakan sudah memeriksa ulang hasil ini dan menandatanganinya sebagai verifikasi kedua.'
			: 'Anda menolak hasil ini. Hasil dibatalkan dan pemeriksaan harus diinput ulang.';
		tt?.minta(judul, makna, aksiVerif, async (sandi) => {
			const r = await aksiVerif.jalankan(
				() => api<{ setuju: boolean; lookback_id?: string | null }>('POST', jalur, { setuju, sandi, alasan: alasan ?? null }),
				setuju ? 'Hasil diverifikasi.' : 'Hasil ditolak; silakan input ulang.'
			);
			if (r) {
				if (r.lookback_id) lookbackId = r.lookback_id;
				donasi.muat();
				tolak = null;
				donasi.muat();
			}
		});
	}

	// ---------- Golongan ----------
	const aksiGol = new Aksi();
	let gol = $state({ anti_a: '', anti_b: '', sel_a: '', sel_b: '', anti_d: '', metode: 'gel' });
	async function simpanGolongan(e: SubmitEvent) {
		e.preventDefault();
		if (await aksiGol.jalankan(() => api('POST', `/donasi/${id}/golongan`, gol), 'Hasil golongan disimpan; menunggu verifikasi kedua.')) {
			gol = { anti_a: '', anti_b: '', sel_a: '', sel_b: '', anti_d: '', metode: gol.metode };
			donasi.muat();
		}
	}

	// ---------- IMLTD ----------
	const aksiImltd = new Aksi();
	let imltd = $state<Record<string, { metode: string; hasil: string }>>(
		Object.fromEntries(Object.keys(PARAMETER_IMLTD).map((p) => [p, { metode: 'chlia', hasil: '' }]))
	);
	async function simpanImltd(e: SubmitEvent, parameter: string) {
		e.preventDefault();
		// Pengganti `required` radio bawaan: Pilihan tidak punya prop required.
		if (!imltd[parameter].hasil) {
			aksiImltd.isian = { [`hasil_${parameter}`]: `Pilih hasil ${PARAMETER_IMLTD[parameter]}: non-reaktif atau reaktif.` };
			return;
		}
		const body = { parameter, ...imltd[parameter] };
		if (await aksiImltd.jalankan(() => api('POST', `/donasi/${id}/imltd`, body), `Hasil ${PARAMETER_IMLTD[parameter]} disimpan; menunggu verifikasi kedua.`)) {
			imltd[parameter].hasil = '';
			donasi.muat();
		}
	}

	// ---------- Pengolahan ----------
	const aksiOlah = new Aksi();
	let olah = $state<Record<string, { pilih: boolean; volume_ml: string; alat_id: string }>>(
		Object.fromEntries(Object.keys(KOMPONEN).map((k) => [k, { pilih: false, volume_ml: '', alat_id: '' }]))
	);
	const alatUntuk = (k: string) => {
		const jenis = aturan.data?.find((a) => a.komponen === k)?.jenis_alat;
		return (alat.data ?? []).filter((a) => a.aktif && a.jenis === jenis);
	};
	const JENIS_ALAT: Record<string, string> = { lemari: 'lemari darah', freezer: 'freezer plasma', inkubator: 'inkubator trombosit' };
	async function simpanOlah(e: SubmitEvent) {
		e.preventDefault();
		const komponen = Object.entries(olah)
			.filter(([, v]) => v.pilih)
			.map(([k, v]) => ({ komponen: k, volume_ml: Number(v.volume_ml), alat_id: v.alat_id }));
		if (await aksiOlah.jalankan(() => api('POST', `/donasi/${id}/olah`, { komponen }), 'Donasi diolah; kantong masuk karantina sampai diluluskan.')) donasi.muat();
	}

	// ---------- Donasi gagal ----------
	const aksiGagal = new Aksi();
	let alasanGagal = $state('');
	async function tandaiGagal(e: SubmitEvent) {
		e.preventDefault();
		if (await aksiGagal.jalankan(() => api('POST', `/donasi/${id}/gagal`, { alasan: alasanGagal }), 'Donasi ditandai gagal.')) donasi.muat();
	}

	// ---------- Pelulusan ----------
	const aksiLulus = new Aksi();
	let lulusNomor = $state('');
	function luluskan(k: Kantong) {
		tt?.minta(
			`Luluskan ${k.nomor}`,
			'Anda menandatangani pelulusan: kantong ini memenuhi syarat dan boleh masuk stok tersedia.',
			aksiLulus,
			async (sandi) => {
				lulusNomor = k.nomor;
				if (await aksiLulus.jalankan(() => api('POST', `/kantong/${k.id}/luluskan`, { sandi }), `${k.nomor} diluluskan dan tersedia di stok.`)) donasi.muat();
			}
		);
	}
</script>

{#snippet tombolVerifikasi(h: Hasil, jalur: string, judul: string)}
	{#if !h.terverifikasi && sesi.boleh('uji.verifikasi')}
		{#if saya(h)}
			<p class="text-xs text-redup">Anda penginput hasil ini; verifikasi kedua harus oleh petugas lain.</p>
		{:else if tolak?.jalur === jalur}
			<form
				class="flex flex-col gap-2"
				onsubmit={(e) => {
					e.preventDefault();
					verifikasi(jalur, `Tolak ${judul}`, false, tolak?.alasan);
				}}
			>
				<Isian label="Alasan penolakan" id="tolak-{h.id}" galat={aksiVerif.isian.alasan}>
					<input id="tolak-{h.id}" class="input" bind:value={tolak.alasan} required />
				</Isian>
				<div class="flex gap-2">
					<button class="btn btn--bahaya">Tanda tangani penolakan</button>
					<button type="button" class="btn" onclick={() => (tolak = null)}>Batal</button>
				</div>
			</form>
		{:else}
			<div class="flex flex-wrap gap-2">
				<button type="button" class="btn btn--utama" onclick={() => verifikasi(jalur, `Verifikasi ${judul}`, true)}>
					<BadgeCheck class="size-4" aria-hidden="true" />Verifikasi
				</button>
				<button type="button" class="btn" onclick={() => (tolak = { jalur, alasan: '' })}>Tolak</button>
			</div>
		{/if}
	{/if}
{/snippet}

{#snippet statusVerif(h: Hasil)}
	{#if h.terverifikasi}
		<Chip nada="aman">Terverifikasi</Chip>
	{:else}
		<Chip nada="waspada">Menunggu verifikasi kedua</Chip>
	{/if}
	<span class="block text-xs text-redup">Diinput {h.petugas}, {waktu(h.dicatat_pada)}</span>
{/snippet}

<svelte:head><title>Donasi {donasi.data?.nomor ?? ''} — {merek.judul}</title></svelte:head>

<Keadaan sumber={donasi} apakahKosong={() => false}>
	{#snippet children(d)}
		{@const st = STATUS_DONASI[d.status]}
		{@const reaktif = d.imltd.filter((u) => u.hasil === 'reaktif')}
		{@const hasil = [...(d.golongan ? [{ h: d.golongan as Hasil, label: 'Golongan darah' }] : []), ...d.imltd.map((u) => ({ h: u as Hasil, label: `IMLTD ${PARAMETER_IMLTD[u.parameter] ?? u.parameter}` }))]}
		<KepalaHalaman judul={d.nomor} mono jalur={[['Donasi & pelulusan', '/donasi'], [d.nomor]]} />

		<div class="flex flex-col gap-4">
			<section class="panel flex flex-wrap items-center gap-x-6 gap-y-4" aria-labelledby="j-donor">
				<div class="flex min-w-72 flex-1 items-center gap-4">
					<span class="grid size-13 shrink-0 place-items-center rounded-full bg-halus text-lg font-bold text-teks-2" aria-hidden="true">{inisial(d.donor.nama)}</span>
					<div class="min-w-0">
						<div class="flex flex-wrap items-center gap-2">
							<h2 id="j-donor" class="text-xl font-extrabold tracking-tight"><a class="hover:underline" href="/donor/{d.donor.id}">{d.donor.nama}</a></h2>
							<Chip nada={st?.[1]}>{st?.[0] ?? d.status}</Chip>
							<Chip polos>{d.jenis === 'pengganti' ? 'Pengganti' : 'Sukarela'}</Chip>
							{#if reaktif.length}<Chip nada="bahaya">Reaktif</Chip>{/if}
						</div>
						<p class="mt-0.5 text-sm text-redup">
							<span class="mono font-semibold text-teks-2">{d.nomor}</span> · NIK <span class="mono">{d.donor.nik}</span> · diambil {waktu(d.waktu)}
						</p>
					</div>
				</div>
				<dl class="flex flex-wrap gap-x-7 gap-y-2 text-sm">
					<div><dt class="text-xs font-semibold text-redup">Volume</dt><dd class="font-bold">{d.volume_ml} mL</dd></div>
					<div><dt class="text-xs font-semibold text-redup">Kantong</dt><dd class="font-bold capitalize">{d.jenis_kantong}</dd></div>
					{#if d.jenis === 'pengganti'}
						<div><dt class="text-xs font-semibold text-redup">Pengganti untuk</dt><dd class="mono font-bold">{d.untuk_no_rm}</dd></div>
					{/if}
					{#if d.catatan}<div class="max-w-64"><dt class="text-xs font-semibold text-redup">Catatan</dt><dd class="font-bold">{d.catatan}</dd></div>{/if}
				</dl>
				<div class="flex flex-col items-center gap-1">
					{#if d.golongan}
						<GolonganLencana abo={d.golongan.abo} rhesus={d.golongan.rhesus} ukuran="besar" />
						<span class="text-xs font-semibold {d.golongan.terverifikasi ? 'text-aman' : 'text-waspada'}">{d.golongan.terverifikasi ? 'Terverifikasi' : 'Belum diverifikasi'}</span>
					{:else}
						<Chip nada="waspada">Golongan belum diperiksa</Chip>
					{/if}
				</div>
			</section>

			<Tahap {tahap} {sekarang} label="Tahap donasi" />

			{#if reaktif.length}
				<div class="peringatan" role="alert">
					<TriangleAlert class="mt-0.5 size-5 shrink-0" aria-hidden="true" />
					<div>
						<p>Reaktif {reaktif.map((u) => PARAMETER_IMLTD[u.parameter]).join(', ')}{reaktif.some((u) => !u.terverifikasi) ? ' (sebagian belum diverifikasi)' : ''}.</p>
						<p class="text-sm font-normal">
							Setelah verifikasi kedua: donor dicekal permanen sampai uji konfirmasi, semua kantong dari donasi ini tetap karantina dan tidak boleh
							diluluskan, dan look-back dibuka atas donasi-donasi sebelumnya dari donor ini.
							{#if lookbackId ?? donasi.data?.lookback_id}
								<a class="underline" href="/lookback/{lookbackId ?? donasi.data?.lookback_id}">Buka look-back yang baru dibuat</a>.
							{:else if reaktif.some((u) => u.terverifikasi)}
								<a class="underline" href="/lookback">Lihat daftar look-back</a>.
							{/if}
						</p>
					</div>
				</div>
			{/if}

			<PesanAksi aksi={aksiVerif} />

			<div class="grid items-start gap-5 xl:grid-cols-[1fr_360px]">
				<div class="flex min-w-0 flex-col gap-4">
					<section class="panel" aria-labelledby="judul-golongan">
						<h2 id="judul-golongan">Golongan darah donasi</h2>
						<p class="mb-4 text-sm text-redup">Cell dan serum grouping. Golongan tercetak di label kantong, jadi wajib diverifikasi petugas kedua.</p>
						{#if d.golongan}
							{@const g = d.golongan}
							<div class="flex flex-wrap items-center gap-5">
								<GolonganLencana abo={g.abo} rhesus={g.rhesus} ukuran="besar" />
								<dl class="grid flex-1 grid-cols-3 gap-x-4 gap-y-2 text-sm sm:grid-cols-6">
									{#each REAKSI as [k, l] (k)}
										<div><dt class="text-xs font-semibold text-redup">{l}</dt><dd class="font-semibold">{g[k] === 'pos' ? 'Positif' : 'Negatif'}</dd></div>
									{/each}
									<div><dt class="text-xs font-semibold text-redup">Metode</dt><dd class="font-semibold">{METODE_GOL[g.metode] ?? g.metode}</dd></div>
								</dl>
							</div>
							<div class="mt-4 flex flex-col gap-2 border-t border-garis pt-4">
								<div>{@render statusVerif(g)}</div>
								{@render tombolVerifikasi(g, `/golongan-donasi/${g.id}/verifikasi`, 'golongan donasi')}
							</div>
						{:else if sesi.boleh('uji.input')}
							<form class="flex flex-col gap-4" onsubmit={simpanGolongan}>
								<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
									{#each REAKSI as [k, l] (k)}
										<Pilihan legend={l} name="gol-{k}" opsi={POS_NEG} bind:value={gol[k]} galat={k === 'anti_a' || k === 'sel_a' ? aksiGol.isian[k] : undefined} />
									{/each}
									<Isian label="Metode" id="gol-metode" galat={aksiGol.isian.metode}>
										<select id="gol-metode" class="input" bind:value={gol.metode}>
											{#each Object.entries(METODE_GOL) as [v, l] (v)}<option value={v}>{l}</option>{/each}
										</select>
									</Isian>
								</div>
								<PesanAksi aksi={aksiGol} />
								<div class="flex justify-end gap-2.5 border-t border-garis pt-4">
									<button class="btn btn--utama" disabled={aksiGol.sibuk}>Simpan hasil golongan</button>
								</div>
							</form>
						{:else}
							<p class="text-sm text-redup">Golongan belum diperiksa.</p>
						{/if}
					</section>

					<section class="panel" aria-labelledby="judul-imltd">
						<h2 id="judul-imltd">Uji saring IMLTD</h2>
						<p class="mb-3 text-sm text-redup">HIV, HBsAg, HCV, dan sifilis wajib; malaria opsional.</p>
						<PesanAksi aksi={aksiImltd} />
						<ul class="divide-y divide-garis">
							{#each Object.keys(PARAMETER_IMLTD) as p (p)}
								{@const u = d.imltd.find((x) => x.parameter === p)}
								<li class="py-3 last:pb-0">
									<div class="flex flex-wrap items-start justify-between gap-3">
										<p class="font-semibold">
											{PARAMETER_IMLTD[p]}
											<span class="text-xs font-medium text-redup">{IMLTD_WAJIB.includes(p) ? 'wajib' : 'opsional'}</span>
										</p>
										{#if u}
											<div class="text-right">
												<Chip nada={u.hasil === 'reaktif' ? 'bahaya' : 'aman'}>{u.hasil === 'reaktif' ? 'Reaktif' : 'Non-reaktif'}</Chip>
												<span class="ml-1 text-xs text-redup">{METODE_IMLTD[u.metode] ?? u.metode}</span>
											</div>
										{/if}
									</div>
									{#if u}
										<div class="mt-2 flex flex-col gap-2">
											<div>{@render statusVerif(u)}</div>
											{@render tombolVerifikasi(u, `/imltd/${u.id}/verifikasi`, `uji ${PARAMETER_IMLTD[p]}`)}
										</div>
									{:else if sesi.boleh('uji.input')}
										<form class="mt-2 flex flex-wrap items-end gap-3" onsubmit={(e) => simpanImltd(e, p)}>
											<div>
												<label class="label" for="metode-{p}">Metode</label>
												<select id="metode-{p}" class="input" bind:value={imltd[p].metode}>
													{#each Object.entries(METODE_IMLTD) as [v, l] (v)}<option value={v}>{l}</option>{/each}
												</select>
											</div>
											<div class="min-w-56 flex-1">
												<Pilihan
													legend="Hasil {PARAMETER_IMLTD[p]}"
													name="hasil-{p}"
													opsi={[
														['nonreaktif', 'Non-reaktif'],
														['reaktif', 'Reaktif']
													]}
													bind:value={imltd[p].hasil}
													galat={aksiImltd.isian[`hasil_${p}`]}
													bahaya={['reaktif']}
												/>
											</div>
											<button class="btn" disabled={aksiImltd.sibuk}>Simpan</button>
										</form>
									{:else}
										<p class="mt-1 text-sm text-redup">Belum diperiksa.</p>
									{/if}
								</li>
							{/each}
						</ul>
					</section>

					{#if d.status === 'diambil' && sesi.boleh('donor.kelola')}
						<form class="panel flex flex-col gap-4" onsubmit={simpanOlah} aria-labelledby="judul-olah">
							<div>
								<h2 id="judul-olah">Pengolahan komponen</h2>
								<p class="text-sm text-redup">Pilih komponen hasil pemisahan. Darah lengkap (WB) tidak bisa digabung dengan komponen lain. Kedaluwarsa dihitung dari aturan komponen.</p>
							</div>
							{#if !d.golongan?.terverifikasi}
								<p class="catatan">Golongan donasi harus terverifikasi lebih dulu karena tercetak di label kantong.</p>
							{:else if alat.galat || aturan.galat}
								<p class="text-sm text-bahaya" role="alert">Daftar alat simpan tidak dapat dimuat: {alat.galat ?? aturan.galat}</p>
							{:else if !alat.data || !aturan.data}
								<p class="text-sm text-redup" aria-busy="true">Memuat alat simpan…</p>
							{:else}
								<div class="overflow-x-auto">
									<table class="tabel">
										<thead><tr><th scope="col">Komponen</th><th scope="col">Volume (mL)</th><th scope="col">Alat simpan</th></tr></thead>
										<tbody>
											{#each Object.keys(KOMPONEN) as k (k)}
												{@const daftarAlat = alatUntuk(k)}
												<tr>
													<td>
														<label class="flex items-center gap-2 font-medium">
															<input
																type="checkbox"
																bind:checked={olah[k].pilih}
																disabled={k === 'wb' ? Object.entries(olah).some(([x, v]) => x !== 'wb' && v.pilih) : olah.wb.pilih}
															/>{NAMA_KOMPONEN[k]}
														</label>
													</td>
													<td>
														<label class="sr-only" for="vol-{k}">Volume {KOMPONEN[k]} (mL)</label>
														<input id="vol-{k}" class="input w-28" type="number" min="1" bind:value={olah[k].volume_ml} disabled={!olah[k].pilih} required={olah[k].pilih} />
													</td>
													<td>
														<label class="sr-only" for="alat-{k}">Alat simpan {KOMPONEN[k]}</label>
														<select id="alat-{k}" class="input" bind:value={olah[k].alat_id} disabled={!olah[k].pilih} required={olah[k].pilih}>
															<option value="" disabled>{daftarAlat.length ? `Pilih ${JENIS_ALAT[daftarAlat[0].jenis]}` : 'Tidak ada alat yang sesuai'}</option>
															{#each daftarAlat as a (a.id)}<option value={a.id}>{a.kode} — {a.nama}</option>{/each}
														</select>
													</td>
												</tr>
											{/each}
										</tbody>
									</table>
								</div>
								{#if aksiOlah.isian.komponen}<p class="text-sm text-bahaya" role="alert">{aksiOlah.isian.komponen}</p>{/if}
								<PesanAksi aksi={aksiOlah} />
								<div class="flex justify-end gap-2.5 border-t border-garis pt-4">
									<button class="btn btn--utama" disabled={aksiOlah.sibuk || !Object.values(olah).some((v) => v.pilih)}>Olah donasi</button>
								</div>
							{/if}
						</form>
					{/if}

					<section class="panel" aria-labelledby="judul-kantong">
						<h2 id="judul-kantong">Kantong hasil pengolahan</h2>
						<p class="mb-3 text-sm text-redup">Kantong tetap karantina sampai diluluskan.</p>
						{#if aksiLulus.galat}
							<div class="peringatan mb-3" role="alert">
								<CircleAlert class="mt-0.5 size-5 shrink-0" aria-hidden="true" />
								<p>{lulusNomor} tidak dapat diluluskan: {aksiLulus.galat}</p>
							</div>
						{:else}
							<PesanAksi aksi={aksiLulus} />
						{/if}
						{#if d.kantong.length === 0}
							<p class="text-sm text-redup">Donasi belum diolah.</p>
						{:else}
							<div class="overflow-x-auto">
								<table class="tabel">
									<thead>
										<tr>
											<th scope="col">Nomor kantong</th>
											<th scope="col">Komponen</th>
											<th scope="col">Golongan</th>
											<th scope="col">Volume</th>
											<th scope="col">Kedaluwarsa</th>
											<th scope="col">Status</th>
											<th scope="col"><span class="sr-only">Aksi</span></th>
										</tr>
									</thead>
									<tbody>
										{#each d.kantong as k (k.id)}
											<tr>
												<td class="mono font-semibold">{k.nomor}</td>
												<td>{KOMPONEN[k.komponen] ?? k.komponen}</td>
												<td><GolonganLencana abo={k.abo} rhesus={k.rhesus} ukuran="kecil" /></td>
												<td class="tabular-nums">{k.volume_ml} mL</td>
												<td class="whitespace-nowrap">{waktu(k.kedaluwarsa)}<span class="block text-xs text-redup">{relatif(k.kedaluwarsa)}</span></td>
												<td>
													<span class="flex flex-wrap gap-1">
														<StatusKantong status={k.status} />
														{#if k.lulus}<Chip nada="aman">Diluluskan</Chip>{/if}
													</span>
													{#if k.alasan_status}<span class="block text-xs text-redup">{k.alasan_status}</span>{/if}
												</td>
												<td class="text-right">
													{#if k.status === 'karantina' && !k.lulus && sesi.boleh('donor.lulus')}
														<button type="button" class="btn btn--utama" onclick={() => luluskan(k)} disabled={aksiLulus.sibuk}>Luluskan</button>
													{/if}
												</td>
											</tr>
										{/each}
									</tbody>
								</table>
							</div>
						{/if}
					</section>
				</div>

				<aside class="flex flex-col gap-4" aria-label="Ringkasan donasi">
					<section class="panel" aria-labelledby="judul-verif">
						<h2 id="judul-verif">Verifikasi kedua</h2>
						<p class="mb-3 text-sm text-redup">Hasil manual baru berlaku setelah diperiksa petugas lain.</p>
						{#if hasil.length === 0}
							<p class="text-sm text-redup">Belum ada hasil yang diinput.</p>
						{:else}
							<ul class="flex flex-col gap-3">
								{#each hasil as { h, label } (h.id)}
									<li class="flex items-start justify-between gap-3">
										<span class="min-w-0">
											<span class="block text-sm font-semibold">{label}</span>
											<span class="block text-xs text-redup">{h.petugas} · {waktu(h.dicatat_pada)}</span>
										</span>
										<Chip nada={h.terverifikasi ? 'aman' : 'waspada'}>{h.terverifikasi ? 'Terverifikasi' : 'Menunggu'}</Chip>
									</li>
								{/each}
							</ul>
						{/if}
					</section>

					{#if d.status === 'diambil' && sesi.boleh('donor.kelola')}
						<form class="panel flex flex-col gap-4" onsubmit={tandaiGagal} aria-labelledby="judul-gagal">
							<div>
								<h2 id="judul-gagal">Donasi gagal</h2>
								<p class="text-sm text-redup">Untuk pengambilan yang tidak bisa diolah (aliran berhenti, volume kurang, kantong rusak).</p>
							</div>
							<Isian label="Alasan" id="alasan-gagal" galat={aksiGagal.isian.alasan}>
								<input id="alasan-gagal" class="input" bind:value={alasanGagal} required />
							</Isian>
							<PesanAksi aksi={aksiGagal} />
							<div class="flex justify-end gap-2.5 border-t border-garis pt-4">
								<button class="btn btn--bahaya" disabled={aksiGagal.sibuk}>Tandai gagal</button>
							</div>
						</form>
					{/if}
				</aside>
			</div>
		</div>
	{/snippet}
</Keadaan>

<TandaTangan bind:this={tt} />
