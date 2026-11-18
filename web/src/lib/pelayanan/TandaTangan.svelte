<!--
  Satu DialogSandi per halaman (id isiannya tetap, jadi tidak boleh ada dua). Pemanggil memberi judul, makna, dan
  fungsi kirim; dialog selalu tertutup setelah kirim karena DialogSandi tidak menampilkan galat di dalamnya:
  galat tampil di PesanAksi milik aksi pemanggil.
-->
<script lang="ts" module>
	export type OpsiTtd = {
		judul: string;
		makna: string;
		petugasLain?: boolean;
		labelPetugas?: string;
		kirim: (d: { sandi: string; nama_pengguna: string }) => Promise<unknown>;
	};
	export type Minta = (o: OpsiTtd) => void;
</script>

<script lang="ts">
	import { tick } from 'svelte';
	import DialogSandi from '#lib/DialogSandi.svelte';

	let o = $state<OpsiTtd>({ judul: '', makna: '', kirim: async () => {} });
	let dialog: DialogSandi | undefined = $state();

	export async function minta(opsi: OpsiTtd) {
		o = opsi;
		await tick();
		dialog?.buka();
	}
</script>

<DialogSandi
	bind:this={dialog}
	judul={o.judul}
	makna={o.makna}
	petugasLain={o.petugasLain}
	labelPetugas={o.labelPetugas}
	onkonfirmasi={async (d) => {
		await o.kirim(d);
		return true;
	}}
/>
