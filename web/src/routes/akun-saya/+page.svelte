<script lang="ts">
	import PilihTema from '#lib/PilihTema.svelte';
	import { goto } from '$app/navigation';
	import { api } from '#lib/api.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Isian from '#lib/Isian.svelte';
	import { sesi, type Profil } from '#lib/sesi.svelte.ts';

	let sandi_lama = $state('');
	let sandi_baru = $state('');
	let ulang = $state('');
	const aksi = new Aksi();

	async function simpan(e: SubmitEvent) {
		e.preventDefault();
		if (sandi_baru !== ulang) {
			aksi.isian = { ulang: 'Kedua isian kata sandi baru tidak sama.' };
			return;
		}
		const p = await aksi.jalankan(() => api<Profil>('POST', '/auth/ganti-sandi', { sandi_lama, sandi_baru }), 'Kata sandi diganti.');
		if (p) {
			const wajib = sesi.profil?.wajib_ganti_sandi;
			sesi.profil = p;
			sandi_lama = sandi_baru = ulang = '';
			if (wajib) goto('/');
		}
	}
</script>

<svelte:head><title>Akun saya — utd-rs</title></svelte:head>

<h1 class="mb-1">Akun saya</h1>
<p class="mb-4 text-sm text-redup">{sesi.profil?.nama} · {sesi.profil?.nama_pengguna} · {sesi.profil?.peran.join(', ')}</p>
{#if sesi.profil?.wajib_ganti_sandi}
	<p class="catatan mb-4">Ganti kata sandi sementara Anda sebelum memakai aplikasi.</p>
{/if}
<form class="panel flex max-w-md flex-col gap-4" onsubmit={simpan}>
	<h2>Ganti kata sandi</h2>
	<Isian label="Kata sandi saat ini" id="lama" galat={aksi.isian.sandi_lama}>
		<input id="lama" class="input" type="password" autocomplete="current-password" bind:value={sandi_lama} required />
	</Isian>
	<Isian label="Kata sandi baru" id="baru" galat={aksi.isian.sandi_baru} bantuan="Minimal 10 karakter. Kalimat pendek lebih mudah diingat.">
		<input id="baru" class="input" type="password" autocomplete="new-password" bind:value={sandi_baru} required />
	</Isian>
	<Isian label="Ulangi kata sandi baru" id="ulang" galat={aksi.isian.ulang}>
		<input id="ulang" class="input" type="password" autocomplete="new-password" bind:value={ulang} required />
	</Isian>
	<PesanAksi {aksi} />
	<button class="btn btn--utama self-start" disabled={aksi.sibuk}>Simpan</button>
</form>

<section class="panel mt-4 flex max-w-md flex-col gap-3" aria-labelledby="j-tampilan">
	<h2 id="j-tampilan">Tampilan</h2>
	<p class="text-sm text-redup">Berlaku di browser ini. "Ikuti sistem" memakai setelan terang/gelap komputer atau tablet. Dokumen cetak selalu terang.</p>
	<PilihTema />
</section>
