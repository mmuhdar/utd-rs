# Query sqlx dicek saat runtime dan diuji, bukan dicek saat kompilasi

Berbeda dari rustSIMRS, utd-rs memakai `sqlx::query`/`query_as` biasa, tidak memakai makro `query!` yang memeriksa skema saat kompilasi. Build di Docker dan di mesin dev tanpa compiler C (cargo dijalankan di container, lihat `scripts/cargo.sh`) jadi tidak butuh database atau direktori `.sqlx` yang harus selalu disinkronkan. Kebenaran query dijaga oleh uji `#[sqlx::test]` ujung ke ujung untuk setiap alur.
