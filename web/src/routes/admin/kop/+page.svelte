<!-- Pengaturan kop surat untuk semua dokumen cetak (pola sama dengan lis-rebuild): teks identitas, dua logo, pratinjau. -->
<script lang="ts">
	import { merek } from '#lib/merek.svelte.ts';
	import { api } from '#lib/api.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Isian from '#lib/Isian.svelte';
	import Kop from '#lib/Kop.svelte';
	import { kopSurat, type Kop as DataKop } from '#lib/kop.svelte.ts';
	import { sesi } from '#lib/sesi.svelte.ts';
	import ImagePlus from '@lucide/svelte/icons/image-plus';
	import X from '@lucide/svelte/icons/x';

	const LOGO_MAKS = 512 * 1024;
	const TEKS: { k: keyof DataKop; label: string; bantuan: string; lebar?: boolean }[] = [
		{ k: 'nama_rs', label: 'Nama rumah sakit', bantuan: 'Kosong = memakai nama di profil instalasi.', lebar: true },
		{ k: 'alamat', label: 'Alamat', bantuan: 'Jalan, kecamatan, kabupaten/kota.', lebar: true },
		{ k: 'pemerintah', label: 'Pemerintah daerah', bantuan: 'Baris kecil di atas nama RS. Boleh dikosongkan.' },
		{ k: 'unit', label: 'Unit', bantuan: 'Mis. Instalasi Bank Darah. Boleh dikosongkan.' },
		{ k: 'kontak', label: 'Kontak', bantuan: 'Telepon, surel, atau situs. Boleh dikosongkan.' },
		{ k: 'akreditasi', label: 'Akreditasi', bantuan: 'Mis. Terakreditasi Paripurna. Boleh dikosongkan.' }
	];
	const LOGO: { k: 'logo_kiri' | 'logo_kanan'; label: string }[] = [
		{ k: 'logo_kiri', label: 'Logo kiri' },
		{ k: 'logo_kanan', label: 'Logo kanan' }
	];

	let isi = $state<DataKop | null>(null);
	$effect(() => {
		if (!kopSurat.data) kopSurat.muat();
	});
	$effect(() => {
		if (kopSurat.data && !isi) isi = { ...kopSurat.data };
	});
	const boleh = $derived(sesi.boleh('master.kelola'));
	const berubah = $derived(!!isi && !!kopSurat.data && (Object.keys(isi) as (keyof DataKop)[]).some((k) => isi![k] !== kopSurat.data![k]));
	const aksi = new Aksi();

	function pilihLogo(k: 'logo_kiri' | 'logo_kanan', f: File | undefined) {
		if (!f || !isi) return;
		aksi.isian = { ...aksi.isian, [k]: '' };
		if (!['image/png', 'image/jpeg'].includes(f.type)) return (aksi.isian = { ...aksi.isian, [k]: 'Logo harus berkas PNG atau JPEG.' });
		if (f.size > LOGO_MAKS) return (aksi.isian = { ...aksi.isian, [k]: 'Logo paling besar 512 KB.' });
		const r = new FileReader();
		r.onload = () => isi && (isi[k] = String(r.result));
		r.readAsDataURL(f);
	}

	async function simpan(e: SubmitEvent) {
		e.preventDefault();
		if (!isi) return;
		const { nama_rs_profil: _, ...kirim } = isi;
		const r = await aksi.jalankan(() => api<DataKop>('PUT', '/master/kop', kirim), 'Kop surat disimpan. Berlaku untuk semua dokumen cetak.');
		if (r) {
			kopSurat.data = { ...r, nama_rs_profil: kopSurat.data?.nama_rs_profil };
			isi = { ...kopSurat.data };
			merek.muat(); // judul dan logo aplikasi mengikuti kop
		}
	}
</script>

<svelte:head><title>Kop surat — {merek.judul}</title></svelte:head>

<div class="mb-5">
	<h1>Kop surat</h1>
	<p class="mt-1 font-medium text-redup">
		Juga menjadi judul dan logo aplikasi (logo kanan; kosong = logo kiri). Dipakai di semua dokumen cetak: formulir permintaan, berita acara, serah terima, laporan, dan lembar darurat. Setiap perubahan tercatat di jejak audit.
	</p>
</div>

<Keadaan sumber={kopSurat} apakahKosong={() => false} baris={6}>
	{#snippet children()}
		{#if isi}
			<form class="flex flex-col gap-5" onsubmit={simpan}>
				<section class="panel" aria-labelledby="j-pratinjau">
					<h2 id="j-pratinjau" class="mb-1">Pratinjau</h2>
					<p class="mb-4 text-sm text-redup">Kira-kira seperti di kertas A4. Tampilan cetak sebenarnya mengikuti dokumen.</p>
					<div class="rounded-xl border border-garis p-6">
						<Kop kop={isi} />
						<p class="mt-4 text-center text-sm font-bold">BERITA ACARA PEMUSNAHAN KANTONG DARAH</p>
						<p class="mono text-center text-xs">BA-261118-0001</p>
					</div>
				</section>

				<fieldset class="panel flex flex-col gap-5" disabled={!boleh}>
					<legend class="sr-only">Isi kop</legend>
					<h2>Identitas</h2>
					<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
						{#each TEKS as t (t.k)}
							<div class={t.lebar ? 'sm:col-span-2 lg:col-span-3' : ''}>
								<Isian label={t.label} id="kop-{t.k}" galat={aksi.isian[t.k]} bantuan={t.bantuan}>
									<input
										id="kop-{t.k}"
										class="input"
										maxlength="200"
										placeholder={t.k === 'nama_rs' ? kopSurat.data?.nama_rs_profil : ''}
										aria-invalid={!!aksi.isian[t.k]}
										bind:value={isi[t.k]}
									/>
								</Isian>
							</div>
						{/each}
					</div>

					<h2>Logo</h2>
					<div class="grid gap-4 sm:grid-cols-2">
						{#each LOGO as l (l.k)}
							<div>
								<p class="label" id="{l.k}-label">{l.label}</p>
								<div class="flex items-center gap-4">
									<label
										for="kop-{l.k}"
										class="grid size-24 shrink-0 cursor-pointer place-items-center overflow-hidden rounded-xl border-2 border-dashed border-garis-kuat bg-latar p-2 hover:border-fokus"
									>
										{#if isi[l.k]}
											<img src={isi[l.k]} alt="Pratinjau {l.label.toLowerCase()}" class="max-h-full max-w-full object-contain" />
										{:else}
											<span class="flex flex-col items-center gap-1 text-xs font-semibold text-redup"><ImagePlus class="size-5" aria-hidden="true" />Pilih</span>
										{/if}
										<input
											id="kop-{l.k}"
											type="file"
											class="sr-only"
											accept="image/png,image/jpeg"
											aria-describedby="{l.k}-bantuan"
											onchange={(e) => {
												pilihLogo(l.k, e.currentTarget.files?.[0]);
												e.currentTarget.value = '';
											}}
										/>
									</label>
									<div class="flex flex-col items-start gap-2">
										<label for="kop-{l.k}" class="btn cursor-pointer"><ImagePlus class="size-4" aria-hidden="true" />{isi[l.k] ? 'Ganti gambar' : 'Unggah gambar'}</label>
										{#if isi[l.k]}
											<button type="button" class="btn btn--teks" onclick={() => isi && (isi[l.k] = '')}><X class="size-4" aria-hidden="true" />Kosongkan</button>
										{/if}
										<p id="{l.k}-bantuan" class="text-xs text-redup">PNG/JPEG, maks. 512 KB. Kosong = kotak kosong di kop.</p>
										{#if aksi.isian[l.k]}<p class="text-sm text-bahaya" role="alert">{aksi.isian[l.k]}</p>{/if}
									</div>
								</div>
							</div>
						{/each}
					</div>
				</fieldset>

				<PesanAksi {aksi} />
				{#if boleh}
					<div class="flex justify-end gap-2.5">
						<button type="button" class="btn" disabled={!berubah || aksi.sibuk} onclick={() => kopSurat.data && (isi = { ...kopSurat.data })}>Batalkan perubahan</button>
						<button class="btn btn--utama" disabled={!berubah || aksi.sibuk}>{aksi.sibuk ? 'Menyimpan…' : 'Simpan kop surat'}</button>
					</div>
				{/if}
			</form>
		{/if}
	{/snippet}
</Keadaan>
