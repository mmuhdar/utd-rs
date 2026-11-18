<!-- Kedaluwarsa kantong: waktu, relatif, dan tanda teks + warna bila mendekati atau lewat. -->
<script lang="ts">
	import Chip from '#lib/Chip.svelte';
	import { relatif, waktu } from '#lib/format.ts';
	let { nilai, mendekati = false }: { nilai: string; mendekati?: boolean } = $props();
	const lewat = $derived(new Date(nilai).getTime() <= Date.now());
</script>

<span class="whitespace-nowrap">{waktu(nilai)}</span>
<span class="block text-xs {lewat ? 'text-bahaya' : mendekati ? 'text-waspada' : 'text-redup'}">{relatif(nilai)}</span>
{#if lewat}
	<Chip nada="bahaya">Kedaluwarsa</Chip>
{:else if mendekati}
	<Chip nada="waspada">Segera kedaluwarsa</Chip>
{/if}
