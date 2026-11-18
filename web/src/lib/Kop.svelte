<!--
  Kop surat di dokumen cetak: logo kiri, identitas RS di tengah, logo kanan, garis ganda di bawah.
  Baris yang kosong tidak dicetak; logo kosong tetap berupa kotak kosong supaya teks tetap di tengah.
  `cetakSaja`: hanya tampil saat dicetak (halaman kerja yang juga bisa dicetak).
-->
<script lang="ts">
	import { kopSurat, type Kop } from './kop.svelte.ts';
	let { kop = undefined, cetakSaja = false }: { kop?: Kop; cetakSaja?: boolean } = $props();
	$effect(() => {
		if (!kop && !kopSurat.data && !kopSurat.memuat) kopSurat.muat();
	});
	const k = $derived(kop ?? kopSurat.data);
	const nama = $derived(k ? k.nama_rs || k.nama_rs_profil || '' : '');
</script>

{#if k}
	<header class="kop {cetakSaja ? 'hidden print:flex' : 'flex'} items-center gap-4 border-b-4 border-double border-teks pb-3 text-teks">
		<span class="grid size-20 shrink-0 place-items-center">{#if k.logo_kiri}<img src={k.logo_kiri} alt="" class="max-h-full max-w-full object-contain" />{/if}</span>
		<div class="min-w-0 flex-1 text-center leading-snug">
			{#if k.pemerintah}<p class="text-sm font-semibold uppercase">{k.pemerintah}</p>{/if}
			{#if nama}<p class="text-xl font-extrabold tracking-tight uppercase">{nama}</p>{/if}
			{#if k.unit}<p class="text-base font-bold">{k.unit}</p>{/if}
			{#if k.alamat}<p class="text-xs">{k.alamat}</p>{/if}
			{#if k.kontak}<p class="text-xs">{k.kontak}</p>{/if}
			{#if k.akreditasi}<p class="text-xs font-semibold italic">{k.akreditasi}</p>{/if}
		</div>
		<span class="grid size-20 shrink-0 place-items-center">{#if k.logo_kanan}<img src={k.logo_kanan} alt="" class="max-h-full max-w-full object-contain" />{/if}</span>
	</header>
{/if}
