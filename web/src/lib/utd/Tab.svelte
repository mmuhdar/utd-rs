<!--
  Tab sebagai tautan `?tab=…`: tab aktif tersimpan di URL sehingga bertahan saat dimuat ulang dan bisa dibagikan.
  Induk menghitung tab aktif dengan `tabAktif(daftar)` lalu merender isinya sendiri.
-->
<script lang="ts" module>
	import { page } from '$app/state';
	export type ItemTab = { id: string; label: string };
	/** Tab dari URL bila dikenal (dan boleh dilihat), selain itu tab pertama. */
	export const tabAktif = (daftar: ItemTab[]) =>
		daftar.find((t) => t.id === page.url.searchParams.get('tab'))?.id ?? daftar[0]?.id;
</script>

<script lang="ts">
	let { daftar, label }: { daftar: ItemTab[]; label: string } = $props();
	const aktif = $derived(tabAktif(daftar));
</script>

<nav aria-label={label} class="tanpa-cetak mb-4 flex flex-wrap gap-1 border-b border-garis">
	{#each daftar as t (t.id)}
		<a
			href="?tab={t.id}"
			data-sveltekit-replacestate
			data-sveltekit-noscroll
			aria-current={aktif === t.id ? 'page' : undefined}
			class="-mb-px border-b-2 border-transparent px-3 py-2 text-sm font-medium text-redup hover:text-teks aria-[current=page]:border-utama aria-[current=page]:text-utama"
			>{t.label}</a
		>
	{/each}
</nav>
