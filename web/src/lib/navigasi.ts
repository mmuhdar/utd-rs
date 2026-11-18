import type { Component } from 'svelte';
import ArchiveX from '@lucide/svelte/icons/archive-x';
import Bed from '@lucide/svelte/icons/bed';
import ChartColumn from '@lucide/svelte/icons/chart-column';
import ClipboardList from '@lucide/svelte/icons/clipboard-list';
import Database from '@lucide/svelte/icons/database';
import FilePlus from '@lucide/svelte/icons/file-plus';
import HandHeart from '@lucide/svelte/icons/hand-heart';
import HeartPulse from '@lucide/svelte/icons/heart-pulse';
import History from '@lucide/svelte/icons/history';
import LayoutDashboard from '@lucide/svelte/icons/layout-dashboard';
import Package from '@lucide/svelte/icons/package';
import Plug from '@lucide/svelte/icons/plug';
import Printer from '@lucide/svelte/icons/printer';
import ScanSearch from '@lucide/svelte/icons/scan-search';
import Send from '@lucide/svelte/icons/send';
import ShieldCheck from '@lucide/svelte/icons/shield-check';
import TestTubes from '@lucide/svelte/icons/test-tubes';
import Thermometer from '@lucide/svelte/icons/thermometer';
import Truck from '@lucide/svelte/icons/truck';
import Users from '@lucide/svelte/icons/users';

// Menu per kelompok; setiap entri tampil bila pengguna punya salah satu hak `hak` (kosong: semua pengguna), dan mode UTD-RS bila `utd`.
export type Menu = { href: string; label: string; hak: string[]; ikon: Component; utd?: boolean };
export type Grup = { judul: string; menu: Menu[] };

export const GRUP: Grup[] = [
	{ judul: '', menu: [{ href: '/', label: 'Beranda', hak: [], ikon: LayoutDashboard }] },
	{
		judul: 'Pelayanan',
		menu: [
			{ href: '/permintaan', ikon: ClipboardList, label: 'Permintaan darah', hak: ['permintaan.lihat'] },
			{ href: '/permintaan/baru', ikon: FilePlus, label: 'Buat permintaan', hak: ['permintaan.buat'] },
			{ href: '/transfusi', ikon: Bed, label: 'Transfusi di ruang', hak: ['transfusi.catat'] },
			{ href: '/reaksi', ikon: HeartPulse, label: 'Reaksi & insiden', hak: ['reaksi.telusur', 'laporan.lihat'] }
		]
	},
	{
		judul: 'Bank darah',
		menu: [
			{ href: '/stok', ikon: Package, label: 'Stok kantong', hak: ['stok.lihat'] },
			{ href: '/stok/terima', ikon: Truck, label: 'Penerimaan dari UTD', hak: ['stok.kelola'] },
			{ href: '/permintaan-utd', ikon: Send, label: 'Permintaan ke UTD', hak: ['stok.kelola'] },
			{ href: '/berita-acara', ikon: ArchiveX, label: 'Pengembalian & pemusnahan', hak: ['stok.lihat'] },
			{ href: '/suhu', ikon: Thermometer, label: 'Suhu alat simpan', hak: ['stok.lihat', 'suhu.catat'] },
			{ href: '/lacak', ikon: ScanSearch, label: 'Lacak kantong', hak: ['stok.lihat'] },
			{ href: '/lookback', ikon: History, label: 'Look-back', hak: ['stok.lihat'] }
		]
	},
	{
		judul: 'UTD',
		menu: [
			{ href: '/donor', ikon: HandHeart, label: 'Donor', hak: ['donor.kelola'], utd: true },
			{ href: '/donasi', ikon: TestTubes, label: 'Donasi & pelulusan', hak: ['donor.kelola'], utd: true }
		]
	},
	{
		judul: 'Laporan',
		menu: [
			{ href: '/laporan', ikon: ChartColumn, label: 'Laporan & mutu', hak: ['laporan.lihat'] },
			{ href: '/darurat', ikon: Printer, label: 'Lembar darurat (cetak)', hak: ['stok.lihat'] }
		]
	},
	{
		judul: 'Pengaturan',
		menu: [
			{ href: '/admin/akun', ikon: Users, label: 'Akun pengguna', hak: ['akun.kelola'] },
			{ href: '/admin/master', ikon: Database, label: 'Data induk & aturan', hak: ['master.kelola', 'aturan.kelola'] },
			{ href: '/admin/audit', ikon: ShieldCheck, label: 'Jejak audit', hak: ['audit.lihat'] },
			{ href: '/admin/integrasi', ikon: Plug, label: 'Integrasi SATUSEHAT', hak: ['integrasi.kelola'] }
		]
	}
];
