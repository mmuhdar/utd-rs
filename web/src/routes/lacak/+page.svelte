<script lang="ts">
	import { goto } from '$app/navigation';
	import Pindai from '#lib/Pindai.svelte';
</script>

<svelte:head><title>Lacak kantong — utd-rs</title></svelte:head>

<h1 class="mb-4">Lacak kantong</h1>
<div class="panel max-w-md">
	<Pindai label="Nomor kantong" id="lacak" autofocus onpindai={(v) => goto(`/lacak/${encodeURIComponent(v)}`)} />
	<p class="mt-2 text-sm text-redup">Menampilkan asal, uji, alokasi, pengeluaran, transfusi, reaksi, penyimpangan suhu, dan akhir kantong.</p>
</div>
