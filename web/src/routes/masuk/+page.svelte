<script lang="ts">
	import { merek } from '#lib/merek.svelte.ts';
	import { goto } from '$app/navigation';
	import { api } from '#lib/api.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import PilihTema from '#lib/PilihTema.svelte';
	import Merek from '#lib/Merek.svelte';
	import { sesi, type Profil } from '#lib/sesi.svelte.ts';
	import Droplet from '@lucide/svelte/icons/droplet';
	import ShieldCheck from '@lucide/svelte/icons/shield-check';
	import ScanBarcode from '@lucide/svelte/icons/scan-barcode';
	import PenLine from '@lucide/svelte/icons/pen-line';
	import User from '@lucide/svelte/icons/user';
	import Lock from '@lucide/svelte/icons/lock';
	import LogIn from '@lucide/svelte/icons/log-in';

	let nama_pengguna = $state('');
	let sandi = $state('');
	const aksi = new Aksi();

	async function masuk(e: SubmitEvent) {
		e.preventDefault();
		const p = await aksi.jalankan(() => api<Profil>('POST', '/auth/masuk', { nama_pengguna, sandi }));
		sandi = '';
		if (p) {
			sesi.profil = p;
			goto(p.wajib_ganti_sandi ? '/akun-saya' : '/');
		}
	}
</script>

<svelte:head><title>Masuk — {merek.judul}</title></svelte:head>

<main class="flex min-h-screen bg-kartu">
	<section
		class="hidden flex-1 flex-col justify-between bg-linear-to-tr from-[#2a0612] via-[#7a0b25] to-[#c2133a] p-14 text-white lg:flex"
		aria-label="Tentang aplikasi"
	>
		<div class="max-w-md"><Merek ukuran={48} besar /></div>
		<div class="max-w-xl">
			<p class="text-5xl leading-[1.1] font-extrabold tracking-tight">Setiap kantong tertelusur, dari UTD sampai samping tempat tidur.</p>
			<p class="mt-5 max-w-lg text-lg leading-relaxed font-medium text-[#ffd5de]">
				Sistem informasi bank darah rumah sakit sesuai Permenkes 91 Tahun 2015 — penerimaan, uji pratransfusi, pengeluaran, dan pemantauan
				transfusi dalam satu alur.
			</p>
		</div>
		<ul class="flex flex-wrap gap-2.5 text-sm font-semibold">
			<li class="flex items-center gap-2 rounded-full border border-white/20 bg-white/10 px-3.5 py-2">
				<ShieldCheck class="size-4" aria-hidden="true" />Jejak audit berantai
			</li>
			<li class="flex items-center gap-2 rounded-full border border-white/20 bg-white/10 px-3.5 py-2">
				<ScanBarcode class="size-4" aria-hidden="true" />Pindai ISBT 128
			</li>
			<li class="flex items-center gap-2 rounded-full border border-white/20 bg-white/10 px-3.5 py-2">
				<PenLine class="size-4" aria-hidden="true" />Tanda tangan elektronik
			</li>
		</ul>
	</section>

	<div class="relative flex w-full items-center justify-center p-6 lg:w-[620px] lg:shrink-0">
		<div class="absolute top-5 right-5 w-56"><PilihTema /></div>
		<form class="flex w-full max-w-[400px] flex-col gap-5" onsubmit={masuk}>
			<div class="mb-1 lg:hidden"><Merek ukuran={40} terang={false} /></div>
			<div>
				<h1>Masuk</h1>
				<p class="mt-1 font-medium text-redup">{merek.unit} · {merek.judul}</p>
			</div>
			<div>
				<label class="label" for="nama">Nama pengguna</label>
				<div class="relative">
					<User class="pointer-events-none absolute top-1/2 left-3.5 size-5 -translate-y-1/2 text-redup" aria-hidden="true" />
					<input id="nama" class="input min-h-12 rounded-xl pl-11 text-base" autocomplete="username" bind:value={nama_pengguna} required />
				</div>
			</div>
			<div>
				<label class="label" for="sandi">Kata sandi</label>
				<div class="relative">
					<Lock class="pointer-events-none absolute top-1/2 left-3.5 size-5 -translate-y-1/2 text-redup" aria-hidden="true" />
					<input
						id="sandi"
						class="input min-h-12 rounded-xl pl-11 text-base"
						type="password"
						autocomplete="current-password"
						aria-describedby="sandi-bantuan"
						bind:value={sandi}
						required
					/>
				</div>
				<p id="sandi-bantuan" class="mt-1.5 text-xs text-redup">Akun terkunci sementara setelah 5 kali salah sandi.</p>
			</div>
			<PesanAksi {aksi} />
			<button class="btn btn--utama btn--besar w-full" disabled={aksi.sibuk}>
				<LogIn class="size-5" aria-hidden="true" />{aksi.sibuk ? 'Memeriksa…' : 'Masuk'}
			</button>
			<hr class="border-garis" />
			<p class="flex items-start gap-2.5 text-sm leading-relaxed font-medium text-redup">
				<ShieldCheck class="mt-0.5 size-[18px] shrink-0" aria-hidden="true" />
				Akun bersifat pribadi, jangan dipinjamkan. Sesi berakhir setelah 30 menit tanpa aktivitas, dan setiap tindakan tercatat atas nama Anda.
			</p>
			<p class="text-xs text-redup">
				Browser menampilkan peringatan koneksi tidak aman?
				<a class="font-semibold text-teks-2 underline" href="http://{location.host}/sertifikat">Pasang sertifikat utd-rs</a> sekali di perangkat ini.
			</p>
		</form>
	</div>
</main>
