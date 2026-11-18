<script lang="ts">
	import { page } from '$app/state';
	import { api } from '#lib/api.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Isian from '#lib/Isian.svelte';
	import Chip from '#lib/Chip.svelte';
	import DialogSandi from '#lib/DialogSandi.svelte';
	import StatusKantong from '#lib/StatusKantong.svelte';
	import { sesi } from '#lib/sesi.svelte.ts';
	import { KOMPONEN, golongan, waktu } from '#lib/format.ts';
	import { suhu } from '#lib/stok/stok.ts';
	import Kedaluwarsa from '#lib/stok/Kedaluwarsa.svelte';

	const p = new Sumber(() => api<any>('GET', `/penyimpangan-suhu/${page.params.id}`));
	$effect(() => {
		void page.params.id;
		p.muat();
	});

	const boleh = $derived(sesi.boleh('karantina.lepas'));
	let barisAktif = $state('');
	const aksiNilai = new Aksi();

	async function simpanNilai(e: SubmitEvent, kantong_id: string) {
		e.preventDefault();
		barisAktif = kantong_id;
		const f = new FormData(e.currentTarget as HTMLFormElement);
		const r = await aksiNilai.jalankan(
			() => api('POST', `/penyimpangan-suhu/${page.params.id}/nilai`, { kantong_id, keputusan: f.get('keputusan'), alasan: f.get('alasan') }),
			'Penilaian disimpan.'
		);
		if (r) p.muat();
	}

	let kesimpulan = $state('');
	let ttd: DialogSandi | undefined = $state();
	const aksiTutup = new Aksi();
	async function tutup(sandi: string) {
		const r = await aksiTutup.jalankan(() => api('POST', `/penyimpangan-suhu/${page.params.id}/tutup`, { kesimpulan, sandi }), 'Penyimpangan ditutup.');
		if (r) p.muat();
		return true;
	}
</script>

<svelte:head><title>Penyimpangan suhu — utd-rs</title></svelte:head>

<a class="mb-4 inline-block text-sm text-utama hover:underline" href="/suhu">Kembali ke suhu alat simpan</a>

<Keadaan sumber={p}>
	{#snippet children(d)}
		{@const kantong = d.kantong ?? []}
		{@const belum = kantong.filter((k: any) => !k.keputusan).length}
		{@const terbuka = d.status === 'terbuka'}
		<div class="mb-4 flex flex-wrap items-center gap-3">
			<h1>Penyimpangan suhu <span class="mono">{d.alat.kode}</span></h1>
			{#if terbuka}<Chip nada="bahaya">Terbuka</Chip>{:else}<Chip>Ditutup</Chip>{/if}
		</div>
		<dl class="panel mb-4 grid gap-x-6 gap-y-2 text-sm sm:grid-cols-2 lg:grid-cols-4">
			<div><dt class="text-redup">Alat</dt><dd>{d.alat.nama} ({d.alat.nomor_seri ?? 'tanpa nomor seri'})</dd></div>
			<div><dt class="text-redup">Rentang diizinkan</dt><dd>{suhu(d.alat.suhu_min)} – {suhu(d.alat.suhu_maks)}</dd></div>
			<div><dt class="text-redup">Mulai</dt><dd>{waktu(d.mulai)}</dd></div>
			<div><dt class="text-redup">Suhu ekstrem</dt><dd class="font-semibold text-bahaya">{suhu(d.suhu_ekstrem)}</dd></div>
			{#if !terbuka}
				<div class="sm:col-span-2 lg:col-span-4"><dt class="text-redup">Kesimpulan ({waktu(d.ditutup_pada)})</dt><dd>{d.kesimpulan}</dd></div>
			{/if}
		</dl>

		<section class="panel mb-4" aria-labelledby="judul-kantong">
			<h2 id="judul-kantong" class="mb-1">Penilaian kantong</h2>
			<p class="mb-3 text-sm text-redup">
				Kantong layak kembali tersedia (kecuali hasil donasi yang belum diluluskan); kantong tidak layak tetap karantina untuk dikembalikan atau dimusnahkan.
			</p>
			<div class="mb-2"><PesanAksi aksi={aksiNilai} /></div>
			{#if !kantong.length}
				<p class="text-sm text-redup">Tidak ada kantong di alat ini saat penyimpangan terjadi.</p>
			{:else}
				<div class="overflow-x-auto">
					<table class="tabel">
						<thead>
							<tr>
								<th scope="col">Kantong</th>
								<th scope="col">Kedaluwarsa</th>
								<th scope="col">Status</th>
								<th scope="col">Penilaian</th>
							</tr>
						</thead>
						<tbody>
							{#each kantong as k (k.kantong_id)}
								<tr>
									<td>
										<a class="mono text-utama hover:underline" href="/lacak/{encodeURIComponent(k.nomor)}">{k.nomor}</a>
										<span class="block text-xs">{KOMPONEN[k.komponen]} {golongan(k.abo, k.rhesus)}</span>
									</td>
									<td><Kedaluwarsa nilai={k.kedaluwarsa} /></td>
									<td><StatusKantong status={k.status} /></td>
									<td class="min-w-72">
										{#if k.keputusan}
											<Chip nada={k.keputusan === 'layak' ? 'aman' : 'bahaya'}>{k.keputusan === 'layak' ? 'Layak' : 'Tidak layak'}</Chip>
											<span class="block text-xs">{k.alasan}</span>
											<span class="block text-xs text-redup">{waktu(k.dinilai_pada)}</span>
										{:else if terbuka && boleh}
											{@const galat = barisAktif === k.kantong_id ? aksiNilai.isian : {}}
											<form class="flex flex-col gap-2" onsubmit={(e) => simpanNilai(e, k.kantong_id)}>
												<fieldset class="flex gap-4 text-sm">
													<legend class="sr-only">Keputusan untuk {k.nomor}</legend>
													<label class="flex items-center gap-1"><input type="radio" class="size-4" name="keputusan" value="layak" required />Layak</label>
													<label class="flex items-center gap-1"><input type="radio" class="size-4" name="keputusan" value="tidak_layak" />Tidak layak</label>
												</fieldset>
												{#if galat.keputusan}<p class="text-sm text-bahaya" role="alert">{galat.keputusan}</p>{/if}
												<Isian label="Dasar penilaian" id="alasan-{k.kantong_id}" galat={galat.alasan}>
													<input id="alasan-{k.kantong_id}" class="input" name="alasan" required aria-invalid={!!galat.alasan} />
												</Isian>
												<button class="btn self-start" disabled={aksiNilai.sibuk}>Simpan penilaian</button>
											</form>
										{:else}
											<Chip nada="waspada">Belum dinilai</Chip>
										{/if}
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{/if}
		</section>

		{#if terbuka && boleh}
			<form
				class="panel flex max-w-2xl flex-col gap-4"
				aria-labelledby="judul-tutup"
				onsubmit={(e) => {
					e.preventDefault();
					ttd?.buka();
				}}
			>
				<h2 id="judul-tutup">Tutup penyimpangan</h2>
				{#if belum}<p class="catatan">Masih ada {belum} kantong yang belum dinilai. Nilai semuanya sebelum menutup.</p>{/if}
				<Isian label="Kesimpulan dan tindakan perbaikan" id="kesimpulan" galat={aksiTutup.isian.kesimpulan}>
					<textarea id="kesimpulan" class="input" rows="3" bind:value={kesimpulan} required aria-invalid={!!aksiTutup.isian.kesimpulan}></textarea>
				</Isian>
				<PesanAksi aksi={aksiTutup} />
				<button class="btn btn--utama self-start" disabled={aksiTutup.sibuk || belum > 0}>Tutup dengan tanda tangan</button>
			</form>
			<DialogSandi
				bind:this={ttd}
				judul="Tanda tangani penutupan penyimpangan"
				makna="Anda menyatakan semua kantong telah dinilai dan tindakan perbaikan sudah ditetapkan."
				onkonfirmasi={({ sandi }) => tutup(sandi)}
			/>
		{/if}
	{/snippet}
</Keadaan>
