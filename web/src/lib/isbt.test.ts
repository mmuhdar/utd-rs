import { test } from 'node:test';
import assert from 'node:assert/strict';
import { urai } from './isbt.ts';

test('nomor donasi ISBT 128', () => {
	assert.deepEqual(urai('=G12342620001200'), { jenis: 'din', nomor_donasi: 'G123426200012', nomor: 'G123426200012' });
	assert.equal(urai('=G123426200012').jenis, 'din');
});

test('golongan, produk, kedaluwarsa', () => {
	assert.deepEqual(urai('=%5100'), { jenis: 'golongan', abo: 'O', rhesus: 'pos' });
	assert.deepEqual(urai('=%2800'), { jenis: 'golongan', abo: 'AB', rhesus: 'neg' });
	assert.deepEqual(urai('=<E0336V00'), { jenis: 'produk', kode_produk: 'E0336V00' });
	const k = urai('=>026032');
	assert.equal(k.jenis, 'kedaluwarsa');
	if (k.jenis === 'kedaluwarsa') assert.equal(k.waktu.toDateString(), new Date(2026, 1, 1).toDateString());
	const j = urai('&>0260321430');
	if (j.jenis === 'kedaluwarsa') assert.equal(j.waktu.getHours(), 14);
});

test('format lokal tidak dikenal tetap dikembalikan apa adanya', () => {
	assert.deepEqual(urai(' PMI-2026-0001 '), { jenis: 'lain', teks: 'PMI-2026-0001' });
});
