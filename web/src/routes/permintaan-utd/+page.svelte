<script lang="ts">
	import { merek } from '#lib/merek.svelte.ts';
	import { api } from '#lib/api.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Isian from '#lib/Isian.svelte';
	import Chip from '#lib/Chip.svelte';
	import { sesi } from '#lib/sesi.svelte.ts';
	import { KOMPONEN, NAMA_KOMPONEN, waktu } from '#lib/format.ts';
	import KepalaHalaman from '#lib/KepalaHalaman.svelte';
	import Golongan from '#lib/Golongan.svelte';
	import Pilihan from '#lib/pelayanan/Pilihan.svelte';
	import Send from '@lucide/svelte/icons/send';
	import { ABO, RHESUS, type Utd } from '#lib/stok/stok.ts';
	import Plus from '@lucide/svelte/icons/plus';
	import Trash from '@lucide/svelte/icons/trash-2';

	type Baris = { komponen: string; abo: string; rhesus: string; jumlah: number };
	const STATUS: Record<string, [string, string]> = {
		dikirim: ['Dikirim', 'waspada'],
		dipenuhi: ['Dipenuhi', 'aman'],
		batal: ['Batal', 'netral']
	};
	const JENIS: Record<string, string> = { rutin: 'Rutin', khusus: 'Khusus', darurat: 'Darurat' };

	const daftar = new Sumber(() => api<any[]>('GET', '/permintaan-utd'));
	const utdList = new Sumber(() => api<Utd[]>('GET', '/master/utd'));
	$effect(() => {
		daftar.muat();
		utdList.muat();
	});

	const kelola = $derived(sesi.boleh('stok.kelola'));
	let utd_id = $state('');
	let jenis = $state('rutin');
	let catatan = $state('');
	const barisBaru = (): Baris => ({ komponen: 'prc', abo: 'O', rhesus: 'pos', jumlah: 1 });
	let baris = $state<Baris[]>([barisBaru()]);
	const aksi = new Aksi();
	const aksiStatus = new Aksi();

	async function simpan(e: SubmitEvent) {
		e.preventDefault();
		const r = await aksi.jalankan(() => api('POST', '/permintaan-utd', { utd_id, jenis, baris, catatan }), 'Permintaan ke UTD dibuat.');
		if (r) {
			baris = [barisBaru()];
			catatan = '';
			jenis = 'rutin';
			daftar.muat();
		}
	}

	async function ubah(id: string, status: 'dipenuhi' | 'batal', nomor: string) {
		if (status === 'batal' && !confirm(`Batalkan permintaan ${nomor}?`)) return;
		const r = await aksiStatus.jalankan(
			() => api('POST', `/permintaan-utd/${id}/status`, { status }),
			`Permintaan ${nomor} ditandai ${status === 'batal' ? 'batal' : 'dipenuhi'}.`
		);
		if (r) daftar.muat();
	}
</script>

<svelte:head><title>Permintaan ke UTD — {merek.judul}</title></svelte:head>

<KepalaHalaman
	judul="Permintaan ke UTD"
	sub={daftar.data ? `${daftar.data.filter((r) => r.status === 'dikirim').length} menunggu dipenuhi · ${daftar.data.length} permintaan` : undefined}
	jalur={[['Stok kantong', '/stok'], ['Permintaan ke UTD']]}
/>

{#if kelola}
	<form class="panel mb-5 flex flex-col gap-5" onsubmit={simpan} aria-labelledby="judul-baru">
		<h2 id="judul-baru">Buat permintaan</h2>
		<div class="grid items-start gap-4 sm:grid-cols-2">
			<Isian label="UTD mitra" id="utd" galat={aksi.isian.utd_id}>
				<select id="utd" class="input" bind:value={utd_id} required>
					<option value="" disabled>Pilih UTD</option>
					{#each (utdList.data ?? []).filter((u) => u.aktif) as u (u.id)}<option value={u.id}>{u.nama}</option>{/each}
				</select>
			</Isian>
			<Pilihan legend="Jenis permintaan" name="jenis" opsi={Object.entries(JENIS)} bind:value={jenis} galat={aksi.isian.jenis} bahaya={['darurat']} />
		</div>

		<fieldset class="rounded-2xl border border-garis bg-latar/40 p-4">
			<legend class="label px-1">Baris komponen</legend>
			<ol class="flex flex-col gap-3">
				{#each baris as b, n (n)}
					<li class="grid grid-cols-2 items-end gap-3 sm:grid-cols-[auto_2fr_1fr_1fr_1fr_auto]">
						<span class="hidden min-h-10 items-center sm:flex" aria-hidden="true"><Golongan abo={b.abo} rhesus={b.rhesus} /></span>
						<div class="col-span-2 sm:col-span-1">
							<label class="label text-xs" for="b{n}-komponen">Komponen baris {n + 1}</label>
							<select id="b{n}-komponen" class="input" bind:value={b.komponen}>
								{#each Object.entries(NAMA_KOMPONEN) as [v, l] (v)}<option value={v}>{l}</option>{/each}
							</select>
						</div>
						<div>
							<label class="label text-xs" for="b{n}-abo">ABO</label>
							<select id="b{n}-abo" class="input" bind:value={b.abo}>
								{#each ABO as a (a)}<option value={a}>{a}</option>{/each}
							</select>
						</div>
						<div>
							<label class="label text-xs" for="b{n}-rhesus">Rhesus</label>
							<select id="b{n}-rhesus" class="input" bind:value={b.rhesus}>
								{#each Object.entries(RHESUS) as [v, l] (v)}<option value={v}>{l}</option>{/each}
							</select>
						</div>
						<div>
							<label class="label text-xs" for="b{n}-jumlah">Jumlah kantong</label>
							<input id="b{n}-jumlah" class="input tabular-nums" type="number" min="1" bind:value={b.jumlah} required />
						</div>
						<button type="button" class="btn btn--teks" disabled={baris.length === 1} onclick={() => baris.splice(n, 1)}>
							<Trash class="size-4" aria-hidden="true" /><span class="sr-only">Hapus baris {n + 1}</span>
						</button>
					</li>
				{/each}
			</ol>
			{#if aksi.isian.baris}<p class="mt-1 text-sm text-bahaya" role="alert">{aksi.isian.baris}</p>{/if}
			<button type="button" class="btn mt-3" onclick={() => baris.push(barisBaru())}><Plus class="size-4" aria-hidden="true" />Tambah baris</button>
		</fieldset>

		<Isian
			label={jenis === 'darurat' ? 'Catatan (wajib untuk darurat)' : 'Catatan'}
			id="catatan"
			galat={aksi.isian.catatan}
			bantuan={jenis === 'darurat' ? 'Tulis alasan darurat dan data pasien bila darah langka (mis. Rh negatif).' : undefined}
		>
			<textarea id="catatan" class="input" rows="3" bind:value={catatan} required={jenis === 'darurat'} aria-invalid={!!aksi.isian.catatan}></textarea>
		</Isian>
		<PesanAksi {aksi} />
		<div class="flex justify-end gap-2.5 border-t border-garis pt-4">
			<button class="btn btn--utama" disabled={aksi.sibuk}><Send class="size-4" aria-hidden="true" />Kirim permintaan</button>
		</div>
	</form>
{/if}

<section class="panel" aria-labelledby="judul-daftar">
	<div class="mb-3 flex flex-wrap items-baseline justify-between gap-2">
		<h2 id="judul-daftar">Daftar permintaan</h2>
		{#if daftar.data}<p class="text-sm text-redup">{daftar.data.length} permintaan</p>{/if}
	</div>
	<div class="mb-2"><PesanAksi aksi={aksiStatus} /></div>
	<Keadaan sumber={daftar}>
		{#snippet kosong()}<p>Belum ada permintaan ke UTD.</p>{/snippet}
		{#snippet children(data)}
			<div class="overflow-x-auto">
				<table class="tabel">
					<thead>
						<tr>
							<th scope="col">Nomor</th>
							<th scope="col">Dibuat</th>
							<th scope="col">UTD</th>
							<th scope="col">Isi</th>
							<th scope="col">Status</th>
							<th scope="col"><span class="sr-only">Aksi</span></th>
						</tr>
					</thead>
					<tbody>
						{#each data as r (r.id)}
							{@const st = STATUS[r.status] ?? [r.status, 'netral']}
							<tr>
								<td class="mono font-semibold">{r.nomor}</td>
								<td class="whitespace-nowrap text-teks-2">{waktu(r.dibuat_pada)}</td>
								<td>{r.utd}</td>
								<td>
									<ul class="flex flex-col gap-1">
										{#each r.baris as b, i (i)}
											<li class="flex items-center gap-2">
												<Golongan abo={b.abo} rhesus={b.rhesus} ukuran="kecil" />
												<span class="text-teks-2"><span class="font-semibold tabular-nums">{b.jumlah}</span> × {KOMPONEN[b.komponen]}</span>
											</li>
										{/each}
									</ul>
									{#if r.catatan}<p class="mt-1 max-w-72 text-xs text-redup">{r.catatan}</p>{/if}
								</td>
								<td>
									<span class="flex flex-wrap items-start gap-1.5">
										<Chip nada={st[1]}>{st[0]}</Chip>
										{#if r.jenis === 'darurat'}<Chip nada="bahaya" polos>Darurat</Chip>{:else}<Chip polos>{JENIS[r.jenis]}</Chip>{/if}
									</span>
								</td>
								<td>
									{#if kelola && r.status === 'dikirim'}
										<div class="flex flex-wrap justify-end gap-1">
											<button type="button" class="btn btn--teks" disabled={aksiStatus.sibuk} onclick={() => ubah(r.id, 'dipenuhi', r.nomor)}>Tandai dipenuhi</button>
											<button type="button" class="btn btn--teks" disabled={aksiStatus.sibuk} onclick={() => ubah(r.id, 'batal', r.nomor)}>Batalkan</button>
										</div>
									{/if}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/snippet}
	</Keadaan>
</section>
