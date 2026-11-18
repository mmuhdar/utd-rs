<!-- Galat umum dan konfirmasi berhasil untuk satu Aksi. -->
<script lang="ts">
	import CircleAlert from '@lucide/svelte/icons/circle-alert';
	import CircleCheck from '@lucide/svelte/icons/circle-check';
	import type { Aksi } from './aksi.svelte.ts';
	let { aksi }: { aksi: Aksi } = $props();
</script>

{#if aksi.galat}
	<div class="peringatan" role="alert">
		<CircleAlert class="mt-0.5 size-5 shrink-0" aria-hidden="true" />
		<p>{aksi.galat}</p>
	</div>
{:else if aksi.berhasil}
	<p class="flex items-center gap-2 text-sm font-medium text-aman" role="status">
		<CircleCheck class="size-4" aria-hidden="true" />{aksi.berhasil}
	</p>
{/if}
