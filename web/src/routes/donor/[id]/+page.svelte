<script lang="ts">
	import { merek } from '#lib/merek.svelte.ts';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { api } from '#lib/api.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import { sesi } from '#lib/sesi.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Isian from '#lib/Isian.svelte';
	import Chip from '#lib/Chip.svelte';
	import Pindai from '#lib/Pindai.svelte';
	import TandaTangan from '#lib/utd/TandaTangan.svelte';
	import { urai } from '#lib/isbt.ts';
	import { golongan, hariIni, tanggal, waktu } from '#lib/format.ts';
	import Ban from '@lucide/svelte/icons/ban';
	import Droplet from '@lucide/svelte/icons/droplet';

	type Cekal = {
		id: string;
		jenis: 'sementara' | 'permanen';
		mulai: string;
		sampai: string | null;
		alasan: string;
		sumber: string;
		dibuat_pada: string;
		dicabut_pada: string | null;
		alasan_cabut: string | null;
	};
	type Seleksi = {
		id: string;
		waktu: string;
		berat_kg: number;
		hb: number;
		sistolik: number;
		diastolik: number;
		nadi: number;
		suhu: number;
		kuesioner_lolos: boolean;
		keputusan: string;
		alasan: string | null;
		donasi_id: string | null;
	};
	type Donasi = { id: string; nomor: string; waktu: string; jenis: string; untuk_no_rm: string | null; volume_ml: number; status: string };
	type Donor = {
		id: string;
		nik: string;
		nama: string;
		tanggal_lahir: string;
		jenis_kelamin: string;
		alamat: string | null;
		telepon: string | null;
		abo: string | null;
		rhesus: string | null;
		cekal_aktif: string | null;
		cekal: Cekal[];
		seleksi: Seleksi[];
		donasi: Donasi[];
	};
	type Aturan = Record<string, number>;

	const id = $derived(page.params.id ?? '');
	const donor = new Sumber(() => api<Donor>('GET', `/donor/${id}`));
	const aturan = new Sumber(() => api<Aturan>('GET', '/aturan-donor'));
	$effect(() => {
		if (id) donor.muat();
		aturan.muat();
	});

	const KEPUTUSAN: Record<string, [string, string]> = {
		diterima: ['Diterima', 'aman'],
		ditolak_sementara: ['Ditolak sementara', 'waspada'],
		ditolak_permanen: ['Ditolak permanen', 'bahaya']
	};
	const SUMBER_CEKAL: Record<string, string> = { seleksi: 'Seleksi', imltd: 'Uji saring IMLTD', manual: 'Manual' };

	function statusCekal(c: Cekal): [string, string] {
		if (c.dicabut_pada) return ['Dicabut', 'netral'];
		if (c.jenis === 'sementara' && c.sampai && c.sampai < hariIni()) return ['Berakhir', 'netral'];
		return ['Aktif', 'bahaya'];
	}

	const umur = (lahir: string) => {
		const l = new Date(lahir);
		const n = new Date();
		return n.getFullYear() - l.getFullYear() - (n < new Date(n.getFullYear(), l.getMonth(), l.getDate()) ? 1 : 0);
	};

	let tt: TandaTangan | undefined = $state();

	// ---------- Cabut cekal ----------
	const aksiCabut = new Aksi();
	let cabutId = $state<string | null>(null);
	let alasanCabut = $state('');
	function cabut(e: SubmitEvent) {
		e.preventDefault();
		const c = cabutId;
		if (!c) return;
		tt?.minta('Cabut cekal donor', 'Anda menandatangani pencabutan cekal ini. Donor dapat diseleksi kembali.', aksiCabut, async (sandi) => {
			const ok = await aksiCabut.jalankan(() => api('POST', `/cekal/${c}/cabut`, { alasan: alasanCabut, sandi }), 'Cekal dicabut.');
			if (ok) {
				cabutId = null;
				alasanCabut = '';
				donor.muat();
			}
		});
	}

	// ---------- Buat cekal ----------
	const aksiCekal = new Aksi();
	let cekal = $state({ jenis: 'sementara', sampai: '', alasan: '' });
	async function buatCekal(e: SubmitEvent) {
		e.preventDefault();
		const body = { jenis: cekal.jenis, alasan: cekal.alasan, sampai: cekal.jenis === 'sementara' ? cekal.sampai || null : null };
		if (await aksiCekal.jalankan(() => api('POST', `/donor/${id}/cekal`, body), 'Cekal dicatat.')) {
			cekal = { jenis: 'sementara', sampai: '', alasan: '' };
			donor.muat();
		}
	}

	// ---------- Seleksi ----------
	const aksiSeleksi = new Aksi();
	const kosongSeleksi = () => ({
		berat_kg: '',
		hb: '',
		sistolik: '',
		diastolik: '',
		nadi: '',
		suhu: '',
		kuesioner_lolos: '',
		keputusan: '',
		alasan: '',
		sampai: ''
	});
	let sel = $state(kosongSeleksi());

	/** Teks peringatan bila nilai di luar ambang aturan donor (pratinjau; server tetap memutuskan). */
	function luar(v: string, min?: number, maks?: number): string | undefined {
		if (v === '' || !aturan.data) return undefined;
		const n = Number(v);
		if (min !== undefined && n < min) return `Di bawah ambang (${min}).`;
		if (maks !== undefined && n > maks) return `Di atas ambang (${maks}).`;
		return undefined;
	}

	async function seleksi(e: SubmitEvent) {
		e.preventDefault();
		const body = {
			berat_kg: Number(sel.berat_kg),
			hb: Number(sel.hb),
			sistolik: Number(sel.sistolik),
			diastolik: Number(sel.diastolik),
			nadi: Number(sel.nadi),
			suhu: Number(sel.suhu),
			kuesioner_lolos: sel.kuesioner_lolos === 'ya',
			keputusan: sel.keputusan,
			alasan: sel.keputusan === 'diterima' ? null : sel.alasan || null,
			sampai: sel.keputusan === 'ditolak_sementara' ? sel.sampai || null : null
		};
		const pesan = sel.keputusan === 'diterima' ? 'Seleksi dicatat: donor diterima. Lanjutkan pengambilan darah.' : 'Seleksi dicatat; donor dicekal otomatis.';
		if (await aksiSeleksi.jalankan(() => api('POST', `/donor/${id}/seleksi`, body), pesan)) {
			sel = kosongSeleksi();
			donor.muat();
		}
	}

	// Seleksi diterima dalam 12 jam terakhir yang belum diambil darahnya.
	const siapAmbil = $derived(
		donor.data?.seleksi.find(
			(s) => s.keputusan === 'diterima' && !s.donasi_id && Date.now() - new Date(s.waktu).getTime() < 12 * 3600_000
		)
	);

	// ---------- Ambil darah ----------
	const aksiAmbil = new Aksi();
	let ambil = $state({ nomor: '', jenis: 'sukarela', untuk_no_rm: '', volume_ml: '350', jenis_kantong: 'double', catatan: '' });
	function pindai(v: string) {
		const p = urai(v);
		ambil.nomor = p.jenis === 'din' ? p.nomor_donasi : p.jenis === 'lain' ? p.teks : '';
		aksiAmbil.isian = p.jenis === 'din' || p.jenis === 'lain' ? {} : { nomor: 'Yang dipindai bukan nomor donasi. Pindai barcode nomor donasi pada label kantong.' };
	}
	async function ambilDarah(e: SubmitEvent) {
		e.preventDefault();
		const s = siapAmbil;
		if (!s) return;
		if (!ambil.nomor) {
			aksiAmbil.isian = { nomor: 'Pindai nomor donasi dari label kantong.' };
			return;
		}
		const body = {
			nomor: ambil.nomor,
			jenis: ambil.jenis,
			untuk_no_rm: ambil.jenis === 'pengganti' ? ambil.untuk_no_rm : null,
			volume_ml: Number(ambil.volume_ml),
			jenis_kantong: ambil.jenis_kantong,
			catatan: ambil.catatan || null
		};
		const d = await aksiAmbil.jalankan(() => api<{ id: string }>('POST', `/seleksi/${s.id}/donasi`, body));
		if (d) goto(`/donasi/${d.id}`);
	}
</script>

<svelte:head><title>{donor.data?.nama ?? 'Donor'} — {merek.judul}</title></svelte:head>

<p class="mb-2 text-sm"><a class="text-utama hover:underline" href="/donor">← Cari donor</a></p>

<Keadaan sumber={donor} apakahKosong={() => false}>
	{#snippet children(d)}
		{@const a = aturan.data}
		<h1 class="mb-1">{d.nama}</h1>
		<p class="mb-4 text-sm text-redup">
			NIK <span class="mono">{d.nik}</span> · {d.jenis_kelamin === 'L' ? 'Laki-laki' : 'Perempuan'} · lahir {tanggal(d.tanggal_lahir)} ({umur(d.tanggal_lahir)} tahun) · golongan
			<span class="font-semibold text-teks">{golongan(d.abo, d.rhesus)}</span>
			{#if d.telepon}· {d.telepon}{/if}
			{#if d.alamat}· {d.alamat}{/if}
		</p>

		{#if d.cekal_aktif}
			<div class="peringatan mb-4" role="alert">
				<Ban class="mt-0.5 size-5 shrink-0" aria-hidden="true" />
				<div>
					<p>Donor sedang dicekal ({d.cekal_aktif}).</p>
					<p class="text-sm font-normal">Donor tidak boleh diseleksi atau diambil darahnya, dan kantong dari donor ini tidak dapat diluluskan.</p>
				</div>
			</div>
		{/if}

		<div class="grid gap-4 xl:grid-cols-2">
			{#if siapAmbil && !d.cekal_aktif}
				<form class="panel flex flex-col gap-4 xl:col-span-2" onsubmit={ambilDarah} aria-labelledby="judul-ambil">
					<h2 id="judul-ambil" class="flex items-center gap-2"><Droplet class="size-5 text-bahaya" aria-hidden="true" />Pengambilan darah</h2>
					<p class="text-sm text-redup">Seleksi diterima {waktu(siapAmbil.waktu)}. Darah harus diambil dalam 12 jam setelah seleksi.</p>
					<div class="form-grid">
						<div>
							<Pindai label="Nomor donasi" id="nomor-donasi" onpindai={pindai} autofocus />
							{#if aksiAmbil.isian.nomor}
								<p class="mt-1 text-sm text-bahaya" role="alert">{aksiAmbil.isian.nomor}</p>
							{:else if ambil.nomor}
								<p class="mt-1 text-sm">Nomor donasi: <span class="mono font-semibold">{ambil.nomor}</span></p>
							{:else}
								<p class="mt-1 text-xs text-redup">Pindai barcode nomor donasi pada label kantong.</p>
							{/if}
						</div>
						<fieldset>
							<legend class="label">Jenis donasi</legend>
							<div class="flex min-h-10 items-center gap-4">
								<label class="flex items-center gap-2"><input type="radio" name="jenis-donasi" value="sukarela" bind:group={ambil.jenis} />Sukarela</label>
								<label class="flex items-center gap-2"><input type="radio" name="jenis-donasi" value="pengganti" bind:group={ambil.jenis} />Pengganti</label>
							</div>
						</fieldset>
						{#if ambil.jenis === 'pengganti'}
							<Isian label="No. RM pasien yang dibantu" id="untuk-rm" galat={aksiAmbil.isian.untuk_no_rm} bantuan="Pasien harus sudah terdaftar di utd-rs.">
								<input id="untuk-rm" class="input mono" bind:value={ambil.untuk_no_rm} required />
							</Isian>
						{/if}
						<Isian label="Volume (mL)" id="volume" galat={aksiAmbil.isian.volume_ml} bantuan="100–550 mL.">
							<input id="volume" class="input" type="number" min="100" max="550" bind:value={ambil.volume_ml} required />
						</Isian>
						<Isian label="Jenis kantong" id="jenis-kantong" galat={aksiAmbil.isian.jenis_kantong}>
							<select id="jenis-kantong" class="input" bind:value={ambil.jenis_kantong}>
								<option value="single">Single</option>
								<option value="double">Double</option>
								<option value="triple">Triple</option>
								<option value="quadruple">Quadruple</option>
							</select>
						</Isian>
						<Isian label="Catatan (opsional)" id="catatan-ambil">
							<input id="catatan-ambil" class="input" bind:value={ambil.catatan} />
						</Isian>
					</div>
					<PesanAksi aksi={aksiAmbil} />
					<button class="btn btn--utama self-start" disabled={aksiAmbil.sibuk}>Catat donasi</button>
				</form>
			{/if}

			<form class="panel flex flex-col gap-4 xl:col-span-2" onsubmit={seleksi} aria-labelledby="judul-seleksi">
				<h2 id="judul-seleksi">Seleksi donor</h2>
				{#if d.cekal_aktif}
					<p class="text-sm text-redup">Seleksi tidak bisa dicatat selama donor dicekal.</p>
				{:else}
					{#if a}
						<p class="text-sm text-redup">
							Ambang: umur {a.umur_min}–{a.umur_maks} tahun, jarak antardonasi minimal {a.hari_antar_donasi} hari
							{#if d.donasi[0]}(donasi terakhir {tanggal(d.donasi[0].waktu)}){/if}.
						</p>
					{/if}
					<fieldset class="grid gap-4 sm:grid-cols-3">
						<legend class="sr-only">Pemeriksaan</legend>
						<Isian label="Berat badan (kg)" id="berat" galat={aksiSeleksi.isian.berat_kg ?? luar(sel.berat_kg, a?.berat_min)} bantuan={a ? `Minimal ${a.berat_min} kg.` : undefined}>
							<input id="berat" class="input" type="number" step="0.1" bind:value={sel.berat_kg} required />
						</Isian>
						<Isian label="Hb (g/dL)" id="hb" galat={luar(sel.hb, a?.hb_min, a?.hb_maks)} bantuan={a ? `${a.hb_min}–${a.hb_maks} g/dL.` : undefined}>
							<input id="hb" class="input" type="number" step="0.1" bind:value={sel.hb} required />
						</Isian>
						<Isian label="Suhu (°C)" id="suhu" galat={luar(sel.suhu, undefined, a?.suhu_maks)} bantuan={a ? `Maksimal ${a.suhu_maks} °C.` : undefined}>
							<input id="suhu" class="input" type="number" step="0.1" bind:value={sel.suhu} required />
						</Isian>
						<Isian label="Sistolik (mmHg)" id="sistolik" galat={luar(sel.sistolik, a?.sistolik_min, a?.sistolik_maks)} bantuan={a ? `${a.sistolik_min}–${a.sistolik_maks} mmHg.` : undefined}>
							<input id="sistolik" class="input" type="number" bind:value={sel.sistolik} required />
						</Isian>
						<Isian label="Diastolik (mmHg)" id="diastolik" galat={luar(sel.diastolik, a?.diastolik_min, a?.diastolik_maks)} bantuan={a ? `${a.diastolik_min}–${a.diastolik_maks} mmHg.` : undefined}>
							<input id="diastolik" class="input" type="number" bind:value={sel.diastolik} required />
						</Isian>
						<Isian label="Nadi (/menit)" id="nadi" galat={luar(sel.nadi, a?.nadi_min, a?.nadi_maks)} bantuan={a ? `${a.nadi_min}–${a.nadi_maks} /menit.` : undefined}>
							<input id="nadi" class="input" type="number" bind:value={sel.nadi} required />
						</Isian>
					</fieldset>
					<div class="form-grid">
						<fieldset>
							<legend class="label">Kuesioner riwayat kesehatan</legend>
							<div class="flex min-h-10 items-center gap-4">
								<label class="flex items-center gap-2"><input type="radio" name="kuesioner" value="ya" bind:group={sel.kuesioner_lolos} required />Lolos</label>
								<label class="flex items-center gap-2"><input type="radio" name="kuesioner" value="tidak" bind:group={sel.kuesioner_lolos} />Tidak lolos</label>
							</div>
						</fieldset>
						<Isian label="Keputusan" id="keputusan" galat={aksiSeleksi.isian.keputusan}>
							<select id="keputusan" class="input" bind:value={sel.keputusan} required aria-invalid={!!aksiSeleksi.isian.keputusan}>
								<option value="" disabled>Pilih keputusan</option>
								<option value="diterima">Diterima</option>
								<option value="ditolak_sementara">Ditolak sementara</option>
								<option value="ditolak_permanen">Ditolak permanen</option>
							</select>
						</Isian>
						{#if sel.keputusan === 'ditolak_sementara'}
							<Isian label="Ditolak sampai tanggal" id="sel-sampai" galat={aksiSeleksi.isian.sampai} bantuan="Donor dicekal sampai tanggal ini.">
								<input id="sel-sampai" class="input" type="date" min={hariIni()} bind:value={sel.sampai} required />
							</Isian>
						{/if}
						{#if sel.keputusan && sel.keputusan !== 'diterima'}
							<Isian label="Alasan penolakan" id="sel-alasan" galat={aksiSeleksi.isian.alasan} bantuan="Kosongkan untuk memakai daftar nilai di luar ambang.">
								<input id="sel-alasan" class="input" bind:value={sel.alasan} />
							</Isian>
						{/if}
					</div>
					{#if sel.keputusan && sel.keputusan !== 'diterima'}
						<p class="catatan">Penolakan otomatis membuat cekal {sel.keputusan === 'ditolak_permanen' ? 'permanen' : 'sementara'} pada donor.</p>
					{/if}
					<PesanAksi aksi={aksiSeleksi} />
					<button class="btn btn--utama self-start" disabled={aksiSeleksi.sibuk}>Simpan seleksi</button>
				{/if}
			</form>

			<section class="panel" aria-labelledby="judul-cekal">
				<h2 id="judul-cekal" class="mb-3">Riwayat cekal</h2>
				<PesanAksi aksi={aksiCabut} />
				{#if d.cekal.length === 0}
					<p class="text-sm text-redup">Donor belum pernah dicekal.</p>
				{:else}
					<ul class="divide-y divide-halus">
						{#each d.cekal as c (c.id)}
							{@const [lbl, nada] = statusCekal(c)}
							<li class="py-2">
								<div class="flex items-start justify-between gap-3">
									<div>
										<p class="font-medium">{c.jenis === 'permanen' ? 'Permanen' : `Sementara sampai ${tanggal(c.sampai)}`}</p>
										<p class="text-sm">{c.alasan}</p>
										<p class="text-xs text-redup">Mulai {tanggal(c.mulai)} · sumber: {SUMBER_CEKAL[c.sumber] ?? c.sumber}</p>
										{#if c.dicabut_pada}
											<p class="text-xs text-redup">Dicabut {waktu(c.dicabut_pada)}: {c.alasan_cabut}</p>
										{/if}
									</div>
									<div class="flex flex-col items-end gap-2">
										<Chip {nada}>{lbl}</Chip>
										{#if !c.dicabut_pada && sesi.boleh('donor.lulus') && cabutId !== c.id}
											<button type="button" class="btn btn--teks" onclick={() => ((cabutId = c.id), (alasanCabut = ''))}>Cabut</button>
										{/if}
									</div>
								</div>
								{#if cabutId === c.id}
									<form class="mt-2 flex flex-col gap-2" onsubmit={cabut}>
										<Isian label="Alasan pencabutan" id="alasan-cabut-{c.id}" galat={aksiCabut.isian.alasan} bantuan="Mis. hasil uji konfirmasi negatif.">
											<input id="alasan-cabut-{c.id}" class="input" bind:value={alasanCabut} required />
										</Isian>
										<div class="flex gap-2">
											<button class="btn btn--utama">Tanda tangani pencabutan</button>
											<button type="button" class="btn" onclick={() => (cabutId = null)}>Batal</button>
										</div>
									</form>
								{/if}
							</li>
						{/each}
					</ul>
				{/if}
			</section>

			<form class="panel flex flex-col gap-4" onsubmit={buatCekal} aria-labelledby="judul-buat-cekal">
				<h2 id="judul-buat-cekal">Cekal donor</h2>
				<fieldset>
					<legend class="label">Jenis cekal</legend>
					<div class="flex min-h-10 items-center gap-4">
						<label class="flex items-center gap-2"><input type="radio" name="jenis-cekal" value="sementara" bind:group={cekal.jenis} />Sementara</label>
						<label class="flex items-center gap-2"><input type="radio" name="jenis-cekal" value="permanen" bind:group={cekal.jenis} />Permanen</label>
					</div>
				</fieldset>
				{#if cekal.jenis === 'sementara'}
					<Isian label="Sampai tanggal" id="cekal-sampai" galat={aksiCekal.isian.sampai}>
						<input id="cekal-sampai" class="input" type="date" min={hariIni()} bind:value={cekal.sampai} required />
					</Isian>
				{/if}
				<Isian label="Alasan" id="cekal-alasan" galat={aksiCekal.isian.alasan}>
					<input id="cekal-alasan" class="input" bind:value={cekal.alasan} required />
				</Isian>
				<PesanAksi aksi={aksiCekal} />
				<button class="btn btn--bahaya self-start" disabled={aksiCekal.sibuk}><Ban class="size-4" aria-hidden="true" />Cekal donor</button>
			</form>

			<section class="panel xl:col-span-2" aria-labelledby="judul-riwayat">
				<h2 id="judul-riwayat" class="mb-3">Riwayat seleksi dan donasi</h2>
				{#if d.seleksi.length === 0}
					<p class="text-sm text-redup">Belum ada seleksi.</p>
				{:else}
					<div class="overflow-x-auto">
						<table class="tabel">
							<thead>
								<tr>
									<th scope="col">Waktu</th>
									<th scope="col">Hb</th>
									<th scope="col">Tekanan darah</th>
									<th scope="col">Nadi</th>
									<th scope="col">Berat</th>
									<th scope="col">Suhu</th>
									<th scope="col">Kuesioner</th>
									<th scope="col">Keputusan</th>
									<th scope="col">Donasi</th>
								</tr>
							</thead>
							<tbody>
								{#each d.seleksi as s (s.id)}
									{@const k = KEPUTUSAN[s.keputusan]}
									{@const n = d.donasi.find((x) => x.id === s.donasi_id)}
									<tr>
										<td class="whitespace-nowrap">{waktu(s.waktu)}</td>
										<td class="tabular-nums">{s.hb}</td>
										<td class="tabular-nums">{s.sistolik}/{s.diastolik}</td>
										<td class="tabular-nums">{s.nadi}</td>
										<td class="tabular-nums">{s.berat_kg} kg</td>
										<td class="tabular-nums">{s.suhu} °C</td>
										<td>{s.kuesioner_lolos ? 'Lolos' : 'Tidak lolos'}</td>
										<td>
											<Chip nada={k?.[1]}>{k?.[0] ?? s.keputusan}</Chip>
											{#if s.alasan}<span class="block text-xs text-redup">{s.alasan}</span>{/if}
										</td>
										<td>
											{#if n}
												<a class="mono text-utama hover:underline" href="/donasi/{n.id}">{n.nomor}</a>
												<span class="block text-xs text-redup">{n.jenis === 'pengganti' ? `Pengganti (${n.untuk_no_rm})` : 'Sukarela'} · {n.volume_ml} mL</span>
											{:else}
												<span class="text-redup">—</span>
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
