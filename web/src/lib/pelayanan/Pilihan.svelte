<!-- Pilihan tunggal sebagai tombol radio berlabel (positif/negatif, ya/tidak, jenis). Target sentuh ≥ 40px. -->
<script lang="ts" generics="T">
	let {
		legend,
		name,
		opsi,
		value = $bindable(),
		galat = undefined,
		besar = false,
		disabled = false
	}: {
		legend: string;
		name: string;
		opsi: [T, string][];
		value?: T;
		galat?: string;
		besar?: boolean;
		disabled?: boolean;
	} = $props();
</script>

<fieldset aria-describedby={galat ? `${name}-galat` : undefined} {disabled}>
	<legend class="label">{legend}</legend>
	<div class="flex flex-wrap gap-2">
		{#each opsi as [v, l] (l)}
			<label
				class="inline-flex cursor-pointer items-center gap-2 rounded-md border border-garis bg-kartu px-3 text-sm has-checked:border-utama has-checked:bg-info-latar has-checked:font-semibold {besar
					? 'min-h-12 text-base'
					: 'min-h-10'}"
			>
				<input type="radio" {name} value={v} bind:group={value} />{l}
			</label>
		{/each}
	</div>
	{#if galat}<p class="mt-1 text-sm text-bahaya" id="{name}-galat" role="alert">{galat}</p>{/if}
</fieldset>
