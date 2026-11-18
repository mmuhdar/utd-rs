<!-- Sampel pratransfusi: daftar dengan batas usia, dan penerimaan/penolakan sampel baru. -->
<script lang="ts">
	import { api } from '#lib/api.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Isian from '#lib/Isian.svelte';
	import Chip from '#lib/Chip.svelte';
	import { sesi } from '#lib/sesi.svelte.ts';
	import { keRfc3339, lokal, relatif, waktu } from '#lib/format.ts';
	import Pilihan from '#lib/pelayanan/Pilihan.svelte';
	import { ALASAN_TOLAK_SAMPEL, JENIS_SAMPEL, PENYIMPANAN, STATUS_SAMPEL, batasUsiaJam } from '#lib/pelayanan/darah.ts';

	let { p, aktif, muat }: { p: any; aktif: boolean; muat: () => Promise<void> } = $props();

	const kosong = () => ({ diambil_pada: lokal(), diambil_oleh: '', jenis: 'edta', penyimpanan: 'suhu_ruang', diterima: true as boolean, alasan_tolak: '' });
	let f = $state(kosong());
	const usia = $derived(batasUsiaJam(f.jenis, f.penyimpanan));
	const aksi = new Aksi();

	async function kirim(e: SubmitEvent) {
		e.preventDefault();
		const r = await aksi.jalankan(
			() =>
				api('POST', `/permintaan/${p.id}/sampel`, {
					...f,
					diambil_pada: keRfc3339(f.diambil_pada),
					alasan_tolak: f.diterima ? undefined : f.alasan_tolak
				}),
			f.diterima ? 'Sampel diterima.' : 'Penolakan sampel dicatat; minta sampel baru dari ruang.'
		);
		if (r) {
			f = kosong();
			await muat();
		}
	}
</script>

<section class="panel flex flex-col gap-4" aria-labelledby="j-sampel">
	<h2 id="j-sampel">Sampel pratransfusi</h2>
	{#if p.sampel.length}
		<div class="overflow-x-auto">
			<table class="tabel">
				<thead>
					<tr><th scope="col">Diambil</th><th scope="col">Jenis</th><th scope="col">Pengambil</th><th scope="col">Batas pakai</th><th scope="col">Status</th></tr>
				</thead>
				<tbody>
					{#each p.sampel as s (s.id)}
						{@const st = STATUS_SAMPEL[s.status]}
						{@const lewat = new Date(s.batas_pakai) <= new Date()}
						<tr>
							<td>{waktu(s.diambil_pada)}<span class="block text-xs text-redup">diterima {waktu(s.diterima_pada)}</span></td>
							<td>{JENIS_SAMPEL[s.jenis] ?? s.jenis}<span class="block text-xs text-redup">{PENYIMPANAN[s.penyimpanan] ?? s.penyimpanan}</span></td>
							<td>{s.diambil_oleh}</td>
							<td>
								{waktu(s.batas_pakai)}
								{#if s.status !== 'ditolak'}
									<span class="block text-xs {lewat ? 'font-semibold text-bahaya' : 'text-redup'}">{lewat ? 'Lewat batas usia; minta sampel baru' : relatif(s.batas_pakai)}</span>
								{/if}
							</td>
							<td>
								<Chip nada={st?.[1]}>{st?.[0] ?? s.status}</Chip>
								{#if s.alasan_tolak}<span class="block text-xs text-redup">{s.alasan_tolak}</span>{/if}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{:else}
		<p class="text-sm text-redup">Belum ada sampel. Permintaan diproses setelah sampel darah pasien diterima.</p>
	{/if}

	{#if aktif && sesi.boleh('permintaan.proses')}
		<form class="tanpa-cetak flex flex-col gap-4 border-t border-garis pt-4" onsubmit={kirim}>
			<h3 class="font-semibold">Terima sampel</h3>
			<div class="form-grid">
				<Isian label="Waktu pengambilan" id="s-diambil" galat={aksi.isian.diambil_pada}>
					<input id="s-diambil" class="input" type="datetime-local" bind:value={f.diambil_pada} required />
				</Isian>
				<Isian label="Nama pengambil sampel" id="s-oleh" galat={aksi.isian.diambil_oleh}>
					<input id="s-oleh" class="input" bind:value={f.diambil_oleh} required />
				</Isian>
				<Pilihan legend="Jenis sampel" name="s-jenis" opsi={Object.entries(JENIS_SAMPEL)} bind:value={f.jenis} />
				<Pilihan legend="Penyimpanan" name="s-simpan" opsi={Object.entries(PENYIMPANAN)} bind:value={f.penyimpanan} galat={aksi.isian.penyimpanan} />
			</div>
			<p class="text-sm {usia ? 'text-redup' : 'text-bahaya'}">
				{#if usia && f.diambil_pada}
					Batas usia {usia} jam: sampel ini dapat dipakai sampai {waktu(new Date(new Date(f.diambil_pada).getTime() + usia * 3_600_000).toISOString())}.
				{:else if !usia}
					Serum/plasma hanya boleh disimpan dingin (4 °C).
				{/if}
			</p>
			<Pilihan
				legend="Keputusan"
				name="s-keputusan"
				opsi={[
					[true, 'Terima'],
					[false, 'Tolak']
				]}
				bind:value={f.diterima}
			/>
			{#if !f.diterima}
				<Isian label="Alasan penolakan" id="s-alasan" galat={aksi.isian.alasan_tolak}>
					<select id="s-alasan" class="input" bind:value={f.alasan_tolak} required>
						<option value="" disabled>Pilih alasan</option>
						{#each ALASAN_TOLAK_SAMPEL as a (a)}<option value={a}>{a}</option>{/each}
					</select>
				</Isian>
			{/if}
			<PesanAksi {aksi} />
			<button class="btn {f.diterima ? 'btn--utama' : 'btn--bahaya'} self-start" disabled={aksi.sibuk}>
				{f.diterima ? 'Terima sampel' : 'Catat penolakan sampel'}
			</button>
		</form>
	{/if}
</section>
