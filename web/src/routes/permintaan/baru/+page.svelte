<script lang="ts">
	import { merek } from '#lib/merek.svelte.ts';
	import { goto } from '$app/navigation';
	import { api } from '#lib/api.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Isian from '#lib/Isian.svelte';
	import { sesi } from '#lib/sesi.svelte.ts';
	import { NAMA_KOMPONEN, golongan, hariIni, lokal, tanggal } from '#lib/format.ts';
	import Pilihan from '#lib/pelayanan/Pilihan.svelte';
	import { ABO, JENIS_PERMINTAAN, RH, YA_TIDAK, angka } from '#lib/pelayanan/darah.ts';
	import Plus from '@lucide/svelte/icons/plus';
	import Trash from '@lucide/svelte/icons/trash-2';

	type Pasien = { no_rm: string; nama: string; tanggal_lahir: string; jenis_kelamin: string };

	// Pasien: pilih dari hasil pencarian, atau isi identitas baru.
	let q = $state('');
	let hasil = $state<Pasien[]>([]);
	let mencari = $state(false);
	let pasien = $state<Pasien | null>(null);
	let baru = $state(false);
	let identitas = $state({ no_rm: '', nama: '', tanggal_lahir: '', jenis_kelamin: '' });

	$effect(() => {
		const kata = q.trim();
		if (kata.length < 3) {
			hasil = [];
			return;
		}
		const t = setTimeout(async () => {
			mencari = true;
			try {
				hasil = await api<Pasien[]>('GET', `/pasien?q=${encodeURIComponent(kata)}`);
			} catch {
				hasil = [];
			} finally {
				mencari = false;
			}
		}, 250);
		return () => clearTimeout(t);
	});

	const dokterSendiri = $derived(!!sesi.profil?.peran.includes('dokter'));
	const f = $state({
		ruang: '',
		dokter: '',
		diagnosis: '',
		indikasi: '',
		jenis: 'rutin',
		alasan_darurat: '',
		hb: '' as string | number,
		trombosit: '' as string | number,
		abo_klinis: '',
		rhesus_klinis: '',
		riwayat_transfusi: undefined as boolean | undefined,
		riwayat_reaksi: undefined as boolean | undefined,
		rencana_transfusi: hariIni()
	});
	let baris = $state([{ komponen: 'prc', jumlah: 1 as number, volume_ml: '' as string | number }]);
	const maks = lokal(new Date(Date.now() + 3 * 86_400_000)).slice(0, 10);

	const aksi = new Aksi();
	async function kirim(e: SubmitEvent) {
		e.preventDefault();
		if (!pasien && !baru) {
			aksi.galat = 'Pilih pasien dari hasil pencarian atau isi identitas pasien baru.';
			return;
		}
		const r = await aksi.jalankan(() =>
			api<{ id: string }>('POST', '/permintaan', {
				...(baru ? { pasien: identitas } : { no_rm: pasien?.no_rm }),
				ruang: f.ruang,
				dokter: dokterSendiri ? undefined : f.dokter,
				diagnosis: f.diagnosis,
				indikasi: f.indikasi,
				jenis: f.jenis,
				alasan_darurat: f.jenis === 'darurat' ? f.alasan_darurat : undefined,
				hb: angka(f.hb),
				trombosit: angka(f.trombosit),
				abo_klinis: f.abo_klinis || undefined,
				rhesus_klinis: f.rhesus_klinis || undefined,
				riwayat_transfusi: f.riwayat_transfusi,
				riwayat_reaksi: f.riwayat_reaksi,
				rencana_transfusi: f.rencana_transfusi || undefined,
				baris: baris.map((b) => ({ komponen: b.komponen, jumlah: Number(b.jumlah), volume_ml: angka(b.volume_ml) }))
			})
		);
		if (r) goto(`/permintaan/${r.id}`);
	}
</script>

<svelte:head><title>Buat permintaan darah — {merek.judul}</title></svelte:head>

<h1 class="mb-1">Buat permintaan darah</h1>
<p class="mb-4 text-sm text-redup">Isian sesuai formulir permintaan darah (Permenkes 91/2015 §5.6.1). Kirim sampel darah pasien bersama permintaan.</p>

<form class="flex max-w-4xl flex-col gap-4" onsubmit={kirim} novalidate>
	<section class="panel flex flex-col gap-4" aria-labelledby="j-pasien">
		<h2 id="j-pasien">Pasien</h2>
		{#if pasien}
			<div class="flex flex-wrap items-center justify-between gap-3 rounded-md bg-halus p-3">
				<p>
					<span class="font-semibold">{pasien.nama}</span>
					<span class="mono text-sm">{pasien.no_rm}</span>
					<span class="block text-sm text-redup">Lahir {tanggal(pasien.tanggal_lahir)} · {pasien.jenis_kelamin === 'L' ? 'Laki-laki' : 'Perempuan'}</span>
				</p>
				<button type="button" class="btn" onclick={() => (pasien = null)}>Ganti pasien</button>
			</div>
		{:else if baru}
			<div class="form-grid">
				<Isian label="No. rekam medis" id="no_rm" galat={aksi.isian.no_rm}>
					<input id="no_rm" class="input mono" bind:value={identitas.no_rm} required />
				</Isian>
				<Isian label="Nama pasien" id="nama" galat={aksi.isian.nama}>
					<input id="nama" class="input" bind:value={identitas.nama} required />
				</Isian>
				<Isian label="Tanggal lahir" id="tanggal_lahir" galat={aksi.isian.tanggal_lahir}>
					<input id="tanggal_lahir" class="input" type="date" max={hariIni()} bind:value={identitas.tanggal_lahir} required />
				</Isian>
				<Pilihan
					legend="Jenis kelamin"
					name="jenis_kelamin"
					opsi={[
						['L', 'Laki-laki'],
						['P', 'Perempuan']
					]}
					bind:value={identitas.jenis_kelamin}
					galat={aksi.isian.jenis_kelamin}
				/>
			</div>
			<button type="button" class="btn btn--teks self-start" onclick={() => (baru = false)}>Cari pasien terdaftar</button>
		{:else}
			<Isian label="Cari pasien (no. RM atau nama, minimal 3 huruf)" id="cari" galat={aksi.isian.no_rm}>
				<input id="cari" class="input" type="search" autocomplete="off" bind:value={q} />
			</Isian>
			<div aria-live="polite">
				{#if mencari}
					<p class="text-sm text-redup">Mencari…</p>
				{:else if q.trim().length >= 3 && !hasil.length}
					<p class="text-sm text-redup">Pasien tidak ditemukan. Isi identitas pasien baru bila belum pernah terdaftar.</p>
				{/if}
				{#if hasil.length}
					<ul class="divide-y divide-halus rounded-md border border-garis">
						{#each hasil as h (h.no_rm)}
							<li>
								<button type="button" class="flex min-h-11 w-full flex-wrap items-center gap-x-3 px-3 py-2 text-left hover:bg-halus" onclick={() => (pasien = h)}>
									<span class="font-medium">{h.nama}</span>
									<span class="mono text-sm">{h.no_rm}</span>
									<span class="text-sm text-redup">lahir {tanggal(h.tanggal_lahir)} · {h.jenis_kelamin}</span>
								</button>
							</li>
						{/each}
					</ul>
				{/if}
			</div>
			<button type="button" class="btn self-start" onclick={() => (baru = true)}>Pasien baru</button>
		{/if}
	</section>

	<section class="panel flex flex-col gap-4" aria-labelledby="j-klinis">
		<h2 id="j-klinis">Data klinis</h2>
		<div class="form-grid">
			<Isian label="Ruang perawatan" id="ruang" galat={aksi.isian.ruang}>
				<input id="ruang" class="input" bind:value={f.ruang} required />
			</Isian>
			{#if dokterSendiri}
				<div>
					<p class="label">Dokter peminta</p>
					<p class="text-sm">{sesi.profil?.nama} (akun Anda)</p>
				</div>
			{:else}
				<Isian label="Nama DPJP/dokter peminta" id="dokter" galat={aksi.isian.dokter}>
					<input id="dokter" class="input" bind:value={f.dokter} required />
				</Isian>
			{/if}
			<Isian label="Diagnosis klinis" id="diagnosis" galat={aksi.isian.diagnosis}>
				<input id="diagnosis" class="input" bind:value={f.diagnosis} required />
			</Isian>
			<Isian label="Indikasi transfusi" id="indikasi" galat={aksi.isian.indikasi}>
				<input id="indikasi" class="input" bind:value={f.indikasi} required />
			</Isian>
			<Isian label="Hb terakhir (g/dL)" id="hb" galat={aksi.isian.hb} bantuan="Isi Hb atau trombosit, minimal salah satu.">
				<input id="hb" class="input" type="number" step="0.1" min="0" bind:value={f.hb} />
			</Isian>
			<Isian label="Trombosit terakhir (/µL)" id="trombosit" galat={aksi.isian.trombosit}>
				<input id="trombosit" class="input" type="number" step="1000" min="0" bind:value={f.trombosit} />
			</Isian>
			<Isian label="Golongan ABO menurut catatan klinis (bila ada)" id="abo_klinis" galat={aksi.isian.abo_klinis}>
				<select id="abo_klinis" class="input" bind:value={f.abo_klinis}>
					<option value="">Tidak diketahui</option>
					{#each ABO as a (a)}<option value={a}>{a}</option>{/each}
				</select>
			</Isian>
			<Isian label="Rhesus menurut catatan klinis (bila ada)" id="rhesus_klinis">
				<select id="rhesus_klinis" class="input" bind:value={f.rhesus_klinis}>
					<option value="">Tidak diketahui</option>
					{#each RH as [v, l] (v)}<option value={v}>{l}</option>{/each}
				</select>
			</Isian>
			<Pilihan legend="Pernah transfusi sebelumnya?" name="riwayat_transfusi" opsi={YA_TIDAK} bind:value={f.riwayat_transfusi} galat={aksi.isian.riwayat_transfusi} />
			<Pilihan legend="Pernah reaksi transfusi?" name="riwayat_reaksi" opsi={YA_TIDAK} bind:value={f.riwayat_reaksi} galat={aksi.isian.riwayat_reaksi} />
		</div>
		{#if f.abo_klinis}
			<p class="text-xs text-redup">Golongan klinis {golongan(f.abo_klinis, f.rhesus_klinis)} hanya keterangan; bank darah tetap memeriksa golongan dari sampel.</p>
		{/if}
	</section>

	<section class="panel flex flex-col gap-4" aria-labelledby="j-minta">
		<h2 id="j-minta">Darah yang diminta</h2>
		<div class="form-grid">
			<Pilihan legend="Jenis permintaan" name="jenis" opsi={Object.entries(JENIS_PERMINTAAN).map(([k, [l]]) => [k, l] as [string, string])} bind:value={f.jenis} galat={aksi.isian.jenis} />
			<Isian label="Rencana transfusi" id="rencana" galat={aksi.isian.rencana_transfusi} bantuan="Paling lambat 3 hari dari hari ini.">
				<input id="rencana" class="input" type="date" min={hariIni()} max={maks} bind:value={f.rencana_transfusi} required />
			</Isian>
		</div>
		{#if f.jenis === 'darurat'}
			<Isian label="Alasan permintaan darurat" id="alasan_darurat" galat={aksi.isian.alasan_darurat} bantuan="Wajib diisi. Darah darurat dapat dikeluarkan sebelum uji silang selesai dengan persetujuan dokter.">
				<textarea id="alasan_darurat" class="input" rows="2" bind:value={f.alasan_darurat} required></textarea>
			</Isian>
		{/if}

		<fieldset class="flex flex-col gap-3" aria-describedby={aksi.isian.baris ? 'baris-galat' : undefined}>
			<legend class="label">Komponen dan jumlah kantong</legend>
			{#each baris as b, i (i)}
				<div class="grid items-end gap-3 rounded-md border border-garis p-3 sm:grid-cols-[2fr_1fr_1fr_auto]">
					<div>
						<label class="label" for="komponen-{i}">Komponen</label>
						<select id="komponen-{i}" class="input" bind:value={b.komponen}>
							{#each Object.entries(NAMA_KOMPONEN) as [k, l] (k)}<option value={k}>{l}</option>{/each}
						</select>
					</div>
					<div>
						<label class="label" for="jumlah-{i}">Jumlah kantong</label>
						<input id="jumlah-{i}" class="input" type="number" min="1" max="20" bind:value={b.jumlah} required />
					</div>
					<div>
						<label class="label" for="volume-{i}">Volume (mL, opsional)</label>
						<input id="volume-{i}" class="input" type="number" min="1" bind:value={b.volume_ml} />
					</div>
					<button
						type="button"
						class="btn btn--teks"
						disabled={baris.length === 1}
						onclick={() => baris.splice(i, 1)}
						aria-label="Hapus baris {i + 1}"><Trash class="size-4" aria-hidden="true" /></button
					>
				</div>
			{/each}
			{#if aksi.isian.baris}<p class="text-sm text-bahaya" id="baris-galat" role="alert">{aksi.isian.baris}</p>{/if}
			<button type="button" class="btn self-start" onclick={() => baris.push({ komponen: 'ffp', jumlah: 1, volume_ml: '' })}>
				<Plus class="size-4" aria-hidden="true" />Tambah komponen
			</button>
		</fieldset>
	</section>

	<PesanAksi {aksi} />
	<div class="flex gap-2">
		<button class="btn btn--utama btn--besar" disabled={aksi.sibuk}>{aksi.sibuk ? 'Mengirim…' : 'Kirim permintaan'}</button>
		<a class="btn btn--besar" href="/permintaan">Batal</a>
	</div>
</form>
