<!--
  Setiap wilayah yang memuat data punya empat keadaan: memuat (kerangka berbentuk isinya),
  kosong (apa yang seharusnya ada dan satu aksi), galat (apa yang rusak dan coba lagi), dan penuh.
-->
<script lang="ts" generics="T">
	import type { Snippet } from 'svelte';
	import CircleAlert from '@lucide/svelte/icons/circle-alert';
	import Inbox from '@lucide/svelte/icons/inbox';
	import type { Sumber } from './sumber.svelte.ts';

	let {
		sumber,
		kosong,
		children,
		apakahKosong = (d: T) => Array.isArray(d) && d.length === 0,
		baris = 5
	}: {
		sumber: Sumber<T>;
		kosong?: Snippet;
		children: Snippet<[T]>;
		apakahKosong?: (d: T) => boolean;
		baris?: number;
	} = $props();
</script>

{#if sumber.galat}
	<div class="keadaan keadaan--galat" role="alert">
		<CircleAlert class="size-6" />
		<p class="font-medium">{sumber.galat}</p>
		{#if sumber.bisaDiulang}
			<button type="button" class="btn" onclick={() => sumber.muat()}>Coba lagi</button>
		{/if}
	</div>
{:else if sumber.data === undefined}
	<div class="kerangka" aria-busy="true" aria-label="Memuat data">
		{#each { length: baris } as _, i (i)}
			<span class="kerangka__baris"></span>
		{/each}
	</div>
{:else if kosong && apakahKosong(sumber.data)}
	<div class="keadaan">
		<Inbox class="size-6 opacity-60" />
		{@render kosong()}
	</div>
{:else}
	{@render children(sumber.data)}
{/if}
