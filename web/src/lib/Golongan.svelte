<!-- Lencana golongan darah berwarna label ISBT 128; teksnya tetap "A+"/"O−" sehingga terbaca tanpa warna. -->
<script lang="ts">
	import { golongan } from './format.ts';
	let { abo, rhesus, ukuran = 'sedang' }: { abo?: string | null; rhesus?: string | null; ukuran?: 'kecil' | 'sedang' | 'besar' } = $props();
</script>

{#if abo}
	<span
		class="gol gol--{abo.toLowerCase()}"
		class:gol--neg={rhesus === 'neg'}
		class:gol--kecil={ukuran === 'kecil'}
		class:gol--besar={ukuran === 'besar'}
		title={golongan(abo, rhesus)}
		><span aria-hidden="true">{abo}{rhesus === 'neg' ? '−' : rhesus === 'pos' ? '+' : ''}</span><span class="sr-only"
			>{golongan(abo, rhesus)}</span
		></span
	>
{:else}
	<span class="text-redup">—</span>
{/if}
