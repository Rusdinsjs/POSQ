# Prompt Induk Antigravity untuk Menuntaskan POSQ

**Keputusan arsitektur yang telah disetujui:** SQLite pada perangkat lokal dan PostgreSQL pada server.

Dokumen ini berfungsi sebagai instruksi eksekusi, kontrol mutu, dan mekanisme melanjutkan pekerjaan lintas sesi. Tempelkan seluruh bagian **PROMPT UTAMA** ke Agent Coding Antigravity dari root repository POSQ.

## 1. Makna Keputusan Database

Keputusan tersebut harus diterapkan dengan topologi berikut.

```text
Tauri Desktop
  -> transaksi atomik ke SQLite lokal
  -> domain event ditulis ke sync_outbox dalam transaksi yang sama
  -> Sync Worker mengirim batch melalui HTTPS
  -> Sync API memvalidasi dan menulis secara idempoten
  -> PostgreSQL server menyimpan data konsolidasi dan server event log
  -> Desktop menarik event baru berdasarkan cursor
  -> event diterapkan ke SQLite melalui sync_inbox secara idempoten
```

Konsekuensinya:

1. SQLite adalah sumber kebenaran operasional saat perangkat sedang offline.
2. PostgreSQL adalah sumber kebenaran konsolidasi lintas perangkat dan lintas outlet.
3. Desktop tidak boleh memiliki kredensial PostgreSQL dan tidak boleh terhubung langsung ke PostgreSQL.
4. Sinkronisasi hanya melalui API server yang terautentikasi.
5. Checkout tidak boleh menunggu server. Transaksi lokal harus selesai meskipun server tidak tersedia.
6. Tidak boleh ada pola dual-write, yaitu menulis SQLite dan PostgreSQL secara langsung dalam satu use case.
7. Outbox harus ditulis dalam transaksi SQLite yang sama dengan perubahan domain.
8. Pengiriman ulang event tidak boleh menggandakan order, pembayaran, pergerakan stok, atau jurnal.

## 2. Pembagian Otoritas Data

| Kelompok data | Otoritas utama | Pola perubahan |
|---|---|---|
| Order, payment, refund, shift, cash movement | Perangkat atau outlet asal | Append-only setelah diposting; koreksi melalui reversal/adjustment |
| Inventory movement | Perangkat atau outlet asal | Sinkronkan movement, bukan angka saldo yang dapat ditimpa |
| Saldo persediaan konsolidasi | Hasil proyeksi | Dihitung dari ledger; bukan payload bebas dari frontend |
| Produk, kategori, price list, pelanggan | Master terversi | Optimistic concurrency dengan `version` dan aturan konflik |
| Merchant, outlet, perangkat, paket, entitlement | Server | Diturunkan ke lokal sebagai snapshot bertanda tangan atau tervalidasi |
| Preset, capability, outlet profile | Server atau owner berizin | Terversi, diaudit, dan tersedia offline setelah tersinkronisasi |
| Audit log | Kedua sisi | Append-only, memiliki actor, device, outlet, waktu, dan correlation ID |
| Lampiran/foto | Object storage melalui server | Metadata disinkronkan terpisah; jangan dimasukkan ke batch event utama |

Aturan tambahan:

- Gunakan UUIDv7 atau ULID yang dapat dibuat secara lokal. Jangan memakai integer auto-increment sebagai identitas global.
- Simpan waktu dalam UTC dan tampilkan sesuai zona waktu outlet.
- Simpan uang sebagai integer satuan mata uang terkecil. Jangan memakai float.
- Gunakan `deleted_at` atau tombstone untuk data tersinkronisasi. Jangan mengandalkan hard delete.
- Transaksi keuangan yang sudah diposting tidak boleh diedit secara diam-diam.
- Setiap event wajib memiliki `event_id`, `event_type`, `schema_version`, `aggregate_type`, `aggregate_id`, `aggregate_version`, `merchant_id`, `outlet_id`, `device_id`, `actor_id`, `occurred_at`, `correlation_id`, `causation_id`, dan `payload`.

## 3. Urutan Pekerjaan yang Wajib

### Gelombang A: kebenaran dan keamanan core

1. `MM-000`: selaraskan status proyek menjadi pre-alpha selama gate produksi belum lulus.
2. `MM-001`: tulis ADR final SQLite lokal dan PostgreSQL server.
3. `MM-002`: keluarkan private signing key lisensi dari desktop.
4. `MM-003`: implementasikan `SessionContext` yang tervalidasi backend.
5. `MM-004`: jadikan backend sumber kebenaran harga, diskon, pajak, service charge, HPP, dan total.
6. `MM-005`: implementasikan backup dan restore SQLite nyata, termasuk integrity check dan pre-restore backup.
7. `MM-006`: isolasi KDS menggunakan capability guard.
8. `MM-007`: tambahkan characterization test untuk perilaku lama yang kritis.

### Gelombang B: fondasi SQLite dan PostgreSQL

1. `SYNC-001`: dokumentasikan ownership, source of truth, dan conflict policy setiap aggregate.
2. `SYNC-002`: normalkan ID global, timestamp UTC, version, tenant scope, dan tombstone pada SQLite.
3. `SYNC-003`: buat migrasi lokal untuk `devices`, `sync_outbox`, `sync_inbox`, `sync_cursors`, `sync_conflicts`, dan `sync_dead_letters`.
4. `SYNC-004`: bangun control plane atau sync service dengan PostgreSQL, migrasi server, autentikasi perangkat, dan tenant isolation.
5. `SYNC-005`: implementasikan endpoint registrasi perangkat, bootstrap, push, pull, acknowledgment, dan health.
6. `SYNC-006`: implementasikan background sync worker dengan batching, retry exponential backoff, jitter, timeout, dan circuit breaker.
7. `SYNC-007`: implementasikan idempotency dan deduplication pada lokal serta server.
8. `SYNC-008`: implementasikan cursor server yang monotonik dan pull berhalaman.
9. `SYNC-009`: implementasikan optimistic concurrency dan conflict inbox untuk master data.
10. `SYNC-010`: sinkronkan vertical slice pertama: outlet profile, produk, price list, pelanggan, order, payment, dan inventory movement.
11. `SYNC-011`: buat halaman status sinkronisasi, retry aman, konflik, perangkat, dan audit tanpa memberi izin mengedit ledger mentah.
12. `SYNC-012`: implementasikan PostgreSQL backup server-side. Jangan menjalankan `pg_dump` dari desktop.
13. `SYNC-013`: uji upgrade dari database pengguna lama tanpa kehilangan data.
14. `SYNC-014`: uji dua perangkat, offline panjang, event duplikat, event terlambat, crash, dan pemulihan.
15. `SYNC-015`: tambah observability dengan structured log, correlation ID, metrik backlog, latency, error rate, dan dead-letter count tanpa membocorkan data sensitif.

### Gelombang C: capability kernel

1. `MM-100`: tabel preset, capability, dependency, entitlement, dan outlet profile.
2. `MM-101`: registry capability pada Rust.
3. `MM-102`: resolver deterministik untuk preset + override + entitlement.
4. `MM-103`: hilangkan `localStorage` sebagai sumber kebenaran mode.
5. `MM-104`: menu resolver dan route guard pada frontend.
6. `MM-105`: command guard pada backend dengan error terstruktur `CAPABILITY_DISABLED`.
7. `MM-106`: onboarding wizard berbasis karakter operasional usaha.

### Gelombang D: mode sebagai vertical slice

Urutan rilis:

1. General Flexible.
2. Retail Standard.
3. F&B Quick Service.
4. Retail Serialized.
5. F&B Table Service.
6. Appointment Service.
7. Repair/Workshop.
8. Grocery dan barang timbang.
9. Wholesale dan distribusi.
10. Rental.
11. Membership.
12. Consignment, resale, dan trade-in.
13. Digital voucher.
14. Custom production.
15. Ticketing.
16. Accommodation.
17. Pharmacy/lot.
18. Professional project.
19. Preset nonkomersial.
20. Parking serta fuel/energy sebagai pack integrasi perangkat keras.

Setiap mode wajib mencakup skema, migrasi, backend service, state machine, command guard, permission, audit, sinkronisasi, UI operasional, laporan, backup/restore, dokumentasi, dan pengujian. Kartu mode atau halaman pengaturan saja tidak dianggap implementasi.

## 4. Gate Teknis Wajib

Pekerjaan tidak boleh dinyatakan selesai sebelum seluruh gate yang relevan lulus.

### Gate penyimpanan lokal

- SQLite memakai WAL dan foreign key aktif.
- Migrasi bersifat deterministik dan dapat dijalankan dari versi produksi sebelumnya.
- Backup mengambil snapshot konsisten dari database aktif, bukan menyalin file secara serampangan ketika WAL aktif.
- Restore memverifikasi format, checksum, versi skema, dan `PRAGMA integrity_check`.
- Sebelum restore, aplikasi membuat backup pemulihan dari database yang sedang aktif.

### Gate sinkronisasi

- Menekan checkout saat offline tetap berhasil secara lokal.
- Event outbox terbentuk atomik bersama transaksi domain.
- Push yang sama sebanyak 100 kali hanya menghasilkan satu perubahan server.
- Pull event yang sama sebanyak 100 kali hanya diterapkan satu kali di lokal.
- Event yang datang tidak berurutan tidak merusak aggregate.
- Cursor tidak maju sebelum seluruh event pada halaman berhasil diterapkan.
- Event gagal masuk dead-letter dengan alasan, jumlah percobaan, dan mekanisme retry aman.
- Isolasi tenant dan outlet diuji secara negatif.
- Token perangkat dapat dicabut.
- Tidak ada password atau DSN PostgreSQL pada binary, source frontend, log, atau konfigurasi desktop.

### Gate konflik

- Order, payment, refund, cash movement, dan inventory movement bersifat append-only.
- Master data menggunakan `aggregate_version` atau ETag.
- Konflik tidak diselesaikan dengan last-write-wins tanpa aturan domain eksplisit.
- Perubahan stok tidak menyinkronkan `quantity_on_hand` sebagai nilai yang saling menimpa.
- Overselling lintas perangkat ketika sama-sama offline diakui sebagai risiko bisnis dan diselesaikan dengan kebijakan reservasi, rekonsiliasi, atau pembatasan mode. Jangan menyembunyikannya sebagai masalah yang dianggap sudah hilang.

### Gate kualitas

- Formatter, lint, type check, frontend build, Rust test, integration test, dan migration test lulus.
- Tidak ada test yang dinonaktifkan untuk membuat pipeline hijau.
- Tidak ada placeholder, fake success, hard-coded secret, atau mock produksi.
- Warning baru harus nol. Warning lama dicatat dan tidak boleh bertambah.
- Perubahan finansial dan sinkronisasi memiliki failure-path test.

## 5. PROMPT UTAMA

```text
Anda adalah principal software engineer yang bertanggung jawab menuntaskan
transformasi POSQ menjadi aplikasi POS offline-first multimode yang layak pilot.
Anda bekerja langsung pada repository POSQ dan harus menghasilkan kode yang
berjalan, bukan hanya analisis, contoh, mock, atau rekomendasi.

KEPUTUSAN PRODUK YANG SUDAH FINAL

1. Database operasional lokal adalah SQLite.
2. Database server adalah PostgreSQL.
3. Desktop tidak boleh terhubung langsung ke PostgreSQL. Semua komunikasi
   menggunakan API sync/control-plane melalui HTTPS.
4. Checkout dan operasi outlet utama harus tetap berjalan saat server mati.
5. Gunakan transactional outbox/inbox. Dilarang melakukan dual-write langsung
   ke SQLite dan PostgreSQL.
6. PostgreSQL menyimpan konsolidasi lintas perangkat/outlet, server event log,
   konfigurasi pusat, entitlement, dan proyeksi pelaporan.
7. Mode bisnis adalah preset + capability + workflow + outlet profile, bukan
   percabangan if/else global dan bukan localStorage.

SUMBER KEBENARAN YANG WAJIB DIBACA

- AGENTS.md dan instruksi repository lain yang berlaku;
- seluruh ADR, DECISIONS.md, STATUS.md, DATA_MODEL.md, TASK_BACKLOG.md;
- LOCAL_POSTGRESQL_STRATEGY.md dan BACKUP_KEY_RECOVERY.md jika masih ada;
- docs/POSQ_MULTIMODE_BLUEPRINT.md;
- POSQ_ANTIGRAVITY_MASTER_PROMPT.md;
- source, migrations, tests, package scripts, dan CI aktual.

Jika nama atau lokasi dokumen berbeda, temukan padanannya. Kode dan test aktual
lebih kuat daripada klaim status lama. Catat setiap ketidaksesuaian.

TUJUAN AKHIR

Tuntaskan seluruh task secara berurutan: Gelombang A, Gelombang B, Gelombang C,
kemudian Gelombang D. Jangan berhenti setelah membuat rencana. Mulai dari audit
baseline singkat, lalu langsung implementasikan task pertama yang belum selesai.
Lanjutkan otomatis ke task berikutnya selama tidak ada blocker yang benar-benar
membutuhkan keputusan manusia, kredensial, akses eksternal, atau tindakan
destruktif yang tidak diizinkan.

PROTOKOL KERJA

1. Periksa git status dan lindungi seluruh perubahan yang sudah ada. Jangan
   menghapus, mereset, atau menimpa pekerjaan pengguna yang tidak terkait.
2. Jalankan baseline test yang tersedia dan rekam hasilnya.
3. Buat atau perbarui docs/implementation/ANTIGRAVITY_PROGRESS.md dengan:
   task ID, status, acceptance criteria, file terdampak, migrasi, hasil test,
   risiko, dan task berikutnya.
4. Kerjakan satu task ID pada satu waktu, tetapi setelah task lulus lanjutkan
   otomatis ke task berikutnya. Jangan meminta pengguna mengetik "lanjut".
5. Sebelum mengubah perilaku lama, buat characterization test yang relevan.
6. Implementasikan vertical slice terkecil yang benar dan dapat diuji.
7. Setelah setiap task, jalankan test terfokus. Setelah setiap gelombang,
   jalankan seluruh suite, build, lint, dan migration test.
8. Perbarui ADR, DATA_MODEL.md, STATUS.md, TASK_BACKLOG.md, dokumentasi operator,
   dan recovery note agar sesuai dengan implementasi nyata.
9. Bila git repository bersih sebelum mulai dan kebijakan repository mengizinkan,
   buat commit atomik setelah satu task lulus. Jangan push, release, deploy,
   mengubah layanan produksi, atau menyentuh secret tanpa izin eksplisit.
10. Jangan menandai task selesai jika ada acceptance criterion yang belum lulus.

INVARIANT YANG TIDAK BOLEH DILANGGAR

- Semua command mutasi menggunakan SessionContext tervalidasi, tenant/outlet
  scope, permission guard, license guard, capability guard, dan idempotency key.
- Backend menghitung ulang harga, diskon, pajak, service charge, HPP, total,
  refund limit, dan movement. Frontend hanya mengirim intent dan pilihan sah.
- ID global dibuat dengan UUIDv7 atau ULID. Integer lokal tidak boleh menjadi
  identitas sinkronisasi.
- Uang disimpan sebagai integer satuan terkecil dan waktu disimpan dalam UTC.
- Order yang sudah diposting, payment, refund, cash movement, inventory movement,
  dan audit log bersifat append-only. Koreksi memakai reversal atau adjustment.
- Sinkronkan inventory movement, bukan angka saldo yang saling menimpa.
- Data tersinkronisasi yang dihapus memakai tombstone.
- Setiap event membawa schema_version dan aggregate_version.
- Apply event lokal dan server bersifat idempoten.
- Desktop tidak menyimpan private signing key, PostgreSQL DSN, database password,
  atau service secret.
- KDS hanya aktif ketika capability kitchen/KDS efektif tersedia.
- Mode non-F&B tidak boleh menghasilkan kitchen ticket.
- Backup dan restore harus bekerja pada database nyata serta dibuktikan dengan
  restore drill, row count, checksum, dan integrity check.

KONTRAK SINKRONISASI MINIMUM

Bangun kontrak versi, misalnya /api/v1/sync, yang mencakup:

- device registration dan revocation;
- bootstrap snapshot berhalaman;
- push batch event dengan event_id unik;
- acknowledgment per event, bukan hanya per request;
- pull berdasarkan server cursor monotonik;
- inbox deduplication;
- outbox retry dengan exponential backoff dan jitter;
- dead-letter untuk kegagalan permanen;
- optimistic concurrency untuk master data;
- conflict record yang dapat diaudit;
- schema compatibility dan penolakan versi yang tidak didukung;
- batas ukuran batch, timeout, dan rate limiting;
- correlation ID dan structured error code.

Gunakan server sequence/cursor hanya untuk urutan distribusi. Jangan menganggap
jam perangkat sebagai urutan global. Jangan menghapus outbox sebelum acknowledgment
per event tersimpan secara atomik.

KEBIJAKAN KONFLIK MINIMUM

- Transactional aggregates: append-only, deduplicate by event/idempotency ID.
- Product/customer/price master: optimistic concurrency dengan version eksplisit.
- Inventory: gabungkan movement ledger; buat adjustment terotorisasi bila hasil
  rekonsiliasi berbeda.
- Outlet profile/capability/entitlement: server-authoritative dan dicache lokal.
- Konflik yang tidak aman tidak boleh diselesaikan diam-diam. Masukkan ke
  sync_conflicts dan tampilkan tindakan resolusinya kepada role berizin.

URUTAN IMPLEMENTASI

Ikuti seluruh task dan acceptance criteria pada bagian Gelombang A sampai D di
POSQ_ANTIGRAVITY_MASTER_PROMPT.md. Prioritaskan kebenaran sistem dan sync core
sebelum menambah kartu mode. Setiap mode harus menjadi vertical slice lengkap:
schema + migration + state machine + backend + permission + capability guard +
sync + UI + audit + report + backup/restore + test + operator docs.

PENGUJIAN WAJIB SINKRONISASI

Tambahkan automated test untuk:

1. checkout lokal ketika server tidak tersedia;
2. crash setelah commit domain tetapi sebelum push;
3. push event yang sama 100 kali;
4. pull event yang sama 100 kali;
5. acknowledgment parsial;
6. event datang terlambat dan tidak berurutan;
7. dua perangkat membuat transaksi saat offline;
8. konflik edit master data;
9. token perangkat dicabut;
10. akses silang tenant/outlet ditolak;
11. upgrade schema dari database lama;
12. backup dan restore SQLite;
13. backup dan restore PostgreSQL pada service/server;
14. backlog besar setelah offline panjang;
15. payload rusak atau schema_version tidak didukung.

ATURAN DIAGNOSIS DAN PERBAIKAN

Jika test gagal, cari akar masalah dan perbaiki dalam scope task. Jangan hanya
menambah retry, menonaktifkan test, memperlonggar assertion, memakai sleep tetap,
atau menyembunyikan error. Bila dependensi tidak tersedia, gunakan perintah
repository yang setara atau dokumentasikan bukti blocker secara presisi.

STOP CONDITION YANG SAH

Anda hanya boleh berhenti sebelum tujuan akhir apabila:

- perubahan membutuhkan keputusan produk yang belum tercakup dalam dokumen;
- diperlukan kredensial atau akses layanan yang memang tidak tersedia;
- diperlukan migrasi destruktif dengan risiko kehilangan data;
- ditemukan perubahan pengguna yang konflik langsung dan tidak dapat digabung;
- terdapat pelanggaran keamanan yang membutuhkan persetujuan pemilik.

Jika berhenti, jangan memberi laporan umum. Sebutkan task ID, bukti blocker,
file/baris terkait, opsi penyelesaian, rekomendasi, dan konsekuensi setiap opsi.
Tetap selesaikan seluruh pekerjaan lain yang tidak terblokir.

DEFINITION OF DONE PRODUK

Jangan menyatakan POSQ "tuntas", "production-ready", atau "selesai" sebelum:

- seluruh task target berstatus lulus dengan bukti test;
- tidak ada fake backup, fake success, private key klien, users LIMIT 1 untuk
  actor aktif, pricing yang dipercaya dari frontend, atau mode source of truth
  di localStorage;
- SQLite lokal bekerja offline dan PostgreSQL tersinkronisasi secara idempoten;
- recovery drill dan migration test lulus;
- tiga mode GA pertama lulus skenario pilot end-to-end;
- security, tenant isolation, reconciliation, dan observability gate lulus;
- dokumentasi instalasi, operasi, backup, restore, konflik, dan troubleshooting
  sesuai dengan kode aktual.

LAPORAN AKHIR

Saat pekerjaan benar-benar selesai atau terblokir secara sah, laporkan:

1. hasil akhir per task dan per gelombang;
2. keputusan arsitektur yang diimplementasikan;
3. migrasi dan kompatibilitas data lama;
4. file utama yang berubah;
5. perintah test dan hasil aktual;
6. hasil sync, conflict, security, backup, dan restore drill;
7. risiko yang masih terbuka;
8. langkah pengguna yang benar-benar diperlukan.

Mulai sekarang. Audit baseline secukupnya, buat ADR keputusan database, lalu
langsung kerjakan task pertama yang belum lulus. Jangan berhenti pada rencana.
```

## 6. Prompt Lanjutan Jika Sesi Antigravity Terputus

```text
Lanjutkan implementasi POSQ dari repository dan kondisi kerja aktual.
Baca AGENTS.md, POSQ_ANTIGRAVITY_MASTER_PROMPT.md,
docs/POSQ_MULTIMODE_BLUEPRINT.md, serta
docs/implementation/ANTIGRAVITY_PROGRESS.md. Verifikasi git diff dan test aktual.
Jangan mengulang task yang sudah lulus. Mulai dari task pertama berstatus
in_progress, blocked yang sudah teratasi, atau pending. Ikuti seluruh invariant,
acceptance criteria, test gate, dan stop condition pada prompt induk. Lanjutkan
otomatis ke task berikutnya tanpa menunggu perintah "lanjut".
```

## 7. Arahan Penggunaan

1. Salin dokumen ini ke root repository POSQ dengan nama yang sama.
2. Salin blueprint lama ke `docs/POSQ_MULTIMODE_BLUEPRINT.md` bila belum berada di repository.
3. Berikan bagian **PROMPT UTAMA** kepada Antigravity.
4. Jangan meminta semua mode dibuat dalam satu perubahan besar. Prompt tetap mengarahkan seluruh roadmap, tetapi Antigravity harus menyelesaikan dan menguji satu task ID pada satu waktu.
5. Saat sesi terputus atau konteks Agent habis, gunakan **Prompt Lanjutan**. Berkas progres di repository menjadi sumber kesinambungan kerja.
6. Tinjau hanya keputusan yang masuk stop condition. Untuk kegagalan build, test, migrasi non-destruktif, dan masalah implementasi biasa, minta Agent memperbaikinya tanpa melempar pekerjaan kembali kepada Anda.

## 8. Keputusan Teknis yang Tidak Perlu Ditanyakan Lagi

- SQLite lokal dan PostgreSQL server sudah final.
- POSQ tetap offline-first.
- Desktop hanya berbicara kepada server melalui API.
- Sinkronisasi menggunakan outbox/inbox dan idempotency.
- Mode adalah preset dan capability per outlet.
- Harga dan aturan finansial diputuskan backend lokal, bukan frontend.
- Data transaksi dan ledger yang sudah diposting bersifat append-only.
- Secret server tidak boleh masuk ke aplikasi desktop.

Keputusan yang masih boleh diajukan kepada pemilik produk hanya mencakup regulasi domain, prioritas komersial, kebijakan overselling lintas perangkat saat seluruh perangkat offline, penyedia layanan eksternal, dan tindakan migrasi destruktif.
