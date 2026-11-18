<!-- Lembar cetak untuk downtime (NFR-12): dicetak berkala supaya pelayanan tetap jalan saat sistem mati. -->
<script lang="ts">
	import { merek } from '#lib/merek.svelte.ts';
	import Kop from '#lib/Kop.svelte';
	import { api } from '#lib/api.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import { sesi } from '#lib/sesi.svelte.ts';
	import { KOMPONEN, STATUS_PERMINTAAN, golongan, tanggal, waktu } from '#lib/format.ts';
	import { GOL } from '#lib/stok/stok.ts';
	import Printer from '@lucide/svelte/icons/printer';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';
	import TriangleAlert from '@lucide/svelte/icons/triangle-alert';
	import KepalaHalaman from '#lib/KepalaHalaman.svelte';
	import Golongan from '#lib/Golongan.svelte';
	import Chip from '#lib/Chip.svelte';

	type Ringkas = { komponen: string; abo: string; rhesus: string; tersedia: number; mendekati_kedaluwarsa: number };
	const ringkasan = new Sumber(() => api<Ringkas[]>('GET', '/stok/ringkasan'));
	const tersedia = new Sumber(() => api<any[]>('GET', '/stok?status=tersedia'));
	// Daftar permintaan sudah memuat golongan pasien terverifikasi.
	const permintaan = new Sumber(() => api<any[]>('GET', '/permintaan'));
	let dicetak = $state(new Date().toISOString());

	function muat() {
		dicetak = new Date().toISOString();
		ringkasan.muat();
		tersedia.muat();
		if (sesi.boleh('permintaan.lihat')) permintaan.muat();
	}
	$effect(() => {
		muat();
	});

	const sel = (k: string, a: string, r: string) => ringkasan.data?.find((x) => x.komponen === k && x.abo === a && x.rhesus === r);
	const golPasien = (p: any) =>
		p.golongan_terverifikasi
			? golongan(p.golongan_terverifikasi.abo, p.golongan_terverifikasi.rhesus)
			: `Belum terverifikasi${p.abo_klinis ? ` (klinis ${golongan(p.abo_klinis, p.rhesus_klinis)})` : ''}`;
	const teralokasi = (p: any): string[] => p.kantong_teralokasi ?? [];
</script>

<svelte:head><title>Lembar darurat — {merek.judul}</title></svelte:head>

<div class="tanpa-cetak">
	<KepalaHalaman judul="Lembar darurat" sub="Cetak lembar ini setiap pergantian dinas dan simpan di meja BDRS untuk dipakai saat sistem tidak bisa diakses.">
		{#snippet aksi()}
			<button type="button" class="btn" onclick={muat}><RefreshCw class="size-4" aria-hidden="true" />Muat ulang</button>
			<button type="button" class="btn btn--utama" onclick={() => window.print()}><Printer class="size-4" aria-hidden="true" />Cetak</button>
		{/snippet}
	</KepalaHalaman>
</div>

<div class="flex flex-col gap-5 print:gap-3 print:text-[9pt]">
	<header class="panel print:border-0 print:p-0 print:shadow-none">
		<Kop />
		<h2 class="mt-4 text-xl font-extrabold tracking-tight">Lembar darurat stok dan golongan pasien</h2>
		<p class="mt-1 text-sm text-teks-2">Dicetak {waktu(dicetak)} oleh {sesi.profil?.nama}. Data sesudah waktu ini tidak tercantum.</p>
	</header>

	<section class="catatan break-inside-avoid flex-col gap-2 print:border-0 print:bg-transparent print:p-0 print:text-teks" aria-labelledby="judul-petunjuk">
		<h2 id="judul-petunjuk" class="flex items-center gap-2.5"><TriangleAlert class="size-5 shrink-0 print:hidden" aria-hidden="true" />Prosedur manual saat sistem mati</h2>
		<ol class="list-decimal space-y-1 pl-5 text-sm font-medium">
			<li>Catat setiap permintaan, sampel, uji silang, pengeluaran, dan transfusi di formulir kertas cadangan, dengan nomor kantong lengkap dan jam.</li>
			<li>Pakai lembar ini sebagai stok awal. Coret kantong yang dikeluarkan dan tulis jam keluar serta no. RM penerimanya.</li>
			<li>Golongan pasien di lembar ini hanya yang sudah terverifikasi. Pasien lain tetap diperiksa golongan dan uji silang manual dengan verifikasi petugas kedua.</li>
			<li>Darurat tanpa golongan terverifikasi: hanya O Rh− untuk sel darah merah dan AB untuk plasma, dengan persetujuan dokter tertulis.</li>
			<li>Jangan keluarkan kantong yang kedaluwarsa atau berstatus karantina.</li>
			<li>Setelah sistem pulih, masukkan semua catatan kertas ke utd-rs secara berurutan, lalu simpan kertasnya sebagai arsip.</li>
		</ol>
	</section>

	<section class="panel break-inside-avoid print:border-0 print:p-0" aria-labelledby="judul-ringkasan">
		<h2 id="judul-ringkasan" class="mb-2">Stok tersedia per komponen dan golongan</h2>
		<Keadaan sumber={ringkasan} apakahKosong={() => false}>
			{#snippet children()}
				<div class="overflow-x-auto">
					<table class="tabel">
						<thead>
							<tr>
								<th scope="col">Komponen</th>
								{#each GOL as [a, r] (a + r)}<th scope="col" class="text-center"><Golongan abo={a} rhesus={r} ukuran="kecil" /></th>{/each}
							</tr>
						</thead>
						<tbody>
							{#each Object.keys(KOMPONEN) as k (k)}
								<tr>
									<th scope="row" class="text-left text-sm font-medium normal-case">{KOMPONEN[k]}</th>
									{#each GOL as [a, r] (a + r)}
										{@const x = sel(k, a, r)}
										<td class="text-center tabular-nums">
											<span class={x?.tersedia ? 'text-base font-bold' : 'text-redup'}>{x?.tersedia ?? 0}</span>
											{#if x?.mendekati_kedaluwarsa}<span class="block text-xs text-waspada">{x.mendekati_kedaluwarsa} segera kedaluwarsa</span>{/if}
										</td>
									{/each}
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{/snippet}
		</Keadaan>
	</section>

	{#if sesi.boleh('permintaan.lihat')}
		<section class="panel print:border-0 print:p-0" aria-labelledby="judul-permintaan">
			<h2 id="judul-permintaan" class="mb-2">Permintaan darah berjalan dan golongan pasien</h2>
			<Keadaan sumber={permintaan}>
				{#snippet kosong()}<p>Tidak ada permintaan darah yang sedang berjalan.</p>{/snippet}
				{#snippet children(data)}
					<div class="overflow-x-auto">
						<table class="tabel">
							<thead>
								<tr>
									<th scope="col">Permintaan</th>
									<th scope="col">Pasien</th>
									<th scope="col">Golongan pasien</th>
									<th scope="col">Diminta</th>
									<th scope="col">Kantong teralokasi</th>
									<th scope="col">Status</th>
								</tr>
							</thead>
							<tbody>
								{#each data as p (p.id)}
									<tr class="break-inside-avoid">
										<td>
											<span class="mono font-semibold">{p.nomor}</span>
											<span class="block text-xs text-redup">{p.ruang} · {p.jenis === 'darurat' ? 'DARURAT' : p.jenis}</span>
										</td>
										<td>
											<span class="font-semibold">{p.pasien.nama}</span>
											<span class="mono block text-xs text-redup">{p.no_rm}</span>
											<span class="block text-xs text-redup">Lahir {tanggal(p.pasien.tanggal_lahir)} · {p.pasien.jenis_kelamin}</span>
										</td>
										<td>
											{#if p.golongan_terverifikasi}
												<Golongan abo={p.golongan_terverifikasi.abo} rhesus={p.golongan_terverifikasi.rhesus} />
											{:else}
												<span class="text-sm font-semibold text-waspada">{golPasien(p)}</span>
											{/if}
										</td>
										<td>{#each p.baris as b, i (i)}<span class="block whitespace-nowrap">{b.jumlah} × {KOMPONEN[b.komponen]}</span>{/each}</td>
										<td class="mono">{#each teralokasi(p) as n (n)}<span class="block">{n}</span>{:else}—{/each}</td>
										<td><Chip nada={STATUS_PERMINTAAN[p.status]?.[1]}>{STATUS_PERMINTAAN[p.status]?.[0] ?? p.status}</Chip></td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/snippet}
			</Keadaan>
		</section>
	{/if}

	<section class="panel print:border-0 print:p-0" aria-labelledby="judul-tersedia">
		<h2 id="judul-tersedia" class="mb-2">Kantong tersedia, kedaluwarsa terdekat lebih dulu</h2>
		<Keadaan sumber={tersedia}>
			{#snippet kosong()}<p>Tidak ada kantong tersedia.</p>{/snippet}
			{#snippet children(data)}
				<div class="overflow-x-auto">
					<table class="tabel">
						<thead>
							<tr>
								<th scope="col">Nomor kantong</th>
								<th scope="col">Komponen</th>
								<th scope="col">Golongan</th>
								<th scope="col">Kedaluwarsa</th>
								<th scope="col">Alat</th>
								<th scope="col">Keluar (jam / no. RM)</th>
							</tr>
						</thead>
						<tbody>
							{#each data as k (k.id)}
								<tr class="break-inside-avoid">
									<td class="mono font-semibold">{k.nomor}</td>
									<td>{KOMPONEN[k.komponen]}</td>
									<td><Golongan abo={k.abo} rhesus={k.rhesus} ukuran="kecil" /></td>
									<td class="whitespace-nowrap">
										{waktu(k.kedaluwarsa)}
										{#if k.lewat_kedaluwarsa}<span class="block text-xs font-medium text-bahaya">Kedaluwarsa, jangan dipakai</span>
										{:else if k.mendekati_kedaluwarsa}<span class="block text-xs text-waspada">Segera kedaluwarsa</span>{/if}
									</td>
									<td class="mono">{k.alat ?? '—'}</td>
									<td class="min-w-32 border-l border-dashed border-garis-kuat"></td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{/snippet}
		</Keadaan>
	</section>
</div>

<style>
	@page {
		size: A4;
		margin: 12mm;
	}
</style>
