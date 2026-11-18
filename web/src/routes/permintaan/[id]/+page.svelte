<script lang="ts">
	import { merek } from '#lib/merek.svelte.ts';
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
	import { KOMPONEN, NAMA_KOMPONEN, STATUS_PERMINTAAN, golongan, tanggal, waktu, pukul, inisial } from '#lib/format.ts';
	import TandaTangan, { type Minta } from '#lib/pelayanan/TandaTangan.svelte';
	import { JENIS_PERMINTAAN } from '#lib/pelayanan/darah.ts';
	import Sampel from './Sampel.svelte';
	import Golongan from './Golongan.svelte';
	import UjiSilang from './UjiSilang.svelte';
	import Alokasi from './Alokasi.svelte';
	import Printer from '@lucide/svelte/icons/printer';
	import KepalaHalaman from '#lib/KepalaHalaman.svelte';
	import Siren from '@lucide/svelte/icons/siren';
	import GolonganLencana from '#lib/Golongan.svelte';
	import Tahap from './Tahap.svelte';
	import { tahapSekarang } from './tahap.ts';
	import Samping from './Samping.svelte';

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

<svelte:head><title>{detail.data?.nomor ?? 'Permintaan darah'} — {merek.judul}</title></svelte:head>

<TandaTangan bind:this={ttd} />

<Keadaan sumber={detail} apakahKosong={() => false} baris={10}>
	{#snippet children(p)}
		{@const st = STATUS_PERMINTAAN[p.status]}
		{@const aktif = ['diminta', 'diproses', 'siap'].includes(p.status)}
		{@const gol = p.golongan_terverifikasi}
		<div class="mb-3"><Kop cetakSaja /></div>
		<p class="mb-2 hidden text-sm print:block">Formulir permintaan darah · dicetak {waktu(new Date().toISOString())}</p>
		<KepalaHalaman judul={p.nomor} mono jalur={[['Permintaan darah', '/permintaan'], [p.nomor]]} sub={p.alasan_status ?? undefined}>
			{#snippet aksi()}
				{#each Object.entries(AKHIR) as [k, a] (k)}
					{#if a.hak() && (a.dari as readonly string[]).includes(p.status)}
						<button type="button" class="btn {k === 'selesai' ? '' : 'btn--teks'}" aria-expanded={akhir === k} onclick={() => ((akhir = k as Akhir), (alasan = ''))}>{a.label}</button>
					{/if}
				{/each}
				<button type="button" class="btn" onclick={() => window.print()}><Printer class="size-4" aria-hidden="true" />Cetak formulir</button>
			{/snippet}
		</KepalaHalaman>

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
			<div class="peringatan mb-4"><Siren aria-hidden="true" /><p><span class="font-bold">Permintaan darurat:</span> {p.alasan_darurat}</p></div>
		{/if}

		<div class="flex flex-col gap-4">
			<section class="panel flex flex-wrap items-center gap-x-6 gap-y-4" aria-labelledby="j-pasien">
				<div class="flex min-w-72 flex-1 items-center gap-4">
					<span class="grid size-13 shrink-0 place-items-center rounded-full bg-halus text-lg font-bold text-teks-2" aria-hidden="true">{inisial(p.pasien.nama)}</span>
					<div class="min-w-0">
						<div class="flex flex-wrap items-center gap-2">
							<h2 id="j-pasien" class="text-xl font-extrabold tracking-tight">{p.pasien.nama}</h2>
							<Chip nada={st?.[1]}>{st?.[0] ?? p.status}</Chip>
							<Chip nada={JENIS_PERMINTAAN[p.jenis]?.[1]} polos>{JENIS_PERMINTAAN[p.jenis]?.[0] ?? p.jenis}</Chip>
						</div>
						<p class="mt-0.5 text-sm text-redup">
							<a class="mono font-semibold text-teks-2 hover:underline" href="/pasien/{encodeURIComponent(p.no_rm)}">{p.no_rm}</a>
							· {p.pasien.jenis_kelamin === 'L' ? 'Laki-laki' : 'Perempuan'} · {tanggal(p.pasien.tanggal_lahir)} · {p.ruang} · {p.dokter}
						</p>
					</div>
				</div>
				<dl class="flex flex-wrap gap-x-7 gap-y-2 text-sm">
					<div><dt class="text-xs font-semibold text-redup">Diagnosis</dt><dd class="font-bold">{p.diagnosis}</dd></div>
					<div><dt class="text-xs font-semibold text-redup">Hb</dt><dd class="font-bold">{p.hb != null ? Number(p.hb).toLocaleString('id-ID') : '—'} g/dL</dd></div>
					<div>
						<dt class="text-xs font-semibold text-redup">Diminta</dt>
						<dd class="font-bold">{p.baris.map((b: any) => `${KOMPONEN[b.komponen]} ${b.jumlah}`).join(', ')} · {pukul(p.diminta_pada)}</dd>
					</div>
				</dl>
				<div class="flex flex-col items-center gap-1">
					{#if gol}
						<GolonganLencana abo={gol.abo} rhesus={gol.rhesus} ukuran="besar" />
						<span class="text-xs font-semibold text-aman">Terverifikasi</span>
					{:else}
						<Chip nada="waspada">Golongan belum terverifikasi</Chip>
					{/if}
				</div>
			</section>

			<Tahap {p} {aturan} />

			<div class="grid items-start gap-5 xl:grid-cols-[1fr_360px]">
				<div class="flex min-w-0 flex-col gap-4">
					<Sampel {p} {aktif} {muat} sekarang={tahapSekarang(p, aturan)} />
					<Golongan {p} {aktif} {muat} {minta} sekarang={tahapSekarang(p, aturan)} />
					{#if p.baris.some((b: any) => aturan[b.komponen] !== 'tidak_perlu') || p.uji_silang.length}
						<UjiSilang {p} {aktif} {aturan} {muat} {minta} sekarang={tahapSekarang(p, aturan)} />
					{/if}
					<Alokasi {p} {aktif} {aturan} {muat} {minta} sekarang={tahapSekarang(p, aturan)} />
				</div>
				<Samping {p} />
			</div>
		</div>
		<p class="mt-8 hidden text-sm print:block">
			Komponen: {p.baris.map((b: any) => `${b.jumlah} ${KOMPONEN[b.komponen]}`).join(', ')}. Tanda tangan petugas BDRS: ____________________
		</p>
	{/snippet}
</Keadaan>
