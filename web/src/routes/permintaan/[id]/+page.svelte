<script lang="ts">
	import { page } from '$app/state';
	import { api } from '#lib/api.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Isian from '#lib/Isian.svelte';
	import Chip from '#lib/Chip.svelte';
	import { sesi } from '#lib/sesi.svelte.ts';
	import { KOMPONEN, NAMA_KOMPONEN, STATUS_PERMINTAAN, golongan, tanggal, waktu } from '#lib/format.ts';
	import TandaTangan, { type Minta } from '#lib/pelayanan/TandaTangan.svelte';
	import { JENIS_PERMINTAAN } from '#lib/pelayanan/darah.ts';
	import Sampel from './Sampel.svelte';
	import Golongan from './Golongan.svelte';
	import UjiSilang from './UjiSilang.svelte';
	import Alokasi from './Alokasi.svelte';
	import Printer from '@lucide/svelte/icons/printer';

	const id = $derived(page.params.id);
	const detail = new Sumber(() => api<any>('GET', `/permintaan/${id}`));
	const aturanSumber = new Sumber(() => api<any[]>('GET', '/master/aturan-komponen'));
	$effect(() => {
		void id;
		detail.muat();
	});
	$effect(() => {
		aturanSumber.muat();
	});
	const aturan = $derived(Object.fromEntries((aturanSumber.data ?? []).map((a) => [a.komponen, a.uji_silang])));
	const muat = () => detail.muat();

	let ttd: TandaTangan | undefined = $state();
	const minta: Minta = (o) => ttd?.minta(o);

	// Mengakhiri permintaan: tolak, batal, atau selesai.
	const AKHIR = {
		tolak: { label: 'Tolak permintaan', alasan: 'Alasan penolakan (formulir tidak lengkap atau tidak sesuai)', dari: ['diminta', 'diproses'], hak: () => sesi.boleh('permintaan.proses') },
		batal: { label: 'Batalkan permintaan', alasan: 'Alasan pembatalan', dari: ['diminta', 'diproses', 'siap'], hak: () => sesi.boleh('permintaan.buat') || sesi.boleh('permintaan.proses') },
		selesai: { label: 'Tandai selesai', alasan: 'Keterangan (opsional)', dari: ['diproses', 'siap'], hak: () => sesi.boleh('permintaan.proses') }
	} as const;
	type Akhir = keyof typeof AKHIR;
	let akhir = $state<Akhir | null>(null);
	let alasan = $state('');
	const aksiAkhir = new Aksi();

	async function kirimAkhir(e: SubmitEvent) {
		e.preventDefault();
		if (!akhir) return;
		const r = await aksiAkhir.jalankan(
			() => api('POST', `/permintaan/${id}/${akhir}`, { alasan: alasan || undefined }),
			'Status permintaan diperbarui. Kantong yang belum dikeluarkan kembali ke stok.'
		);
		if (r) {
			akhir = null;
			alasan = '';
			await muat();
		}
	}
</script>

<svelte:head><title>{detail.data?.nomor ?? 'Permintaan darah'} — utd-rs</title></svelte:head>

<TandaTangan bind:this={ttd} />

<Keadaan sumber={detail} apakahKosong={() => false} baris={10}>
	{#snippet children(p)}
		{@const st = STATUS_PERMINTAAN[p.status]}
		{@const aktif = ['diminta', 'diproses', 'siap'].includes(p.status)}
		{@const gol = p.golongan_terverifikasi}
		<p class="mb-2 hidden text-sm print:block">{sesi.profil?.nama_rs} · Formulir permintaan darah · dicetak {waktu(new Date().toISOString())}</p>
		<div class="mb-4 flex flex-wrap items-start justify-between gap-3">
			<div>
				<p class="tanpa-cetak text-sm"><a class="text-utama hover:underline" href="/permintaan">Permintaan darah</a></p>
				<h1 class="mono">{p.nomor}</h1>
				<div class="mt-1 flex flex-wrap gap-2">
					<Chip nada={st?.[1]}>{st?.[0] ?? p.status}</Chip>
					<Chip nada={JENIS_PERMINTAAN[p.jenis]?.[1]}>{JENIS_PERMINTAAN[p.jenis]?.[0] ?? p.jenis}</Chip>
				</div>
				{#if p.alasan_status}<p class="mt-1 text-sm text-redup">{p.alasan_status}</p>{/if}
			</div>
			<div class="tanpa-cetak flex flex-wrap gap-2">
				<button type="button" class="btn" onclick={() => window.print()}><Printer class="size-4" aria-hidden="true" />Cetak formulir</button>
				{#each Object.entries(AKHIR) as [k, a] (k)}
					{#if a.hak() && (a.dari as readonly string[]).includes(p.status)}
						<button type="button" class="btn {k === 'selesai' ? '' : 'btn--teks'}" aria-expanded={akhir === k} onclick={() => ((akhir = k as Akhir), (alasan = ''))}>{a.label}</button>
					{/if}
				{/each}
			</div>
		</div>

		{#if akhir}
			<form class="panel tanpa-cetak mb-4 flex flex-col gap-3" onsubmit={kirimAkhir}>
				<h2>{AKHIR[akhir].label}</h2>
				<Isian label={AKHIR[akhir].alasan} id="akhir-alasan" galat={aksiAkhir.isian.alasan}>
					<input id="akhir-alasan" class="input" bind:value={alasan} required={akhir !== 'selesai'} />
				</Isian>
				<PesanAksi aksi={aksiAkhir} />
				<div class="flex gap-2">
					<button class="btn {akhir === 'selesai' ? 'btn--utama' : 'btn--bahaya'}" disabled={aksiAkhir.sibuk}>{AKHIR[akhir].label}</button>
					<button type="button" class="btn" onclick={() => (akhir = null)}>Batal</button>
				</div>
			</form>
		{:else}
			<div class="mb-4"><PesanAksi aksi={aksiAkhir} /></div>
		{/if}

		{#if p.jenis === 'darurat' && aktif}
			<p class="peringatan mb-4">Permintaan darurat: {p.alasan_darurat}</p>
		{/if}

		<div class="flex flex-col gap-4">
			<div class="grid gap-4 lg:grid-cols-2">
				<section class="panel" aria-labelledby="j-pasien">
					<h2 id="j-pasien" class="mb-3">Pasien</h2>
					<dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-sm">
						<dt class="text-redup">Nama</dt><dd class="font-semibold">{p.pasien.nama}</dd>
						<dt class="text-redup">No. RM</dt><dd><a class="mono text-utama hover:underline" href="/pasien/{encodeURIComponent(p.no_rm)}">{p.no_rm}</a></dd>
						<dt class="text-redup">Tanggal lahir</dt><dd>{tanggal(p.pasien.tanggal_lahir)}</dd>
						<dt class="text-redup">Jenis kelamin</dt><dd>{p.pasien.jenis_kelamin === 'L' ? 'Laki-laki' : 'Perempuan'}</dd>
						<dt class="text-redup">Golongan terverifikasi</dt>
						<dd>
							{#if gol}
								<span class="text-lg font-semibold">{golongan(gol.abo, gol.rhesus)}</span>
							{:else}
								<Chip nada="waspada">Belum terverifikasi</Chip>
							{/if}
						</dd>
						<dt class="text-redup">Golongan klinis</dt><dd>{golongan(p.abo_klinis, p.rhesus_klinis)}</dd>
						<dt class="text-redup">Riwayat transfusi</dt><dd>{p.riwayat_transfusi ? 'Ya' : 'Tidak'}</dd>
						<dt class="text-redup">Riwayat reaksi</dt><dd class={p.riwayat_reaksi ? 'font-semibold text-bahaya' : ''}>{p.riwayat_reaksi ? 'Ya' : 'Tidak'}</dd>
					</dl>
				</section>
				<section class="panel" aria-labelledby="j-permintaan">
					<h2 id="j-permintaan" class="mb-3">Permintaan</h2>
					<dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-sm">
						<dt class="text-redup">Ruang</dt><dd>{p.ruang}</dd>
						<dt class="text-redup">Dokter peminta</dt><dd>{p.dokter}</dd>
						<dt class="text-redup">Diagnosis</dt><dd>{p.diagnosis}</dd>
						<dt class="text-redup">Indikasi</dt><dd>{p.indikasi}</dd>
						<dt class="text-redup">Hb / trombosit</dt><dd>{p.hb ?? '—'} g/dL / {p.trombosit?.toLocaleString('id-ID') ?? '—'} /µL</dd>
						<dt class="text-redup">Diminta</dt><dd>{waktu(p.diminta_pada)}</dd>
						<dt class="text-redup">Rencana transfusi</dt><dd>{tanggal(p.rencana_transfusi)}</dd>
						<dt class="text-redup">Komponen</dt>
						<dd>
							<ul>
								{#each p.baris as b, i (i)}
									{@const n = p.alokasi.filter((a: any) => !a.dilepas_pada && a.kantong.komponen === b.komponen).length}
									<li>
										{b.jumlah} kantong {NAMA_KOMPONEN[b.komponen] ?? b.komponen}{b.volume_ml ? ` (${b.volume_ml} mL)` : ''}
										<span class="text-redup">· {n} dialokasikan</span>
									</li>
								{/each}
							</ul>
						</dd>
						{#if p.rujukan_simrs}<dt class="text-redup">Rujukan SIMRS</dt><dd class="mono">{p.rujukan_simrs}</dd>{/if}
					</dl>
				</section>
			</div>

			<Sampel {p} {aktif} {muat} />
			<Golongan {p} {aktif} {muat} {minta} />
			{#if p.baris.some((b: any) => aturan[b.komponen] !== 'tidak_perlu') || p.uji_silang.length}
				<UjiSilang {p} {aktif} {aturan} {muat} {minta} />
			{/if}
			<Alokasi {p} {aktif} {aturan} {muat} {minta} />
		</div>
		<p class="mt-8 hidden text-sm print:block">
			Komponen: {p.baris.map((b: any) => `${b.jumlah} ${KOMPONEN[b.komponen]}`).join(', ')}. Tanda tangan petugas BDRS: ____________________
		</p>
	{/snippet}
</Keadaan>
