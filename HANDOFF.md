# Handoff — utd-rs

Tanggal: 2026-11-18

## Tujuan proyek
Aplikasi web **UTD-RS** (Unit Transfusi Darah Rumah Sakit / Bank Darah RS) yang memenuhi standar & regulasi pemerintah Indonesia.

- Backend: **Rust**
- Frontend: **Svelte**, jalan di browser
- Instalasi di server RS: **Docker**
- Skill riset: skill Matt Pocock (`research`, lalu `grill-with-docs` / `domain-modeling` / `to-spec` / `to-tickets`)
- Skill UI: **`ui-ux-pro-max`** (dipakai saat desain frontend)

## Sudah selesai
- Repo dibuat: `/home/mat/AgenticProjects/utd-rs`, `git init`, branch `main`, **belum ada commit**.

## Terhenti di
- Riset regulasi lewat skill `research` sempat berjalan (agen latar belakang) lalu **dihentikan sebelum menulis file apa pun**.
  Agen sedang membaca CPOB UTD BPOM (Aneks 2: sistem komputerisasi, audit trail) saat dihentikan. Tidak ada hasil tersimpan — riset harus diulang.

## Langkah berikutnya
1. Jalankan ulang `/research` dengan brief di bawah. Output: `docs/research/regulasi-utd-rs.md`.
2. Uji kebutuhan hasil riset dengan `/grill-with-docs` (jawab "Pertanyaan terbuka").
3. `/domain-modeling` → glosarium istilah (kantong darah, crossmatch, IMLTD, hemovigilans, dll.).
4. `/to-spec` lalu `/to-tickets`.
5. Baru setelah itu: scaffold Rust + Svelte + Docker, desain UI dengan `/ui-ux-pro-max`.

## Brief riset (pakai apa adanya)
Tulis dalam Bahasa Indonesia ke `docs/research/regulasi-utd-rs.md`. Hanya sumber primer (peraturan.bpk.go.id, jdih.kemkes.go.id, peraturan.go.id, satusehat.kemkes.go.id, BPOM, Komdigi/BSSN, WHO). Setiap klaim diberi URL + pasal. Tandai sumber yang tidak bisa diverifikasi, dan regulasi yang dicabut/diganti (UU 36/2009 → UU 17/2023, PP 28/2024; cek status PP 7/2011, Permenkes 83/2014, Permenkes 91/2015, Peraturan BPOM CPOB UTD).

Topik:
1. Pelayanan darah: dasar hukum, peran UTD vs BDRS, permintaan darah, crossmatch, ABO/Rh, skrining IMLTD (HIV, HBV, HCV, sifilis), penyimpanan & suhu, kedaluwarsa per komponen (WB, PRC, TC, FFP, cryo), distribusi, pemusnahan, ketertelusuran donor→kantong→pasien (hemovigilans), pelaporan reaksi transfusi, pelaporan wajib ke Kemenkes, retensi dokumen.
2. CPOB UTD (BPOM): dokumentasi, integritas data, validasi sistem komputer, audit trail, format label (ISBT 128 atau format nasional?).
3. RME — Permenkes 24/2022: kewajiban, retensi, keamanan, interoperabilitas, tenggat integrasi SATUSEHAT.
4. SATUSEHAT: resource FHIR yang relevan untuk transfusi, OAuth2 client-credentials, sandbox/production.
5. UU 27/2022 PDP: data kesehatan sebagai data spesifik, persetujuan, DPO, notifikasi kebocoran 3×24 jam.
6. Keamanan sistem elektronik: PP 71/2019, Permenkominfo 5/2020 (registrasi PSE), regulasi BSSN.
7. Integrasi SIMRS (Permenkes 82/2013) dan bridging BPJS bila relevan.

Akhiri dengan:
- **Requirement untuk utd-rs**: FR-xx (fungsional) dan NFR-xx (keamanan, audit trail, backup, retensi, ketersediaan, aksesibilitas, deploy Docker on-prem), masing-masing dengan sumbernya.
- **Pertanyaan terbuka**: mis. lingkup UTD-RS vs BDRS saja, apakah RS juga menerima donor.
