import { test } from 'node:test';
import assert from 'node:assert/strict';
import { kesimpulan, kompatibel, tafsirAbo } from './darah.ts';

// Sama dengan uji di src/pratransfusi.rs supaya cermin di browser tidak menyimpang dari server.
test('kompatibilitas ABO/Rh', () => {
	for (const p of ['A', 'B', 'AB', 'O']) assert.ok(kompatibel('prc', p, 'pos', 'O', 'pos'));
	assert.ok(!kompatibel('prc', 'A', 'pos', 'AB', 'pos'));
	assert.ok(kompatibel('prc', 'AB', 'pos', 'A', 'pos'));
	assert.ok(!kompatibel('wb', 'A', 'pos', 'O', 'pos'));
	assert.ok(kompatibel('ffp', 'A', 'neg', 'AB', 'pos'));
	assert.ok(!kompatibel('ffp', 'A', 'pos', 'O', 'pos'));
	assert.ok(!kompatibel('prc', 'O', 'neg', 'O', 'pos'));
	assert.ok(!kompatibel('tc', 'A', 'neg', 'A', 'pos'));
});

test('tafsir cell dan serum grouping', () => {
	assert.deepEqual(tafsirAbo('pos', 'neg', 'neg', 'pos'), ['A', true]);
	assert.deepEqual(tafsirAbo('neg', 'neg', 'pos', 'pos'), ['O', true]);
	assert.deepEqual(tafsirAbo('pos', 'neg', 'pos', 'pos'), ['A', false]);
});

test('kesimpulan uji silang', () => {
	assert.equal(kesimpulan('prc', 'neg', 'neg', 'neg'), 'serasi');
	assert.equal(kesimpulan('prc', 'neg', 'pos', 'neg'), 'hanya_prc');
	assert.equal(kesimpulan('wb', 'neg', 'neg', 'pos'), 'tidak_serasi');
	assert.equal(kesimpulan('ffp', undefined, 'neg', undefined), 'serasi');
});
