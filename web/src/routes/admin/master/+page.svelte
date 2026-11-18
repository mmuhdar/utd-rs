<script lang="ts">
	import { sesi } from '#lib/sesi.svelte.ts';
	import Tab, { tabAktif } from '#lib/utd/Tab.svelte';
	import Profil from './Profil.svelte';
	import UtdMitra from './UtdMitra.svelte';
	import AlatSimpan from './AlatSimpan.svelte';
	import AturanKomponen from './AturanKomponen.svelte';
	import AturanDonor from './AturanDonor.svelte';

	const daftar = $derived(
		[
			{ id: 'profil', label: 'Profil instalasi', boleh: sesi.boleh('master.kelola') },
			{ id: 'utd', label: 'UTD mitra', boleh: sesi.boleh('master.kelola') },
			{ id: 'alat', label: 'Alat simpan', boleh: sesi.boleh('master.kelola') },
			{ id: 'aturan-komponen', label: 'Aturan komponen', boleh: sesi.boleh('aturan.kelola') },
			{ id: 'aturan-donor', label: 'Aturan donor', boleh: sesi.boleh('aturan.kelola') && sesi.modeUtd }
		].filter((t) => t.boleh)
	);
	const tab = $derived(tabAktif(daftar));
</script>

<svelte:head><title>Data induk & aturan — utd-rs</title></svelte:head>

<h1 class="mb-4">Data induk & aturan</h1>

{#if !daftar.length}
	<p class="catatan">Anda tidak berhak mengelola data induk atau aturan.</p>
{:else}
	<Tab {daftar} label="Bagian data induk" />
	{#if tab === 'profil'}<Profil />
	{:else if tab === 'utd'}<UtdMitra />
	{:else if tab === 'alat'}<AlatSimpan />
	{:else if tab === 'aturan-komponen'}<AturanKomponen />
	{:else if tab === 'aturan-donor'}<AturanDonor />
	{/if}
{/if}
