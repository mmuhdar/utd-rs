<!-- Field berlabel dengan pesan galat di bawahnya (galat dari server per isian). -->
<script lang="ts">
	import type { Snippet } from 'svelte';
	let {
		label,
		id,
		galat = undefined,
		bantuan = undefined,
		children
	}: { label: string; id: string; galat?: string; bantuan?: string; children: Snippet } = $props();
</script>

<div>
	<label class="label" for={id}>{label}</label>
	{@render children()}
	{#if galat}
		<p class="mt-1 text-sm text-bahaya" id="{id}-galat" role="alert">{galat}</p>
	{:else if bantuan}
		<p class="mt-1 text-xs text-redup">{bantuan}</p>
	{/if}
</div>
