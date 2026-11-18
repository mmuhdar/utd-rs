<!--
  Pilihan tunggal sebagai tombol segmen (positif/negatif, ya/tidak, jenis). Tetap radio bawaan (sr-only) supaya
  keyboard dan pembaca layar bekerja. `bahaya`: nilai yang saat dipilih tampil merah, mis. uji silang positif.
-->
<script lang="ts" generics="T">
	let {
		legend,
		name,
		opsi,
		value = $bindable(),
		galat = undefined,
		besar = false,
		disabled = false,
		bahaya = [],
		required = false
	}: {
		legend: string;
		name: string;
		opsi: [T, string][];
		value?: T;
		galat?: string;
		besar?: boolean;
		disabled?: boolean;
		bahaya?: T[];
		required?: boolean;
	} = $props();
	const rata = $derived(opsi.length <= 3);
</script>

<fieldset aria-describedby={galat ? `${name}-galat` : undefined} {disabled}>
	<legend class="label">{legend}</legend>
	<div
		class="flex flex-wrap gap-0.5 rounded-[10px] bg-halus p-1 {galat ? 'ring-2 ring-bahaya-titik' : ''} {rata ? 'w-full' : 'w-fit max-w-full'}"
	>
		{#each opsi as [v, l], i (l)}
			{@const merah = bahaya.includes(v)}
			<label
				class="inline-flex cursor-pointer items-center justify-center rounded-lg px-3 text-sm whitespace-nowrap font-medium text-redup transition-colors hover:text-teks has-checked:font-semibold has-checked:shadow-kartu has-focus-visible:outline-3 has-focus-visible:outline-fokus {merah
					? 'has-checked:bg-bahaya-isi has-checked:text-white'
					: 'has-checked:bg-kartu has-checked:text-teks'} {besar ? 'min-h-11 text-base' : 'min-h-9'} {rata ? 'flex-auto' : ''}"
			>
				<input type="radio" class="sr-only" {name} value={v} bind:group={value} required={required && i === 0} />{l}
			</label>
		{/each}
	</div>
	{#if galat}<p class="mt-1 text-sm text-bahaya" id="{name}-galat" role="alert">{galat}</p>{/if}
</fieldset>
