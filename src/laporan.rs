//! Laporan (SPEK M6): Formulir 3 bulanan BDRS (Permenkes 83/2014 Pasal 51; Permenkes 91/2015 §7.2.2), indikator
//! mutu (rasio C:T, §5.11), perencanaan kebutuhan (Pasal 43), ketersediaan darah (PP 28/2024 Pasal 299), laporan UTD.
//! Periode bulan dihitung dalam zona waktu Asia/Jakarta.

use crate::AppState;
use crate::auth::Pengguna;
use crate::error::{Hasil, tolak};
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::Value;

pub fn rute() -> Router<AppState> {
    Router::new()
        .route("/laporan/formulir3", get(formulir3))
        .route("/laporan/mutu", get(mutu))
        .route("/laporan/perencanaan", get(perencanaan))
        .route("/laporan/ketersediaan", get(ketersediaan))
        .route("/laporan/utd", get(utd))
}

#[derive(Deserialize)]
struct Periode {
    bulan: Option<String>,
}

/// "YYYY-MM" yang valid; bawaan bulan berjalan.
fn bulan(p: &Periode) -> Hasil<String> {
    match p.bulan.as_deref() {
        None => {
            Ok(time::OffsetDateTime::now_utc().to_offset(time::macros::offset!(+7)).date().to_string()[..7].to_string())
        }
        Some(b)
            if b.len() == 7
                && b.as_bytes()[4] == b'-'
                && b[..4].parse::<u16>().is_ok()
                && b[5..].parse::<u8>().is_ok_and(|m| (1..=12).contains(&m)) =>
        {
            Ok(b.to_string())
        }
        Some(_) => Err(tolak("bulan", "Format bulan YYYY-MM.")),
    }
}

// Batas periode sebagai CTE yang dipakai semua bagian laporan.
const PERIODE: &str = "with periode as (select (($1 || '-01')::date)::timestamp at time zone 'Asia/Jakarta' as dari,
                                               ((($1 || '-01')::date) + interval '1 month')::timestamp at time zone 'Asia/Jakarta' as sampai)";

/// GET /api/laporan/formulir3?bulan=YYYY-MM
async fn formulir3(State(s): State<AppState>, p: Pengguna, Query(q): Query<Periode>) -> Hasil<Json<Value>> {
    p.wajib("laporan.lihat")?;
    let b = bulan(&q)?;
    let sql = format!(
        "{PERIODE}
         select jsonb_build_object(
           'bulan', $1::text,
           'profil', (select to_jsonb(x) from profil_instalasi x),
           'a_permintaan_ke_utd', (select coalesce(jsonb_agg(jsonb_build_object('nomor', r.nomor, 'tanggal', r.dibuat_pada, 'utd', u.nama,
                                       'jenis', r.jenis, 'baris', r.baris, 'status', r.status) order by r.dibuat_pada), '[]')
                                   from permintaan_utd r join utd_mitra u on u.id = r.utd_id, periode
                                   where r.dibuat_pada >= periode.dari and r.dibuat_pada < periode.sampai),
           'b_penerimaan_dari_utd', (select coalesce(jsonb_agg(x order by x->>'komponen', x->>'abo', x->>'rhesus'), '[]') from (
                                       select jsonb_build_object('komponen', k.komponen, 'abo', k.abo, 'rhesus', k.rhesus,
                                         'diterima', count(*) filter (where coalesce(b.catatan, '') not like 'Ditolak saat penerimaan%'),
                                         'ditolak', count(*) filter (where b.catatan like 'Ditolak saat penerimaan%')) x
                                       from kantong k join penerimaan t on t.id = k.penerimaan_id left join berita_acara b on b.id = k.berita_acara_id,
                                       periode where t.diterima_pada >= periode.dari and t.diterima_pada < periode.sampai
                                       group by k.komponen, k.abo, k.rhesus) y),
           'c_pemakaian_per_ruang', (select coalesce(jsonb_agg(x order by x->>'ruang'), '[]') from (
                                       select jsonb_build_object('ruang', r.ruang,
                                         'permintaan', count(distinct r.id),
                                         'kantong_diminta', (select coalesce(sum((bb->>'jumlah')::int), 0) from permintaan r2, jsonb_array_elements(r2.baris) bb, periode p2
                                                             where r2.ruang = r.ruang and r2.diminta_pada >= p2.dari and r2.diminta_pada < p2.sampai),
                                         'ditransfusikan', jsonb_build_object(
                                           'wb', count(t.id) filter (where k.komponen = 'wb'), 'prc', count(t.id) filter (where k.komponen = 'prc'),
                                           'tc', count(t.id) filter (where k.komponen = 'tc'), 'ffp', count(t.id) filter (where k.komponen = 'ffp'),
                                           'cryo', count(t.id) filter (where k.komponen = 'cryo'))) x
                                       from permintaan r left join alokasi a on a.permintaan_id = r.id
                                       left join transfusi t on t.kantong_id = a.kantong_id left join kantong k on k.id = t.kantong_id, periode
                                       where r.diminta_pada >= periode.dari and r.diminta_pada < periode.sampai
                                       group by r.ruang) y),
           'd_reaksi_transfusi', (select coalesce(jsonb_agg(jsonb_build_object('waktu', x.waktu_reaksi, 'nama', ps.nama,
                                       'usia', extract(year from age(x.waktu_reaksi, ps.tanggal_lahir::timestamptz))::int, 'jenis_kelamin', ps.jenis_kelamin,
                                       'diagnosis', r.diagnosis, 'riwayat_transfusi', r.riwayat_transfusi, 'komponen', k.komponen,
                                       'nomor_kantong', k.nomor, 'gejala', x.gejala, 'kategori', x.kategori, 'status', x.status,
                                       'kesimpulan', x.kesimpulan) order by x.waktu_reaksi), '[]')
                                   from reaksi x join transfusi t on t.id = x.transfusi_id join kantong k on k.id = t.kantong_id
                                   join pasien ps on ps.no_rm = t.no_rm join pengeluaran g on g.id = t.pengeluaran_id
                                   join alokasi a on a.id = g.alokasi_id join permintaan r on r.id = a.permintaan_id, periode
                                   where x.waktu_reaksi >= periode.dari and x.waktu_reaksi < periode.sampai),
           'e_kembali_ke_utd', (select coalesce(jsonb_agg(jsonb_build_object('berita_acara', b.nomor, 'tanggal', b.dibuat_pada, 'utd', u.nama,
                                       'nomor_kantong', k.nomor, 'komponen', k.komponen, 'abo', k.abo, 'rhesus', k.rhesus, 'alasan', k.alasan_status)
                                       order by b.dibuat_pada, k.nomor), '[]')
                                 from berita_acara b join utd_mitra u on u.id = b.utd_id join kantong k on k.berita_acara_id = b.id, periode
                                 where b.jenis = 'pengembalian' and b.dibuat_pada >= periode.dari and b.dibuat_pada < periode.sampai),
           'pemusnahan', (select coalesce(jsonb_agg(jsonb_build_object('berita_acara', b.nomor, 'tanggal', b.dibuat_pada, 'nomor_kantong', k.nomor,
                                 'komponen', k.komponen, 'alasan', k.alasan_status) order by b.dibuat_pada), '[]')
                          from berita_acara b join kantong k on k.berita_acara_id = b.id, periode
                          where b.jenis = 'pemusnahan' and b.dibuat_pada >= periode.dari and b.dibuat_pada < periode.sampai),
           'persediaan_saat_ini', (select coalesce(jsonb_agg(x order by x->>'komponen', x->>'abo', x->>'rhesus'), '[]') from (
                                     select jsonb_build_object('komponen', komponen, 'abo', abo, 'rhesus', rhesus, 'jumlah', count(*)) x
                                     from kantong where status in ('tersedia', 'dialokasikan') and kedaluwarsa > now()
                                     group by komponen, abo, rhesus) y))"
    );
    let v: Value = sqlx::query_scalar(sqlx::AssertSqlSafe(sql)).bind(&b).fetch_one(&s.db).await?;
    Ok(Json(v))
}

/// GET /api/laporan/mutu?bulan=YYYY-MM — rasio C:T (> 2,5 = uji silang berlebihan), kedaluwarsa, permintaan tidak
/// terpenuhi, waktu tunggu permintaan darurat.
async fn mutu(State(s): State<AppState>, p: Pengguna, Query(q): Query<Periode>) -> Hasil<Json<Value>> {
    p.wajib("laporan.lihat")?;
    let b = bulan(&q)?;
    let sql = format!(
        "{PERIODE}, uji as (select k.komponen, count(distinct u.kantong_id) n from uji_silang u join kantong k on k.id = u.kantong_id, periode
                           where u.batal_alasan is null and u.dicatat_pada >= periode.dari and u.dicatat_pada < periode.sampai group by k.komponen),
                    tr as (select k.komponen, count(*) n from transfusi t join kantong k on k.id = t.kantong_id, periode
                           where t.mulai >= periode.dari and t.mulai < periode.sampai group by k.komponen)
         select jsonb_build_object(
           'bulan', $1::text,
           'rasio_ct', (select coalesce(jsonb_agg(jsonb_build_object('komponen', coalesce(uji.komponen, tr.komponen), 'uji_silang', coalesce(uji.n, 0),
                            'transfusi', coalesce(tr.n, 0),
                            'rasio', case when coalesce(tr.n, 0) > 0 then round(coalesce(uji.n, 0)::numeric / tr.n, 2) end)), '[]')
                        from uji full join tr on tr.komponen = uji.komponen),
           'batas_rasio_ct', 2.5,
           'kedaluwarsa', (select count(*) from kantong k join berita_acara b on b.id = k.berita_acara_id, periode
                           where k.alasan_status = 'kedaluwarsa' and b.dibuat_pada >= periode.dari and b.dibuat_pada < periode.sampai),
           'kedaluwarsa_di_stok', (select count(*) from kantong where status in ('tersedia', 'karantina', 'dialokasikan') and kedaluwarsa <= now()),
           'dimusnahkan', (select count(*) from kantong k join berita_acara b on b.id = k.berita_acara_id, periode
                           where b.jenis = 'pemusnahan' and b.dibuat_pada >= periode.dari and b.dibuat_pada < periode.sampai),
           'permintaan_tidak_terpenuhi', (select count(*) from permintaan r, periode
                           where r.diminta_pada >= periode.dari and r.diminta_pada < periode.sampai and r.status in ('selesai', 'batal')
                             and (select coalesce(sum((bb->>'jumlah')::int), 0) from jsonb_array_elements(r.baris) bb)
                               > (select count(*) from pengeluaran g join alokasi a on a.id = g.alokasi_id where a.permintaan_id = r.id)),
           'menit_tunggu_darurat', (select round(avg(extract(epoch from g.pertama - r.diminta_pada) / 60)::numeric, 1)
                           from permintaan r join lateral (select min(g.dikeluarkan_pada) pertama from pengeluaran g join alokasi a on a.id = g.alokasi_id
                                                           where a.permintaan_id = r.id) g on g.pertama is not null, periode
                           where r.jenis = 'darurat' and r.diminta_pada >= periode.dari and r.diminta_pada < periode.sampai),
           'insiden', (select count(*) from insiden i, periode where i.waktu >= periode.dari and i.waktu < periode.sampai),
           'reaksi', (select count(*) from reaksi x, periode where x.waktu_reaksi >= periode.dari and x.waktu_reaksi < periode.sampai))"
    );
    let v: Value = sqlx::query_scalar(sqlx::AssertSqlSafe(sql)).bind(&b).fetch_one(&s.db).await?;
    Ok(Json(v))
}

/// GET /api/laporan/perencanaan — rerata pemakaian mingguan 26 minggu terakhir per komponen × golongan
/// (Permenkes 91/2015 §5.3); stok minimal = 1 minggu, ideal = 2 minggu, dibandingkan dengan stok tersedia.
async fn perencanaan(State(s): State<AppState>, p: Pengguna) -> Hasil<Json<Value>> {
    p.wajib("laporan.lihat")?;
    let v: Value = sqlx::query_scalar(
        "with pakai as (select k.komponen, k.abo, k.rhesus, count(*)::numeric / 26 rerata from transfusi t join kantong k on k.id = t.kantong_id
                        where t.mulai >= now() - interval '26 weeks' group by 1, 2, 3),
              stok as (select komponen, abo, rhesus, count(*) n from kantong where status = 'tersedia' and kedaluwarsa > now() group by 1, 2, 3)
         select coalesce(jsonb_agg(jsonb_build_object('komponen', coalesce(pakai.komponen, stok.komponen), 'abo', coalesce(pakai.abo, stok.abo),
                    'rhesus', coalesce(pakai.rhesus, stok.rhesus), 'rerata_mingguan', round(coalesce(pakai.rerata, 0), 1),
                    'stok_minimal', ceil(coalesce(pakai.rerata, 0)), 'stok_ideal', ceil(2 * coalesce(pakai.rerata, 0)),
                    'tersedia', coalesce(stok.n, 0), 'kurang', greatest(0, ceil(coalesce(pakai.rerata, 0)) - coalesce(stok.n, 0)))
                order by coalesce(pakai.komponen, stok.komponen), coalesce(pakai.abo, stok.abo), coalesce(pakai.rhesus, stok.rhesus)), '[]')
         from pakai full join stok on stok.komponen = pakai.komponen and stok.abo = pakai.abo and stok.rhesus = pakai.rhesus",
    )
    .fetch_one(&s.db)
    .await?;
    Ok(Json(v))
}

/// GET /api/laporan/ketersediaan — ketersediaan darah untuk kanal nasional (PP 28/2024 Pasal 299; SPEK §keputusan 5).
async fn ketersediaan(State(s): State<AppState>, p: Pengguna) -> Hasil<Json<Value>> {
    p.wajib("laporan.lihat")?;
    let v: Value = sqlx::query_scalar(
        "select jsonb_build_object('waktu', now(), 'kode_faskes', (select kode_faskes from profil_instalasi),
                 'nama_rs', (select nama_rs from profil_instalasi),
                 'stok', (select coalesce(jsonb_agg(jsonb_build_object('komponen', komponen, 'abo', abo, 'rhesus', rhesus, 'jumlah', n)
                                  order by komponen, abo, rhesus), '[]')
                          from (select komponen, abo, rhesus, count(*) n from kantong where status = 'tersedia' and kedaluwarsa > now()
                                group by 1, 2, 3) x))",
    )
    .fetch_one(&s.db)
    .await?;
    Ok(Json(v))
}

/// GET /api/laporan/utd?bulan=YYYY-MM — ringkasan kegiatan UTD (mode UTD-RS; Permenkes 83/2014 Pasal 35).
async fn utd(State(s): State<AppState>, p: Pengguna, Query(q): Query<Periode>) -> Hasil<Json<Value>> {
    p.wajib("laporan.lihat")?;
    let b = bulan(&q)?;
    let sql = format!(
        "{PERIODE}
         select jsonb_build_object('bulan', $1::text,
           'seleksi', (select jsonb_build_object('total', count(*), 'diterima', count(*) filter (where keputusan = 'diterima'),
                           'ditolak_sementara', count(*) filter (where keputusan = 'ditolak_sementara'),
                           'ditolak_permanen', count(*) filter (where keputusan = 'ditolak_permanen'))
                       from seleksi, periode where waktu >= periode.dari and waktu < periode.sampai),
           'donasi', (select jsonb_build_object('total', count(*), 'sukarela', count(*) filter (where jenis = 'sukarela'),
                          'pengganti', count(*) filter (where jenis = 'pengganti'), 'gagal', count(*) filter (where status = 'gagal'))
                      from donasi, periode where waktu >= periode.dari and waktu < periode.sampai),
           'imltd_reaktif', (select coalesce(jsonb_object_agg(parameter, n), '{{}}') from (
                                select u.parameter, count(*) n from uji_imltd u join donasi d on d.id = u.donasi_id, periode
                                where u.hasil = 'reaktif' and u.tanda_tangan_id is not null and d.waktu >= periode.dari and d.waktu < periode.sampai
                                group by u.parameter) x),
           'komponen_diolah', (select coalesce(jsonb_object_agg(komponen, n), '{{}}') from (
                                  select k.komponen, count(*) n from kantong k join donasi d on d.id = k.donasi_id, periode
                                  where d.waktu >= periode.dari and d.waktu < periode.sampai group by k.komponen) x))"
    );
    let v: Value = sqlx::query_scalar(sqlx::AssertSqlSafe(sql)).bind(&b).fetch_one(&s.db).await?;
    Ok(Json(v))
}
