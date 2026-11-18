<!--
  Tanda tangan elektronik (ADR 0005): konfirmasi ulang kata sandi. Mode `petugas_lain` meminta nama pengguna
  dan sandi petugas kedua (verifikasi samping tempat tidur, persetujuan darurat). Memakai <dialog> bawaan
  sehingga fokus terkurung dan Esc menutup.
-->
<script lang="ts">
	import PenLine from '@lucide/svelte/icons/pen-line';
	let {
		judul,
		makna,
		petugasLain = false,
		labelPetugas = 'Nama pengguna petugas kedua',
		onkonfirmasi
	}: {
		judul: string;
		makna: string;
		petugasLain?: boolean;
		labelPetugas?: string;
		onkonfirmasi: (d: { sandi: string; nama_pengguna: string }) => Promise<boolean>;
	} = $props();
	let dialog: HTMLDialogElement | undefined = $state();
	let sandi = $state('');
	let nama = $state('');
	let sibuk = $state(false);

	export function buka() {
		sandi = '';
		nama = '';
		dialog?.showModal();
	}

	async function kirim(e: SubmitEvent) {
		e.preventDefault();
		sibuk = true;
		const ok = await onkonfirmasi({ sandi, nama_pengguna: nama });
		sibuk = false;
		sandi = '';
		if (ok) dialog?.close();
	}
</script>

<dialog bind:this={dialog} class="m-auto w-full max-w-md rounded-lg border border-garis bg-kartu p-0 text-teks backdrop:bg-black/50">
	<form class="flex flex-col gap-4 p-5" onsubmit={kirim}>
		<h2 class="flex items-center gap-2"><PenLine class="size-5" aria-hidden="true" />{judul}</h2>
		<p class="text-sm text-redup">{makna}</p>
		{#if petugasLain}
			<div>
				<label class="label" for="ttd-nama">{labelPetugas}</label>
				<input id="ttd-nama" class="input" autocomplete="off" bind:value={nama} required />
			</div>
		{/if}
		<div>
			<label class="label" for="ttd-sandi">{petugasLain ? 'Kata sandi petugas tersebut' : 'Kata sandi Anda'}</label>
			<input id="ttd-sandi" class="input" type="password" autocomplete="current-password" bind:value={sandi} required />
		</div>
		<div class="flex justify-end gap-2">
			<button type="button" class="btn" onclick={() => dialog?.close()}>Batal</button>
			<button class="btn btn--utama" disabled={sibuk}>{sibuk ? 'Memproses…' : 'Tanda tangani'}</button>
		</div>
	</form>
</dialog>
