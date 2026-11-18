<!-- Penunjuk tahap alur pratransfusi sampai transfusi (SPEK M3–M5), dihitung dari detail permintaan. -->
<script lang="ts">
	import Check from '@lucide/svelte/icons/check';
	import { golongan, pukul } from '#lib/format.ts';
	let { p, aturan }: { p: any; aturan: Record<string, string> } = $props();

	const tahap = $derived.by(() => {
		const total = p.baris.reduce((n: number, b: any) => n + b.jumlah, 0);
		const totalUji = p.baris.filter((b: any) => aturan[b.komponen] !== 'tidak_perlu').reduce((n: number, b: any) => n + b.jumlah, 0);
		const layak = p.sampel.find((s: any) => s.layak);
		const gol = p.golongan_terverifikasi;
		const serasi = p.uji_silang.filter((u: any) => u.terverifikasi && ['serasi', 'hanya_prc'].includes(u.kesimpulan)).length;
		const alok = p.alokasi.filter((a: any) => !a.dilepas_pada);
		const keluar = alok.filter((a: any) => a.pengeluaran).length;
		const transfusi = alok.filter((a: any) => a.transfusi).length;
		return [
			{ nama: 'Sampel', selesai: !!layak, ket: layak ? `Diterima ${pukul(layak.diterima_pada)}` : p.sampel.length ? 'Tidak layak' : 'Belum diterima' },
			{ nama: 'Golongan darah', selesai: !!gol, ket: gol ? `${golongan(gol.abo, gol.rhesus)} terverifikasi` : 'Belum terverifikasi' },
			{
				nama: 'Uji silang serasi',
				selesai: serasi >= totalUji,
				ket: totalUji ? `${serasi} dari ${totalUji} serasi` : 'Tidak diperlukan'
			},
			{ nama: 'Alokasi', selesai: alok.length >= total, ket: `${alok.length} dari ${total} kantong` },
			{ nama: 'Pengeluaran', selesai: keluar >= total, ket: `${keluar} dari ${total} kantong` },
			{ nama: 'Transfusi', selesai: transfusi >= total, ket: `${transfusi} dari ${total} kantong` }
		];
	});
	const berjalan = $derived(['diminta', 'diproses', 'siap'].includes(p.status));
	const sekarang = $derived(berjalan ? tahap.findIndex((t) => !t.selesai) : -1);
</script>

<ol class="panel tanpa-cetak flex flex-wrap gap-x-2 gap-y-3 py-3.5" aria-label="Tahap permintaan">
	{#each tahap as t, i (t.nama)}
		{@const aktif = i === sekarang}
		<li class="flex min-w-40 flex-1 items-center gap-2.5" aria-current={aktif ? 'step' : undefined}>
			<span
				class="grid size-7 shrink-0 place-items-center rounded-full text-xs font-bold {t.selesai
					? 'bg-aman-titik text-white'
					: aktif
						? 'bg-utama text-pada-utama'
						: 'bg-halus text-redup'}"
			>
				{#if t.selesai}<Check class="size-4" strokeWidth={3} aria-hidden="true" /><span class="sr-only">Selesai:</span>{:else}{i + 1}{/if}
			</span>
			<span class="min-w-0">
				<span class="block text-sm {aktif ? 'font-bold' : 'font-semibold'} {t.selesai || aktif ? '' : 'text-redup'}">{t.nama}</span>
				<span class="block text-xs text-redup">{t.ket}</span>
			</span>
			{#if i < tahap.length - 1}
				<span class="hidden h-0.5 min-w-4 flex-1 rounded xl:block {t.selesai ? 'bg-aman-titik' : 'bg-garis'}" aria-hidden="true"></span>
			{/if}
		</li>
	{/each}
</ol>
