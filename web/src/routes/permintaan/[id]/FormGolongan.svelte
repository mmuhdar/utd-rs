<!-- Input reaksi pemeriksaan golongan pasien untuk satu sampel; tafsir ABO/Rh tampil sebelum dikirim. -->
<script lang="ts">
	import { api } from '#lib/api.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Isian from '#lib/Isian.svelte';
	import { golongan } from '#lib/format.ts';
	import Pilihan from '#lib/pelayanan/Pilihan.svelte';
	import { METODE_GOLONGAN, POS_NEG, tafsirAbo } from '#lib/pelayanan/darah.ts';

	let { sampel, onselesai }: { sampel: string; onselesai: () => Promise<void> } = $props();

	const f = $state({ anti_a: '', anti_b: '', sel_a: '', sel_b: '', anti_d: '', metode: 'gel' });
	const lengkap = $derived(!!(f.anti_a && f.anti_b && f.sel_a && f.sel_b && f.anti_d));
	const tafsir = $derived(lengkap ? tafsirAbo(f.anti_a, f.anti_b, f.sel_a, f.sel_b) : null);
	const aksi = new Aksi();
	const n = (x: string) => `${x}-${sampel.slice(0, 8)}`;

	async function kirim(e: SubmitEvent) {
		e.preventDefault();
		if (await aksi.jalankan(() => api('POST', `/sampel/${sampel}/golongan`, f), 'Hasil tersimpan.')) await onselesai();
	}
</script>

<form class="tanpa-cetak flex flex-col gap-4" onsubmit={kirim}>
	<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
		<Pilihan legend="Anti-A (cell grouping)" name={n('anti_a')} opsi={POS_NEG} bind:value={f.anti_a} galat={aksi.isian.anti_a} />
		<Pilihan legend="Anti-B (cell grouping)" name={n('anti_b')} opsi={POS_NEG} bind:value={f.anti_b} />
		<Pilihan legend="Anti-D (Rhesus)" name={n('anti_d')} opsi={POS_NEG} bind:value={f.anti_d} />
		<Pilihan legend="Sel A (serum grouping)" name={n('sel_a')} opsi={POS_NEG} bind:value={f.sel_a} galat={aksi.isian.sel_a} />
		<Pilihan legend="Sel B (serum grouping)" name={n('sel_b')} opsi={POS_NEG} bind:value={f.sel_b} />
		<Isian label="Metode" id={n('metode')} galat={aksi.isian.metode}>
			<select id={n('metode')} class="input" bind:value={f.metode}>
				{#each Object.entries(METODE_GOLONGAN) as [k, l] (k)}<option value={k}>{l}</option>{/each}
			</select>
		</Isian>
	</div>
	{#if tafsir}
		{#if tafsir[1]}
			<p class="text-sm" aria-live="polite">Tafsir: <strong class="text-base">{golongan(tafsir[0], f.anti_d)}</strong> (cell dan serum grouping konsisten)</p>
		{:else}
			<p class="catatan" aria-live="polite">Cell grouping menunjukkan {tafsir[0]} tetapi serum grouping tidak sesuai. Ulangi pemeriksaan; bila tetap, rujuk sampel ke UTD.</p>
		{/if}
	{/if}
	<PesanAksi {aksi} />
	<button class="btn btn--utama self-start" disabled={aksi.sibuk || !lengkap}>Simpan hasil golongan</button>
</form>
