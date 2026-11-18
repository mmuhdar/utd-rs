<!-- Logo + judul aplikasi dari kop surat; tanpa logo memakai tanda tetes darah bawaan. `terang`: teks putih (latar gelap). -->
<script lang="ts">
	import Droplet from '@lucide/svelte/icons/droplet';
	import { merek } from './merek.svelte.ts';
	let { ukuran = 36, terang = true, sub = true, besar = false }: { ukuran?: number; terang?: boolean; sub?: boolean; besar?: boolean } = $props();
</script>

<span class="flex min-w-0 items-center gap-2.5">
	{#if merek.logo}
		<span class="grid shrink-0 place-items-center overflow-hidden rounded-[11px] bg-white p-1" style="width: {ukuran}px; height: {ukuran}px">
			<img src={merek.logo} alt="" class="max-h-full max-w-full object-contain" onerror={() => (merek.rusak = true)} />
		</span>
	{:else}
		<span
			class="grid shrink-0 place-items-center rounded-[11px] bg-linear-to-br from-[#f43f5e] to-merek-gelap"
			style="width: {ukuran}px; height: {ukuran}px"
		>
			<Droplet class="text-white" style="width: {ukuran * 0.55}px; height: {ukuran * 0.55}px" strokeWidth={2.5} aria-hidden="true" />
		</span>
	{/if}
	<span class="min-w-0">
		<span class="line-clamp-2 {besar ? 'text-lg' : 'text-sm'} leading-tight font-extrabold {terang ? 'text-white' : 'text-teks'}">{merek.judul}</span>
		{#if sub}<span class="block truncate {besar ? 'text-sm' : 'text-xs'} font-medium {terang ? 'text-white/70' : 'text-redup'}">{merek.unit}</span>{/if}
	</span>
</span>
