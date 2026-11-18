<script lang="ts">
	import { merek } from '#lib/merek.svelte.ts';
	import { goto } from '$app/navigation';
	import Pindai from '#lib/Pindai.svelte';
	import KepalaHalaman from '#lib/KepalaHalaman.svelte';
</script>

<svelte:head><title>Lacak kantong — {merek.judul}</title></svelte:head>

<KepalaHalaman judul="Lacak kantong" sub="Ketertelusuran satu kantong dari asal sampai akhir" />
<div class="panel max-w-2xl border-2 border-fokus/40">
	<Pindai label="Nomor kantong" id="lacak" autofocus onpindai={(v) => goto(`/lacak/${encodeURIComponent(v)}`)} />
	<p class="mt-3 text-sm text-redup">Menampilkan asal, uji, alokasi, pengeluaran, transfusi, reaksi, penyimpangan suhu, dan akhir kantong.</p>
</div>
