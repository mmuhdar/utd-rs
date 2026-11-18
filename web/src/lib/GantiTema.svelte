<!-- Tombol ringkas di bilah samping: tiap klik berganti terang → gelap → ikuti sistem. -->
<script lang="ts">
	import Sun from '@lucide/svelte/icons/sun';
	import Moon from '@lucide/svelte/icons/moon';
	import Monitor from '@lucide/svelte/icons/monitor';
	import { tema, type Pilihan } from './tema.svelte.ts';
	const URUT: Pilihan[] = ['terang', 'gelap', 'sistem'];
	const NAMA: Record<Pilihan, string> = { terang: 'terang', gelap: 'gelap', sistem: 'ikuti sistem' };
	const Ikon = $derived(tema.pilihan === 'terang' ? Sun : tema.pilihan === 'gelap' ? Moon : Monitor);
	const berikut = $derived(URUT[(URUT.indexOf(tema.pilihan) + 1) % URUT.length]);
</script>

<button
	type="button"
	class="grid size-9 shrink-0 cursor-pointer place-items-center rounded-lg text-samping-redup hover:bg-samping hover:text-white"
	aria-label="Tema {NAMA[tema.pilihan]}. Ganti ke {NAMA[berikut]}"
	title="Tema: {NAMA[tema.pilihan]} (klik untuk ganti ke {NAMA[berikut]})"
	onclick={() => tema.atur(berikut)}><Ikon class="size-[18px]" aria-hidden="true" /></button
>
