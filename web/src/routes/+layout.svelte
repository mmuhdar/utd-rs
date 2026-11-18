<script lang="ts">
	import '../app.css';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { sesi } from '#lib/sesi.svelte.ts';
	import { GRUP } from '#lib/navigasi.ts';
	import Droplet from '@lucide/svelte/icons/droplet';
	import Menu from '@lucide/svelte/icons/menu';
	import LogOut from '@lucide/svelte/icons/log-out';

	let { children } = $props();
	let menuTerbuka = $state(false);
	const publik = $derived(page.url.pathname === '/masuk');

	$effect(() => {
		sesi.muat();
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
			menu: g.menu.filter((m) => m.hak.some((h) => sesi.boleh(h)) && (!m.utd || sesi.modeUtd))
		})).filter((g) => g.menu.length)
	);

	const aktif = (href: string) =>
		page.url.pathname === href || (href !== '/' && page.url.pathname.startsWith(href + '/') && href !== '/permintaan' && href !== '/stok');
</script>

{#if publik}
	{@render children()}
{:else if !sesi.siap || !sesi.profil}
	<div class="keadaan" aria-busy="true">Memuat…</div>
{:else}
	<div class="min-h-screen lg:flex">
		<a href="#isi" class="sr-only focus:not-sr-only focus:absolute focus:top-2 focus:left-2 focus:z-50 btn">Lewati ke isi</a>
		<header class="tanpa-cetak flex items-center gap-3 border-b border-garis bg-kartu px-4 py-2 lg:hidden">
			<button class="btn btn--teks" aria-label="Buka menu" aria-expanded={menuTerbuka} onclick={() => (menuTerbuka = !menuTerbuka)}>
				<Menu class="size-5" aria-hidden="true" />
			</button>
			<span class="font-semibold">utd-rs</span>
		</header>
		<nav
			class="tanpa-cetak {menuTerbuka ? 'block' : 'hidden'} w-full shrink-0 border-r border-garis bg-kartu lg:sticky lg:top-0 lg:block lg:h-screen lg:w-60 lg:overflow-y-auto"
			aria-label="Menu utama"
		>
			<a href="/" class="flex items-center gap-2 border-b border-garis px-4 py-3" onclick={() => (menuTerbuka = false)}>
				<Droplet class="size-6 text-bahaya" aria-hidden="true" />
				<span>
					<span class="block font-semibold">utd-rs</span>
					<span class="block text-xs text-redup">{sesi.profil.nama_rs}</span>
				</span>
			</a>
			{#each grup as g (g.judul)}
				<div class="px-2 py-2">
					<p class="px-2 pb-1 text-xs font-semibold tracking-wide text-redup uppercase">{g.judul}</p>
					<ul>
						{#each g.menu as m (m.href)}
							<li>
								<a
									href={m.href}
									class="block rounded-md px-2 py-1.5 text-sm hover:bg-halus aria-[current=page]:bg-info-latar aria-[current=page]:font-semibold aria-[current=page]:text-utama"
									aria-current={aktif(m.href) ? 'page' : undefined}
									onclick={() => (menuTerbuka = false)}>{m.label}</a
								>
							</li>
						{/each}
					</ul>
				</div>
			{/each}
			<div class="border-t border-garis px-4 py-3 text-sm">
				<a href="/akun-saya" class="block font-medium hover:underline">{sesi.profil.nama}</a>
				<p class="text-xs text-redup">{sesi.profil.peran.join(', ')}</p>
				<button
					class="btn btn--teks mt-2 -ml-3"
					onclick={async () => {
						await sesi.keluar();
						goto('/masuk');
					}}><LogOut class="size-4" aria-hidden="true" />Keluar</button
				>
			</div>
		</nav>
		<main id="isi" class="min-w-0 flex-1 p-4 lg:p-6">
			{@render children()}
		</main>
	</div>
{/if}
