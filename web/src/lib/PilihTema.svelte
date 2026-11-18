<!-- Pilihan tema: terang, gelap, atau ikuti sistem. Dipakai di halaman Masuk dan Akun saya. -->
<script lang="ts">
	import Sun from '@lucide/svelte/icons/sun';
	import Moon from '@lucide/svelte/icons/moon';
	import Monitor from '@lucide/svelte/icons/monitor';
	import { tema, type Pilihan } from './tema.svelte.ts';
	const PILIHAN: [Pilihan, string, typeof Sun][] = [
		['terang', 'Terang', Sun],
		['gelap', 'Gelap', Moon],
		['sistem', 'Ikuti sistem', Monitor]
	];
</script>

<div role="group" aria-label="Tema tampilan" class="flex gap-0.5 rounded-[10px] bg-halus p-1">
	{#each PILIHAN as [p, label, Ikon] (p)}
		<button
			type="button"
			class="flex flex-1 cursor-pointer items-center justify-center gap-1.5 rounded-lg px-2 py-1.5 text-xs font-semibold text-redup transition-colors hover:text-teks aria-pressed:bg-kartu aria-pressed:text-teks aria-pressed:shadow-kartu"
			aria-pressed={tema.pilihan === p}
			title={label}
			onclick={() => tema.atur(p)}
		>
			<Ikon class="size-4" aria-hidden="true" /><span class={p === 'sistem' ? 'sr-only' : ''}>{label}</span>
		</button>
	{/each}
</div>
