<script lang="ts">
	import { goto } from '$app/navigation';
	import { api } from '#lib/api.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Isian from '#lib/Isian.svelte';
	import Chip from '#lib/Chip.svelte';
	import StatusKantong from '#lib/StatusKantong.svelte';
	import { sesi } from '#lib/sesi.svelte.ts';
	import { KOMPONEN, golongan, waktu } from '#lib/format.ts';
	import { ALASAN_AKHIR, type Utd } from '#lib/stok/stok.ts';
	import Kedaluwarsa from '#lib/stok/Kedaluwarsa.svelte';

	const daftar = new Sumber(() => api<any[]>('GET', '/berita-acara'));
	const kelola = $derived(sesi.boleh('stok.kelola'));
	const calon = new Sumber(() => api<any[]>('GET', '/stok?status=karantina,tersedia,dikeluarkan'));
	const utdList = new Sumber(() => api<Utd[]>('GET', '/master/utd'));
	$effect(() => {
		daftar.muat();
		if (kelola) {
			calon.muat();
			utdList.muat();
		}
	});

	let jenis = $state<'pengembalian' | 'pemusnahan'>('pemusnahan');
	let utd_id = $state('');
	let metode = $state('');
	let saksi = $state('');
	let catatan = $state('');
	// id kantong → alasan; kantong terpilih bila ada di peta ini.
	let pilih = $state<Record<string, string>>({});
	const aksi = new Aksi();
	const jumlahPilih = $derived(Object.keys(pilih).length);

	function centang(k: any, v: boolean) {
		if (v) pilih[k.id] = k.lewat_kedaluwarsa ? 'kedaluwarsa' : '';
		else delete pilih[k.id];
	}

	async function simpan(e: SubmitEvent) {
		e.preventDefault();
		const kata = jenis === 'pemusnahan' ? 'dimusnahkan' : 'dikembalikan ke UTD';
		if (!confirm(`${jumlahPilih} kantong akan ${kata}. Tindakan ini tidak dapat dibatalkan. Lanjutkan?`)) return;
		const r = await aksi.jalankan(() =>
			api<{ id: string }>('POST', '/berita-acara', {
				jenis,
				utd_id: jenis === 'pengembalian' ? utd_id || null : null,
				metode,
				saksi,
				catatan,
				kantong: Object.entries(pilih).map(([id, alasan]) => ({ id, alasan }))
			})
		);
		if (r) goto(`/berita-acara/${r.id}`);
	}
</script>

<svelte:head><title>Pengembalian & pemusnahan — utd-rs</title></svelte:head>

<h1 class="mb-4">Pengembalian dan pemusnahan</h1>

{#if kelola}
	<form class="panel mb-6 flex flex-col gap-4" onsubmit={simpan} aria-labelledby="judul-baru">
		<h2 id="judul-baru">Buat berita acara</h2>
		<fieldset>
			<legend class="label">Jenis</legend>
			<div class="flex flex-wrap gap-4 text-sm">
				<label class="flex items-center gap-2"><input type="radio" class="size-4" bind:group={jenis} value="pemusnahan" />Pemusnahan</label>
				<label class="flex items-center gap-2"><input type="radio" class="size-4" bind:group={jenis} value="pengembalian" />Pengembalian ke UTD</label>
			</div>
		</fieldset>
		<div class="form-grid">
			{#if jenis === 'pengembalian'}
				<Isian label="UTD tujuan" id="utd" galat={aksi.isian.utd_id}>
					<select id="utd" class="input" bind:value={utd_id} required aria-invalid={!!aksi.isian.utd_id}>
						<option value="" disabled>Pilih UTD</option>
						{#each utdList.data ?? [] as u (u.id)}<option value={u.id}>{u.nama}</option>{/each}
					</select>
				</Isian>
			{:else}
				<Isian label="Metode pemusnahan" id="metode" galat={aksi.isian.metode} bantuan="Mis. insinerator limbah medis RS.">
					<input id="metode" class="input" bind:value={metode} required aria-invalid={!!aksi.isian.metode} />
				</Isian>
			{/if}
			<Isian label="Saksi" id="saksi">
				<input id="saksi" class="input" bind:value={saksi} />
			</Isian>
		</div>
		<Isian label="Catatan" id="catatan">
			<textarea id="catatan" class="input" rows="2" bind:value={catatan}></textarea>
		</Isian>

		<fieldset>
			<legend class="label">Kantong (karantina, tersedia, atau dikeluarkan tetapi tidak ditransfusikan)</legend>
			{#if aksi.isian.kantong}<p class="mb-2 text-sm text-bahaya" role="alert">{aksi.isian.kantong}</p>{/if}
			<Keadaan sumber={calon}>
				{#snippet kosong()}<p>Tidak ada kantong yang bisa dikembalikan atau dimusnahkan.</p>{/snippet}
				{#snippet children(data)}
					<div class="max-h-[32rem] overflow-auto">
						<table class="tabel">
							<thead>
								<tr>
									<th scope="col">Pilih</th>
									<th scope="col">Nomor kantong</th>
									<th scope="col">Komponen</th>
									<th scope="col">Kedaluwarsa</th>
									<th scope="col">Status</th>
									<th scope="col">Alasan</th>
								</tr>
							</thead>
							<tbody>
								{#each data as k (k.id)}
									<tr>
										<td>
											<input
												type="checkbox"
												class="size-4"
												id="p-{k.id}"
												checked={k.id in pilih}
												onchange={(e) => centang(k, e.currentTarget.checked)}
											/>
										</td>
										<td><label class="mono" for="p-{k.id}">{k.nomor}</label></td>
										<td class="whitespace-nowrap">{KOMPONEN[k.komponen]} {golongan(k.abo, k.rhesus)}</td>
										<td><Kedaluwarsa nilai={k.kedaluwarsa} mendekati={k.mendekati_kedaluwarsa} /></td>
										<td>
											<StatusKantong status={k.status} />
											{#if k.alasan_status}<span class="block max-w-60 text-xs text-redup">{k.alasan_status}</span>{/if}
										</td>
										<td>
											{#if k.id in pilih}
												<label class="sr-only" for="a-{k.id}">Alasan untuk {k.nomor}</label>
												<select id="a-{k.id}" class="input" bind:value={pilih[k.id]} required>
													<option value="" disabled>Pilih alasan</option>
													{#each ALASAN_AKHIR as a (a)}<option value={a}>{a}</option>{/each}
												</select>
											{/if}
										</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/snippet}
			</Keadaan>
		</fieldset>
		<PesanAksi {aksi} />
		<button class="btn btn--bahaya self-start" disabled={aksi.sibuk || !jumlahPilih}>
			Buat berita acara {jenis} ({jumlahPilih} kantong)
		</button>
	</form>
{/if}

<section class="panel" aria-labelledby="judul-daftar">
	<h2 id="judul-daftar" class="mb-3">Daftar berita acara</h2>
	<Keadaan sumber={daftar}>
		{#snippet kosong()}<p>Belum ada berita acara pengembalian atau pemusnahan.</p>{/snippet}
		{#snippet children(data)}
			<div class="overflow-x-auto">
				<table class="tabel">
					<thead>
						<tr>
							<th scope="col">Nomor</th>
							<th scope="col">Tanggal</th>
							<th scope="col">Jenis</th>
							<th scope="col">Tujuan / metode</th>
							<th scope="col" class="text-right">Kantong</th>
						</tr>
					</thead>
					<tbody>
						{#each data as b (b.id)}
							<tr>
								<td><a class="mono text-utama hover:underline" href="/berita-acara/{b.id}">{b.nomor}</a></td>
								<td class="whitespace-nowrap">{waktu(b.dibuat_pada)}</td>
								<td><Chip nada={b.jenis === 'pemusnahan' ? 'bahaya' : 'info'}>{b.jenis === 'pemusnahan' ? 'Pemusnahan' : 'Pengembalian'}</Chip></td>
								<td>
									{b.jenis === 'pengembalian' ? b.utd : b.metode}
									{#if b.catatan}<span class="block text-xs text-redup">{b.catatan}</span>{/if}
								</td>
								<td class="text-right tabular-nums">{b.jumlah}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/snippet}
	</Keadaan>
</section>
