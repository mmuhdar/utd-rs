<!-- Penunjuk tahap alur UTD (pola sama dengan routes/permintaan/[id]/Tahap.svelte). `sekarang` = indeks tahap aktif, -1 bila alur berhenti. -->
<script lang="ts">
	import Check from '@lucide/svelte/icons/check';
	let { tahap, sekarang, label }: { tahap: { nama: string; selesai: boolean; ket: string }[]; sekarang: number; label: string } = $props();
</script>

<ol class="panel tanpa-cetak flex flex-wrap gap-x-2 gap-y-3 py-3.5" aria-label={label}>
	{#each tahap as t, i (t.nama)}
		{@const aktif = i === sekarang}
		<li class="flex min-w-36 flex-1 items-center gap-2.5" aria-current={aktif ? 'step' : undefined}>
			<span
				class="grid size-7 shrink-0 place-items-center rounded-full text-xs font-bold {t.selesai
					? 'bg-aman-titik text-white'
					: aktif
						? 'bg-utama text-pada-utama'
						: 'bg-halus text-redup'}"
			>
				{#if t.selesai}<Check class="size-4" strokeWidth={3} aria-hidden="true" /><span class="sr-only">Selesai:</span>{:else}{i + 1}{/if}
			</span>
			<span class="min-w-0">
				<span class="block text-sm {aktif ? 'font-bold' : 'font-semibold'} {t.selesai || aktif ? '' : 'text-redup'}">{t.nama}</span>
				<span class="block text-xs text-redup">{t.ket}</span>
			</span>
			{#if i < tahap.length - 1}
				<span class="hidden h-0.5 min-w-4 flex-1 rounded xl:block {t.selesai ? 'bg-aman-titik' : 'bg-garis'}" aria-hidden="true"></span>
			{/if}
		</li>
	{/each}
</ol>
