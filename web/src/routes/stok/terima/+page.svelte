<script lang="ts">
	import { merek } from '#lib/merek.svelte.ts';
	import { goto } from '$app/navigation';
	import { api } from '#lib/api.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Isian from '#lib/Isian.svelte';
	import Pindai from '#lib/Pindai.svelte';
	import Chip from '#lib/Chip.svelte';
	import { sesi } from '#lib/sesi.svelte.ts';
	import { NAMA_KOMPONEN, keRfc3339, lokal, waktu } from '#lib/format.ts';
	import { urai } from '#lib/isbt.ts';
	import { ABO, RHESUS, alatUntuk, labelAlat, suhu, type Alat, type Utd } from '#lib/stok/stok.ts';
	import Trash from '@lucide/svelte/icons/trash-2';

	type Baris = {
		nomor: string;
		nomor_donasi: string;
		kode_produk: string;
		komponen: string;
		abo: string;
		rhesus: string;
		volume_ml: number | null;
		kedaluwarsa: string;
		imltd_nonreaktif: boolean;
		diterima: boolean;
		alasan_tolak: string;
		alat_id: string;
	};

	const utdList = new Sumber(() => api<Utd[]>('GET', '/master/utd'));
	const alat = new Sumber(() => api<Alat[]>('GET', '/master/alat'));
	const permintaanUtd = new Sumber(() => api<any[]>('GET', '/permintaan-utd'));
	const riwayat = new Sumber(() => api<any[]>('GET', '/penerimaan'));
	$effect(() => {
		utdList.muat();
		alat.muat();
		permintaanUtd.muat();
		riwayat.muat();
	});

	let utd_id = $state('');
	let permintaan_utd_id = $state('');
	let nomor_dokumen_utd = $state('');
	let suhu_min_transport = $state<number | null>(null);
	let suhu_maks_transport = $state<number | null>(null);
	let petugas_utd = $state('');
	let catatan = $state('');
	let kantong = $state<Baris[]>([]);
	let pesanPindai = $state('');
	const aksi = new Aksi();

	const utd = $derived(utdList.data?.find((u) => u.id === utd_id));
	const isbt = $derived(utd?.format_label === 'isbt128');

	const baru = (nomor: string, nomor_donasi = ''): Baris => ({
		nomor,
		nomor_donasi,
		kode_produk: '',
		komponen: '',
		abo: '',
		rhesus: '',
		volume_ml: null,
		kedaluwarsa: '',
		imltd_nonreaktif: false,
		diterima: true,
		alasan_tolak: '',
		alat_id: ''
	});

	function tambah(b: Baris) {
		if (kantong.some((k) => k.nomor === b.nomor)) {
			pesanPindai = `Kantong ${b.nomor} sudah ada di daftar.`;
			return;
		}
		kantong.push(b);
		pesanPindai = `Kantong ${b.nomor} ditambahkan (baris ${kantong.length}).`;
	}

	function pindai(v: string) {
		if (!isbt) return tambah(baru(v));
		const p = urai(v);
		if (p.jenis === 'din') return tambah(baru(p.nomor, p.nomor_donasi));
		const akhir = kantong.at(-1);
		if (p.jenis === 'lain') {
			pesanPindai = `Pindaian "${p.teks}" tidak dikenali sebagai label ISBT 128. Isi manual bila perlu.`;
			return;
		}
		if (!akhir) {
			pesanPindai = 'Pindai nomor donasi (DIN) lebih dulu untuk memulai baris kantong.';
			return;
		}
		if (p.jenis === 'golongan') {
			akhir.abo = p.abo;
			akhir.rhesus = p.rhesus;
			pesanPindai = `Golongan ${p.abo} ${p.rhesus} diisi ke ${akhir.nomor}.`;
		} else if (p.jenis === 'produk') {
			akhir.kode_produk = p.kode_produk;
			pesanPindai = `Kode produk ${p.kode_produk} diisi ke ${akhir.nomor}. Pilih komponennya.`;
		} else {
			akhir.kedaluwarsa = lokal(p.waktu);
			pesanPindai = `Kedaluwarsa ${waktu(p.waktu.toISOString())} diisi ke ${akhir.nomor}.`;
		}
	}

	const g = (n: number, f: string) => aksi.isian[`kantong.${n}.${f}`];

	async function simpan(e: SubmitEvent) {
		e.preventDefault();
		const r = await aksi.jalankan(() =>
			api<{ id: string }>('POST', '/penerimaan', {
				utd_id,
				permintaan_utd_id: permintaan_utd_id || null,
				nomor_dokumen_utd,
				suhu_min_transport,
				suhu_maks_transport,
				petugas_utd,
				catatan,
				kantong: kantong.map((k) => ({
					...k,
					volume_ml: k.volume_ml || null,
					kedaluwarsa: keRfc3339(k.kedaluwarsa),
					alat_id: k.diterima ? k.alat_id || null : null,
					alasan_tolak: k.diterima ? null : k.alasan_tolak
				}))
			})
		);
		if (r) goto(`/stok/terima/${r.id}`);
	}
</script>

<svelte:head><title>Penerimaan dari UTD — {merek.judul}</title></svelte:head>

<h1 class="mb-4">Penerimaan dari UTD</h1>

{#if sesi.boleh('stok.kelola')}
	<form class="mb-6 flex flex-col gap-4" onsubmit={simpan}>
		<section class="panel" aria-labelledby="judul-serah">
			<h2 id="judul-serah" class="mb-3">Berita serah terima</h2>
			<div class="form-grid">
				<Isian label="UTD mitra" id="utd" galat={aksi.isian.utd_id}>
					<select id="utd" class="input" bind:value={utd_id} required>
						<option value="" disabled>Pilih UTD</option>
						{#each (utdList.data ?? []).filter((u) => u.aktif) as u (u.id)}
							<option value={u.id}>{u.nama} ({u.format_label === 'isbt128' ? 'label ISBT 128' : 'label lokal'})</option>
						{/each}
					</select>
				</Isian>
				<Isian label="Permintaan ke UTD (bila ada)" id="pu">
					<select id="pu" class="input" bind:value={permintaan_utd_id}>
						<option value="">Tanpa permintaan</option>
						{#each (permintaanUtd.data ?? []).filter((p) => p.utd_id === utd_id && p.status === 'dikirim') as p (p.id)}
							<option value={p.id}>{p.nomor} · {waktu(p.dibuat_pada)}</option>
						{/each}
					</select>
				</Isian>
				<Isian label="Nomor dokumen UTD" id="dok" galat={aksi.isian.nomor_dokumen_utd}>
					<input id="dok" class="input mono" bind:value={nomor_dokumen_utd} />
				</Isian>
				<Isian label="Petugas UTD yang menyerahkan" id="petugas" galat={aksi.isian.petugas_utd}>
					<input id="petugas" class="input" bind:value={petugas_utd} required aria-invalid={!!aksi.isian.petugas_utd} />
				</Isian>
				<Isian label="Suhu minimum selama transportasi (°C)" id="smin">
					<input id="smin" class="input" type="number" step="0.1" bind:value={suhu_min_transport} />
				</Isian>
				<Isian label="Suhu maksimum selama transportasi (°C)" id="smaks">
					<input id="smaks" class="input" type="number" step="0.1" bind:value={suhu_maks_transport} />
				</Isian>
			</div>
			<div class="mt-4">
				<Isian label="Catatan" id="catatan">
					<textarea id="catatan" class="input" rows="2" bind:value={catatan}></textarea>
				</Isian>
			</div>
		</section>

		<section class="panel" aria-labelledby="judul-kantong">
			<h2 id="judul-kantong" class="mb-1">Kantong</h2>
			{#if !utd}
				<p class="text-sm text-redup">Pilih UTD mitra lebih dulu; cara memindai mengikuti format labelnya.</p>
			{:else}
				<p class="mb-3 text-sm text-redup">
					{#if isbt}
						Pindai nomor donasi (DIN) untuk memulai baris baru, lalu pindai golongan, kode produk, dan kedaluwarsa kantong yang sama.
					{:else}
						Pindai atau ketik nomor kantong; isian lain diisi dari label.
					{/if}
				</p>
				<div class="max-w-md">
					<Pindai label="Pindai label kantong" id="pindai" onpindai={pindai} autofocus />
				</div>
				<p class="mt-1 min-h-5 text-sm" role="status">{pesanPindai}</p>
			{/if}
			{#if aksi.isian.kantong}<p class="mt-2 text-sm text-bahaya" role="alert">{aksi.isian.kantong}</p>{/if}

			<ol class="mt-3 flex flex-col gap-3">
				{#each kantong as k, n (n)}
					{@const pilihan = alatUntuk(alat.data ?? [], k.komponen)}
					<li class="rounded-md border p-3 {k.diterima ? 'border-garis' : 'border-bahaya/40 bg-bahaya-latar'}">
						<div class="mb-2 flex items-center justify-between gap-2">
							<p class="text-sm font-semibold">Kantong {n + 1} {#if !k.diterima}<Chip nada="bahaya">Ditolak</Chip>{/if}</p>
							<button type="button" class="btn btn--teks" onclick={() => kantong.splice(n, 1)}>
								<Trash class="size-4" aria-hidden="true" />Hapus baris<span class="sr-only"> kantong {n + 1}</span>
							</button>
						</div>
						<div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
							<Isian label="Nomor kantong" id="k{n}-nomor" galat={g(n, 'nomor')}>
								<input id="k{n}-nomor" class="input mono" bind:value={k.nomor} required aria-invalid={!!g(n, 'nomor')} />
							</Isian>
							<Isian label="Komponen" id="k{n}-komponen" galat={g(n, 'komponen')} bantuan={k.kode_produk ? `Kode produk ${k.kode_produk}` : undefined}>
								<select id="k{n}-komponen" class="input" bind:value={k.komponen} required aria-invalid={!!g(n, 'komponen')}>
									<option value="" disabled>Pilih</option>
									{#each Object.entries(NAMA_KOMPONEN) as [v, l] (v)}<option value={v}>{l}</option>{/each}
								</select>
							</Isian>
							<Isian label="Golongan ABO" id="k{n}-abo" galat={g(n, 'abo')}>
								<select id="k{n}-abo" class="input" bind:value={k.abo} required aria-invalid={!!g(n, 'abo')}>
									<option value="" disabled>Pilih</option>
									{#each ABO as a (a)}<option value={a}>{a}</option>{/each}
								</select>
							</Isian>
							<Isian label="Rhesus" id="k{n}-rhesus">
								<select id="k{n}-rhesus" class="input" bind:value={k.rhesus} required aria-invalid={!!g(n, 'abo')}>
									<option value="" disabled>Pilih</option>
									{#each Object.entries(RHESUS) as [v, l] (v)}<option value={v}>{l}</option>{/each}
								</select>
							</Isian>
							<Isian label="Volume (mL)" id="k{n}-volume" galat={g(n, 'volume_ml')}>
								<input id="k{n}-volume" class="input" type="number" min="1" bind:value={k.volume_ml} />
							</Isian>
							<Isian label="Kedaluwarsa" id="k{n}-kedaluwarsa" galat={g(n, 'kedaluwarsa')}>
								<input id="k{n}-kedaluwarsa" class="input" type="datetime-local" bind:value={k.kedaluwarsa} required aria-invalid={!!g(n, 'kedaluwarsa')} />
							</Isian>
							<Isian label="Keputusan" id="k{n}-diterima">
								<select id="k{n}-diterima" class="input" bind:value={k.diterima}>
									<option value={true}>Diterima</option>
									<option value={false}>Ditolak (dikembalikan)</option>
								</select>
							</Isian>
							{#if k.diterima}
								<Isian label="Alat simpan" id="k{n}-alat" galat={g(n, 'alat_id')} bantuan={k.komponen ? undefined : 'Pilih komponen lebih dulu.'}>
									<select id="k{n}-alat" class="input" bind:value={k.alat_id} required aria-invalid={!!g(n, 'alat_id')}>
										<option value="" disabled>Pilih alat</option>
										{#each pilihan as a (a.id)}<option value={a.id}>{labelAlat(a)}</option>{/each}
									</select>
								</Isian>
							{:else}
								<Isian label="Alasan penolakan" id="k{n}-alasan" galat={g(n, 'alasan_tolak')}>
									<input id="k{n}-alasan" class="input" bind:value={k.alasan_tolak} required aria-invalid={!!g(n, 'alasan_tolak')} />
								</Isian>
							{/if}
						</div>
						<div class="mt-3">
							<label class="flex items-center gap-2 text-sm">
								<input type="checkbox" class="size-4" bind:checked={k.imltd_nonreaktif} aria-describedby={g(n, 'imltd_nonreaktif') ? `k${n}-imltd-galat` : undefined} />
								Hasil uji saring IMLTD dari UTD non-reaktif
							</label>
							{#if g(n, 'imltd_nonreaktif')}<p class="mt-1 text-sm text-bahaya" id="k{n}-imltd-galat" role="alert">{g(n, 'imltd_nonreaktif')}</p>{/if}
						</div>
					</li>
				{/each}
			</ol>
		</section>

		<PesanAksi {aksi} />
		<div>
			<button class="btn btn--utama btn--besar" disabled={aksi.sibuk || !kantong.length}>
				Simpan berita serah terima ({kantong.length} kantong)
			</button>
		</div>
	</form>
{/if}

<section class="panel" aria-labelledby="judul-riwayat">
	<h2 id="judul-riwayat" class="mb-3">Penerimaan sebelumnya</h2>
	<Keadaan sumber={riwayat}>
		{#snippet kosong()}<p>Belum ada penerimaan dari UTD.</p>{/snippet}
		{#snippet children(data)}
			<div class="overflow-x-auto">
				<table class="tabel">
					<thead>
						<tr>
							<th scope="col">Nomor</th>
							<th scope="col">Diterima</th>
							<th scope="col">UTD</th>
							<th scope="col">Dokumen UTD</th>
							<th scope="col">Suhu transport</th>
							<th scope="col" class="text-right">Kantong</th>
						</tr>
					</thead>
					<tbody>
						{#each data as r (r.id)}
							<tr>
								<td><a class="mono text-utama hover:underline" href="/stok/terima/{r.id}">{r.nomor}</a></td>
								<td class="whitespace-nowrap">{waktu(r.diterima_pada)}</td>
								<td>{r.utd}<span class="block text-xs text-redup">{r.petugas_utd}</span></td>
								<td class="mono">{r.nomor_dokumen_utd ?? '—'}</td>
								<td class="whitespace-nowrap">{suhu(r.suhu_min_transport)} – {suhu(r.suhu_maks_transport)}</td>
								<td class="text-right tabular-nums">
									{r.jumlah}
									{#if r.ditolak}<span class="block text-xs text-bahaya">{r.ditolak} ditolak</span>{/if}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/snippet}
	</Keadaan>
</section>
