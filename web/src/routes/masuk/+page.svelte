<script lang="ts">
	import { goto } from '$app/navigation';
	import { api } from '#lib/api.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import { sesi, type Profil } from '#lib/sesi.svelte.ts';
	import Droplet from '@lucide/svelte/icons/droplet';

	let nama_pengguna = $state('');
	let sandi = $state('');
	const aksi = new Aksi();

	async function masuk(e: SubmitEvent) {
		e.preventDefault();
		const p = await aksi.jalankan(() => api<Profil>('POST', '/auth/masuk', { nama_pengguna, sandi }));
		sandi = '';
		if (p) {
			sesi.profil = p;
			goto(p.wajib_ganti_sandi ? '/akun-saya' : '/');
		}
	}
</script>

<svelte:head><title>Masuk — utd-rs</title></svelte:head>

<main class="flex min-h-screen items-center justify-center p-4">
	<form class="panel flex w-full max-w-sm flex-col gap-4 p-6" onsubmit={masuk}>
		<div class="flex items-center gap-2">
			<Droplet class="size-7 text-bahaya" aria-hidden="true" />
			<div>
				<h1>utd-rs</h1>
				<p class="text-sm text-redup">Bank darah rumah sakit</p>
			</div>
		</div>
		<div>
			<label class="label" for="nama">Nama pengguna</label>
			<input id="nama" class="input" autocomplete="username" bind:value={nama_pengguna} required />
		</div>
		<div>
			<label class="label" for="sandi">Kata sandi</label>
			<input id="sandi" class="input" type="password" autocomplete="current-password" bind:value={sandi} required />
		</div>
		<PesanAksi {aksi} />
		<button class="btn btn--utama btn--besar" disabled={aksi.sibuk}>{aksi.sibuk ? 'Memeriksa…' : 'Masuk'}</button>
		<p class="text-xs text-redup">Akun bersifat pribadi. Jangan meminjamkan akun; setiap tindakan tercatat atas nama Anda.</p>
	</form>
</main>
