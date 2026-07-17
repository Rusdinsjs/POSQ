# Rekomendasi Penyempurnaan Aplikasi POSQ

Analisis berbasis bukti kode (`apps/desktop/src-tauri/src/*.rs`, `services/control-plane-api/src/*`) vs dokumen desain (`docs/DECISIONS.md`, `DATA_MODEL.md`, `LOCAL_POSTGRESQL_STRATEGY.md`, `SECURITY_MODEL.md`).

Tujuan: desain + implementasi (sesuai klarifikasi user).

## 0. Temuan Kritis (Harus Diselesaikan Duluan)

### F-1. Kontradiksi arsitektur: SQLite digunakan, bukan PostgreSQL lokal — **KEPUTUSAN: TETAP SQLITE, SAHKAN DESAIN**
- **Fakta**: `apps/desktop/src-tauri/src/db.rs` memakai `sqlx::sqlite::SqlitePool` + file `posq.db`; `Cargo.toml` hanya mengaktifkan fitur `sqlite` (tidak ada `postgres`). `backup.rs:60` jatuh ke "MOCK BACKUP" karena `pg_dump` tidak ada.
- **Desain lama**: `DEC-005`, `DEC-035`, `LOCAL_POSTGRESQL_STRATEGY.md`, `DATA_MODEL.md` secara eksplisit mewajibkan **PostgreSQL lokal per device** ("Must not use SQLite as primary operational DB" terdaftar di *Explicitly Rejected Directions*).
- **Keputusan (2026-07-17)**: User memilih **Opsi B — tetap SQLite untuk MVP dan sahkan secara tertulis**. Alasan: MVP desktop ringan & offline-handal, kode sudah jalan di SQLite, menghindari friction instalasi PG di UMKM Windows.
- **Tindakan wajib (bukan opsional)**:
  - Buat **ADR-0013-sqlite-local-operational-db.md** yang mengubah keputusan DB operasional lokal menjadi SQLite (per-device file). Catat tradeoff: concurrency single-writer (cocok single-cashier MVP), tidak ada `RETURNING`/`FOR UPDATE`/enum/role DB, backup pakai `VACUUM INTO`/file copy.
  - Perbarui `DEC-005` & `DEC-035` → status "Accepted (amended to SQLite per ADR-0013)".
  - Perbarui `DATA_MODEL.md §3` → hapus `numeric(18,3)`, enum tipe, `RETURNING`, `FOR UPDATE`; gunakan `REAL`/`INTEGER`, `text` status, `ON CONFLICT` SQLite.
  - Perbarui `LOCAL_POSTGRESQL_STRATEGY.md` → judul/isi jadi "Local Operational DB Strategy (SQLite)"; hapus wajib-PG. `init.sql` menjadi **server-only** (control plane).
  - Tambah ke *Explicitly Rejected Directions* (revisi): "PostgreSQL sebagai DB operasional lokal MVP" diganti penjelasan bahwa SQLite dipilih; PostgreSQL tetap untuk control-plane server jika diperlukan.
- **Dampak yang harus dikelola di kode**: WAL mode sudah `Normal` (`db.rs:21`) — baik; pastikan single-writer (sudah, max 5 koneksi tapi SQLite serializes writes). Currency pakai `i64` (lihat I-4), bukan `f64` untuk uang.

### F-2. Private signing key ada di desktop app (pelanggaran keamanan)
- **Fakta**: `license.rs:69-72` & `:223-226` menyimpan `mock_server_private_key` (Ed25519) **di dalam binary desktop** dan menggunakannya untuk menandatangani/refresh token. `SERVER_PUBLIC_KEY` juga hardcoded.
- **Desain**: `DEC-029` melarang keras private key di desktop; `SECURITY_MODEL.md` 3.1 & 4.1 melarangnya; `ADR-0011` mewajibkan device-bound + server-signed.
- **Dampak**: Siapa pun bisa mem-(*patch*) binary untuk memalsu lisensi tak terbatas. Ini stub demo, bukan produksi.
- **Rekomendasi**:
  - Aktivasi & refresh HARUS memanggil Control Plane API (`services/control-plane-api/src/routes/licenses.rs`). Desktop hanya memverifikasi signature via `SERVER_PUBLIC_KEY` (public saja).
  - Hapus `mock_server_private_key` dari `license.rs`. Ganti `activate_device`/`refresh_license` jadi panggilan HTTP ke server (challenge-response per ADR-0011).
  - Tambah test: `SECURITY_TEST` gagal build jika private key ditemukan di crate desktop (grep CI).

### F-3. Status.md tidak akurat ("M14 Completed")
- **Fakta**: Banyak modul masih stub (`license.rs` mock, `backup.rs` mock, `server.rs`/`sync_worker.rs` perlu dicek), tapi STATUS.md menyatakan semua M1–M14 selesai.
- **Rekomendasi**: Turunkan status ke "Implementation in progress / not pilot-ready". Perbarui tabel milestone dengan kolom "Code status: real / stub / mock".

## 1. Penyempurnaan Desain (dokumen)

### D-1. Selesaikan 17 keputusan menggantung (PDEC-001..023)
Banyak PDEC di `DECISIONS.md §2` masih "Open" padahal STATUS.md bilang MVP selesai. Contoh kritis:
- PDEC-002 OS target (Windows only?),
- PDEC-003 grace period (rekomendasi 7 hari),
- PDEC-004/PDEC-022 QRIS manual vs gateway,
- PDEC-011 recovery key (user-held vs escrow),
- PDEC-014 license lease (3/7/14 hari).
Tulis keputusan final + buat ADR bila mengubah batas arsitektur.

### D-2. Validasi pajak/PPN & struk (Indonesia compliance)
- `pos.rs:834` hardcode `tax = (subtotal - discount) * 0.11`. `DEC-041` bilang pajak harus *configurable*. `INDONESIA_COMPLIANCE.md` butuh validasi attorney.
- **Rekomendasi**: Pindahkan tarif & enable/disable ke tabel `outlet_settings`/`tax_rules`; dukung PPN 11%, bisa 0, multi-rate (service charge terpisah). Struk harus mencantumkan NPWP/merchant sesuai regulasi (butuh legal review sebelum rilis).

### D-3. Konsistensi skema lokal vs server
- `DATA_MODEL.md §3` (local) pakai `numeric(18,3)` & tipe `text` status; implementasi SQLite pakai `f64`/`i64`. Jika tetap SQLite (Opsi B), sederhanakan DATA_MODEL agar tidak menjanjikan fitur PostgreSQL (`RETURNING`, enum, `FOR UPDATE`).
- `DATA_MODEL.md §3.21 device_licenses` vs `license.rs` struct tidak sinkron (field `tamper_seal`, `last_server_time` tidak dipakai di kode).

## 2. Penyempurnaan Implementasi (kode)

### I-1. License & Subscription nyata (gantikan F-2)
- Desktop: `verify_license()` pertahankan (verifikasi signature + clock-rollback + grace). `activate_device`/`refresh_license` → HTTP ke CP API.
- Server: implementasikan issue/rotate token di `licenses.rs` dengan private key **hanya di server** (baca dari `LICENSE_SIGNING_KEY_PATH` di `docker-compose.yml`, sudah disiapkan!).
- Tambah `device_activation_challenges` + nonce replay (sudah ada di `init.sql`) untuk activation handshake.

### I-2. Backup/Restore nyata (gantikan mock di `backup.rs`)
- **Karena SQLite (F-1 LOCKED)**: gunakan `VACUUM INTO 'backup.db'` untuk backup atomik + `File::copy` manifest; enkripsi AES-256-GCM sudah benar (`encrypt_data`), pertahankan. Hapus jalur "MOCK BACKUP" `pg_dump`.
- **Wajib**: pre-restore backup (saat ini `backup.rs:156` "skipped for brevity") — implementasikan sebelum restore menimpa DB.
- Upload metadata ke CP API (`upload_backup_metadata` saat ini POST ke `127.0.0.1:3000` tanpa auth) → pakai token + idempotency key.

### I-3. Perkuat auth & session
- `auth.rs:184` `login_user` membandingkan PIN terhadap **semua user aktif** (loop). OK untuk skala kecil, tapi pastikan `failed_login_attempts` & `locked_until` benar-benar diterapkan per-user (saat ini reset dilakukan pada user yang cocok saja; user terkunci lain tetap bisa dicek). Tambah rate-limit.
- Session `expires_at` 12 jam (`auth.rs:234`); pertimbangkan idle-timeout + `last_activity_at` benar-benar digunakan untuk invalidasi.

### I-4. Transaksi & angka (currency)
- `checkout()` sudah transactional (baik). Tapi `f64` untuk uang (SQLite Opsi B) rawan rounding. Gunakan `i64` minor unit (sen) untuk semua perhitungan; `f64` hanya untuk qty stok.
- Validasi server-side: `subtotal/discount/tax/grand_total` dihitung ulang di Rust, jangan percaya 100% pada nilai dari UI (mitigasi tamper).

### I-5. RBAC sudah baik — pertahankan & perluas
- `security_policy.rs` (authorization_policies + grants + supervisor PIN + self-approval block) dan `auth::has_permission` layak dipertahankan.
- Pastikan **setiap** command sensitif (refund, void, stock adjustment, price override) memanggil `evaluate_action_policy` + `validate_and_consume_grant` di Rust, bukan hanya UI.

### I-6. Hardware abstraction
- `hardware.rs` + `serialport` sudah ada. Pastikan printer ESC/POS dan barcode scanner lewat adapter (sudah sesuai ADR-0012). Tambah mock printer test di CI.

### I-7. Health check & instalasi
- `LOCAL_POSTGRESQL_STRATEGY.md §8` menuntut health check (OK/WARNING/ACTION_REQUIRED/BLOCKED). Belum ada di `db.rs`. Tambah `check_db_health()` yang memverifikasi tabel wajib + disk space.
- Installer Windows (NSIS di `tauri.conf.json`) harus bundle/setup DB lokal (PG atau SQLite) + panduan repair.

## 3. Pengurangan / Penghapusan

- **Hapus** semua "MOCK" (license signing, backup dump) sebelum pilot — ganti dengan implementasi nyata atau nonaktifkan fitur dengan jelas.
- **Hapus** `mock_server_private_key` (F-2).
- **Hapus** ketergantungan pada `http://127.0.0.1:3000` hardcoded di desktop; gunakan `PUBLIC_API_URL`/config + TLS.
- **Kurangi** scope marketplace sync & payment gateway (sudah direject di DEC-016/017) — jangan ditambah sebelum core stabil.

## 4. Rencana Eksekusi (urutan)

1. **F-1 (LOCKED → SQLite)**: Buat ADR-0013 + perbarui `DEC-005/035`, `DATA_MODEL.md §3`, `LOCAL_POSTGRESQL_STRATEGY.md`, `init.sql` (server-only). *Mengunci semua langkah berikut.*
2. **F-2** license: pindahkan signing ke server (`services/control-plane-api/src/routes/licenses.rs` membaca `LICENSE_SIGNING_KEY_PATH`), hapus `mock_server_private_key` di `license.rs`, wiring `activate_device`/`refresh_license` ke CP API (challenge-response + nonce per ADR-0011). Desktop hanya verifikasi via `SERVER_PUBLIC_KEY`.
3. **F-3** STATUS.md: turunkan ke "Implementation in progress / not pilot-ready"; tambah kolom "Code status: real / stub / mock" per milestone.
4. **D-1** Selesaikan PDEC (terutama OS target, grace 7 hari, QRIS manual, recovery key user-held, lease 7 hari).
5. **I-2** Backup/restore nyata (SQLite: `VACUUM INTO`/file copy + AES-256-GCM sudah benar) + **pre-restore safety** (saat ini `backup.rs:156` skipped).
6. **I-4** Currency `i64` minor-unit; recalculation total di Rust (jangan percaya UI).
7. **D-2** Tax configurable (tabel `tax_rules`); struk per regulasi (legal review).
8. **I-7** Health check (`check_db_health`) + installer Windows bundle SQLite + repair flow.
9. Security test gate: private key absence (grep CI), token tamper, clock rollback, expired-mode blocks Rust command.
10. Pilot dengan 1 merchant terbatas.

## 5. Risiko Terbuka
- Fully compromised local machine tetap bisa direverse (residual, diterima di SECURITY_MODEL §8).
- Backup key loss → unrecoverable (mitigasi: UX peringatan + escrow Enterprise, PDEC-011).
- Jika Opsi A (PG), instalasi PG di UMKM Windows adalah friction UX tinggi — butuh embedded/portable PG.

## 6. Validasi
- `cargo test` (desktop) + `sqlx::migrate` idempotency test (sudah ada).
- `npm run check` (Svelte/TS) — pastikan 0 error.
- Security checklist `SECURITY_MODEL.md §6` harus lulus 100% sebelum pilot.
- Schema guardrail test `services/control-plane-api/tests/schema_guardrail.rs` untuk memastikan server TIDAK menyimpan tabel operasional.
