# 🔄 Offline Sync Architecture & Guide (Local-First)

POSQ adalah aplikasi **Local-First POS** yang beroperasi secara mandiri penuh di database SQLite lokal. Koneksi cloud digunakan untuk sinkronisasi async di background tanpa pernah memblokir transaksi bisnis kasir.

---

## 🏗️ Pola Desain (Outbox Pattern)

1. **Atomicity**:
   - Setiap transaksi bisnis (`process_checkout`, `process_stock_movement`, dsb) menyimpan data ke tabel utama DAN menyisipkan entri ke `sync_outbox` dalam satu transaksi SQLite atomik (`tx.commit()`).

2. **Idempotency Key**:
   - Setiap entri outbox menyertakan UUID v4 unik (`event_id`). Server cloud menggunakan key ini untuk mendeteksi retries agar tidak terjadi duplikasi data saat koneksi terputus.

3. **Background Worker (`SyncWorker`)**:
   - Rust tokio background task mengecek entri `pending` secara berkala (5 detik).
   - Menggunakan **Exponential Backoff dengan Jitter** (1s, 2s, 4s, 8s, ..., max 64s + random jitter 0-1000ms) untuk mencegah *thundering herd*.

4. **Penanganan Konflik (`conflict_resolver.rs`)**:
   - Konflik diselesaikan menggunakan strategi `ServerWins`, `ClientWins`, atau `ManualMerge`.

---

## 📊 Indikator Status Visual

- **Emerald (Hijau)**: Semua data tersinkronisasi.
- **Amber (Kuning)**: Ada transaksi pending yang sedang diantrekan.
- **Rose (Merah)**: Terdapat transaksi gagal sync yang memerlukan perhatian.
- **Slate (Abu-abu)**: Perangkat sedang offline (Local-First active).
