<script lang="ts">
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
	import { KOMPONEN, NAMA_KOMPONEN, golongan, relatif, waktu } from '#lib/format.ts';
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

<svelte:head><title>Donasi {donasi.data?.nomor ?? ''} — utd-rs</title></svelte:head>

<p class="mb-2 text-sm"><a class="text-utama hover:underline" href="/donasi">← Daftar donasi</a></p>

<Keadaan sumber={donasi} apakahKosong={() => false}>
	{#snippet children(d)}
		{@const st = STATUS_DONASI[d.status]}
		{@const reaktif = d.imltd.filter((u) => u.hasil === 'reaktif')}
		<div class="mb-4 flex flex-wrap items-start justify-between gap-3">
			<div>
				<h1 class="mono">{d.nomor}</h1>
				<p class="text-sm text-redup">
					Donor <a class="text-utama hover:underline" href="/donor/{d.donor.id}">{d.donor.nama}</a> · {waktu(d.waktu)} ·
					{d.jenis === 'pengganti' ? `pengganti untuk no. RM ${d.untuk_no_rm}` : 'sukarela'} · {d.volume_ml} mL · kantong {d.jenis_kantong}
				</p>
				{#if d.catatan}<p class="text-sm text-redup">Catatan: {d.catatan}</p>{/if}
			</div>
			<Chip nada={st?.[1]}>{st?.[0] ?? d.status}</Chip>
		</div>

		{#if reaktif.length}
			<div class="peringatan mb-4" role="alert">
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

		<div class="mt-4 grid gap-4 xl:grid-cols-2">
			<section class="panel" aria-labelledby="judul-golongan">
				<h2 id="judul-golongan" class="mb-3">Golongan darah donasi</h2>
				{#if d.golongan}
					{@const g = d.golongan}
					<p class="mb-2 text-2xl font-semibold">{golongan(g.abo, g.rhesus)}</p>
					<dl class="mb-3 grid grid-cols-3 gap-x-4 gap-y-1 text-sm sm:grid-cols-6">
						{#each REAKSI as [k, l] (k)}
							<div><dt class="text-xs text-redup">{l}</dt><dd>{g[k] === 'pos' ? 'Positif' : 'Negatif'}</dd></div>
						{/each}
						<div><dt class="text-xs text-redup">Metode</dt><dd>{METODE_GOL[g.metode] ?? g.metode}</dd></div>
					</dl>
					<div class="flex flex-col gap-2">
						<div>{@render statusVerif(g)}</div>
						{@render tombolVerifikasi(g, `/golongan-donasi/${g.id}/verifikasi`, 'golongan donasi')}
					</div>
				{:else if sesi.boleh('uji.input')}
					<form class="flex flex-col gap-3" onsubmit={simpanGolongan}>
						<div class="overflow-x-auto">
							<table class="tabel">
								<caption class="sr-only">Hasil reaksi pemeriksaan golongan</caption>
								<thead><tr><th scope="col">Reagen</th><th scope="col">Positif</th><th scope="col">Negatif</th></tr></thead>
								<tbody>
									{#each REAKSI as [k, l] (k)}
										<tr>
											<th scope="row" class="text-left text-sm font-medium normal-case">{l}</th>
											<td><label class="flex items-center gap-2"><input type="radio" name="gol-{k}" value="pos" bind:group={gol[k]} required /><span>Positif<span class="sr-only"> {l}</span></span></label></td>
											<td><label class="flex items-center gap-2"><input type="radio" name="gol-{k}" value="neg" bind:group={gol[k]} /><span>Negatif<span class="sr-only"> {l}</span></span></label></td>
										</tr>
									{/each}
								</tbody>
							</table>
						</div>
						<Isian label="Metode" id="gol-metode" galat={aksiGol.isian.metode}>
							<select id="gol-metode" class="input" bind:value={gol.metode}>
								{#each Object.entries(METODE_GOL) as [v, l] (v)}<option value={v}>{l}</option>{/each}
							</select>
						</Isian>
						{#if aksiGol.isian.anti_a || aksiGol.isian.sel_a}
							<p class="text-sm text-bahaya" role="alert">{aksiGol.isian.anti_a ?? aksiGol.isian.sel_a}</p>
						{/if}
						<PesanAksi aksi={aksiGol} />
						<button class="btn btn--utama self-start" disabled={aksiGol.sibuk}>Simpan hasil golongan</button>
					</form>
				{:else}
					<p class="text-sm text-redup">Golongan belum diperiksa.</p>
				{/if}
			</section>

			<section class="panel" aria-labelledby="judul-imltd">
				<h2 id="judul-imltd" class="mb-1">Uji saring IMLTD</h2>
				<p class="mb-3 text-xs text-redup">HIV, HBsAg, HCV, dan sifilis wajib; malaria opsional.</p>
				<PesanAksi aksi={aksiImltd} />
				<ul class="divide-y divide-halus">
					{#each Object.keys(PARAMETER_IMLTD) as p (p)}
						{@const u = d.imltd.find((x) => x.parameter === p)}
						<li class="py-3">
							<div class="flex flex-wrap items-start justify-between gap-3">
								<p class="font-medium">
									{PARAMETER_IMLTD[p]}
									<span class="text-xs font-normal text-redup">{IMLTD_WAJIB.includes(p) ? 'wajib' : 'opsional'}</span>
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
									<fieldset>
										<legend class="label">Hasil {PARAMETER_IMLTD[p]}</legend>
										<div class="flex min-h-10 items-center gap-4">
											<label class="flex items-center gap-2"><input type="radio" name="hasil-{p}" value="nonreaktif" bind:group={imltd[p].hasil} required />Non-reaktif</label>
											<label class="flex items-center gap-2"><input type="radio" name="hasil-{p}" value="reaktif" bind:group={imltd[p].hasil} />Reaktif</label>
										</div>
									</fieldset>
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
				<form class="panel flex flex-col gap-4 xl:col-span-2" onsubmit={simpanOlah} aria-labelledby="judul-olah">
					<h2 id="judul-olah">Pengolahan komponen</h2>
					{#if !d.golongan?.terverifikasi}
						<p class="catatan">Golongan donasi harus terverifikasi lebih dulu karena tercetak di label kantong.</p>
					{:else}
						{#if alat.galat || aturan.galat}
							<p class="text-sm text-bahaya" role="alert">Daftar alat simpan tidak dapat dimuat: {alat.galat ?? aturan.galat}</p>
						{:else if !alat.data || !aturan.data}
							<p class="text-sm text-redup" aria-busy="true">Memuat alat simpan…</p>
						{:else}
							<p class="text-sm text-redup">Pilih komponen hasil pemisahan. Darah lengkap (WB) tidak bisa digabung dengan komponen lain. Kedaluwarsa dihitung dari aturan komponen.</p>
							<div class="overflow-x-auto">
								<table class="tabel">
									<thead><tr><th scope="col">Komponen</th><th scope="col">Volume (mL)</th><th scope="col">Alat simpan</th></tr></thead>
									<tbody>
										{#each Object.keys(KOMPONEN) as k (k)}
											{@const daftarAlat = alatUntuk(k)}
											<tr>
												<td>
													<label class="flex items-center gap-2">
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
							<button class="btn btn--utama self-start" disabled={aksiOlah.sibuk || !Object.values(olah).some((v) => v.pilih)}>Olah donasi</button>
						{/if}
					{/if}
				</form>
				<form class="panel flex flex-col gap-3 xl:col-span-2" onsubmit={tandaiGagal} aria-labelledby="judul-gagal">
					<h2 id="judul-gagal">Donasi gagal</h2>
					<p class="text-sm text-redup">Untuk pengambilan yang tidak bisa diolah (aliran berhenti, volume kurang, kantong rusak).</p>
					<Isian label="Alasan" id="alasan-gagal" galat={aksiGagal.isian.alasan}>
						<input id="alasan-gagal" class="input" bind:value={alasanGagal} required />
					</Isian>
					<PesanAksi aksi={aksiGagal} />
					<button class="btn btn--bahaya self-start" disabled={aksiGagal.sibuk}>Tandai gagal</button>
				</form>
			{/if}

			<section class="panel xl:col-span-2" aria-labelledby="judul-kantong">
				<h2 id="judul-kantong" class="mb-3">Kantong hasil pengolahan</h2>
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
										<td class="mono">{k.nomor}</td>
										<td>{KOMPONEN[k.komponen] ?? k.komponen}</td>
										<td>{golongan(k.abo, k.rhesus)}</td>
										<td class="tabular-nums">{k.volume_ml} mL</td>
										<td class="whitespace-nowrap">{waktu(k.kedaluwarsa)}<span class="block text-xs text-redup">{relatif(k.kedaluwarsa)}</span></td>
										<td>
											<StatusKantong status={k.status} />
											{#if k.lulus}<Chip nada="aman">Diluluskan</Chip>{/if}
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
	{/snippet}
</Keadaan>

<TandaTangan bind:this={tt} />
