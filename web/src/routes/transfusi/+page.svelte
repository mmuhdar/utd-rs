<!-- Layar ruang (tablet): kantong yang menunggu, verifikasi samping tempat tidur, dan transfusi yang berjalan. -->
<script lang="ts">
	import { goto } from '$app/navigation';
	import { api, GalatApi, pesanGalat } from '#lib/api.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import Isian from '#lib/Isian.svelte';
	import Chip from '#lib/Chip.svelte';
	import Pindai from '#lib/Pindai.svelte';
	import { sesi } from '#lib/sesi.svelte.ts';
	import { KOMPONEN, golongan, hariIni, pukul, relatif } from '#lib/format.ts';
	import TandaTangan from '#lib/pelayanan/TandaTangan.svelte';
	import OctagonAlert from '@lucide/svelte/icons/octagon-alert';
	import CircleAlert from '@lucide/svelte/icons/circle-alert';
	import ScanLine from '@lucide/svelte/icons/scan-line';
	import TriangleAlert from '@lucide/svelte/icons/triangle-alert';
	import Clock from '@lucide/svelte/icons/clock';
	import User from '@lucide/svelte/icons/user';
	import Droplet from '@lucide/svelte/icons/droplet';
	import Syringe from '@lucide/svelte/icons/syringe';
	import Golongan from '#lib/Golongan.svelte';

	const keluar = new Sumber(() => api<any[]>('GET', '/pengeluaran/aktif'));
	const aktif = new Sumber(() => api<any[]>('GET', '/transfusi/aktif'));
	$effect(() => {
		keluar.muat();
		aktif.muat();
		const t = setInterval(() => {
			keluar.muat();
			aktif.muat();
		}, 30_000);
		return () => clearInterval(t);
	});

	// Verifikasi samping tempat tidur.
	let no_rm = $state('');
	let nomor = $state('');
	let tanggal_lahir = $state('');
	let rawat_jalan = $state(false);
	let alasan_terlambat = $state('');
	let mintaAlasan = $state<string | null>(null);
	let blokir = $state<string | null>(null);
	let galat = $state<string | null>(null);
	let isian = $state<Record<string, string>>({});
	let sibuk = $state(false);
	let ttd: TandaTangan | undefined = $state();

	// Kantong yang dipindai, bila ada di daftar pengeluaran aktif: tampilkan tujuannya. Pencocokan tetap di server.
	const kantong = $derived(keluar.data?.find((k) => k.nomor_kantong === nomor));
	const terlambat = $derived(kantong?.terlambat ?? false);
	const lengkap = $derived(!!(no_rm && nomor && tanggal_lahir && (!(terlambat || mintaAlasan) || alasan_terlambat.trim())));

	function ulang() {
		no_rm = nomor = tanggal_lahir = alasan_terlambat = '';
		rawat_jalan = false;
		mintaAlasan = blokir = galat = null;
		isian = {};
	}

	function verifikasi(e: SubmitEvent) {
		e.preventDefault();
		ttd?.minta({
			judul: 'Tanda tangan petugas kedua',
			makna: `Petugas kedua menyatakan telah mencocokkan sendiri gelang pasien ${no_rm} dan label kantong ${nomor} di samping tempat tidur.`,
			petugasLain: true,
			kirim: async ({ nama_pengguna, sandi }) => {
				sibuk = true;
				galat = null;
				isian = {};
				try {
					const t = await api<{ id: string }>('POST', '/transfusi/verifikasi', {
						no_rm,
						nomor_kantong: nomor,
						tanggal_lahir,
						rawat_jalan,
						alasan_terlambat: alasan_terlambat || undefined,
						petugas_2: nama_pengguna,
						sandi_petugas_2: sandi
					});
					goto(`/transfusi/${t.id}`);
				} catch (e) {
					const pesan = pesanGalat(e);
					if (e instanceof GalatApi && e.status === 409 && pesan.startsWith('JANGAN TRANSFUSIKAN')) {
						blokir = pesan;
						keluar.muat();
					} else {
						galat = pesan;
						if (e instanceof GalatApi) {
							isian = e.perIsian;
							if (isian.alasan_terlambat) mintaAlasan = isian.alasan_terlambat;
						}
					}
				} finally {
					sibuk = false;
				}
			}
		});
	}
</script>

<svelte:head><title>Transfusi di ruang — utd-rs</title></svelte:head>

<TandaTangan bind:this={ttd} />

<h1 class="mb-4">Transfusi di ruang</h1>

<div class="grid gap-4 xl:grid-cols-2">
	{#if sesi.boleh('transfusi.catat')}
		<section class="panel flex flex-col gap-4 xl:col-span-2" aria-labelledby="j-verif">
			<h2 id="j-verif" class="text-lg">Verifikasi samping tempat tidur</h2>
			{#if blokir}
				<div class="peringatan p-6" role="alert">
					<OctagonAlert class="size-7! p-2.5!" aria-hidden="true" />
					<div class="flex flex-col gap-2">
						<p class="text-2xl leading-tight font-extrabold">{blokir}</p>
						<p class="text-base">Pasien <span class="mono">{no_rm}</span> · kantong <span class="mono">{nomor}</span>. Kejadian ini otomatis dicatat sebagai Kejadian Nyaris Cedera (KNC).</p>
					</div>
				</div>
				<div class="rounded-2xl border border-garis p-5">
					<h3 class="mb-3 text-lg font-bold">Yang harus dilakukan sekarang</h3>
					<ol class="flex flex-col gap-3 text-base">
						{#each ['Jangan pasang kantong ini. Tutup kembali dan beri tanda "ditolak di ruang".', 'Bawa kantong ke bank darah dalam 30 menit bersama formulir permintaan.', 'Hubungi bank darah untuk kantong pengganti yang sesuai.'] as l, i (i)}
							<li class="flex items-center gap-3">
								<span class="grid size-7 shrink-0 place-items-center rounded-full bg-bahaya-latar text-sm font-bold text-bahaya">{i + 1}</span>{l}
							</li>
						{/each}
					</ol>
				</div>
				<button type="button" class="btn btn--besar self-start" onclick={ulang}>Mulai verifikasi baru</button>
			{:else}
				<form class="flex flex-col gap-4" onsubmit={verifikasi}>
					<p class="text-sm text-redup">Dua petugas di samping tempat tidur. Pindai gelang pasien dan label kantong, tanyakan tanggal lahir pasien.</p>
					<div class="grid gap-4 md:grid-cols-2">
						<div class="flex flex-col gap-3 rounded-2xl border border-garis p-5">
							<h3 class="flex items-center gap-3 text-lg font-bold">
								<span class="grid size-10 place-items-center rounded-xl bg-halus"><User class="size-5" aria-hidden="true" /></span>Gelang pasien
							</h3>
							<Pindai label="1. Pindai gelang pasien (no. RM)" id="v-rm" onpindai={(v) => (no_rm = v)} autofocus />
							<p class="flex min-h-12 items-center gap-3 rounded-xl px-3.5 py-2 text-base {no_rm ? 'bg-info-latar' : 'bg-halus'}" aria-live="polite">
								{#if no_rm}<ScanLine class="size-5 shrink-0 text-info" aria-hidden="true" /><span class="sr-only">Terbaca:</span>{/if}
								<span class="w-20 shrink-0 font-semibold text-redup">No. RM</span><span class="mono font-semibold">{no_rm || '—'}</span>
							</p>
							{#if isian.no_rm}<p class="text-sm text-bahaya">{isian.no_rm}</p>{/if}
						</div>
						<div class="flex flex-col gap-3 rounded-2xl border border-garis p-5">
							<h3 class="flex items-center gap-3 text-lg font-bold">
								<span class="grid size-10 place-items-center rounded-xl bg-halus"><Droplet class="size-5" aria-hidden="true" /></span>Label kantong
							</h3>
							<Pindai label="2. Pindai label kantong" id="v-kantong" onpindai={(v) => (nomor = v)} />
							<p class="flex min-h-12 items-center gap-3 rounded-xl px-3.5 py-2 text-base {nomor ? 'bg-info-latar' : 'bg-halus'}" aria-live="polite">
								{#if nomor}<ScanLine class="size-5 shrink-0 text-info" aria-hidden="true" /><span class="sr-only">Terbaca:</span>{/if}
								<span class="w-20 shrink-0 font-semibold text-redup">Kantong</span><span class="mono font-semibold">{nomor || '—'}</span>
							</p>
							{#if kantong}
								<p class="flex min-h-12 items-center gap-3 rounded-xl bg-halus px-3.5 py-2 text-base">
									<span class="w-20 shrink-0 font-semibold text-redup">Untuk</span>
									<span class="min-w-0 flex-1">{kantong.pasien.nama} · <span class="mono">{kantong.pasien.no_rm}</span></span>
									<Golongan abo={kantong.abo} rhesus={kantong.rhesus} />
								</p>
								{#if no_rm && kantong.pasien.no_rm !== no_rm}
									<p class="catatan" role="alert">
										<TriangleAlert aria-hidden="true" />No. RM gelang berbeda dengan tujuan kantong. Periksa ulang sebelum melanjutkan.
									</p>
								{/if}
							{/if}
							{#if isian.nomor_kantong}<p class="text-sm text-bahaya">{isian.nomor_kantong}</p>{/if}
						</div>
					</div>
					<div class="grid gap-4 md:grid-cols-2">
						<Isian label="3. Tanggal lahir menurut pasien atau gelang" id="v-lahir" galat={isian.tanggal_lahir}>
							<input id="v-lahir" class="input min-h-12 rounded-xl text-base" type="date" max={hariIni()} bind:value={tanggal_lahir} required />
						</Isian>
						<label class="inline-flex min-h-12 items-center gap-3 self-end text-base">
							<input type="checkbox" class="size-5" bind:checked={rawat_jalan} />
							Pasien rawat jalan (observasi pasca 1 jam)
						</label>
					</div>
					{#if terlambat || mintaAlasan}
						<div class="catatan">
							<Clock aria-hidden="true" />
							<div class="flex w-full flex-col gap-3">
								<p>{mintaAlasan ?? 'Kantong ini sudah lebih dari 30 menit di luar bank darah. Isi alasan bila tetap ditransfusikan, atau kembalikan ke BDRS.'}</p>
								<div class="w-full text-teks">
									<Isian label="Alasan transfusi terlambat" id="v-terlambat">
										<textarea id="v-terlambat" class="input text-base" rows="2" bind:value={alasan_terlambat} required></textarea>
									</Isian>
								</div>
							</div>
						</div>
					{/if}
					{#if galat}
						<div class="peringatan" role="alert"><CircleAlert aria-hidden="true" /><p>{galat}</p></div>
					{/if}
					<div class="flex flex-wrap items-center justify-end gap-3 border-t border-garis pt-4">
						<p class="mr-auto text-sm text-redup">Dicocokkan server setelah petugas kedua menandatangani.</p>
						<button type="button" class="btn btn--besar" onclick={ulang}>Kosongkan</button>
						<button class="btn btn--utama btn--besar" disabled={!lengkap || sibuk}
							><Syringe class="size-5" aria-hidden="true" />{sibuk ? 'Memeriksa…' : 'Cocokkan dan mulai transfusi'}</button
						>
					</div>
				</form>
			{/if}
		</section>
	{/if}

	<section class="panel" aria-labelledby="j-keluar">
		<h2 id="j-keluar" class="mb-3 text-lg">Kantong menunggu ditransfusikan</h2>
		<Keadaan sumber={keluar}>
			{#snippet kosong()}<p>Tidak ada kantong yang sudah keluar dari bank darah dan menunggu ditransfusikan.</p>{/snippet}
			{#snippet children(data)}
				<ul class="flex flex-col gap-2">
					{#each data as k (k.id)}
						<li class="flex flex-wrap items-center justify-between gap-3 rounded-xl border p-3.5 {k.terlambat ? 'border-bahaya-titik bg-bahaya-latar' : 'border-garis'}">
							<div>
								<p class="mono font-semibold">{k.nomor_kantong}</p>
								<p class="flex flex-wrap items-center gap-1.5 text-sm"><Golongan abo={k.abo} rhesus={k.rhesus} ukuran="kecil" /> {KOMPONEN[k.komponen]} · {k.pasien.nama} <span class="mono text-redup">{k.pasien.no_rm}</span></p>
								<p class="text-xs text-redup">{k.ruang} · keluar pukul {pukul(k.dikeluarkan_pada)} · diterima {k.penerima_ruang}</p>
							</div>
							<Chip nada={k.terlambat ? 'bahaya' : k.menit > 20 ? 'waspada' : 'netral'}>
								{k.menit} menit{k.terlambat ? ' — lewat 30 menit' : ''}
							</Chip>
						</li>
					{/each}
				</ul>
			{/snippet}
		</Keadaan>
	</section>

	<section class="panel" aria-labelledby="j-aktif">
		<h2 id="j-aktif" class="mb-3 text-lg">Transfusi berjalan</h2>
		<Keadaan sumber={aktif}>
			{#snippet kosong()}<p>Tidak ada transfusi yang berjalan atau menunggu observasi pasca-transfusi.</p>{/snippet}
			{#snippet children(data)}
				<ul class="flex flex-col gap-2">
					{#each data as t (t.id)}
						{@const menit = (Date.now() - new Date(t.mulai).getTime()) / 60000}
						<li>
							<a href="/transfusi/{t.id}" class="flex min-h-12 flex-wrap items-center justify-between gap-3 rounded-xl border border-garis p-3.5 hover:bg-halus">
								<span>
									<span class="block font-semibold">{t.pasien.nama} <span class="mono text-sm font-normal text-redup">{t.no_rm}</span></span>
									<span class="block text-sm"><span class="mono">{t.nomor_kantong}</span> · {KOMPONEN[t.komponen]} · mulai {relatif(t.mulai)}</span>
									<span class="block text-xs text-redup">Observasi terakhir: {t.observasi_terakhir ? relatif(t.observasi_terakhir) : 'belum ada'}</span>
								</span>
								<span class="flex flex-col items-end gap-1">
									{#if t.selesai}
										<Chip nada="waspada">Menunggu observasi pasca</Chip>
									{:else}
										<Chip nada="info">Berjalan</Chip>
										{#if !t.ada_15_menit && menit >= 15}<Chip nada="bahaya">Observasi 15 menit belum dicatat</Chip>{/if}
									{/if}
								</span>
							</a>
						</li>
					{/each}
				</ul>
			{/snippet}
		</Keadaan>
	</section>
</div>
