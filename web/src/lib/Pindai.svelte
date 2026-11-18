<!--
  Isian untuk pemindai barcode: pemindai mengetik teks lalu Enter. Setelah Enter nilai dikirim ke `onpindai`
  dan isian dikosongkan serta tetap fokus, supaya pemindaian berikutnya langsung masuk.
-->
<script lang="ts">
	import ScanBarcode from '@lucide/svelte/icons/scan-barcode';
	let {
		label,
		id,
		onpindai,
		autofocus = false,
		placeholder = 'Pindai atau ketik, lalu Enter'
	}: { label: string; id: string; onpindai: (v: string) => void; autofocus?: boolean; placeholder?: string } = $props();
	let nilai = $state('');
	let el: HTMLInputElement | undefined = $state();
	$effect(() => {
		if (autofocus) el?.focus();
	});
</script>

<div>
	<label class="label" for={id}>{label}</label>
	<div class="relative">
		<ScanBarcode class="pointer-events-none absolute top-2.5 left-3 size-5 text-redup" aria-hidden="true" />
		<input
			bind:this={el}
			{id}
			class="input mono pl-10"
			autocomplete="off"
			spellcheck="false"
			{placeholder}
			bind:value={nilai}
			onkeydown={(e) => {
				if (e.key === 'Enter') {
					e.preventDefault();
					const v = nilai.trim();
					if (v) onpindai(v);
					nilai = '';
				}
			}}
		/>
	</div>
</div>
