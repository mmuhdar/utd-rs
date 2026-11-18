<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { api } from '#lib/api.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Isian from '#lib/Isian.svelte';
	import DialogSandi from '#lib/DialogSandi.svelte';
	import StatusKantong from '#lib/StatusKantong.svelte';
	import { sesi } from '#lib/sesi.svelte.ts';
	import { KOMPONEN, golongan, waktu } from '#lib/format.ts';
	import { ABO, RHESUS, alatUntuk, labelAlat, type Alat } from '#lib/stok/stok.ts';
	import Kedaluwarsa from '#lib/stok/Kedaluwarsa.svelte';
	import Golongan from '#lib/Golongan.svelte';
	import Pindai from '#lib/Pindai.svelte';
	import PackagePlus from '@lucide/svelte/icons/package-plus';

	type Kantong = {
		id: string;
		nomor: string;
		komponen: string;
		abo: string;
		rhesus: string;
		volume_ml: number | null;
		kedaluwarsa: string;
		status: string;
		alasan_status: string | null;
		alat: string | null;
		alat_id: string | null;
		asal: string;
		dicairkan_pada: string | null;
		permintaan: string | null;
		mendekati_kedaluwarsa: boolean;
		lewat_kedaluwarsa: boolean;
	};

	// Filter disimpan di query URL dan diteruskan apa adanya ke server (nama parameternya sama).
	const stok = new Sumber(() => api<Kantong[]>('GET', `/stok${page.url.search}`));
	const alat = new Sumber(() => api<Alat[]>('GET', '/master/alat'));
	$effect(() => {
		void page.url.search;
		stok.muat();
	});
	$effect(() => {
		alat.muat();
	});

	const filter = (k: string) => page.url.searchParams.get(k) ?? '';
	function ubahFilter(k: string, v: string) {
		const q = new URLSearchParams(page.url.search);
		if (v) q.set(k, v);
		else q.delete(k);
		goto(`?${q}`, { replace: true, reset: false });
	}

	const kelola = $derived(sesi.boleh('stok.kelola'));
	const bisaCair = (k: Kantong) =>
		kelola && (k.komponen === 'ffp' || k.komponen === 'cryo') && !k.dicairkan_pada && ['tersedia', 'dialokasikan'].includes(k.status) && !k.lewat_kedaluwarsa;
	const bisaKarantina = (k: Kantong) => kelola && ['tersedia', 'dialokasikan'].includes(k.status);
	const bisaLepas = (k: Kantong) => sesi.boleh('karantina.lepas') && k.status === 'karantina' && k.asal !== 'donasi';

	// Satu dialog untuk semua aksi per kantong.
	type Jenis = 'cairkan' | 'pindah' | 'karantina' | 'lepas';
	const JUDUL: Record<Jenis, string> = {
		cairkan: 'Cairkan kantong',
		pindah: 'Pindahkan ke alat lain',
		karantina: 'Karantina kantong',
		lepas: 'Lepas karantina'
	};
	let dialog: HTMLDialogElement | undefined = $state();
	let ttd: DialogSandi | undefined = $state();
	let jenis = $state<Jenis>('pindah');
	let kantong = $state<Kantong | null>(null);
	let alat_id = $state('');
	let alasan = $state('');
	const aksi = new Aksi();

	const pilihanAlat = $derived(
		kantong ? alatUntuk(alat.data ?? [], kantong.komponen, jenis === 'cairkan' || !!kantong.dicairkan_pada).filter((a) => a.id !== kantong?.alat_id) : []
	);

	function buka(j: Jenis, k: Kantong) {
		jenis = j;
		kantong = k;
		alat_id = '';
		alasan = '';
		aksi.galat = aksi.berhasil = null;
		aksi.isian = {};
		dialog?.showModal();
	}

	async function kirim(sandi?: string) {
		if (!kantong) return;
		const body = jenis === 'karantina' || jenis === 'lepas' ? { alasan, sandi } : { alat_id: alat_id || null };
		const ok = await aksi.jalankan(() => api('POST', `/kantong/${kantong!.id}/${jenis === 'lepas' ? 'lepas-karantina' : jenis}`, body));
		if (ok !== undefined) {
			dialog?.close();
			stok.muat();
			alat.muat();
		}
	}

	function submit(e: SubmitEvent) {
		e.preventDefault();
		if (jenis === 'lepas') ttd?.buka();
		else kirim();
	}
</script>

<svelte:head><title>Stok kantong — utd-rs</title></svelte:head>

<div class="mb-5 flex flex-wrap items-end justify-between gap-3">
	<div>
		<h1>Stok kantong</h1>
		{#if stok.data}
			<p class="mt-1 font-medium text-redup">
				{stok.data.filter((k) => k.status === 'tersedia').length} tersedia · {stok.data.filter((k) => k.status === 'karantina').length} karantina ·
				{stok.data.filter((k) => k.status === 'dialokasikan').length} dialokasikan
			</p>
		{/if}
	</div>
	{#if kelola}
		<a class="btn btn--utama" href="/stok/terima"><PackagePlus class="size-4" aria-hidden="true" />Terima dari UTD</a>
	{/if}
</div>

<div class="panel mb-4 p-4">
	<Pindai
		id="pindai-stok"
		label="Pindai label kantong atau ketik nomornya, lalu Enter untuk melacak"
		autofocus
		onpindai={(v) => goto(`/lacak/${encodeURIComponent(v)}`)}
	/>
</div>

<form class="mb-4 flex flex-wrap items-end gap-3" aria-label="Filter stok" onsubmit={(e) => e.preventDefault()}>
	<div role="group" aria-label="Komponen" class="flex flex-wrap gap-0.5 rounded-[10px] bg-halus p-1">
		{#each [['', 'Semua'], ...Object.entries(KOMPONEN)] as [k, n] (k)}
			<button
				type="button"
				class="cursor-pointer rounded-lg px-3 py-1.5 text-sm font-medium text-redup hover:text-teks aria-pressed:bg-kartu aria-pressed:font-semibold aria-pressed:text-teks aria-pressed:shadow-kartu"
				aria-pressed={filter('komponen') === k}
				onclick={() => ubahFilter('komponen', k)}>{n}</button
			>
		{/each}
	</div>
	<div>
		<label class="sr-only" for="f-abo">Golongan ABO</label>
		<select id="f-abo" class="input w-auto" value={filter('abo')} onchange={(e) => ubahFilter('abo', e.currentTarget.value)}>
			<option value="">Semua golongan</option>
			{#each ABO as a (a)}<option value={a}>Golongan {a}</option>{/each}
		</select>
	</div>
	<div>
		<label class="sr-only" for="f-rhesus">Rhesus</label>
		<select id="f-rhesus" class="input w-auto" value={filter('rhesus')} onchange={(e) => ubahFilter('rhesus', e.currentTarget.value)}>
			<option value="">Semua rhesus</option>
			{#each Object.entries(RHESUS) as [k, n] (k)}<option value={k}>{n}</option>{/each}
		</select>
	</div>
	<div>
		<label class="sr-only" for="f-status">Status</label>
		<select id="f-status" class="input w-auto" value={filter('status')} onchange={(e) => ubahFilter('status', e.currentTarget.value)}>
			<option value="">Semua di bank darah</option>
			<option value="tersedia">Tersedia</option>
			<option value="karantina">Karantina</option>
			<option value="dialokasikan">Dialokasikan</option>
		</select>
	</div>
</form>

<section class="panel" aria-labelledby="judul-tabel">
	<div class="mb-3 flex flex-wrap items-baseline justify-between gap-2">
		<h2 id="judul-tabel">Kedaluwarsa terdekat lebih dulu (FEFO)</h2>
		{#if stok.data}<p class="text-sm text-redup">{stok.data.length} kantong</p>{/if}
	</div>
	<Keadaan sumber={stok}>
		{#snippet kosong()}<p>Tidak ada kantong yang cocok dengan filter ini.</p>{/snippet}
		{#snippet children(data)}
			<div class="overflow-x-auto">
				<table class="tabel">
					<thead>
						<tr>
							<th scope="col">Nomor kantong</th>
							<th scope="col">Komponen</th>
							<th scope="col">Golongan</th>
							<th scope="col">Kedaluwarsa</th>
							<th scope="col">Status</th>
							<th scope="col">Alat</th>
							<th scope="col"><span class="sr-only">Aksi</span></th>
						</tr>
					</thead>
					<tbody>
						{#each data as k (k.id)}
							<tr>
								<td><a class="mono font-semibold hover:underline" href="/lacak/{encodeURIComponent(k.nomor)}">{k.nomor}</a></td>
								<td>
									{KOMPONEN[k.komponen]}
									{#if k.volume_ml}<span class="block text-xs text-redup">{k.volume_ml} mL</span>{/if}
									{#if k.dicairkan_pada}<span class="block text-xs text-waspada">Dicairkan {waktu(k.dicairkan_pada)}</span>{/if}
								</td>
								<td><Golongan abo={k.abo} rhesus={k.rhesus} /></td>
								<td><Kedaluwarsa nilai={k.kedaluwarsa} mendekati={k.mendekati_kedaluwarsa} /></td>
								<td>
									<StatusKantong status={k.status} />
									{#if k.permintaan}<span class="block text-xs text-redup">untuk {k.permintaan}</span>{/if}
									{#if k.alasan_status && k.status === 'karantina'}<span class="block max-w-60 text-xs text-redup">{k.alasan_status}</span>{/if}
								</td>
								<td class="mono">{k.alat ?? '—'}</td>
								<td>
									<div class="flex flex-wrap justify-end gap-1">
										{#if kelola}<button type="button" class="btn btn--teks" onclick={() => buka('pindah', k)}>Pindah</button>{/if}
										{#if bisaCair(k)}<button type="button" class="btn btn--teks" onclick={() => buka('cairkan', k)}>Cairkan</button>{/if}
										{#if bisaKarantina(k)}<button type="button" class="btn btn--teks" onclick={() => buka('karantina', k)}>Karantina</button>{/if}
										{#if bisaLepas(k)}<button type="button" class="btn btn--teks" onclick={() => buka('lepas', k)}>Lepas karantina</button>{/if}
									</div>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/snippet}
	</Keadaan>
</section>

<dialog bind:this={dialog} class="dialog">
	{#if kantong}
		<form class="flex flex-col gap-4 p-5" onsubmit={submit}>
			<h2>{JUDUL[jenis]}</h2>
			<p class="text-sm">
				<span class="mono font-medium">{kantong.nomor}</span> · {KOMPONEN[kantong.komponen]}
				{golongan(kantong.abo, kantong.rhesus)} · di <span class="mono">{kantong.alat ?? '—'}</span>
			</p>
			{#if jenis === 'cairkan'}
				<p class="text-sm text-redup">Kedaluwarsa dipersingkat sesuai aturan komponen. Plasma cair disimpan di lemari darah.</p>
			{/if}
			{#if jenis === 'karantina' && kantong.status === 'dialokasikan'}
				<p class="catatan">Kantong ini dialokasikan untuk {kantong.permintaan}; alokasinya akan dilepas.</p>
			{/if}
			{#if jenis === 'cairkan' || jenis === 'pindah'}
				<Isian label={jenis === 'cairkan' ? 'Lemari tujuan' : 'Alat simpan tujuan'} id="aksi-alat" galat={aksi.isian.alat_id}>
					<select id="aksi-alat" class="input" bind:value={alat_id} required aria-invalid={!!aksi.isian.alat_id}>
						<option value="" disabled>Pilih alat</option>
						{#each pilihanAlat as a (a.id)}<option value={a.id}>{labelAlat(a)}</option>{/each}
					</select>
				</Isian>
				{#if !pilihanAlat.length}<p class="text-sm text-waspada">Tidak ada alat aktif yang sesuai dan bebas penyimpangan suhu.</p>{/if}
			{:else}
				<Isian label={jenis === 'lepas' ? 'Alasan pelepasan karantina' : 'Alasan karantina'} id="aksi-alasan" galat={aksi.isian.alasan}>
					<textarea id="aksi-alasan" class="input" rows="3" bind:value={alasan} required aria-invalid={!!aksi.isian.alasan}></textarea>
				</Isian>
			{/if}
			<PesanAksi {aksi} />
			<div class="flex justify-end gap-2">
				<button type="button" class="btn" onclick={() => dialog?.close()}>Batal</button>
				<button class="btn {jenis === 'karantina' ? 'btn--bahaya' : 'btn--utama'}" disabled={aksi.sibuk}>
					{jenis === 'lepas' ? 'Lanjut ke tanda tangan' : JUDUL[jenis]}
				</button>
			</div>
		</form>
	{/if}
</dialog>

<DialogSandi
	bind:this={ttd}
	judul="Tanda tangani pelepasan karantina"
	makna="Anda menyatakan kantong ini layak kembali ke stok tersedia. Tindakan ini tercatat atas nama Anda."
	onkonfirmasi={async ({ sandi }) => {
		await kirim(sandi);
		return true;
	}}
/>
