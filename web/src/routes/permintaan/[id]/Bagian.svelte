<!--
  Satu tahap alur permintaan sebagai kartu yang bisa dilipat (desain 04): tahap selesai tampil satu baris ringkasan,
  tahap yang sedang dikerjakan terbuka. Saat dicetak semua isi tampil.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import Check from '@lucide/svelte/icons/check';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';
	let {
		id,
		nomor,
		judul,
		ringkas,
		selesai = false,
		bukaAwal = !selesai,
		children
	}: { id: string; nomor: number; judul: string; ringkas: string; selesai?: boolean; bukaAwal?: boolean; children: Snippet } = $props();
	// Bawaan: hanya tahap yang sedang dikerjakan yang terbuka; pengguna boleh membuka/menutup sendiri.
	let buka = $state<boolean | null>(null);
	const terbuka = $derived(buka ?? bukaAwal);
</script>

<section {id} class="panel scroll-mt-4 p-0" aria-labelledby="{id}-judul">
	<button
		type="button"
		class="flex w-full cursor-pointer items-center gap-3.5 rounded-2xl px-5 py-4 text-left hover:bg-halus/50"
		aria-expanded={terbuka}
		aria-controls="{id}-isi"
		onclick={() => (buka = !terbuka)}
	>
		<span
			class="grid size-8 shrink-0 place-items-center rounded-full text-sm font-bold {selesai ? 'bg-aman-titik text-white' : 'bg-utama text-pada-utama'}"
		>
			{#if selesai}<Check class="size-4" strokeWidth={3} aria-hidden="true" /><span class="sr-only">Selesai.</span>{:else}{nomor}{/if}
		</span>
		<span class="min-w-0 flex-1">
			<h2 id="{id}-judul" class="text-base font-bold">{judul}</h2>
			<span class="block truncate text-sm text-redup">{ringkas}</span>
		</span>
		<ChevronDown class="tanpa-cetak size-5 shrink-0 text-redup transition-transform {terbuka ? 'rotate-180' : ''}" aria-hidden="true" />
	</button>
	<div id="{id}-isi" class="flex flex-col gap-4 border-t border-garis px-5 pt-4 pb-5 {terbuka ? '' : 'hidden print:flex'}">
		{@render children()}
	</div>
</section>
