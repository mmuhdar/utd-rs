<script lang="ts">
	import { merek } from '#lib/merek.svelte.ts';
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
	import { KOMPONEN, waktu } from '#lib/format.ts';
	import KepalaHalaman from '#lib/KepalaHalaman.svelte';
	import Golongan from '#lib/Golongan.svelte';
	import TriangleAlert from '@lucide/svelte/icons/triangle-alert';
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

<svelte:head><title>Penyimpangan suhu — {merek.judul}</title></svelte:head>

<KepalaHalaman
	judul={p.data ? `Penyimpangan suhu ${p.data.alat.kode}` : 'Penyimpangan suhu'}
	sub={p.data ? `${p.data.alat.nama} · mulai ${waktu(p.data.mulai)}` : undefined}
	jalur={[['Suhu alat simpan', '/suhu'], [p.data ? `Penyimpangan ${p.data.alat.kode}` : 'Penyimpangan']]}
>
	{#if p.data}
		<p class="mt-2">{#if p.data.status === 'terbuka'}<Chip nada="bahaya">Terbuka</Chip>{:else}<Chip>Ditutup</Chip>{/if}</p>
	{/if}
</KepalaHalaman>

<Keadaan sumber={p}>
	{#snippet children(d)}
		{@const kantong = d.kantong ?? []}
		{@const belum = kantong.filter((k: any) => !k.keputusan).length}
		{@const terbuka = d.status === 'terbuka'}
		<section class="mb-5 grid gap-4 sm:grid-cols-2 xl:grid-cols-4" aria-label="Ringkasan">
			<div class="panel flex flex-col gap-1.5 border-bahaya-titik bg-bahaya-latar">
				<p class="text-sm font-semibold text-bahaya">Suhu ekstrem</p>
				<p class="text-4xl font-extrabold tracking-tight text-bahaya tabular-nums">{suhu(d.suhu_ekstrem)}</p>
			</div>
			<div class="panel flex flex-col gap-1.5">
				<p class="text-sm font-semibold text-redup">Rentang diizinkan</p>
				<p class="text-2xl font-extrabold tracking-tight tabular-nums">{suhu(d.alat.suhu_min)} – {suhu(d.alat.suhu_maks)}</p>
			</div>
			<div class="panel flex flex-col gap-1.5">
				<p class="text-sm font-semibold text-redup">Mulai</p>
				<p class="text-2xl font-extrabold tracking-tight">{waktu(d.mulai)}</p>
			</div>
			<div class="panel flex flex-col gap-1.5 {belum ? 'border-waspada-titik' : ''}">
				<p class="text-sm font-semibold text-redup">Kantong dinilai</p>
				<p class="text-4xl font-extrabold tracking-tight tabular-nums">{kantong.length - belum}<span class="text-xl text-redup">/{kantong.length}</span></p>
				{#if belum}<p class="text-xs font-semibold text-waspada">{belum} belum dinilai</p>{/if}
			</div>
		</section>

		<div class="grid items-start gap-5 xl:grid-cols-[1fr_360px]">
			<section class="panel min-w-0" aria-labelledby="judul-kantong">
				<h2 id="judul-kantong" class="mb-1">Penilaian kantong</h2>
				<p class="mb-4 text-sm text-redup">
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
									<th scope="col">Golongan</th>
									<th scope="col">Kedaluwarsa</th>
									<th scope="col">Status</th>
									<th scope="col">Penilaian</th>
								</tr>
							</thead>
							<tbody>
								{#each kantong as k (k.kantong_id)}
									<tr>
										<td>
											<a class="mono font-semibold hover:underline" href="/lacak/{encodeURIComponent(k.nomor)}">{k.nomor}</a>
											<span class="block text-xs text-redup">{KOMPONEN[k.komponen]}</span>
										</td>
										<td><Golongan abo={k.abo} rhesus={k.rhesus} /></td>
										<td><Kedaluwarsa nilai={k.kedaluwarsa} /></td>
										<td><StatusKantong status={k.status} /></td>
										<td class="min-w-72">
											{#if k.keputusan}
												<Chip nada={k.keputusan === 'layak' ? 'aman' : 'bahaya'}>{k.keputusan === 'layak' ? 'Layak' : 'Tidak layak'}</Chip>
												<span class="mt-1 block text-xs text-teks-2">{k.alasan}</span>
												<span class="block text-xs text-redup">{waktu(k.dinilai_pada)}</span>
											{:else if terbuka && boleh}
												{@const galat = barisAktif === k.kantong_id ? aksiNilai.isian : {}}
												<form class="flex flex-col gap-2.5" onsubmit={(e) => simpanNilai(e, k.kantong_id)}>
													<fieldset>
														<legend class="sr-only">Keputusan untuk {k.nomor}</legend>
														<div class="flex w-fit gap-0.5 rounded-[10px] bg-halus p-1 {galat.keputusan ? 'ring-2 ring-bahaya-titik' : ''}">
															<label
																class="inline-flex min-h-9 cursor-pointer items-center rounded-lg px-3 text-sm font-medium text-redup hover:text-teks has-checked:bg-kartu has-checked:font-semibold has-checked:text-teks has-checked:shadow-kartu has-focus-visible:outline-3 has-focus-visible:outline-fokus"
																><input type="radio" class="sr-only" name="keputusan" value="layak" required />Layak</label
															>
															<label
																class="inline-flex min-h-9 cursor-pointer items-center rounded-lg px-3 text-sm font-medium text-redup hover:text-teks has-checked:bg-bahaya-isi has-checked:font-semibold has-checked:text-white has-checked:shadow-kartu has-focus-visible:outline-3 has-focus-visible:outline-fokus"
																><input type="radio" class="sr-only" name="keputusan" value="tidak_layak" />Tidak layak</label
															>
														</div>
													</fieldset>
													{#if galat.keputusan}<p class="text-sm text-bahaya" role="alert">{galat.keputusan}</p>{/if}
													<Isian label="Dasar penilaian" id="alasan-{k.kantong_id}" galat={galat.alasan}>
														<input id="alasan-{k.kantong_id}" class="input" name="alasan" required aria-invalid={!!galat.alasan} />
													</Isian>
													<button class="btn self-end" disabled={aksiNilai.sibuk}>Simpan penilaian</button>
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

			<aside class="flex flex-col gap-5">
				<section class="panel" aria-labelledby="judul-alat">
					<h2 id="judul-alat" class="judul-kecil mb-3">Alat simpan</h2>
					<dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2.5 text-sm">
						<dt class="text-redup">Kode</dt><dd class="mono font-semibold">{d.alat.kode}</dd>
						<dt class="text-redup">Nama</dt><dd class="font-medium">{d.alat.nama}</dd>
						<dt class="text-redup">Nomor seri</dt><dd class="mono">{d.alat.nomor_seri ?? 'tanpa nomor seri'}</dd>
						<dt class="text-redup">Rentang</dt><dd class="font-medium tabular-nums">{suhu(d.alat.suhu_min)} – {suhu(d.alat.suhu_maks)}</dd>
					</dl>
					{#if !terbuka}
						<div class="mt-4 border-t border-garis pt-4">
							<p class="judul-kecil mb-1">Kesimpulan · {waktu(d.ditutup_pada)}</p>
							<p class="text-sm">{d.kesimpulan}</p>
						</div>
					{/if}
				</section>

				{#if terbuka && boleh}
					<form
						class="panel flex flex-col gap-4"
						aria-labelledby="judul-tutup"
						onsubmit={(e) => {
							e.preventDefault();
							ttd?.buka();
						}}
					>
						<h2 id="judul-tutup">Tutup penyimpangan</h2>
						{#if belum}
							<p class="catatan"><TriangleAlert aria-hidden="true" />Masih ada {belum} kantong yang belum dinilai. Nilai semuanya sebelum menutup.</p>
						{/if}
						<Isian label="Kesimpulan dan tindakan perbaikan" id="kesimpulan" galat={aksiTutup.isian.kesimpulan}>
							<textarea id="kesimpulan" class="input" rows="4" bind:value={kesimpulan} required aria-invalid={!!aksiTutup.isian.kesimpulan}></textarea>
						</Isian>
						<PesanAksi aksi={aksiTutup} />
						<div class="flex justify-end gap-2.5 border-t border-garis pt-4">
							<button class="btn btn--utama" disabled={aksiTutup.sibuk || belum > 0}>Tutup dengan tanda tangan</button>
						</div>
					</form>
					<DialogSandi
						bind:this={ttd}
						judul="Tanda tangani penutupan penyimpangan"
						makna="Anda menyatakan semua kantong telah dinilai dan tindakan perbaikan sudah ditetapkan."
						onkonfirmasi={({ sandi }) => tutup(sandi)}
					/>
				{/if}
			</aside>
		</div>
	{/snippet}
</Keadaan>
