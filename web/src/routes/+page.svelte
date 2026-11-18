<script lang="ts">
	import { merek } from '#lib/merek.svelte.ts';
	import { api } from '#lib/api.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import Chip from '#lib/Chip.svelte';
	import Golongan from '#lib/Golongan.svelte';
	import { sesi } from '#lib/sesi.svelte.ts';
	import { KOMPONEN, STATUS_PERMINTAAN, relatif, waktu, pukul } from '#lib/format.ts';
	import Thermometer from '@lucide/svelte/icons/thermometer';
	import Clock from '@lucide/svelte/icons/clock';
	import Package from '@lucide/svelte/icons/package';
	import Hourglass from '@lucide/svelte/icons/hourglass';
	import ClipboardList from '@lucide/svelte/icons/clipboard-list';
	import ShieldAlert from '@lucide/svelte/icons/shield-alert';
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	import Plus from '@lucide/svelte/icons/plus';
	import Printer from '@lucide/svelte/icons/printer';

	type Ringkas = { komponen: string; abo: string; rhesus: string; tersedia: number; mendekati_kedaluwarsa: number; lewat_kedaluwarsa: number; karantina: number };
	type Rencana = { abo: string; rhesus: string; stok_minimal: number };
	const stok = new Sumber(() => api<Ringkas[]>('GET', '/stok/ringkasan'));
	const rencana = new Sumber(() => api<Rencana[]>('GET', '/laporan/perencanaan'));
	const permintaan = new Sumber(() => api<any[]>('GET', '/permintaan'));
	const keluar = new Sumber(() => api<any[]>('GET', '/pengeluaran/aktif'));
	const penyimpangan = new Sumber(() => api<any[]>('GET', '/penyimpangan-suhu'));
	const alat = new Sumber(() => api<any[]>('GET', '/master/alat'));
	const kantong = new Sumber(() => api<any[]>('GET', '/stok?status=tersedia'));

	$effect(() => {
		if (sesi.boleh('stok.lihat')) {
			stok.muat();
			penyimpangan.muat();
			alat.muat();
			kantong.muat();
		}
		if (sesi.boleh('laporan.lihat')) rencana.muat();
		if (sesi.boleh('permintaan.lihat')) {
			permintaan.muat();
			keluar.muat();
		}
		// Daftar yang bergerak cepat diperbarui tiap menit (pengingat 30 menit, permintaan darurat).
		const t = setInterval(() => {
			if (sesi.boleh('permintaan.lihat')) {
				permintaan.muat();
				keluar.muat();
			}
		}, 60_000);
		return () => clearInterval(t);
	});

	const GOL: [string, string][] = [
		['O', 'pos'], ['O', 'neg'], ['A', 'pos'], ['A', 'neg'], ['B', 'pos'], ['B', 'neg'], ['AB', 'pos'], ['AB', 'neg']
	];
	const jumlah = (f: (r: Ringkas) => number) => (stok.data ?? []).reduce((n, r) => n + f(r), 0);
	const perGol = $derived(
		GOL.map(([abo, rhesus]) => {
			const baris = (stok.data ?? []).filter((r) => r.abo === abo && r.rhesus === rhesus);
			const minimal = (rencana.data ?? []).filter((r) => r.abo === abo && r.rhesus === rhesus).reduce((n, r) => n + Number(r.stok_minimal), 0);
			const n = baris.reduce((s, r) => s + r.tersedia, 0);
			const komp = baris.filter((r) => r.tersedia).map((r) => `${KOMPONEN[r.komponen]} ${r.tersedia}`).join(' · ') || 'Kosong';
			return { abo, rhesus, n, minimal, kurang: minimal > 0 && n < minimal, komp };
		})
	);
	const terbuka = $derived((penyimpangan.data ?? []).filter((p) => p.status === 'terbuka'));
	const terlambat = $derived((keluar.data ?? []).filter((k) => k.terlambat));
	const segera = $derived((kantong.data ?? []).filter((k) => k.mendekati_kedaluwarsa).slice(0, 5));
	const darurat = $derived((permintaan.data ?? []).filter((r) => r.jenis === 'darurat').length);

	const jam = new Date().getHours();
	const salam = jam < 11 ? 'Selamat pagi' : jam < 15 ? 'Selamat siang' : jam < 18 ? 'Selamat sore' : 'Selamat malam';
	const hari = new Intl.DateTimeFormat('id-ID', { weekday: 'long', day: 'numeric', month: 'long', year: 'numeric' }).format(new Date());
	// Kelas ditulis utuh supaya terbaca pemindai Tailwind.
	const NADA: Record<string, [string, string]> = {
		aman: ['bg-aman-latar text-aman', 'text-aman'],
		waspada: ['bg-waspada-latar text-waspada', 'text-waspada'],
		info: ['bg-info-latar text-info', 'text-info'],
		bahaya: ['bg-bahaya-latar text-bahaya', 'text-bahaya']
	};
	const ringkasBaris = (b: any[]) => (b ?? []).map((x) => `${KOMPONEN[x.komponen] ?? x.komponen} ${x.jumlah}`).join(', ');
	const suhuTeks = (v: number | string) => `${Number(v).toLocaleString('id-ID', { minimumFractionDigits: 1 })} °C`;
	const keluarRentang = (a: any) => a.suhu_terakhir && (Number(a.suhu_terakhir.suhu) < Number(a.suhu_min) || Number(a.suhu_terakhir.suhu) > Number(a.suhu_maks));
</script>

<svelte:head><title>Beranda — {merek.judul}</title></svelte:head>

<div class="flex flex-col gap-5">
	<header class="flex flex-wrap items-end justify-between gap-4">
		<div>
			<h1>{salam}, {sesi.profil?.nama}</h1>
			<p class="mt-1 font-medium text-redup first-letter:uppercase">{hari}</p>
		</div>
		<div class="flex flex-wrap gap-2.5">
			{#if sesi.boleh('stok.lihat')}
				<a class="btn" href="/darurat"><Printer class="size-4" aria-hidden="true" />Lembar darurat</a>
			{/if}
			{#if sesi.boleh('permintaan.buat')}
				<a class="btn btn--utama" href="/permintaan/baru"><Plus class="size-4" aria-hidden="true" />Buat permintaan</a>
			{/if}
		</div>
	</header>

	{#if terbuka.length || terlambat.length}
		<div class="flex flex-col gap-2.5">
			{#if terbuka.length}
				<div class="peringatan" role="alert">
					<Thermometer aria-hidden="true" />
					<div class="flex-1">
						<p class="font-bold">Penyimpangan suhu di {terbuka.map((p) => p.alat).join(', ')}</p>
						<p>Kantong di alat tersebut otomatis dikarantina sampai dinilai penanggung jawab.</p>
					</div>
					<a class="btn shrink-0 text-teks" href="/suhu">Nilai sekarang</a>
				</div>
			{/if}
			{#if terlambat.length}
				<div class="catatan" role="alert">
					<Clock aria-hidden="true" />
					<div class="flex-1">
						<p class="text-base font-bold">{terlambat.length} kantong lebih dari 30 menit di luar bank darah</p>
						<p class="text-sm">
							{terlambat.map((k) => `${k.nomor_kantong} · ${k.ruang}`).join('; ')} — transfusi belum dimulai. Hubungi ruang atau minta kantong dikembalikan.
						</p>
					</div>
				</div>
			{/if}
		</div>
	{/if}

	{#if sesi.boleh('stok.lihat')}
		<section class="grid gap-4 sm:grid-cols-2 xl:grid-cols-4" aria-label="Angka utama">
			{#each [
				{ judul: 'Kantong tersedia', nilai: jumlah((r) => r.tersedia), ket: 'Belum kedaluwarsa, siap dipakai', ikon: Package, nada: 'aman' },
				{ judul: 'Segera kedaluwarsa', nilai: jumlah((r) => r.mendekati_kedaluwarsa), ket: 'Utamakan keluar (FEFO)', ikon: Hourglass, nada: 'waspada' },
				{ judul: 'Permintaan berjalan', nilai: permintaan.data?.length ?? '—', ket: darurat ? `${darurat} darurat` : 'Tidak ada yang darurat', ikon: ClipboardList, nada: 'info' },
				{ judul: 'Karantina', nilai: jumlah((r) => r.karantina), ket: 'Menunggu penilaian atau hasil', ikon: ShieldAlert, nada: 'bahaya' }
			] as k (k.judul)}
				<div class="panel flex flex-col gap-2">
					<div class="flex items-center justify-between">
						<p class="text-sm font-semibold text-redup">{k.judul}</p>
						<span class="grid size-8 place-items-center rounded-lg {NADA[k.nada][0]}"><k.ikon class="size-4" aria-hidden="true" /></span>
					</div>
					<p class="text-4xl font-extrabold tracking-tight tabular-nums">{stok.data ? k.nilai : '—'}</p>
					<p class="text-xs font-semibold {NADA[k.nada][1]}">{k.ket}</p>
				</div>
			{/each}
		</section>

		<section class="panel" aria-labelledby="judul-stok">
			<div class="mb-4 flex items-start justify-between gap-3">
				<div>
					<h2 id="judul-stok">Stok tersedia per golongan</h2>
					<p class="text-sm text-redup">
						Semua komponen{rencana.data ? ' · garis merah bila di bawah stok minimal (rerata pemakaian mingguan)' : ''}
					</p>
				</div>
				<a class="flex items-center gap-1 text-sm font-semibold text-teks-2 hover:underline" href="/stok">Rincian stok<ChevronRight class="size-4" aria-hidden="true" /></a>
			</div>
			<Keadaan sumber={stok} apakahKosong={() => false} baris={2}>
				{#snippet children()}
					<ul class="grid grid-cols-2 gap-3 sm:grid-cols-4 xl:grid-cols-8">
						{#each perGol as g (g.abo + g.rhesus)}
							<li class="flex flex-col items-start gap-2.5 rounded-xl border bg-latar/60 p-3.5 {g.kurang ? 'border-bahaya-titik/60' : 'border-garis'}">
								<Golongan abo={g.abo} rhesus={g.rhesus} />
								<p class="flex items-baseline gap-1">
									<span class="text-3xl font-extrabold tracking-tight tabular-nums {g.kurang ? 'text-bahaya' : ''}">{g.n}</span>
									<span class="text-xs font-semibold text-redup">ktg</span>
								</p>
								{#if g.minimal}
									<span class="block h-1.5 self-stretch overflow-hidden rounded-full bg-garis" aria-hidden="true">
										<span
											class="block h-full rounded-full {g.kurang ? 'bg-bahaya-titik' : 'bg-aman-titik'}"
											style="width: {Math.min(100, Math.max(4, (g.n / (2 * g.minimal)) * 100))}%"
										></span>
									</span>
									{#if g.kurang}<span class="text-xs font-semibold text-bahaya">Di bawah minimal {g.minimal}</span>{/if}
								{/if}
								<span class="text-[0.73rem] font-medium text-redup">{g.komp}</span>
							</li>
						{/each}
					</ul>
				{/snippet}
			</Keadaan>
		</section>
	{/if}

	<div class="grid items-start gap-5 xl:grid-cols-[1fr_380px]">
		<div class="flex min-w-0 flex-col gap-5">
			{#if sesi.boleh('permintaan.lihat')}
				<section class="panel" aria-labelledby="judul-permintaan">
					<div class="mb-3 flex items-start justify-between gap-3">
						<div>
							<h2 id="judul-permintaan">Permintaan berjalan</h2>
							<p class="text-sm text-redup">Diperbarui otomatis tiap menit</p>
						</div>
						<a class="flex items-center gap-1 text-sm font-semibold text-teks-2 hover:underline" href="/permintaan"
							>Semua permintaan<ChevronRight class="size-4" aria-hidden="true" /></a
						>
					</div>
					<Keadaan sumber={permintaan}>
						{#snippet kosong()}<p>Tidak ada permintaan darah yang sedang berjalan.</p>{/snippet}
						{#snippet children(data)}
							<div class="overflow-x-auto">
								<table class="tabel">
									<thead>
										<tr><th scope="col">Pasien</th><th scope="col">Ruang</th><th scope="col">Permintaan</th><th scope="col">Status</th><th scope="col" class="text-right">Masuk</th></tr>
									</thead>
									<tbody>
										{#each data.slice(0, 8) as r (r.id)}
											{@const st = STATUS_PERMINTAAN[r.status]}
											<tr>
												<td>
													<a class="font-semibold hover:underline" href="/permintaan/{r.id}">{r.pasien.nama}</a>
													<span class="mono block text-xs text-redup">{r.no_rm}</span>
												</td>
												<td class="text-teks-2">{r.ruang}</td>
												<td>
													<span class="flex items-center gap-2">
														{#if r.golongan_terverifikasi}
															<Golongan abo={r.golongan_terverifikasi.abo} rhesus={r.golongan_terverifikasi.rhesus} ukuran="kecil" />
														{/if}
														<span class="text-teks-2">{ringkasBaris(r.baris)}</span>
													</span>
												</td>
												<td>
													<span class="flex flex-wrap gap-1.5">
														<Chip nada={st?.[1]}>{st?.[0] ?? r.status}</Chip>
														{#if r.jenis === 'darurat'}<Chip nada="bahaya" polos>Darurat</Chip>{/if}
													</span>
												</td>
												<td class="text-right text-xs whitespace-nowrap text-redup">{relatif(r.diminta_pada)}</td>
											</tr>
										{/each}
									</tbody>
								</table>
							</div>
						{/snippet}
					</Keadaan>
				</section>

				<section class="panel" aria-labelledby="judul-keluar">
					<h2 id="judul-keluar" class="mb-3">Kantong di luar bank darah</h2>
					<Keadaan sumber={keluar}>
						{#snippet kosong()}<p>Tidak ada kantong yang menunggu ditransfusikan.</p>{/snippet}
						{#snippet children(data)}
							<div class="overflow-x-auto">
								<table class="tabel">
									<thead><tr><th scope="col">Kantong</th><th scope="col">Pasien</th><th scope="col">Di luar</th></tr></thead>
									<tbody>
										{#each data as k (k.id)}
											<tr>
												<td>
													<span class="flex items-center gap-2">
														<Golongan abo={k.abo} rhesus={k.rhesus} ukuran="kecil" />
														<span class="mono">{k.nomor_kantong}</span>
													</span>
													<span class="block text-xs text-redup">{KOMPONEN[k.komponen]}</span>
												</td>
												<td>{k.pasien.nama}<span class="block text-xs text-redup">{k.ruang}</span></td>
												<td>
													<Chip nada={k.terlambat ? 'bahaya' : k.menit > 20 ? 'waspada' : 'netral'}>{k.menit} menit</Chip>
													<span class="block text-xs text-redup">sejak {waktu(k.dikeluarkan_pada)}</span>
												</td>
											</tr>
										{/each}
									</tbody>
								</table>
							</div>
						{/snippet}
					</Keadaan>
				</section>
			{/if}
		</div>

		{#if sesi.boleh('stok.lihat')}
			<aside class="flex flex-col gap-5">
				<section class="panel" aria-labelledby="judul-suhu">
					<div class="mb-3 flex items-center justify-between">
						<h2 id="judul-suhu">Suhu alat simpan</h2>
						<a class="flex items-center gap-1 text-sm font-semibold text-teks-2 hover:underline" href="/suhu">Semua alat<ChevronRight class="size-4" aria-hidden="true" /></a>
					</div>
					<Keadaan sumber={alat} baris={3}>
						{#snippet kosong()}<p>Belum ada alat simpan. Tambahkan di Data induk.</p>{/snippet}
						{#snippet children(data)}
							<ul class="flex flex-col gap-2">
								{#each data.filter((a) => a.aktif) as a (a.id)}
									{@const buruk = a.penyimpangan_id || keluarRentang(a)}
									<li class="flex items-center gap-3 rounded-[10px] px-3 py-2.5 {buruk ? 'bg-bahaya-latar' : 'bg-latar/70'}">
										<Thermometer class="size-[18px] shrink-0 {buruk ? 'text-bahaya' : 'text-redup'}" aria-hidden="true" />
										<span class="min-w-0 flex-1">
											<span class="block truncate text-sm font-semibold">{a.nama}</span>
											<span class="block text-xs text-redup">Rentang {suhuTeks(a.suhu_min)} – {suhuTeks(a.suhu_maks)}</span>
										</span>
										<span class="flex flex-col items-end gap-1">
											<span class="text-[0.95rem] font-bold tabular-nums {buruk ? 'text-bahaya' : ''}"
												>{a.suhu_terakhir ? suhuTeks(a.suhu_terakhir.suhu) : '—'}</span
											>
											{#if buruk}
												<Chip nada="bahaya">Menyimpang</Chip>
											{:else if a.suhu_terakhir}
												<span class="text-xs text-redup">pukul {pukul(a.suhu_terakhir.waktu)}</span>
											{:else}
												<span class="text-xs text-redup">belum dicatat</span>
											{/if}
										</span>
									</li>
								{/each}
							</ul>
						{/snippet}
					</Keadaan>
				</section>

				<section class="panel" aria-labelledby="judul-segera">
					<div class="mb-3 flex items-center justify-between">
						<h2 id="judul-segera">Segera kedaluwarsa</h2>
						<a class="flex items-center gap-1 text-sm font-semibold text-teks-2 hover:underline" href="/stok">Stok<ChevronRight class="size-4" aria-hidden="true" /></a>
					</div>
					<Keadaan sumber={kantong} apakahKosong={() => segera.length === 0} baris={3}>
						{#snippet kosong()}<p>Tidak ada kantong tersedia yang mendekati kedaluwarsa.</p>{/snippet}
						{#snippet children()}
							<ul class="flex flex-col gap-3">
								{#each segera as k (k.id)}
									<li class="flex items-center gap-2.5">
										<Golongan abo={k.abo} rhesus={k.rhesus} ukuran="kecil" />
										<span class="min-w-0 flex-1">
											<a class="mono block truncate text-sm hover:underline" href="/lacak/{encodeURIComponent(k.nomor)}">{k.nomor}</a>
											<span class="block text-xs text-redup">{KOMPONEN[k.komponen]} · {k.alat ?? '—'}</span>
										</span>
										<Chip nada="waspada">{relatif(k.kedaluwarsa)}</Chip>
									</li>
								{/each}
							</ul>
						{/snippet}
					</Keadaan>
				</section>
			</aside>
		{/if}
	</div>
</div>
