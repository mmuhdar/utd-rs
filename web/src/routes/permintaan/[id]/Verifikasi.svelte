<!-- Verifikasi kedua satu hasil (golongan pasien atau uji silang): setujui, atau tolak dengan alasan. Keduanya bertanda tangan. -->
<script lang="ts">
	import { api } from '#lib/api.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import { sesi } from '#lib/sesi.svelte.ts';
	import type { Minta } from '#lib/pelayanan/TandaTangan.svelte';

	let {
		url,
		penginput,
		apa,
		minta,
		onselesai
	}: { url: string; penginput: string; apa: string; minta: Minta; onselesai: (r: any) => void } = $props();

	const aksi = new Aksi();
	let menolak = $state(false);
	let alasan = $state('');
	const idAlasan = $derived(`alasan-${url.replaceAll('/', '-')}`);

	function setujui() {
		minta({
			judul: `Verifikasi kedua ${apa}`,
			makna: `Dengan menandatangani, Anda menyatakan telah memeriksa ulang ${apa} ini dan hasilnya benar.`,
			kirim: async ({ sandi }) => {
				const r = await aksi.jalankan(() => api<any>('POST', url, { setuju: true, sandi }), 'Hasil diverifikasi.');
				if (r) onselesai(r);
			}
		});
	}

	function tolak(e: SubmitEvent) {
		e.preventDefault();
		minta({
			judul: `Tolak ${apa}`,
			makna: `Hasil ini dibatalkan dengan alasan "${alasan}" dan penginput dapat memasukkan ulang.`,
			kirim: async ({ sandi }) => {
				const r = await aksi.jalankan(() => api<any>('POST', url, { setuju: false, sandi, alasan }), 'Hasil ditolak dan dibatalkan.');
				if (r) {
					menolak = false;
					alasan = '';
					onselesai(r);
				}
			}
		});
	}
</script>

{#if sesi.profil?.akun_id === penginput}
	<p class="text-xs text-redup">Menunggu verifikasi kedua oleh petugas lain.</p>
{:else}
	<div class="tanpa-cetak flex flex-col gap-2">
		{#if menolak}
			<form class="flex flex-wrap items-end gap-2" onsubmit={tolak}>
				<div class="min-w-48 flex-1">
					<label class="label" for={idAlasan}>Alasan menolak hasil</label>
					<input id={idAlasan} class="input" bind:value={alasan} required />
				</div>
				<button class="btn btn--bahaya">Tolak hasil</button>
				<button type="button" class="btn" onclick={() => (menolak = false)}>Batal</button>
			</form>
		{:else}
			<div class="flex flex-wrap gap-2">
				<button type="button" class="btn btn--utama" onclick={setujui} disabled={aksi.sibuk}>Setujui</button>
				<button type="button" class="btn" onclick={() => (menolak = true)}>Tolak…</button>
			</div>
		{/if}
		<PesanAksi {aksi} />
	</div>
{/if}
