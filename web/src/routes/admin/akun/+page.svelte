<script lang="ts">
	import { minimalSandi, muatKebijakanSandi } from '#lib/kebijakanSandi.svelte.ts';
	import { merek } from '#lib/merek.svelte.ts';
	import { api } from '#lib/api.ts';
	import { Aksi } from '#lib/aksi.svelte.ts';
	import { Sumber } from '#lib/sumber.svelte.ts';
	import Keadaan from '#lib/Keadaan.svelte';
	import PesanAksi from '#lib/PesanAksi.svelte';
	import Isian from '#lib/Isian.svelte';
	import Chip from '#lib/Chip.svelte';
	import { sesi } from '#lib/sesi.svelte.ts';
	import KepalaHalaman from '#lib/KepalaHalaman.svelte';
	import KeyRound from '@lucide/svelte/icons/key-round';
	import Copy from '@lucide/svelte/icons/copy';

	type Akun = { id: string; nama_pengguna: string; nama: string; nip: string | null; peran: string[]; aktif: boolean; terkunci: boolean };

	const PERAN: [string, string][] = [
		['admin', 'Admin TI'],
		['dokter', 'Dokter'],
		['perawat', 'Perawat'],
		['teknisi', 'Teknisi BDRS/UTD'],
		['kepala', 'Penanggung jawab BDRS (kepala)'],
		['auditor', 'Auditor'],
		['simrs', 'Layanan SIMRS (token API)'],
		['logger', 'Layanan data logger suhu (token API)']
	];
	const NAMA_PERAN = Object.fromEntries(PERAN);
	const LAYANAN = ['simrs', 'logger'];
	const layanan = (peran: string[]) => peran.some((r) => LAYANAN.includes(r));

	const akun = new Sumber(() => api<Akun[]>('GET', '/akun'));
	$effect(() => {
		if (sesi.boleh('akun.kelola')) akun.muat();
	});

	// Buat akun
	let baru = $state({ nama_pengguna: '', nama: '', nip: '', peran: [] as string[], sandi_sementara: '' });
	const aksiBuat = new Aksi();
	async function buat(e: SubmitEvent) {
		e.preventDefault();
		const body = { ...baru, sandi_sementara: layanan(baru.peran) ? undefined : baru.sandi_sementara };
		const r = await aksiBuat.jalankan(() => api('POST', '/akun', body), `Akun ${baru.nama_pengguna} dibuat.`);
		if (r) {
			baru = { nama_pengguna: '', nama: '', nip: '', peran: [], sandi_sementara: '' };
			akun.muat();
		}
	}

	// Ubah akun terpilih
	let pilih = $state<Akun | null>(null);
	let ubah = $state({ nama: '', nip: '', peran: [] as string[], aktif: true });
	let sandiReset = $state('');
	let token = $state<string | null>(null);
	const aksiUbah = new Aksi();
	const aksiReset = new Aksi();
	const aksiToken = new Aksi();

	function buka(a: Akun) {
		pilih = a;
		ubah = { nama: a.nama, nip: a.nip ?? '', peran: [...a.peran], aktif: a.aktif };
		sandiReset = '';
		token = null;
		for (const x of [aksiUbah, aksiReset, aksiToken]) x.galat = x.berhasil = null;
	}

	async function simpan(e: SubmitEvent) {
		e.preventDefault();
		if (!pilih) return;
		const r = await aksiUbah.jalankan(() => api('PUT', `/akun/${pilih!.id}`, ubah), 'Perubahan akun disimpan.');
		if (r) {
			pilih = { ...pilih, ...ubah, nip: ubah.nip || null, peran: [...ubah.peran] };
			akun.muat();
		}
	}

	async function reset(e: SubmitEvent) {
		e.preventDefault();
		if (!pilih) return;
		const r = await aksiReset.jalankan(
			() => api('POST', `/akun/${pilih!.id}/reset-sandi`, { sandi_sementara: sandiReset }),
			'Sandi sementara disetel. Pengguna wajib menggantinya saat masuk; semua sesinya diakhiri.'
		);
		if (r) {
			sandiReset = '';
			akun.muat();
		}
	}

	async function terbitkan() {
		if (!pilih) return;
		const r = await aksiToken.jalankan(() => api<{ token: string }>('POST', `/akun/${pilih!.id}/token`));
		if (r) token = r.token;
	}

	async function cabut() {
		if (!pilih || !confirm(`Cabut semua token API milik ${pilih.nama_pengguna}? Sistem yang memakainya akan berhenti terhubung.`)) return;
		token = null;
		const r = await aksiToken.jalankan(() => api<{ dicabut: number }>('POST', `/akun/${pilih!.id}/token/cabut`));
		if (r) aksiToken.berhasil = r.dicabut ? `${r.dicabut} token dicabut.` : 'Tidak ada token aktif.';
	}

	let disalin = $state(false);
	async function salin() {
		if (!token) return;
		await navigator.clipboard.writeText(token);
		disalin = true;
		setTimeout(() => (disalin = false), 2000);
	}
	$effect(muatKebijakanSandi);
</script>

<svelte:head><title>Akun pengguna — {merek.judul}</title></svelte:head>

{#snippet pilihPeran(id: string, peran: string[], galat: string | undefined, ubahPeran: (p: string[]) => void)}
	<fieldset class="sm:col-span-2">
		<legend class="label">Peran</legend>
		<div class="grid gap-1 sm:grid-cols-2">
			{#each PERAN as [kode, label] (kode)}
				<label class="flex items-center gap-2 text-sm">
					<input
						type="checkbox"
						checked={peran.includes(kode)}
						onchange={(e) => ubahPeran(e.currentTarget.checked ? [...peran, kode] : peran.filter((r) => r !== kode))}
						aria-describedby={galat ? `${id}-galat` : undefined}
					/>
					{label}
				</label>
			{/each}
		</div>
		{#if galat}
			<p class="mt-1 text-sm text-bahaya" id="{id}-galat" role="alert">{galat}</p>
		{:else}
			<p class="mt-1 text-xs text-redup">Akun layanan (SIMRS, data logger) memakai token API dan tidak boleh digabung dengan peran lain.</p>
		{/if}
	</fieldset>
{/snippet}

<KepalaHalaman judul="Akun pengguna" jalur={[['Pengaturan'], ['Akun pengguna']]} sub="Akun petugas dan akun layanan. Akun tidak pernah dihapus, hanya dinonaktifkan." />

{#if !sesi.boleh('akun.kelola')}
	<p class="catatan">Anda tidak berhak mengelola akun.</p>
{:else}
	<div class="grid items-start gap-5 xl:grid-cols-[1fr_360px]">
		<section class="panel min-w-0" aria-labelledby="judul-daftar">
			<h2 id="judul-daftar">Daftar akun</h2>
			<p class="mb-3 text-sm text-redup">Pilih Kelola untuk mengubah peran, menonaktifkan, mereset sandi, atau menerbitkan token API.</p>
			<Keadaan sumber={akun}>
				{#snippet kosong()}<p>Belum ada akun.</p>{/snippet}
				{#snippet children(data)}
					<div class="overflow-x-auto">
						<table class="tabel">
							<thead>
								<tr><th>Nama pengguna</th><th>Nama</th><th>NIP</th><th>Peran</th><th>Status</th><th><span class="sr-only">Aksi</span></th></tr>
							</thead>
							<tbody>
								{#each data as a (a.id)}
									<tr class={pilih?.id === a.id ? 'bg-halus' : ''}>
										<td class="mono font-semibold">{a.nama_pengguna}</td>
										<td>{a.nama}</td>
										<td class="mono">{a.nip ?? '—'}</td>
										<td>
											<span class="flex flex-wrap gap-1">
												{#each a.peran as r (r)}<Chip polos nada={LAYANAN.includes(r) ? 'info' : 'netral'}>{NAMA_PERAN[r] ?? r}</Chip>{/each}
											</span>
										</td>
										<td>
											<span class="flex flex-wrap gap-1">
												<Chip nada={a.aktif ? 'aman' : 'netral'}>{a.aktif ? 'Aktif' : 'Nonaktif'}</Chip>
												{#if a.terkunci}<Chip nada="bahaya">Terkunci</Chip>{/if}
											</span>
										</td>
										<td><button class="btn btn--teks" onclick={() => buka(a)} aria-label="Kelola akun {a.nama_pengguna}">Kelola</button></td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/snippet}
			</Keadaan>
		</section>

		<div class="flex flex-col gap-4">
			{#if pilih}
				<section class="panel flex flex-col gap-4" aria-labelledby="judul-ubah">
					<div class="flex items-start justify-between gap-3">
						<div class="min-w-0">
							<h2 id="judul-ubah">Kelola <span class="mono">{pilih.nama_pengguna}</span></h2>
							<p class="text-sm text-redup">{pilih.nama}</p>
						</div>
						<button class="btn btn--teks" onclick={() => (pilih = null)}>Tutup</button>
					</div>
					<form class="form-grid" onsubmit={simpan}>
						<Isian label="Nama lengkap" id="u-nama" galat={aksiUbah.isian.nama}>
							<input id="u-nama" class="input" bind:value={ubah.nama} required />
						</Isian>
						<Isian label="NIP (opsional)" id="u-nip" galat={aksiUbah.isian.nip}>
							<input id="u-nip" class="input mono" bind:value={ubah.nip} />
						</Isian>
						{@render pilihPeran('u-peran', ubah.peran, aksiUbah.isian.peran, (p) => (ubah.peran = p))}
						<label class="flex items-center gap-2 text-sm sm:col-span-2">
							<input type="checkbox" bind:checked={ubah.aktif} />
							Akun aktif (menonaktifkan mengakhiri semua sesinya; akun tidak pernah dihapus)
						</label>
						<div class="flex flex-col gap-2 sm:col-span-2">
							<PesanAksi aksi={aksiUbah} />
							<div class="flex justify-end gap-2.5 border-t border-garis pt-4">
								<button class="btn btn--utama" disabled={aksiUbah.sibuk}>Simpan perubahan</button>
							</div>
						</div>
					</form>

					{#if layanan(pilih.peran)}
						<div class="flex flex-col gap-3 border-t border-garis pt-4">
							<h3 class="flex items-center gap-2 font-semibold"><KeyRound class="size-4" aria-hidden="true" />Token API</h3>
							<p class="text-sm text-redup">Akun layanan masuk dengan header <span class="mono">Authorization: Bearer</span>. Token baru tidak mencabut token lama.</p>
							{#if token}
								<div class="peringatan flex-col" role="alert">
									<p>Salin token ini sekarang. Token hanya ditampilkan sekali dan tidak bisa dilihat lagi.</p>
									<code class="mono block w-full rounded border border-garis bg-kartu p-2 text-sm break-all text-teks">{token}</code>
									<div class="flex flex-wrap gap-2">
										<button class="btn" onclick={salin}><Copy class="size-4" aria-hidden="true" />{disalin ? 'Tersalin' : 'Salin token'}</button>
										<button class="btn" onclick={() => (token = null)}>Saya sudah menyalinnya</button>
									</div>
								</div>
							{/if}
							<PesanAksi aksi={aksiToken} />
							<div class="flex flex-wrap justify-end gap-2.5">
								<button class="btn btn--utama" onclick={terbitkan} disabled={aksiToken.sibuk || !pilih.aktif}>Terbitkan token</button>
								<button class="btn btn--bahaya" onclick={cabut} disabled={aksiToken.sibuk}>Cabut semua token</button>
							</div>
						</div>
					{:else}
						<form class="flex flex-col gap-3 border-t border-garis pt-4" onsubmit={reset}>
							<h3 class="font-semibold">Reset kata sandi</h3>
							<Isian label="Sandi sementara baru" id="u-sandi" galat={aksiReset.isian.sandi_sementara} bantuan="{minimalSandi()}. Sampaikan langsung ke pengguna; ia wajib menggantinya saat masuk. Kunci akun ikut dibuka.">
								<input id="u-sandi" class="input" type="password" autocomplete="new-password" bind:value={sandiReset} required />
							</Isian>
							<PesanAksi aksi={aksiReset} />
							<div class="flex justify-end"><button class="btn" disabled={aksiReset.sibuk}>Reset sandi</button></div>
						</form>
					{/if}
				</section>
			{/if}

			<form class="panel form-grid" onsubmit={buat} aria-labelledby="judul-buat">
				<div class="sm:col-span-2">
					<h2 id="judul-buat">Buat akun</h2>
					<p class="text-sm text-redup">Petugas wajib mengganti sandi sementara saat pertama masuk.</p>
				</div>
				<Isian label="Nama pengguna" id="b-nama-pengguna" galat={aksiBuat.isian.nama_pengguna} bantuan="3–40 huruf kecil, angka, titik, garis bawah, atau tanda hubung.">
					<input id="b-nama-pengguna" class="input mono" autocomplete="off" bind:value={baru.nama_pengguna} required />
				</Isian>
				<Isian label="Nama lengkap" id="b-nama" galat={aksiBuat.isian.nama}>
					<input id="b-nama" class="input" bind:value={baru.nama} required />
				</Isian>
				<Isian label="NIP (opsional)" id="b-nip" galat={aksiBuat.isian.nip}>
					<input id="b-nip" class="input mono" bind:value={baru.nip} />
				</Isian>
				{#if !layanan(baru.peran)}
					<Isian label="Sandi sementara" id="b-sandi" galat={aksiBuat.isian.sandi_sementara} bantuan="{minimalSandi()}; wajib diganti saat pertama masuk.">
						<input id="b-sandi" class="input" type="password" autocomplete="new-password" bind:value={baru.sandi_sementara} required />
					</Isian>
				{/if}
				{@render pilihPeran('b-peran', baru.peran, aksiBuat.isian.peran, (p) => (baru.peran = p))}
				{#if layanan(baru.peran)}
					<p class="catatan sm:col-span-2">Akun layanan tidak memakai sandi. Setelah dibuat, pilih Kelola lalu terbitkan token API.</p>
				{/if}
				<div class="flex flex-col gap-2 sm:col-span-2">
					<PesanAksi aksi={aksiBuat} />
					<div class="flex justify-end gap-2.5 border-t border-garis pt-4">
						<button class="btn btn--utama" disabled={aksiBuat.sibuk}>Buat akun</button>
					</div>
				</div>
			</form>
		</div>
	</div>
{/if}
