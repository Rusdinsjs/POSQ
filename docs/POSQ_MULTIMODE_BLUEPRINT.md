# Blueprint Penyempurnaan POSQ Menjadi POS Multimode Berbasis Kapabilitas

**Repositori yang diaudit:** `rusdinKalem/POSQ`  
**Commit audit:** `702420d54b027880acd8cdd64448787ed7486ec0`  
**Tanggal audit:** 23 Juli 2026  
**Tujuan:** menjadi rancangan teknis dan backlog implementasi yang dapat dieksekusi secara bertahap oleh Agent Coding Antigravity.

## 1. Kesimpulan Eksekutif

POSQ sudah memiliki fondasi yang berguna untuk POS lokal, yaitu SvelteKit 5, Tauri 2, Rust, basis data lokal, checkout atomik, ledger persediaan, shift, RBAC, audit, F&B table management, KDS, retur, dan fondasi omnichannel. Namun, POSQ belum dapat disebut sebagai POS multimode. Saat ini, `businessMode` pada dasarnya masih berupa pilihan antarmuka `retail`, `fb`, atau `jasa` yang disimpan di `localStorage`. Pilihan tersebut belum menjadi konfigurasi outlet yang terversi, belum menjadi sumber kebenaran di backend, belum mengaktifkan kumpulan kapabilitas, dan belum mengubah aturan domain secara konsisten.

Masalah utamanya bukan kekurangan jumlah kartu mode. Masalahnya adalah belum adanya **kernel kapabilitas** yang memisahkan lima unsur berikut:

1. **Preset industri**, misalnya restoran, toko elektronik, salon, bengkel, rental, dan apotek.
2. **Kapabilitas**, misalnya serial, lot, resep, meja, booking, deposit, piutang, dan membership.
3. **Mesin alur kerja**, misalnya checkout langsung, KDS, work order, rental, reservasi, dan entitlement.
4. **Kanal**, misalnya POS, marketplace, toko daring, QR order, dan sales order.
5. **Profil penyimpanan dan terminal**, misalnya standalone, outlet server, mobile cashier, self-checkout, dan kiosk.

Rancangan yang disarankan bukan membuat satu kode terpisah untuk setiap industri. POSQ perlu diubah menjadi:

```text
POS Core
+ Capability Kernel
+ Business Preset
+ Workflow Engine
+ Policy Engine
+ Outlet Configuration
+ Optional Vertical Pack
```

Dengan susunan ini, toko handphone dapat memilih preset **Retail Berserial**, kemudian mengaktifkan kapabilitas **trade-in**, **servis**, **cicilan**, dan **konsinyasi**. Hotel dapat mengaktifkan preset berbeda per outlet: front office memakai akomodasi, restoran memakai F&B, spa memakai appointment, dan toko suvenir memakai retail. Usaha yang tidak cocok dengan preset dapat memakai **Mode Umum/Fleksibel** tanpa kehilangan fungsi inti.

## 2. Metode Audit dan Batas Kesimpulan

Audit dilakukan terhadap struktur repository, migrasi basis data, perintah Tauri, kode Rust, rute Svelte, konfigurasi build, dokumentasi arsitektur, dan status proyek pada commit yang disebutkan di atas. Build frontend berhasil dengan `svelte-check` tanpa error dan menghasilkan static build, tetapi masih terdapat 11 peringatan aksesibilitas. Pengujian Rust tidak dapat diulang pada lingkungan audit karena executable `cargo` tidak tersedia. Oleh sebab itu, klaim `12/12 passed` dalam `STATUS.md` diperlakukan sebagai klaim repository, bukan hasil verifikasi independen audit ini.

Penilaian menggunakan empat status:

| Status | Makna |
|---|---|
| Terimplementasi | Terdapat skema, backend, UI, dan alur yang dapat ditelusuri |
| Parsial | Sebagian lapisan tersedia, tetapi aturan domain belum utuh |
| Mock/Stub | Antarmuka ada, tetapi perilaku produksi belum nyata |
| Belum ada | Tidak ditemukan model data atau alur yang relevan |

## 3. Kondisi Aktual POSQ

### 3.1 Fondasi yang sudah bernilai

| Area | Kondisi aktual | Penilaian |
|---|---|---|
| Desktop shell | Tauri 2 | Terimplementasi |
| Frontend | SvelteKit 5, TypeScript, static adapter | Terimplementasi |
| Backend lokal | Rust melalui Tauri commands | Terimplementasi |
| Database aktif | SQLite dengan WAL dan pool maksimal lima koneksi | Terimplementasi, tetapi bertentangan dengan dokumen |
| Checkout | Order, item, payment, inventory movement, audit, dan outbox dalam transaksi lokal | Parsial-kuat |
| Inventory | Ledger, stock before/after, idempotency, reserved/in-transit/damaged | Parsial-kuat |
| F&B | Meja, dining session, split/join bill, KDS, resep | Parsial-kuat |
| Retail | Produk, barcode dasar, retur penuh | Parsial |
| Jasa | Pemilihan teknisi/terapis pada UI | Mock/Stub |
| Shift dan kas | Shift, cash drawer, cash movement, blind count | Terimplementasi |
| Keamanan | Login PIN, RBAC, supervisor approval, audit chain | Parsial |
| Lisensi | Verifikasi Ed25519 dan restricted mode | Mock dan berisiko kritis |
| Backup | UI dan AES-GCM tersedia | Mock/tidak sesuai database aktif |
| Control plane | Dokumen sangat lengkap, folder service kosong | Belum diimplementasikan |
| Omnichannel | Kolom kanal, reservasi, mock adapter | Fondasi P0 |

### 3.2 Temuan kritis yang harus diselesaikan sebelum memperluas mode

#### A. Kontradiksi SQLite dan PostgreSQL

Dokumen menyatakan PostgreSQL lokal sebagai kontrak yang tidak boleh diubah, tetapi implementasi memakai `SqlitePool`, URL `sqlite://`, pragma WAL, dan migrasi SQL bergaya SQLite. Backup justru mencari `pg_dump.exe` dan `psql.exe`; jika program tersebut tidak ditemukan, POSQ menulis berkas SQL tiruan dan mengembalikan status sukses. Kondisi ini dapat memberi rasa aman palsu karena pengguna mengira data sebenarnya telah dicadangkan.

**Keputusan yang disarankan:** buat ADR baru untuk profil penyimpanan:

- `standalone_sqlite`: satu terminal, satu file lokal, cocok untuk UMKM kecil;
- `outlet_postgres`: beberapa terminal dalam LAN, PostgreSQL pada edge server outlet;
- jangan mengklaim dukungan PostgreSQL sampai adapter, migrasi, backup, restore, dan test benar-benar tersedia.

Jika keputusan produk tetap mewajibkan PostgreSQL untuk seluruh instalasi, hapus jalur SQLite dan selesaikan installer PostgreSQL. Jangan mempertahankan dua realitas yang saling bertentangan.

#### B. Mode hanya tersimpan pada browser lokal

`businessMode` disimpan melalui `localStorage`. Akibatnya:

- mode tidak terikat pada merchant atau outlet;
- mode dapat berbeda tanpa sengaja pada dua terminal;
- backend tidak mengetahui mode yang dipilih;
- perubahan mode tidak tercatat dalam audit;
- tidak ada versi preset atau migrasi konfigurasi;
- lisensi dan entitlement tidak dapat memvalidasi fitur mode;
- mode tidak aman dijadikan dasar aturan bisnis.

Mode harus dipindahkan ke tabel konfigurasi outlet dan dibaca melalui command Rust. `localStorage` hanya boleh menjadi cache UI.

#### C. Backend mempercayai nilai finansial dari frontend

Payload checkout menerima `unit_price`, `discount_total`, `line_total`, `subtotal`, `tax_total`, `service_total`, `grand_total`, `paid_total`, dan `change_total`, lalu menyimpannya tanpa rekalkulasi menyeluruh dari master harga dan kebijakan backend. UI Tauri tetap dapat dimanipulasi. Untuk POS produksi, backend harus memuat harga resmi, daftar harga, promosi, pajak, service charge, pembulatan, dan HPP, kemudian menghasilkan snapshot final.

#### D. Konteks pengguna memakai `SELECT ... FROM users LIMIT 1`

Checkout dan retur mengambil pengguna pertama pada database, bukan sesi yang sedang login. Ini merusak atribusi audit, RBAC, komisi staf, outlet scope, dan pertanggungjawaban transaksi. Seluruh command mutasi harus menerima session token atau membaca session context yang tervalidasi backend.

#### E. KDS tercipta pada transaksi non-F&B

Alur checkout membuat tiket KDS jika tiket belum ditautkan tanpa memastikan capability `kitchen` aktif. Dengan demikian, transaksi retail dan jasa berpotensi menghasilkan tiket dapur. Ini menunjukkan bahwa mode belum mengisolasi perilaku domain.

#### F. Retur belum memenuhi kebutuhan nyata

Retur saat ini mengembalikan seluruh item pesanan, menandai seluruh order `refunded`, menerima nominal refund dari pemanggil, dan tidak membuat payment reversal yang memadai. Belum ada retur parsial per item, exchange, restocking disposition, serial yang sama, lot asal, biaya retur, store credit, atau validasi batas nominal.

#### G. Lisensi menyimpan private signing key di aplikasi desktop

Kode aktivasi dan refresh memuat private key mock yang dapat menerbitkan lisensi baru dari sisi klien. Ini bertentangan langsung dengan ADR keamanan repository. Sebelum distribusi, seluruh signing harus dipindahkan ke control plane dan desktop hanya menyimpan public key.

#### H. Klaim release readiness terlalu dini

`STATUS.md` menyatakan M14 selesai, sedangkan control plane belum memiliki file implementasi, backup nyata belum sesuai database, aktivasi lisensi masih mock, printer dan ECR masih terbatas, dan beberapa keputusan manusia masih terbuka. Status yang lebih jujur adalah **functional prototype / pre-alpha**, bukan release-ready komersial.

## 4. Prinsip Arsitektur Target

### 4.1 Mode adalah preset, bukan percabangan kode global

Hindari pola berikut:

```rust
if business_mode == "fb" { ... }
else if business_mode == "retail" { ... }
else if business_mode == "jasa" { ... }
```

Pola ini akan menghasilkan percabangan yang tidak terkendali ketika jumlah mode bertambah. Gunakan resolver kapabilitas:

```text
Outlet Configuration
  -> resolve preset version
  -> merge capability overrides
  -> verify subscription entitlements
  -> validate dependency/conflict rules
  -> produce EffectiveCapabilitySet
  -> route command to domain workflow
```

### 4.2 Empat lapisan konfigurasi

| Lapisan | Contoh | Dapat diubah pengguna? |
|---|---|---:|
| Core invariant | money integer, audit, idempotency, session, tenant scope | Tidak |
| Preset | `restaurant_table_service@1` | Ya, memilih preset |
| Capability | `table`, `kds`, `recipe`, `split_bill` | Ya, sesuai dependensi |
| Outlet override | service charge 5%, negative stock false | Ya, dengan izin |

### 4.3 Satu organisasi dapat memiliki banyak preset

Konfigurasi harus berada pada outlet atau profit center, bukan hanya merchant.

```text
Merchant Hotel Nusantara
├── Front Office: accommodation
├── Restoran: fnb_table_service
├── Spa: appointment_service
└── Gift Shop: retail_standard
```

Laporan konsolidasi tetap membaca journal penjualan, pembayaran, pajak, biaya, dan stok yang seragam.

### 4.4 Data transaksi harus berupa snapshot

Perubahan master setelah transaksi tidak boleh mengubah sejarah. Setiap line menyimpan:

- nama, SKU, barcode, dan deskripsi saat transaksi;
- harga dasar, daftar harga, diskon, pajak, service charge, dan pembulatan;
- unit ukur dan faktor konversi;
- serial/lot/aset/booking yang dialokasikan;
- HPP atau cost layer yang dikonsumsi;
- staf, kanal, fulfilment, dan policy version;
- alasan override serta pemberi persetujuan.

## 5. Taksonomi Produk dan Tracking Policy

Jangan menyimpulkan jenis transaksi hanya dari `business_mode`. Produk perlu atribut eksplisit.

| Dimensi | Nilai utama |
|---|---|
| `offering_type` | goods, menu, service, rental, accommodation, ticket, membership, digital, custom_product, donation, fee |
| `stock_policy` | none, quantity, weighted, lot, serial, asset, capacity, entitlement |
| `fulfilment_policy` | immediate, pickup, delivery, kitchen, appointment, work_order, rental_checkout, stay, access_scan, recurring |
| `pricing_policy` | fixed, tiered, customer_price_list, time_based, weight_based, dynamic_rate, quote, donation |
| `cost_policy` | none, average, fifo, specific_identification, recipe, project_actual, depreciation_view |
| `return_policy` | none, full, partial, exchange, same_serial, unopened_lot, inspection_required |

Kombinasi dimensi ini mencakup lebih banyak usaha daripada daftar industri yang statis.

## 6. Katalog Mode Bisnis yang Harus Dicakup

### 6.1 Preset komersial

| Kode preset | Mode | Objek transaksi | Kapabilitas pembeda utama | Prioritas |
|---|---|---|---|---:|
| `retail_standard` | Retail umum | Barang siap jual | SKU, varian, barcode, retur, promosi | P1 |
| `retail_serialized` | Elektronik/HP/laptop/kendaraan | Unit unik | serial/IMEI, HPP spesifik, garansi, trade-in | P1 |
| `grocery_weighted` | Minimarket, sembako, buah | Barang unit/berat | timbangan, embedded barcode, lot, ED, waste | P2 |
| `wholesale_distribution` | Grosir/distributor | Barang dan pengiriman | UOM bertingkat, price tier, SO, DO, AR, credit limit | P2 |
| `fnb_quick_service` | Kedai, booth, fast food | Menu | modifier, combo, KDS, antrean, bayar awal | P1 |
| `fnb_table_service` | Restoran/kafe | Menu dan meja | table session, course, split/join bill, open tab | P1 |
| `fnb_bar_tab` | Bar/lounge | Menu dan tab | open tab, pre-auth reference, tip, age policy | P2 |
| `appointment_service` | Salon, spa, barbershop, konsultasi | Slot staf/sumber daya | booking, resource calendar, deposit, no-show, komisi | P2 |
| `repair_workshop` | Bengkel, servis HP/elektronik | Aset pelanggan dan pekerjaan | intake, diagnosis, approval, parts, QC, warranty | P2 |
| `professional_project` | Kontraktor, desain, percetakan proyek | Milestone/hasil kerja | quote, project, timesheet, progress billing, retention | P3 |
| `rental_asset` | Rental mobil, kamera, alat | Hak pakai aset | reservation, checkout/return, deposit, inspection, late fee | P2 |
| `accommodation` | Hotel, vila, homestay | Room-night | rate plan, reservation, check-in, folio, housekeeping | P3 |
| `pharmacy_lot` | Apotek/alkes | Produk lot terkendali | batch, ED, FEFO, recall, authorization, prescription ref | P3 |
| `ticketing_access` | Event, wahana, bioskop | Hak akses/seat | schedule, capacity, QR ticket, check-in, anti-reuse | P3 |
| `membership_subscription` | Gym, coworking, kursus | Entitlement periodik | plan, recurring invoice, credit, freeze, usage | P2 |
| `consignment_resale` | Titip jual/barang bekas | Barang pihak ketiga | ownership, commission, settlement, ageing | P2 |
| `custom_production` | Bakery order, mebel, konveksi | Barang belum dibuat | specification, BOM, work order, WIP, DP, QC | P3 |
| `parking` | Parkir/valet | Durasi dan ruang | gate event, vehicle, progressive tariff, lost ticket | P4 |
| `fuel_energy` | SPBU/charging | Volume/meter | pump/meter, tank reconciliation, shift variance | P4 |
| `digital_voucher` | Pulsa, token, voucher | Kode/fulfilment digital | provider request, pending state, retry, reversal, secret masking | P2 |

### 6.2 Preset nonkomersial dan mode yang bukan “bisnis”

POSQ sebaiknya tidak memaksa seluruh pengguna mengaku sebagai toko. Tambahkan preset berikut dengan accounting behavior yang jelas.

| Kode preset | Penggunaan | Perbedaan utama |
|---|---|---|
| `general_flexible` | Usaha campuran atau belum terklasifikasi | Core checkout, invoice opsional, capability dipilih manual |
| `nonprofit_donation` | Yayasan, rumah ibadah, penggalangan dana | Donor, campaign, restricted fund, receipt donasi, tanpa HPP |
| `cooperative_member_store` | Koperasi anggota | member identity, simpanan/patronage reference, harga anggota |
| `public_service_fee` | Retribusi atau layanan publik | jenis layanan, nomor permohonan, tarif resmi, receipt bernomor, larangan diskon |
| `internal_issue` | Gudang internal, pantry, pengeluaran aset | cost center, requester, approval, issue/return tanpa revenue |
| `school_campus` | Kantin, pembayaran kegiatan, toko sekolah | student/member account, allowance, guardian top-up, restricted items |
| `event_free_access` | Registrasi kegiatan gratis | kuota dan check-in tanpa payment |
| `personal_simple_sale` | Bazar, garage sale, penjualan insidental | onboarding minimal, tanpa procurement kompleks |

Catatan: preset nonkomersial tidak berarti mengubah POSQ menjadi sistem ERP pemerintahan, sistem donasi lengkap, atau sistem akademik. POSQ hanya menangani penerimaan, pengeluaran, akses, atau distribusi pada titik layanan.

## 7. Hal yang Bukan Mode Bisnis

Fitur berikut harus dimodelkan sebagai capability, channel, deployment profile, atau organization policy. Jangan membuat kartu mode baru untuk setiap istilah.

| Istilah | Klasifikasi |
|---|---|
| Online shop, marketplace, social commerce | Sales channel |
| Dine-in, takeaway, delivery, pickup | Fulfilment method |
| B2B, B2C, B2G | Customer/pricing policy |
| Multi-outlet, franchise | Organization topology |
| Offline-first, cloud sync | Deployment/synchronization profile |
| Mobile POS, kiosk, self-checkout | Terminal profile |
| QR ordering | Order capture channel |
| Drive-thru | F&B fulfilment workflow |
| Laundry | Work-order service preset atau subpreset |
| Car wash | Queue-based service subpreset |
| Klinik | Appointment + regulated clinical extension; data medis di luar POS core |
| Sekolah | Member account + invoice/fee extension |

## 8. Matriks Kapabilitas Minimum

Keterangan: `W` wajib, `O` opsional, kosong berarti tidak diaktifkan secara default.

| Preset | Inventory | Serial | Lot/ED | Recipe/BOM | Booking | Resource/asset | Work order | AR/credit | Deposit | Entitlement |
|---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| Retail umum | W | O | O |  |  |  |  | O |  | O |
| Retail berserial | W | W |  |  | O | O | O | O | O |  |
| Grocery | W |  | W | O |  |  |  |  |  |  |
| Grosir | W | O | O |  |  |  |  | W | O |  |
| F&B cepat | W |  | O | W | O | O |  |  |  |  |
| F&B meja | W |  | O | W | O | W |  | O | O |  |
| Appointment | O |  | O |  | W | W | O | O | W | O |
| Repair | W | W | O | O | O | W | W | O | W |  |
| Rental | W | W |  |  | W | W | O | O | W | O |
| Hotel | O |  | O |  | W | W | W | O | W |  |
| Apotek | W | O | W |  |  |  |  | O |  |  |
| Ticketing |  |  |  |  | W | W |  |  | O | W |
| Membership | O |  |  |  | W | W |  | O | O | W |
| Konsinyasi | W | O | O |  |  |  |  | W | O |  |
| Produksi kustom | W | O | O | W | O | W | W | W | W |  |
| Donasi |  |  |  |  | O |  |  | O |  | O |
| Internal issue | W | O | O | O |  | O | O |  |  |  |

## 9. Model Data Target

### 9.1 Konfigurasi preset dan kapabilitas

```sql
business_presets(
  id, code, name, version, status, description,
  default_capabilities_json, created_at
)

capability_definitions(
  key, domain, schema_version, description,
  dependencies_json, conflicts_json, default_config_json
)

outlet_profiles(
  outlet_id, primary_preset_code, preset_version,
  config_version, activated_at, activated_by
)

outlet_capabilities(
  outlet_id, capability_key, enabled,
  config_json, source, updated_at, updated_by
)

outlet_profile_change_log(
  id, outlet_id, before_json, after_json,
  reason, approved_by, created_at
)
```

`source` bernilai `preset`, `user_override`, atau `plan_entitlement`. Resolver harus menentukan konfigurasi efektif secara deterministik.

### 9.2 Katalog generik

```sql
offerings(
  id, merchant_id, offering_type, name, sku,
  stock_policy, fulfilment_policy, pricing_policy,
  cost_policy, return_policy, active
)

offering_variants(...)
units_of_measure(...)
uom_conversions(...)
price_lists(...)
price_list_items(...)
tax_profiles(...)
promotion_rules(...)
```

Untuk migrasi aman, tabel `products` dapat dipertahankan lebih dahulu dan diberi kolom baru. Jangan melakukan rename besar pada fase pertama.

### 9.3 Inventory lanjutan

```sql
inventory_locations(...)
inventory_balances(...)
inventory_ledger(...)
inventory_lots(id, product_id, lot_no, expiry_at, supplier_id, received_at)
serialized_units(id, product_id, serial_no, imei1, imei2, condition, unit_cost, status)
asset_units(id, offering_id, asset_code, status, meter_value, maintenance_due_at)
stock_cost_layers(...)
stock_reservations(...)
```

Saldo hanyalah proyeksi. Ledger merupakan sumber penelusuran. Shopify juga memisahkan inventory item, location, dan inventory level, sehingga kuantitas selalu berhubungan dengan lokasi tertentu, bukan hanya produk global.

### 9.4 Order, pricing, dan pembayaran

```sql
orders(
  ..., order_kind, channel, fulfilment_method,
  pricing_policy_version, payment_status,
  fulfilment_status, workflow_status
)

order_lines(
  ..., offering_type_snapshot, unit_price_snapshot,
  tax_snapshot_json, discount_snapshot_json,
  cost_snapshot_json, fulfilment_ref_type, fulfilment_ref_id
)

payment_intents(...)
payments(...)
payment_allocations(...)
refunds(...)
refund_lines(...)
store_credits(...)
accounts_receivable(...)
```

Satu pembayaran dapat dialokasikan ke beberapa bill/invoice dan satu order dapat dibayar melalui beberapa metode. Jangan menyimpan seluruh semantik pada `orders.paid_total` saja.

### 9.5 Tabel ekstensi domain

```text
F&B           : dining_sessions, kitchen_tickets, courses, modifiers
Appointment   : staff, resources, schedules, appointments, no_show_events
Repair        : customer_assets, repair_tickets, diagnoses, estimates, work_logs
Rental        : rental_contracts, asset_checkouts, inspections, damage_charges
Hotel         : room_types, rooms, rate_plans, reservations, folios, folio_entries
Ticketing     : events, sessions, seats, tickets, access_scans
Membership    : plans, subscriptions, entitlements, entitlement_usages
Consignment   : consignors, consignment_intakes, ownership_units, settlements
Production    : bills_of_material, work_orders, material_issues, production_outputs
Donation      : campaigns, funds, donors, donations, fund_allocations
```

## 10. Mesin Alur Kerja dan State Machine

Satu kolom `status` generik tidak cukup. Setiap aggregate memiliki transisi resmi yang divalidasi backend.

```text
Retail order:
draft -> confirmed -> paid -> fulfilled -> returned/closed

F&B ticket:
open -> sent -> accepted -> preparing -> ready -> served -> closed

Appointment:
tentative -> confirmed -> checked_in -> in_service -> completed
          -> cancelled/no_show

Repair:
received -> diagnosing -> awaiting_approval -> approved
         -> waiting_parts -> repairing -> quality_check
         -> ready -> collected -> warranty_closed

Rental:
quoted -> reserved -> checked_out -> overdue -> returned
       -> inspected -> deposit_settled -> closed

Hotel reservation:
tentative -> confirmed -> checked_in -> checked_out -> closed
          -> cancelled/no_show

Ticket:
held -> issued -> paid -> checked_in -> consumed
     -> cancelled/refunded/expired
```

Setiap transisi memerlukan:

- actor dan permission;
- current version untuk optimistic concurrency;
- idempotency key;
- timestamp dan device;
- reason/approval jika sensitif;
- domain event dalam outbox yang sama;
- compensating action, bukan penghapusan sejarah.

## 11. Rancangan Onboarding Pemilihan Mode

### 11.1 Wizard berbasis pertanyaan operasional

Jangan meminta pengguna memahami istilah teknis. Gunakan pertanyaan berikut:

1. Apa yang terutama Anda kelola: barang, makanan, jasa, aset sewaan, kamar, tiket, membership, penerimaan dana, atau kombinasi?
2. Apakah barang dilacak per jumlah, berat, lot/kedaluwarsa, serial/IMEI, atau aset individual?
3. Apakah pelanggan perlu booking, antrean, meja, teknisi, ruangan, atau jadwal?
4. Apakah pembayaran selalu langsung, memakai DP, termin, piutang, deposit, atau berulang?
5. Apakah ada proses produksi, dapur, diagnosis, inspeksi, atau persetujuan pelanggan?
6. Apakah Anda mengelola satu terminal, beberapa terminal dalam satu outlet, atau banyak outlet?

### 11.2 Hasil wizard

```json
{
  "primary_preset": "retail_serialized",
  "preset_version": 1,
  "enabled_capabilities": [
    "inventory.quantity",
    "inventory.serial",
    "warranty",
    "trade_in",
    "repair_ticket",
    "installment_receivable"
  ],
  "deployment_profile": "standalone_sqlite"
}
```

Sebelum aktivasi, tampilkan ringkasan fitur, menu yang muncul, data tambahan yang wajib diisi, dan dampak apabila mode diubah.

### 11.3 Perubahan mode

Mode tidak boleh dikunci permanen, tetapi perubahan harus aman:

- menambah capability boleh dilakukan setelah validasi dependensi;
- menonaktifkan capability dilarang bila masih ada aggregate aktif, misalnya rental belum kembali;
- perubahan preset menghasilkan audit log;
- data lama tetap dapat dibaca melalui versi capability lama;
- migrasi konfigurasi harus memiliki preview dan rollback metadata.

## 12. Struktur Kode yang Disarankan

```text
apps/desktop/src-tauri/src/
├── core/
│   ├── money.rs
│   ├── session_context.rs
│   ├── tenant_scope.rs
│   ├── idempotency.rs
│   └── domain_event.rs
├── capabilities/
│   ├── registry.rs
│   ├── resolver.rs
│   ├── dependency.rs
│   └── policy.rs
├── domains/
│   ├── catalog/
│   ├── pricing/
│   ├── inventory/
│   ├── sales/
│   ├── payments/
│   ├── fnb/
│   ├── appointments/
│   ├── repair/
│   ├── rental/
│   ├── membership/
│   └── reporting/
├── infrastructure/
│   ├── db/
│   ├── outbox/
│   ├── hardware/
│   ├── backup/
│   └── control_plane/
└── commands/
```

Frontend:

```text
src/lib/
├── capabilities/
│   ├── capabilityStore.svelte.ts
│   ├── menuResolver.ts
│   └── routeGuard.ts
├── features/
│   ├── checkout/
│   ├── inventory/
│   ├── fnb/
│   ├── repair/
│   └── rental/
└── presets/
    ├── onboarding/
    └── settings/
```

File `pos.rs` yang mendekati 1.000 baris dan `inventory.rs` yang melebihi 2.000 baris perlu dipecah menurut use case. Pemecahan dilakukan setelah characterization tests tersedia agar refactor tidak mengubah perilaku tanpa sengaja.

## 13. Urutan Implementasi yang Aman

### Fase 0: Koreksi kebenaran sistem dan keamanan, 2 sampai 3 sprint

| ID | Tugas | Acceptance criteria |
|---|---|---|
| MM-000 | Ubah status proyek dari release-ready menjadi pre-alpha | Dokumen tidak lagi mengklaim fitur mock sebagai produksi |
| MM-001 | Putuskan SQLite/PostgreSQL melalui ADR | Runtime, dokumentasi, installer, backup, dan test konsisten |
| MM-002 | Hilangkan private signing key dari desktop | Pencarian binary/source tidak menemukan key server |
| MM-003 | Implementasikan `SessionContext` | Tidak ada command mutasi memakai `users LIMIT 1` |
| MM-004 | Backend-authoritative pricing | Manipulasi payload frontend tidak mengubah total sah |
| MM-005 | Backup dan restore database aktif | Backup berisi data nyata; drill restore memulihkan checksum dan row count |
| MM-006 | Pisahkan KDS dari checkout generik | Retail/jasa tidak pernah menghasilkan kitchen ticket |
| MM-007 | Characterization tests | Checkout, stock, refund, shift, F&B, license memiliki test failure mode |

### Fase 1: Capability kernel, 2 sprint

| ID | Tugas | Acceptance criteria |
|---|---|---|
| MM-100 | Tambah tabel preset/capability/outlet profile | Konfigurasi tersimpan lokal dan terversi |
| MM-101 | Buat capability registry Rust | Dependency dan conflict tervalidasi backend |
| MM-102 | Buat resolver effective capability | Hasil deterministik dari preset + override + entitlement |
| MM-103 | Ganti `localStorage` sebagai source of truth | UI membaca profile melalui Tauri command |
| MM-104 | Menu dan route resolver | Menu hanya muncul bila capability efektif aktif |
| MM-105 | Command guard | Memanggil command fitur nonaktif menghasilkan `CAPABILITY_DISABLED` |
| MM-106 | Onboarding wizard | User mendapat saran preset dan dapat mengoreksi sebelum aktivasi |

### Fase 2: Perkuat cluster Retail dan F&B, 3 sampai 5 sprint

Urutan: retail standard, retail serialized, F&B quick service, F&B table service, grocery, wholesale. Fokus pada retur parsial, exchange, serial, lot, UOM, modifier, resep, KDS routing, table workflow, price list, dan piutang.

### Fase 3: Jasa dan aset, 4 sampai 6 sprint

Urutan: appointment, repair/workshop, rental, membership, consignment, digital voucher. Masing-masing harus berupa vertical slice yang mencakup skema, service Rust, UI, permission, audit, report, dan test.

### Fase 4: Domain kompleks, setelah pilot

Urutan: custom production, ticketing, accommodation, pharmacy, professional project. Parkir dan energi dikerjakan sebagai integration-heavy pack setelah HAL perangkat matang.

### Fase 5: Preset nonkomersial

Bangun `general_flexible` terlebih dahulu, kemudian donation, cooperative member store, public service fee, internal issue, dan school/campus. Gunakan core yang sama; jangan menyalin checkout.

## 14. Definition of Done Setiap Mode

Sebuah mode tidak boleh diberi label “tersedia” hanya karena kartu muncul pada pengaturan. Mode dinyatakan selesai apabila:

1. preset dan capability dependency terversi;
2. migrasi data tersedia dan dapat diulang;
3. backend menolak command ketika capability tidak aktif;
4. state machine dan invariant diuji;
5. transaksi finansial dihitung backend;
6. inventory/capacity/entitlement movement dapat diaudit;
7. RBAC dan supervisor approval diterapkan di backend;
8. offline workflow utama berhasil tanpa control plane;
9. crash/retry tidak menggandakan transaksi;
10. laporan umum dan laporan domain merekonsiliasi ledger;
11. backup/restore memuat tabel domain;
12. restricted license mode memiliki allowlist dan blocklist;
13. UI keyboard, touch, dan accessibility check lulus;
14. dokumentasi operator dan recovery tersedia;
15. pilot merchant menyelesaikan skenario harian tanpa intervensi developer.

## 15. Test Matrix Minimum

| Kategori | Pengujian wajib |
|---|---|
| Capability | dependency, conflict, entitlement, outlet override, version migration |
| Pricing | tampered frontend, rounding, tax inclusive/exclusive, discount stacking, price list |
| Inventory | negative stock, concurrency, serial uniqueness, FEFO, weighted qty, reservation release |
| Workflow | valid/invalid transition, retry, cancellation, compensation, concurrent terminal |
| Payment | split tender, partial, overpayment, reversal, refund, deposit allocation |
| Offline | server down, restart mid-checkout, outbox retry, long offline period |
| Security | cross-outlet access, stale session, role escalation, supervisor approval reuse |
| Backup | encrypted backup, wrong key, corrupt file, pre-restore backup, full restore drill |
| Mode isolation | retail no KDS, F&B no serial unless enabled, donation no stock mutation |
| Reporting | order-payment-ledger reconciliation, refund net sales, tax and cash shift |

## 16. Prioritas Produk yang Disarankan

Jangan menjanjikan 20 mode sekaligus. Gunakan tingkat kesiapan:

| Tingkat | Mode |
|---|---|
| General Availability pertama | General Flexible, Retail Standard, F&B Quick Service |
| Beta terkontrol | Retail Serialized, F&B Table Service, Appointment, Repair |
| Early access | Grocery, Wholesale, Rental, Membership, Consignment, Digital Voucher |
| Research/pilot | Hotel, Pharmacy, Ticketing, Custom Production, Parking, Fuel/Energy |

Ini bukan pengurangan visi. Strategi ini menjaga agar kernel yang sama dibuktikan pada beberapa pola berbeda sebelum domain regulatif dan perangkat keras khusus ditambahkan.

## 17. Prompt Induk untuk Agent Coding Antigravity

Gunakan prompt berikut setelah blueprint ini ditambahkan ke repository sebagai `docs/POSQ_MULTIMODE_BLUEPRINT.md`.

```text
Anda bekerja pada repository POSQ. Baca seluruh instruksi repository dan
docs/POSQ_MULTIMODE_BLUEPRINT.md sebelum mengubah kode.

Aturan kerja:
1. Kerjakan hanya satu Task ID MM-* dalam satu sesi.
2. Nyatakan acceptance criteria sebelum coding.
3. Jangan mengubah local-first invariant.
4. Jangan menambah if/else global berdasarkan nama mode.
5. Mode adalah preset; backend beroperasi berdasarkan capability efektif.
6. Frontend tidak boleh menjadi sumber kebenaran harga, pajak, diskon, HPP,
   permission, session, license, atau capability.
7. Semua command mutasi wajib memakai SessionContext, tenant/outlet scope,
   permission guard, license guard, capability guard, dan idempotency key.
8. Setiap perubahan skema wajib memiliki forward migration, rollback/recovery
   note, backup impact, dan test migrasi dari database versi sebelumnya.
9. Jangan mengklaim mock/stub sebagai completed.
10. Update STATUS.md, DECISIONS.md, TASK_BACKLOG.md, dan ADR bila diperlukan.

Untuk Task ID yang dipilih:
- lakukan audit file yang terdampak;
- tulis characterization test terlebih dahulu bila menyentuh kode lama;
- implementasikan vertical slice terkecil;
- jalankan formatter, lint, frontend check/build, Rust tests, dan migration tests;
- laporkan file berubah, hasil test, risiko tersisa, dan next task;
- berhenti bila acceptance criteria belum dapat dipenuhi secara aman.
```

## 18. Prompt Pertama yang Disarankan

```text
Task MM-001: selesaikan kontradiksi database POSQ.

Baca AGENTS.md, seluruh ADR, DECISIONS.md, STATUS.md, DATA_MODEL.md,
LOCAL_POSTGRESQL_STRATEGY.md, BACKUP_KEY_RECOVERY.md, dan
POSQ_MULTIMODE_BLUEPRINT.md.

Jangan coding terlebih dahulu. Audit bukti aktual bahwa runtime memakai SQLite,
dokumen mewajibkan PostgreSQL, dan backup memakai pg_dump/mock.

Hasilkan:
1. decision matrix standalone SQLite vs per-device PostgreSQL vs outlet PostgreSQL;
2. rekomendasi arsitektur dan dampak installer, backup, concurrency, multi-terminal;
3. ADR pengganti yang eksplisit;
4. migration plan tanpa kehilangan data;
5. daftar task implementasi kecil beserta acceptance criteria dan test gate.

Tunggu persetujuan manusia terhadap ADR sebelum mengubah storage runtime.
```

Setelah MM-001 diputuskan, task implementasi pertama yang paling penting adalah MM-003 `SessionContext`, kemudian MM-004 backend-authoritative pricing, MM-005 backup nyata, dan MM-006 isolasi KDS. Capability kernel baru dimulai setelah empat risiko tersebut terkendali.

## 19. Keputusan yang Memerlukan Persetujuan Pemilik Produk

1. Apakah POSQ mempertahankan PostgreSQL sebagai satu-satunya database atau menerima profil SQLite standalone?
2. Apakah multi-terminal LAN merupakan target rilis pertama?
3. Mode mana yang menjadi tiga mode General Availability pertama?
4. Apakah kontrol plane harus selesai sebelum pilot berbayar?
5. Apakah data kesehatan, resep, atau identitas sensitif akan diproses dalam POSQ atau hanya direferensikan ke sistem eksternal?
6. Apakah paket domain khusus dijual sebagai add-on atau termasuk paket langganan?
7. Apakah perubahan mode boleh dilakukan oleh owner sendiri atau memerlukan migration assistant?

## 20. Rekomendasi Akhir

POSQ tidak membutuhkan puluhan halaman kasir yang terpisah. POSQ membutuhkan satu core transaksi yang benar, beberapa mesin alur kerja yang terisolasi, dan konfigurasi kapabilitas yang tersimpan serta dipaksakan backend. Fokus pertama bukan menambahkan kartu mode baru, melainkan menghilangkan ketidaksesuaian database, mock backup, private signing key, konteks user palsu, harga yang dipercaya dari frontend, dan KDS yang bocor ke seluruh transaksi.

Setelah fondasi tersebut dibenahi, model **preset + capability + workflow + outlet profile** dapat mencakup bisnis komersial, usaha campuran, dan penggunaan nonkomersial tanpa membuat kode POSQ menjadi kumpulan percabangan yang rapuh. Inilah jalur paling realistis agar POSQ benar-benar luas cakupannya dan tetap dapat dipelihara oleh manusia maupun coding agent.

## Referensi Teknis Utama

- Repository POSQ, commit `702420d54b027880acd8cdd64448787ed7486ec0`.
- Shopify Developer Documentation, model Inventory Item, Inventory Level, dan Location.
- Square Developer Documentation, model Orders API dan fulfilment.
- Dokumentasi resmi Tauri 2 dan SvelteKit untuk desktop static application.
- ADR, `DECISIONS.md`, `STATUS.md`, migrasi, dan source code lokal di repository POSQ.
