<!--
  Tanda tangan elektronik (ADR 0005): konfirmasi ulang kata sandi. Mode `petugas_lain` meminta nama pengguna
  dan sandi petugas kedua (verifikasi samping tempat tidur, persetujuan darurat). Memakai <dialog> bawaan
  sehingga fokus terkurung dan Esc menutup.
-->
<script lang="ts">
	import PenLine from '@lucide/svelte/icons/pen-line';
	import X from '@lucide/svelte/icons/x';
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
	const uid = $props.id();
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

<dialog bind:this={dialog} class="dialog">
	<form class="flex flex-col gap-5 p-7" onsubmit={kirim}>
		<div class="flex items-start gap-3.5">
			<span class="grid size-11 shrink-0 place-items-center rounded-xl bg-halus"><PenLine class="size-5" aria-hidden="true" /></span>
			<div class="min-w-0 flex-1">
				<h2 class="text-xl font-extrabold tracking-tight">{judul}</h2>
				<p class="mt-1 text-sm text-redup">{makna}</p>
			</div>
			<button type="button" class="grid size-9 cursor-pointer place-items-center rounded-lg text-redup hover:bg-halus" aria-label="Tutup" onclick={() => dialog?.close()}
				><X class="size-5" aria-hidden="true" /></button
			>
		</div>
		{#if petugasLain}
			<div>
				<label class="label" for="{uid}-nama">{labelPetugas}</label>
				<input id="{uid}-nama" class="input" autocomplete="off" bind:value={nama} required />
			</div>
		{/if}
		<div>
			<label class="label" for="{uid}-sandi">{petugasLain ? 'Kata sandi petugas tersebut' : 'Kata sandi Anda'}</label>
			<input id="{uid}-sandi" class="input" type="password" autocomplete="current-password" bind:value={sandi} required />
		</div>
		<div class="flex justify-end gap-2.5">
			<button type="button" class="btn" onclick={() => dialog?.close()}>Batal</button>
			<button class="btn btn--utama" disabled={sibuk}><PenLine class="size-4" aria-hidden="true" />{sibuk ? 'Memproses…' : 'Tanda tangani'}</button>
		</div>
	</form>
</dialog>
