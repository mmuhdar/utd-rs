// Tema dipasang sebelum halaman digambar supaya tidak berkedip. Pilihan di localStorage 'utd-tema':
// 'terang' | 'gelap'; tidak ada = ikuti sistem. Logika yang sama ada di src/lib/tema.svelte.ts.
(function () {
	var p = null;
	try {
		p = localStorage.getItem('utd-tema');
	} catch (e) {}
	var gelap = p === 'gelap' || (p !== 'terang' && window.matchMedia('(prefers-color-scheme: dark)').matches);
	document.documentElement.dataset.tema = gelap ? 'gelap' : 'terang';
})();
