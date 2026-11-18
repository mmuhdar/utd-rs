<!-- Status selalu teks + warna (tidak hanya warna). `polos`: tanpa titik, untuk penanda seperti "Cito". -->
<script lang="ts">
	import type { Snippet } from 'svelte';
	let { nada = 'netral', polos = false, children }: { nada?: string; polos?: boolean; children: Snippet } = $props();
</script>

<span class="chip chip--{nada}" class:chip--polos={polos}>{@render children()}</span>
