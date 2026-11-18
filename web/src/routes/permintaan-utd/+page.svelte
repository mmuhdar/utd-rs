<script lang="ts">
	import { api } from '#lib/api.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Isian from '#lib/Isian.svelte';
	import Chip from '#lib/Chip.svelte';
	import { sesi } from '#lib/sesi.svelte.ts';
	import { KOMPONEN, NAMA_KOMPONEN, golongan, waktu } from '#lib/format.ts';
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

<svelte:head><title>Permintaan ke UTD — utd-rs</title></svelte:head>

<h1 class="mb-4">Permintaan ke UTD</h1>

{#if kelola}
	<form class="panel mb-6 flex flex-col gap-4" onsubmit={simpan} aria-labelledby="judul-baru">
		<h2 id="judul-baru">Buat permintaan</h2>
		<div class="form-grid">
			<Isian label="UTD mitra" id="utd" galat={aksi.isian.utd_id}>
				<select id="utd" class="input" bind:value={utd_id} required>
					<option value="" disabled>Pilih UTD</option>
					{#each (utdList.data ?? []).filter((u) => u.aktif) as u (u.id)}<option value={u.id}>{u.nama}</option>{/each}
				</select>
			</Isian>
			<Isian label="Jenis permintaan" id="jenis" galat={aksi.isian.jenis}>
				<select id="jenis" class="input" bind:value={jenis}>
					{#each Object.entries(JENIS) as [v, l] (v)}<option value={v}>{l}</option>{/each}
				</select>
			</Isian>
		</div>

		<fieldset>
			<legend class="label">Baris komponen</legend>
			<ol class="flex flex-col gap-2">
				{#each baris as b, n (n)}
					<li class="grid grid-cols-2 items-end gap-2 sm:grid-cols-[2fr_1fr_1fr_1fr_auto]">
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
							<input id="b{n}-jumlah" class="input" type="number" min="1" bind:value={b.jumlah} required />
						</div>
						<button type="button" class="btn btn--teks" disabled={baris.length === 1} onclick={() => baris.splice(n, 1)}>
							<Trash class="size-4" aria-hidden="true" /><span class="sr-only">Hapus baris {n + 1}</span>
						</button>
					</li>
				{/each}
			</ol>
			{#if aksi.isian.baris}<p class="mt-1 text-sm text-bahaya" role="alert">{aksi.isian.baris}</p>{/if}
			<button type="button" class="btn btn--teks mt-2" onclick={() => baris.push(barisBaru())}><Plus class="size-4" aria-hidden="true" />Tambah baris</button>
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
		<button class="btn btn--utama self-start" disabled={aksi.sibuk}>Kirim permintaan</button>
	</form>
{/if}

<section class="panel" aria-labelledby="judul-daftar">
	<h2 id="judul-daftar" class="mb-3">Daftar permintaan</h2>
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
								<td class="mono">{r.nomor}</td>
								<td class="whitespace-nowrap">{waktu(r.dibuat_pada)}</td>
								<td>{r.utd}</td>
								<td>
									<ul>
										{#each r.baris as b, i (i)}<li>{b.jumlah} × {KOMPONEN[b.komponen]} {golongan(b.abo, b.rhesus)}</li>{/each}
									</ul>
									{#if r.catatan}<p class="text-xs text-redup">{r.catatan}</p>{/if}
								</td>
								<td>
									<span class="flex flex-col items-start gap-1">
										{#if r.jenis === 'darurat'}<Chip nada="bahaya">Darurat</Chip>{:else}<Chip>{JENIS[r.jenis]}</Chip>{/if}
										<Chip nada={st[1]}>{st[0]}</Chip>
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
