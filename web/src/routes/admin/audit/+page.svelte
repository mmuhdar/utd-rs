<script lang="ts">
	import { merek } from '#lib/merek.svelte.ts';
	import { api } from '#lib/api.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Isian from '#lib/Isian.svelte';
	import Chip from '#lib/Chip.svelte';
	import { sesi } from '#lib/sesi.svelte.ts';
	import { waktu } from '#lib/format.ts';
	import KepalaHalaman from '#lib/KepalaHalaman.svelte';
	import ShieldCheck from '@lucide/svelte/icons/shield-check';
	import ShieldAlert from '@lucide/svelte/icons/shield-alert';

	type Baris = {
		urutan: number;
		waktu: string;
		akun_id: string | null;
		nama: string | null;
		aksi: string;
		objek: string;
		objek_id: string | null;
		sebelum: unknown;
		sesudah: unknown;
		alasan: string | null;
		alamat: string | null;
	};

	let filter = $state({ objek: '', objek_id: '', akun_id: '' });
	let terapan = { objek: '', objek_id: '', akun_id: '' };
	const query = (sebelum?: number) => {
		const q = new URLSearchParams(Object.entries(terapan).filter(([, v]) => v.trim()).map(([k, v]) => [k, v.trim()]));
		if (sebelum) q.set('sebelum_urutan', String(sebelum));
		return q.size ? `?${q}` : '';
	};
	const audit = new Sumber(() => api<Baris[]>('GET', `/audit${query()}`));
	const akun = new Sumber(() => api<{ id: string; nama: string; nama_pengguna: string }[]>('GET', '/akun'));
	$effect(() => {
		if (!sesi.boleh('audit.lihat')) return;
		audit.muat();
		if (sesi.boleh('akun.kelola')) akun.muat();
	});

	let habis = $state(false);
	let terbuka = $state<Record<number, boolean>>({});
	function cari(e: SubmitEvent) {
		e.preventDefault();
		terapan = { ...filter };
		habis = false;
		terbuka = {};
		audit.muat();
	}

	const aksiLebih = new Aksi();
	async function lebihLama() {
		const akhir = audit.data?.at(-1)?.urutan;
		if (!akhir) return;
		const r = await aksiLebih.jalankan(() => api<Baris[]>('GET', `/audit${query(akhir)}`));
		if (r) {
			audit.data = [...(audit.data ?? []), ...r];
			habis = r.length < 100; // server mengembalikan 100 per halaman
		}
	}

	const aksiPeriksa = new Aksi();
	let hasil = $state<{ utuh: boolean; putus_di_urutan: number | null; jumlah: number } | null>(null);
	async function periksa() {
		hasil = null;
		hasil = (await aksiPeriksa.jalankan(() => api<NonNullable<typeof hasil>>('GET', '/audit/periksa'))) ?? null;
	}
	const json = (v: unknown) => (v == null ? '—' : JSON.stringify(v, null, 2));
</script>

<svelte:head><title>Jejak audit — {merek.judul}</title></svelte:head>

<KepalaHalaman judul="Jejak audit" jalur={[['Pengaturan'], ['Jejak audit']]} sub="Setiap perubahan data dan setiap pembukaan data pasien atau donor tercatat di sini." />

{#if !sesi.boleh('audit.lihat')}
	<p class="catatan">Anda tidak berhak melihat jejak audit.</p>
{:else}
	<section class="panel mb-4 flex flex-col gap-3" aria-labelledby="judul-keutuhan">
		<div class="flex flex-wrap items-center justify-between gap-3">
			<div>
				<h2 id="judul-keutuhan">Keutuhan rantai</h2>
				<p class="text-sm text-redup">Setiap catatan audit berantai hash dengan catatan sebelumnya. Perubahan atau penghapusan di database akan memutus rantai.</p>
			</div>
			<button class="btn" onclick={periksa} disabled={aksiPeriksa.sibuk}>{aksiPeriksa.sibuk ? 'Memeriksa…' : 'Periksa keutuhan rantai'}</button>
		</div>
		<PesanAksi aksi={aksiPeriksa} />
		{#if hasil?.utuh}
			<p class="flex items-center gap-2 rounded-lg border border-aman/30 bg-aman-latar p-3 font-medium text-aman" role="status">
				<ShieldCheck class="size-5 shrink-0" aria-hidden="true" />Rantai utuh: {hasil.jumlah} catatan diperiksa, tidak ada yang diubah atau dihapus.
			</p>
		{:else if hasil}
			<div class="peringatan" role="alert">
				<ShieldAlert class="mt-0.5 size-5 shrink-0" aria-hidden="true" />
				<p>
					Rantai putus di urutan <span class="mono">{hasil.putus_di_urutan}</span> (dari {hasil.jumlah} catatan). Catatan itu atau sebelumnya telah diubah di luar
					aplikasi. Amankan salinan database dan laporkan ke penanggung jawab.
				</p>
			</div>
		{/if}
	</section>

	<form class="panel mb-4 flex flex-col gap-2" onsubmit={cari} aria-label="Saring jejak audit">
		<div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-[1fr_1fr_1fr_auto] lg:items-end">
			<Isian label="Objek" id="f-objek">
				<input id="f-objek" class="input mono" bind:value={filter.objek} aria-describedby="f-objek-bantuan" placeholder="mis. kantong" />
			</Isian>
			<Isian label="ID objek" id="f-objek-id">
				<input id="f-objek-id" class="input mono" bind:value={filter.objek_id} />
			</Isian>
			{#if sesi.boleh('akun.kelola')}
				<Isian label="Akun pelaku" id="f-akun">
					<select id="f-akun" class="input" bind:value={filter.akun_id}>
						<option value="">Semua akun</option>
						{#each akun.data ?? [] as a (a.id)}<option value={a.id}>{a.nama} ({a.nama_pengguna})</option>{/each}
					</select>
				</Isian>
			{:else}
				<Isian label="ID akun pelaku" id="f-akun">
					<input id="f-akun" class="input mono" bind:value={filter.akun_id} placeholder="UUID akun" />
				</Isian>
			{/if}
			<div><button class="btn btn--utama">Terapkan saringan</button></div>
		</div>
		<p id="f-objek-bantuan" class="text-xs text-redup">Objek mis. <span class="mono">akun</span>, <span class="mono">donor</span>, <span class="mono">kantong</span>, <span class="mono">aturan_komponen</span>. Kosongkan untuk semua.</p>
	</form>

	<section class="panel" aria-labelledby="judul-jejak">
		<h2 id="judul-jejak">Catatan terbaru</h2>
		<p class="mb-3 text-sm text-redup">Terbaru di atas, 100 per halaman.</p>
		<Keadaan sumber={audit}>
			{#snippet kosong()}<p>Tidak ada catatan audit yang cocok dengan saringan.</p>{/snippet}
			{#snippet children(data)}
				<div class="overflow-x-auto">
					<table class="tabel [&_td]:py-2">
						<thead>
							<tr><th>Waktu</th><th>Akun</th><th>Aksi</th><th>Objek</th><th>Alasan</th><th>Urutan</th><th><span class="sr-only">Rincian</span></th></tr>
						</thead>
						<tbody>
							{#each data as b (b.urutan)}
								<tr>
									<td class="whitespace-nowrap">{waktu(b.waktu)}</td>
									<td>{b.nama ?? 'Sistem / belum masuk'}{#if b.alamat}<span class="mono block text-xs text-redup">{b.alamat}</span>{/if}</td>
									<td><Chip polos>{b.aksi}</Chip></td>
									<td class="mono">{b.objek}{#if b.objek_id}<span class="block text-xs break-all text-redup">{b.objek_id}</span>{/if}</td>
									<td>{b.alasan ?? '—'}</td>
									<td class="mono text-xs text-redup tabular-nums">{b.urutan}</td>
									<td>
										{#if b.sebelum != null || b.sesudah != null}
											<button
												class="btn btn--teks"
												aria-expanded={!!terbuka[b.urutan]}
												aria-controls="rincian-{b.urutan}"
												onclick={() => (terbuka[b.urutan] = !terbuka[b.urutan])}>{terbuka[b.urutan] ? 'Tutup' : 'Rincian'}</button
											>
										{/if}
									</td>
								</tr>
								{#if terbuka[b.urutan]}
									<tr id="rincian-{b.urutan}">
										<td colspan="7" class="bg-halus">
											<div class="grid gap-3 md:grid-cols-2">
												<div>
													<p class="label">Sebelum</p>
													<pre class="mono max-h-96 overflow-auto rounded border border-garis bg-kartu p-2 text-xs">{json(b.sebelum)}</pre>
												</div>
												<div>
													<p class="label">Sesudah</p>
													<pre class="mono max-h-96 overflow-auto rounded border border-garis bg-kartu p-2 text-xs">{json(b.sesudah)}</pre>
												</div>
											</div>
										</td>
									</tr>
								{/if}
							{/each}
						</tbody>
					</table>
				</div>
				<div class="mt-3 flex flex-col gap-2">
					<PesanAksi aksi={aksiLebih} />
					{#if habis || data.length < 100}
						<p class="text-sm text-redup">Semua catatan yang cocok sudah ditampilkan ({data.length}).</p>
					{:else}
						<button class="btn self-start" onclick={lebihLama} disabled={aksiLebih.sibuk}>Muat lebih lama</button>
					{/if}
				</div>
			{/snippet}
		</Keadaan>
	</section>
{/if}
