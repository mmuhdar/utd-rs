<!-- Status selalu teks + warna (tidak hanya warna). -->
<script lang="ts">
	import type { Snippet } from 'svelte';
	let { nada = 'netral', children }: { nada?: string; children: Snippet } = $props();
</script>

<span class="chip chip--{nada}">{@render children()}</span>
