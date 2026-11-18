<script lang="ts">
	import Chip from './Chip.svelte';
	import { STATUS_KANTONG } from './format.ts';
	let { status }: { status: string } = $props();
	const [label, nada] = $derived(STATUS_KANTONG[status] ?? [status, 'netral']);
</script>

<Chip {nada}>{label}</Chip>
