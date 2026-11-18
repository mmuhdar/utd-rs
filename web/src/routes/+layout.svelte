<script lang="ts">
	import { inisial } from '#lib/format.ts';
	import '../app.css';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { sesi } from '#lib/sesi.svelte.ts';
	import { GRUP } from '#lib/navigasi.ts';
	import GantiTema from '#lib/GantiTema.svelte';
	import Merek from '#lib/Merek.svelte';
	import { merek } from '#lib/merek.svelte.ts';
	import Droplet from '@lucide/svelte/icons/droplet';
	import Menu from '@lucide/svelte/icons/menu';
	import LogOut from '@lucide/svelte/icons/log-out';

	let { children } = $props();
	let menuTerbuka = $state(false);
	const publik = $derived(page.url.pathname === '/masuk');

	$effect(() => {
		sesi.muat();
		merek.muat();
		const berakhir = () => {
			sesi.profil = null;
			goto('/masuk');
		};
		window.addEventListener('sesi-berakhir', berakhir);
		return () => window.removeEventListener('sesi-berakhir', berakhir);
	});

	$effect(() => {
		if (!sesi.siap) return;
		const jalur = page.url.pathname;
		if (!sesi.profil && jalur !== '/masuk') goto('/masuk');
		else if (sesi.profil?.wajib_ganti_sandi && jalur !== '/akun-saya') goto('/akun-saya');
	});

	const grup = $derived(
		GRUP.map((g) => ({
			...g,
			menu: g.menu.filter((m) => (!m.hak.length || m.hak.some((h) => sesi.boleh(h))) && (!m.utd || sesi.modeUtd))
		})).filter((g) => g.menu.length)
	);


	// Menu aktif = awalan jalur terpanjang, jadi /permintaan/12 menyorot "Permintaan darah", /permintaan/baru menyorot "Buat permintaan".
	const semuaHref = GRUP.flatMap((g) => g.menu.map((m) => m.href));
	const hrefAktif = $derived(
		semuaHref
			.filter((h) => page.url.pathname === h || (h !== '/' && page.url.pathname.startsWith(h + '/')))
			.sort((a, b) => b.length - a.length)[0]
	);
	const aktif = (href: string) => href === hrefAktif;
</script>

<svelte:head><link rel="icon" href={merek.logo ?? '/favicon.svg'} /></svelte:head>

{#if publik}
	{@render children()}
{:else if !sesi.siap || !sesi.profil}
	<div class="keadaan" aria-busy="true">Memuat…</div>
{:else}
	<div class="min-h-screen lg:flex">
		<a href="#isi" class="sr-only focus:not-sr-only focus:absolute focus:top-2 focus:left-2 focus:z-50 btn">Lewati ke isi</a>
		<header class="tanpa-cetak flex items-center gap-3 bg-samping px-4 py-2.5 text-white lg:hidden">
			<button
				class="btn btn--teks text-samping-teks hover:bg-samping-2"
				aria-label="Buka menu"
				aria-expanded={menuTerbuka}
				onclick={() => (menuTerbuka = !menuTerbuka)}
			>
				<Menu class="size-5" aria-hidden="true" />
			</button>
			<Merek ukuran={30} sub={false} />
		</header>
		<nav
			class="tanpa-cetak {menuTerbuka
				? 'flex'
				: 'hidden'} w-full shrink-0 flex-col gap-4 bg-samping px-3.5 py-5 text-samping-teks lg:sticky lg:top-0 lg:flex lg:h-screen lg:w-64 lg:overflow-y-auto [scrollbar-color:var(--color-samping-2)_transparent] [scrollbar-width:thin]"
			aria-label="Menu utama"
		>
			<a href="/" class="px-1.5" onclick={() => (menuTerbuka = false)}><Merek /></a>
			{#each grup as g (g.judul)}
				<div>
					{#if g.judul}
						<p class="px-2.5 pb-1.5 text-[0.73rem] font-bold tracking-widest text-samping-redup uppercase">{g.judul}</p>
					{/if}
					<ul class="flex flex-col gap-0.5">
						{#each g.menu as m (m.href)}
							<li>
								<a
									href={m.href}
									class="group flex items-center gap-3 rounded-[10px] px-2.5 py-[7px] text-sm font-medium transition-colors hover:bg-samping-2 hover:text-white aria-[current=page]:bg-samping-2 aria-[current=page]:font-semibold aria-[current=page]:text-white"
									aria-current={aktif(m.href) ? 'page' : undefined}
									onclick={() => (menuTerbuka = false)}
								>
									<m.ikon class="size-[18px] shrink-0 text-samping-redup group-hover:text-white group-aria-[current=page]:text-white" aria-hidden="true" />
									<span class="flex-1">{m.label}</span>
									{#if aktif(m.href)}<span class="size-1.5 rounded-full bg-[#fb7185]" aria-hidden="true"></span>{/if}
								</a>
							</li>
						{/each}
					</ul>
				</div>
			{/each}
			<div class="mt-auto">
			<div class="flex items-center gap-2.5 rounded-xl bg-samping-2 px-3 py-2.5">
				<span class="grid size-9 shrink-0 place-items-center rounded-full bg-merek text-sm font-bold text-white" aria-hidden="true"
					>{inisial(sesi.profil.nama)}</span
				>
				<a href="/akun-saya" class="min-w-0 flex-1 hover:underline">
					<span class="block truncate text-sm font-semibold text-white">{sesi.profil.nama}</span>
					<span class="block truncate text-xs font-medium text-samping-redup capitalize">{sesi.profil.peran.join(', ')}</span>
				</a>
				<GantiTema />
				<button
					class="grid size-9 cursor-pointer place-items-center rounded-lg text-samping-redup hover:bg-samping hover:text-white"
					aria-label="Keluar"
					title="Keluar"
					onclick={async () => {
						await sesi.keluar();
						goto('/masuk');
					}}><LogOut class="size-[18px]" aria-hidden="true" /></button
				>
			</div>
			</div>
		</nav>
		<main id="isi" class="min-w-0 flex-1 px-4 py-5 lg:px-8 lg:py-6">
			{@render children()}
		</main>
	</div>
{/if}
