-- Koreksi data klinis (FR-34; Permenkes 24/2022 Pasal 30): nilai lama ada di jejak audit, catatan ditandai dikoreksi
-- supaya cetakan menunjukkan bahwa data diubah sejak entri asli (CPOB Aneks 2 butir 30).
alter table observasi add column dikoreksi_pada timestamptz;
alter table transfusi add column dikoreksi_pada timestamptz;
-- Pengeluaran yang terlambat dimulai (> 30 menit) wajib beralasan.
alter table transfusi add column alasan_terlambat text;
