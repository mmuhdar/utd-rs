// Menu per kelompok; setiap entri tampil bila pengguna punya salah satu hak `hak` (dan mode UTD-RS bila `utd`).
export type Menu = { href: string; label: string; hak: string[]; utd?: boolean };
export type Grup = { judul: string; menu: Menu[] };

export const GRUP: Grup[] = [
	{
		judul: 'Pelayanan',
		menu: [
			{ href: '/permintaan', label: 'Permintaan darah', hak: ['permintaan.lihat'] },
			{ href: '/permintaan/baru', label: 'Buat permintaan', hak: ['permintaan.buat'] },
			{ href: '/transfusi', label: 'Transfusi di ruang', hak: ['transfusi.catat'] },
			{ href: '/reaksi', label: 'Reaksi & insiden', hak: ['reaksi.telusur', 'laporan.lihat'] }
		]
	},
	{
		judul: 'Bank darah',
		menu: [
			{ href: '/stok', label: 'Stok kantong', hak: ['stok.lihat'] },
			{ href: '/stok/terima', label: 'Penerimaan dari UTD', hak: ['stok.kelola'] },
			{ href: '/permintaan-utd', label: 'Permintaan ke UTD', hak: ['stok.kelola'] },
			{ href: '/berita-acara', label: 'Pengembalian & pemusnahan', hak: ['stok.lihat'] },
			{ href: '/suhu', label: 'Suhu alat simpan', hak: ['stok.lihat', 'suhu.catat'] },
			{ href: '/lacak', label: 'Lacak kantong', hak: ['stok.lihat'] },
			{ href: '/lookback', label: 'Look-back', hak: ['stok.lihat'] }
		]
	},
	{
		judul: 'UTD',
		menu: [
			{ href: '/donor', label: 'Donor', hak: ['donor.kelola'], utd: true },
			{ href: '/donasi', label: 'Donasi & pelulusan', hak: ['donor.kelola'], utd: true }
		]
	},
	{
		judul: 'Laporan',
		menu: [
			{ href: '/laporan', label: 'Laporan & mutu', hak: ['laporan.lihat'] },
			{ href: '/darurat', label: 'Lembar darurat (cetak)', hak: ['stok.lihat'] }
		]
	},
	{
		judul: 'Pengaturan',
		menu: [
			{ href: '/admin/akun', label: 'Akun pengguna', hak: ['akun.kelola'] },
			{ href: '/admin/master', label: 'Data induk & aturan', hak: ['master.kelola', 'aturan.kelola'] },
			{ href: '/admin/audit', label: 'Jejak audit', hak: ['audit.lihat'] },
			{ href: '/admin/integrasi', label: 'Integrasi SATUSEHAT', hak: ['integrasi.kelola'] }
		]
	}
];
