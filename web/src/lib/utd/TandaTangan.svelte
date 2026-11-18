<!--
  Satu DialogSandi untuk banyak aksi bertanda tangan di satu halaman: `minta(judul, makna, aksi, fn)` membuka dialog lalu
  menjalankan `fn(sandi)`. Dialog selalu ditutup setelahnya supaya galat dari server (mis. sandi salah) terlihat
  di PesanAksi halaman, bukan tertutup modal. Galat isian `sandi` diangkat menjadi galat umum aksi itu.
-->
<script lang="ts">
	import DialogSandi from '#lib/DialogSandi.svelte';
	import type { Aksi } from '#lib/aksi.svelte.ts';

	let judul = $state('');
	let makna = $state('');
	let fn: (sandi: string) => Promise<unknown> = async () => {};
	let aksi: Aksi | undefined;
	let dialog: DialogSandi | undefined = $state();

	export function minta(j: string, m: string, a: Aksi, f: (sandi: string) => Promise<unknown>) {
		judul = j;
		makna = m;
		aksi = a;
		fn = f;
		dialog?.buka();
	}
</script>

<DialogSandi
	bind:this={dialog}
	{judul}
	{makna}
	onkonfirmasi={async ({ sandi }) => {
		await fn(sandi);
		if (aksi?.isian.sandi) aksi.galat = aksi.isian.sandi;
		return true;
	}}
/>
