<script lang="ts">
	import { merek } from '#lib/merek.svelte.ts';
	import { api } from '#lib/api.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import Chip from '#lib/Chip.svelte';
	import { waktu } from '#lib/format.ts';
	import { STATUS_DONASI, golonganGabung } from '#lib/utd/label.ts';

	type Baris = {
		id: string;
		nomor: string;
		waktu: string;
		donor: string;
		donor_id: string;
		jenis: string;
		untuk_no_rm: string | null;
		volume_ml: number;
		status: string;
		golongan: string | null;
		imltd_lengkap: boolean;
		reaktif: boolean;
		karantina: number;
	};

	let status = $state('');
	const daftar = new Sumber(() => api<Baris[]>('GET', `/donasi${status ? `?status=${status}` : ''}`));
	$effect(() => {
		void status;
		daftar.muat();
	});
</script>

<svelte:head><title>Donasi — {merek.judul}</title></svelte:head>

<div class="mb-4 flex flex-wrap items-end justify-between gap-3">
	<h1>Donasi &amp; pelulusan</h1>
	<div>
		<label class="label" for="status">Status donasi</label>
		<select id="status" class="input" bind:value={status}>
			<option value="">Semua</option>
			<option value="diambil">Menunggu pengolahan</option>
			<option value="diolah">Diolah</option>
			<option value="gagal">Gagal</option>
		</select>
	</div>
</div>

<section class="panel">
	<Keadaan sumber={daftar}>
		{#snippet kosong()}
			<p>Belum ada donasi{status ? ' dengan status ini' : ''}. Donasi dicatat dari halaman donor setelah seleksi diterima.</p>
			<a class="btn" href="/donor">Ke halaman donor</a>
		{/snippet}
		{#snippet children(data)}
			<div class="overflow-x-auto">
				<table class="tabel">
					<caption class="sr-only">Daftar donasi beserta status uji dan karantina</caption>
					<thead>
						<tr>
							<th scope="col">Nomor donasi</th>
							<th scope="col">Donor</th>
							<th scope="col">Status</th>
							<th scope="col">Golongan</th>
							<th scope="col">Uji saring IMLTD</th>
							<th scope="col">Kantong karantina</th>
						</tr>
					</thead>
					<tbody>
						{#each data as n (n.id)}
							{@const st = STATUS_DONASI[n.status]}
							<tr>
								<td>
									<a class="mono font-medium text-utama hover:underline" href="/donasi/{n.id}">{n.nomor}</a>
									<span class="block text-xs text-redup">{waktu(n.waktu)}</span>
								</td>
								<td>
									<a class="hover:underline" href="/donor/{n.donor_id}">{n.donor}</a>
									<span class="block text-xs text-redup">{n.jenis === 'pengganti' ? `Pengganti untuk ${n.untuk_no_rm}` : 'Sukarela'} · {n.volume_ml} mL</span>
								</td>
								<td><Chip nada={st?.[1]}>{st?.[0] ?? n.status}</Chip></td>
								<td>
									{#if n.golongan}
										<span class="font-semibold">{golonganGabung(n.golongan)}</span>
									{:else}
										<Chip nada="waspada">Belum terverifikasi</Chip>
									{/if}
								</td>
								<td>
									{#if n.reaktif}
										<Chip nada="bahaya">Reaktif</Chip>
									{:else if n.imltd_lengkap}
										<Chip nada="aman">Lengkap</Chip>
									{:else}
										<Chip nada="waspada">Belum lengkap</Chip>
									{/if}
								</td>
								<td class="tabular-nums">{n.karantina}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/snippet}
	</Keadaan>
</section>
