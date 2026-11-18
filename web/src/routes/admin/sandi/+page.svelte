<!-- Kebijakan kata sandi: panjang minimal yang berlaku saat sandi dibuat, direset, atau diganti. -->
<script lang="ts">
	import { merek } from '#lib/merek.svelte.ts';
	import { api } from '#lib/api.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Isian from '#lib/Isian.svelte';
	import KepalaHalaman from '#lib/KepalaHalaman.svelte';
	import { waktu } from '#lib/format.ts';
	import { kebijakanSandi, type KebijakanSandi } from '#lib/kebijakanSandi.svelte.ts';
	import TriangleAlert from '@lucide/svelte/icons/triangle-alert';

	let panjang = $state<number | null>(null);
	$effect(() => {
		if (!kebijakanSandi.data) kebijakanSandi.muat();
	});
	$effect(() => {
		if (kebijakanSandi.data && panjang === null) panjang = kebijakanSandi.data.panjang_min;
	});
	const aksi = new Aksi();
	const berubah = $derived(panjang !== null && panjang !== kebijakanSandi.data?.panjang_min);

	async function simpan(e: SubmitEvent) {
		e.preventDefault();
		const r = await aksi.jalankan(
			() => api<KebijakanSandi>('PUT', '/kebijakan-sandi', { panjang_min: Number(panjang) }),
			'Kebijakan kata sandi disimpan. Berlaku untuk sandi yang dibuat, direset, atau diganti mulai sekarang.'
		);
		if (r) kebijakanSandi.data = r;
	}
</script>

<svelte:head><title>Kebijakan sandi — {merek.judul}</title></svelte:head>

<KepalaHalaman
	judul="Kebijakan kata sandi"
	jalur={[['Pengaturan'], ['Kebijakan sandi']]}
	sub="Aturan kata sandi untuk semua akun petugas. Setiap perubahan tercatat di jejak audit."
/>

<Keadaan sumber={kebijakanSandi} apakahKosong={() => false} baris={3}>
	{#snippet children(k)}
		<div class="grid items-start gap-5 xl:grid-cols-[1fr_360px]">
			<form class="panel flex flex-col gap-4" onsubmit={simpan}>
				<div>
					<h2>Panjang minimal</h2>
					<p class="text-sm text-redup">Berlaku saat akun dibuat, sandi direset admin, atau pengguna mengganti sandinya. Sandi yang sudah ada tidak ikut berubah.</p>
				</div>
				<Isian label="Panjang minimal kata sandi (karakter)" id="panjang-min" galat={aksi.isian.panjang_min} bantuan="Antara 4 dan 64 karakter. Bawaan 10.">
					<input
						id="panjang-min"
						class="input w-40 text-lg font-semibold tabular-nums"
						type="number"
						min="4"
						max="64"
						required
						aria-invalid={!!aksi.isian.panjang_min}
						bind:value={panjang}
					/>
				</Isian>
				{#if panjang !== null && panjang < 8}
					<div class="catatan" role="status">
						<TriangleAlert aria-hidden="true" />
						<p>
							Sandi kurang dari 8 karakter mudah ditebak. Pakai hanya bila ada alasan kuat, misalnya PIN di tablet ruang, dan pastikan akun terkunci setelah 5 kali
							salah sandi tetap berlaku.
						</p>
					</div>
				{/if}
				<PesanAksi {aksi} />
				<div class="flex justify-end gap-2.5 border-t border-garis pt-4">
					<button type="button" class="btn" disabled={!berubah || aksi.sibuk} onclick={() => (panjang = k.panjang_min)}>Batalkan perubahan</button>
					<button class="btn btn--utama" disabled={!berubah || aksi.sibuk}>{aksi.sibuk ? 'Menyimpan…' : 'Simpan kebijakan'}</button>
				</div>
				<p class="text-xs text-redup">Terakhir diubah {waktu(k.diubah_pada)}.</p>
			</form>

			<aside class="panel flex flex-col gap-3">
				<h2>Pengamanan lain yang selalu berlaku</h2>
				<ul class="flex list-disc flex-col gap-1.5 pl-5 text-sm">
					<li>Akun terkunci 15 menit setelah 5 kali salah sandi.</li>
					<li>Sesi berakhir setelah 30 menit tanpa aktivitas, dan paling lama 12 jam.</li>
					<li>Sandi sementara dari admin wajib diganti saat pertama masuk.</li>
					<li>Sandi baru harus berbeda dari sandi lama.</li>
					<li>Sandi disimpan sebagai hash Argon2; admin pun tidak bisa melihatnya.</li>
				</ul>
			</aside>
		</div>
	{/snippet}
</Keadaan>
