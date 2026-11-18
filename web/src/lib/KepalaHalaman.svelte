<!-- Kepala halaman seragam (design/render): jalur opsional, judul, keterangan, dan tombol aksi di kanan. -->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	let {
		judul,
		sub = undefined,
		jalur = [],
		mono = false,
		aksi = undefined,
		children = undefined
	}: {
		judul: string;
		sub?: string;
		/** [label, href] — elemen terakhir adalah halaman ini dan tidak ditautkan. */
		jalur?: [string, string?][];
		mono?: boolean;
		aksi?: Snippet;
		/** Isi tambahan di bawah judul, mis. chip status. */
		children?: Snippet;
	} = $props();
</script>

<header class="mb-5 flex flex-wrap items-end justify-between gap-3">
	<div class="min-w-0">
		{#if jalur.length}
			<nav class="tanpa-cetak mb-1 flex flex-wrap items-center gap-1.5 text-sm font-medium text-redup" aria-label="Jalur">
				{#each jalur as [label, href], i (i)}
					{#if i}<ChevronRight class="size-3.5" aria-hidden="true" />{/if}
					{#if href}<a class="hover:underline" {href}>{label}</a>{:else}<span class="text-teks-2">{label}</span>{/if}
				{/each}
			</nav>
		{/if}
		<h1 class={mono ? 'mono' : ''}>{judul}</h1>
		{#if sub}<p class="mt-1 font-medium text-redup">{sub}</p>{/if}
		{@render children?.()}
	</div>
	{#if aksi}<div class="tanpa-cetak flex flex-wrap gap-2 empty:hidden">{@render aksi()}</div>{/if}
</header>
