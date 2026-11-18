<script lang="ts">
	import { minimalSandi } from '#lib/kebijakanSandi.svelte.ts';
	import { merek } from '#lib/merek.svelte.ts';
	import PilihTema from '#lib/PilihTema.svelte';
	import KepalaHalaman from '#lib/KepalaHalaman.svelte';
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

<svelte:head><title>Akun saya — {merek.judul}</title></svelte:head>

<KepalaHalaman judul="Akun saya" sub="{sesi.profil?.nama} · {sesi.profil?.nama_pengguna} · {sesi.profil?.peran.join(', ')}" />
{#if sesi.profil?.wajib_ganti_sandi}
	<p class="catatan mb-4">Ganti kata sandi sementara Anda sebelum memakai aplikasi.</p>
{/if}
<div class="grid max-w-5xl items-start gap-5 lg:grid-cols-2">
	<form class="panel flex flex-col gap-4" onsubmit={simpan} aria-labelledby="j-sandi">
		<div>
			<h2 id="j-sandi">Ganti kata sandi</h2>
			<p class="text-sm text-redup">Kata sandi juga dipakai untuk tanda tangan elektronik.</p>
		</div>
		<Isian label="Kata sandi saat ini" id="lama" galat={aksi.isian.sandi_lama}>
			<input id="lama" class="input" type="password" autocomplete="current-password" bind:value={sandi_lama} required />
		</Isian>
		<Isian label="Kata sandi baru" id="baru" galat={aksi.isian.sandi_baru} bantuan="{minimalSandi()}. Kalimat pendek lebih mudah diingat.">
			<input id="baru" class="input" type="password" autocomplete="new-password" bind:value={sandi_baru} required />
		</Isian>
		<Isian label="Ulangi kata sandi baru" id="ulang" galat={aksi.isian.ulang}>
			<input id="ulang" class="input" type="password" autocomplete="new-password" bind:value={ulang} required />
		</Isian>
		<PesanAksi {aksi} />
		<div class="flex justify-end gap-2.5 border-t border-garis pt-4">
			<button class="btn btn--utama" disabled={aksi.sibuk}>Simpan</button>
		</div>
	</form>

	<section class="panel flex flex-col gap-3" aria-labelledby="j-tampilan">
		<h2 id="j-tampilan">Tampilan</h2>
		<p class="text-sm text-redup">Berlaku di browser ini. "Ikuti sistem" memakai setelan terang/gelap komputer atau tablet. Dokumen cetak selalu terang.</p>
		<PilihTema />
	</section>
</div>
