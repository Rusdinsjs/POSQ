# Struktur Menu POS Berdasarkan Preset Bisnis (POSQ Engine)

Dokumen ini mendefinisikan arsitektur menu dan peta kapabilitas (**Feature Flags / Capabilities**) untuk 12 Preset DNA Bisnis pada aplikasi **POSQ**.

Setiap preset menggunakan *Capability Kernel* dinamis sehingga modul yang tidak relevan otomatis disembunyikan tanpa perlu membuat aplikasi terpisah untuk setiap jenis bisnis.

---

## A. Menu Inti Bersama (Core Shared Capabilities)

Menu berikut menjadi fondasi aplikasi, namun visibilitasnya dikendalikan oleh *Feature Flags / Capabilities* sesuai preset yang dipilih:

1. **Dashboard**
   - Ringkasan transaksi
   - Pendapatan hari ini
   - Transaksi terakhir
   - Status kasir / shift
   - Notifikasi stok & operasional
2. **Transaksi (POS Kasir)**
   - Transaksi baru
   - Riwayat transaksi
   - Transaksi tertunda (Hold / Recall)
   - Pembatalan & Refund
   - Cetak ulang struk
3. **Produk / Layanan & Persediaan**
   - Daftar produk / layanan
   - Kategori & Merek
   - Harga & Harga bertingkat
   - Pajak & Service charge
   - Diskon & Promosi
   - Varian, Satuan & Konversi
4. **Pelanggan / Member / Donatur**
   - Data pelanggan / member / donatur / wajib bayar
   - Riwayat transaksi
   - Poin & Loyalitas
   - Saldo deposit atau limit kredit
5. **Kas & Shift**
   - Buka kasir / shift (Modal awal)
   - Tutup kasir / shift
   - Kas masuk & kas keluar (Cash drop)
   - Rekonsiliasi kasir
   - Serah terima shift
6. **Laporan**
   - Laporan penjualan
   - Laporan pembayaran
   - Laporan pajak & service charge
   - Kinerja kasir / operator
   - Pergerakan produk / layanan
   - Audit aktivitas sistem
7. **Pengguna & Hak Akses**
   - Pengguna (Users)
   - Role (Owner, Manager, Cashier, Waiter, Technician)
   - Hak akses menu & kapabilitas
   - Persetujuan (Approval) transaksi khusus
8. **Pengaturan**
   - Profil usaha & Outlet
   - Printer & Perangkat keras (Hardware)
   - Metode pembayaran
   - Pajak & Biaya layanan
   - Format nomor transaksi
   - Integrasi API & Sync Cloud

---

## B. Daftar 12 Preset Bisnis & Spesifikasi Menu Navigasi

### 1. Mode Umum & Fleksibel (`general_flexible`)
- **Kategori**: Umum
- **Ikon**: ⚡
- **Deskripsi**: Preset untuk usaha yang belum memiliki alur operasional khusus dengan kapabilitas universal.
- **Default Capabilities**: `inventory.basic`, `checkout.basic`, `shift.basic`
- **Menu Navigasi UI**:
  - `🛒 Kasir POS` (`/pos`)
  - `🗃️ Produk & Kategori` (`/inventory`)
  - `👥 Pelanggan & Supplier` (`/customers`)
  - `💰 Kas & Pengeluaran` (`/finances`)
  - `📊 Laporan Penjualan` (`/reports`)
  - `🔑 Kas & Shift` (`/shift`)
  - `⚙️ Pengaturan` (`/settings`)

**Fitur Operasional Detail**:
- **Dashboard**: Penjualan hari ini, Transaksi, Laba kotor, Produk terlaris, Stok menipis, Piutang pelanggan.
- **Kasir**: Penjualan barang & jasa, Pencarian produk, Scan barcode, Diskon transaksi, Pembayaran multi-metode, Simpan transaksi, Refund.
- **Produk & Jasa**: Produk fisik, Produk non-stok, Jasa, Paket/bundling, Kategori, Varian, Harga bertingkat.
- **Persediaan**: Stok masuk/keluar, Transfer stok, Penyesuaian stok, Stok opname, Kartu stok.
- **Pembelian**: Purchase order, Penerimaan barang, Retur pembelian, Tagihan supplier.
- **Keuangan**: Pendapatan lain, Pengeluaran, Kas & Bank, Rekonsiliasi.

---

### 2. Toko Retail / Minimarket (`retail_standard`)
- **Kategori**: Retail
- **Ikon**: 🛍️
- **Deskripsi**: Preset untuk toko kelontong, minimarket, toko fashion dengan volume transaksi tinggi & produk berbarcode.
- **Default Capabilities**: `inventory.basic`, `inventory.barcode`, `checkout.basic`, `checkout.refund`, `promotion.basic`, `shift.basic`
- **Menu Navigasi UI**:
  - `🛒 POS Kasir Retail` (`/pos`)
  - `🗃️ Produk, Barcode & Opname` (`/inventory`)
  - `👥 Pelanggan & Member` (`/customers`)
  - `💰 Pembelian & Retur` (`/finances`)
  - `📊 Laporan Margin & Best Seller` (`/reports`)
  - `🔑 Kas Shift` (`/shift`)
  - `⚙️ Pengaturan` (`/settings`)

**Fitur Operasional Detail**:
- **Kasir Cepat**: Scan barcode, Pencarian PLU, Ubah kuantitas cepat, Cek harga, Tahan & panggil transaksi, Diskon supervisor, Split payment (cash/card/QRIS/e-wallet), Retur & Cetak struk.
- **Produk & Promosi**: Master produk, Barcode generator, Satuan & konversi, Harga grosir/cabang, Diskon bertingkat, Beli X Gratis Y, Bundling, Kupon & Voucher.
- **Persediaan**: Stok per outlet, Batch & Kedaluwarsa, Transfer antar gudang, Stok opname barcode scanner, Loss/shrinkage tracking, Minimum stock alert.
- **Member**: Pendaftaran member, Poin loyalitas, Harga khusus member, Riwayat belanja.

---

### 3. Retail Berserial / Elektronik (`retail_serialized`)
- **Kategori**: Retail
- **Ikon**: 📱
- **Deskripsi**: Preset untuk toko elektronik, HP, komputer, alat teknik, & barang ber-IMEI / bergaransi.
- **Default Capabilities**: `inventory.basic`, `inventory.serial`, `checkout.basic`, `checkout.trade_in`, `warranty.tracking`, `repair.tickets`, `shift.basic`
- **Menu Navigasi UI**:
  - `🛒 Kasir POS Serial` (`/pos`)
  - `📱 Stok IMEI & Garansi` (`/inventory`)
  - `🔧 Servis & Garansi` (`/repairs`)
  - `👥 Pelanggan & Cicilan` (`/customers`)
  - `📊 Laporan Serial & Sales` (`/reports`)
  - `🔑 Kas Shift` (`/shift`)
  - `⚙️ Pengaturan` (`/settings`)

**Fitur Operasional Detail**:
- **Kasir Serial**: Scan barcode & IMEI/Serial Number, Validasi nomor serial tersedia, Registrasi pelanggan & Garansi otomatis pada cetakan struk.
- **Nomor Serial & Garansi**: Status serial (Tersedia, Terjual, Dikembalikan, Dalam Servis, Rusak), Masa garansi toko/distributor, Registrasi & Klaim garansi.
- **Servis & RMA**: Penerimaan unit servis, Diagnosa, Estimasi biaya, Penggunaan spare part, Penugasan teknisi, Status perbaikan, Pengembalian ke distributor.

---

### 4. F&B Stand / Kedai Kopi (`fnb_quick_service`)
- **Kategori**: F&B
- **Ikon**: ☕
- **Deskripsi**: Preset untuk coffee shop, booth makanan, bakery, food truck, & quick-service restaurant.
- **Default Capabilities**: `inventory.basic`, `recipe.bom`, `fnb.kds`, `fnb.modifiers`, `checkout.basic`, `shift.basic`
- **Menu Navigasi UI**:
  - `🛒 POS Cepat F&B` (`/pos`)
  - `🍳 Tampilan Dapur / Bar (KDS)` (`/kds`)
  - `🗃️ Resep & Bahan Baku` (`/inventory`)
  - `📊 Penjualan per Menu & Jam Ramai` (`/reports`)
  - `🔑 Kas Shift` (`/shift`)
  - `⚙️ Pengaturan` (`/settings`)

**Fitur Operasional Detail**:
- **Pesanan & Modifiers**: Dine-in / Takeaway / Delivery, Nomor antrean, Nama pelanggan, Ukuran (Small/Medium/Large), Hot/Ice, Topping, Level gula & es.
- **Kitchen Display System (KDS)**: Antrean dapur/bar realtime, Status (Baru -> Sedang dibuat -> Siap diambil -> Selesai), Cetak kitchen ticket.
- **Resep & Bahan Baku (BOM)**: Resep per menu, Deductions bahan otomatis saat checkout, Satuan & Konversi bahan, Pencatatan Waste / Spoilage.

---

### 5. Restoran Meja / Fine Dining (`fnb_table_service`)
- **Kategori**: F&B
- **Ikon**: 🍔
- **Deskripsi**: Preset untuk restoran meja dengan layout denah meja, captain order, reservasi, & split bill.
- **Default Capabilities**: `inventory.basic`, `recipe.bom`, `fnb.kds`, `fnb.table`, `fnb.modifiers`, `fnb.split_bill`, `checkout.basic`, `shift.basic`
- **Menu Navigasi UI**:
  - `🛒 POS Order Meja` (`/pos`)
  - `🍽️ Peta Meja & Reservasi` (`/tables`)
  - `🍳 Kitchen Display (KDS)` (`/kds`)
  - `🗃️ Resep & Purchasing Bahan` (`/inventory`)
  - `📊 Laporan per Meja & Pelayan` (`/reports`)
  - `🔑 Kas Shift` (`/shift`)
  - `⚙️ Pengaturan` (`/settings`)

**Fitur Operasional Detail**:
- **Denah Meja & Reservasi**: Visual layout meja indoor/outdoor, Status meja (Kosong/Terisi/Reserved/Perlu Dibersihkan), Gabung/Pindah meja, Waitlist, Calendar reservasi.
- **Pesanan Meja & Captain Order**: Buka meja, Ordering via tablet waiter, Catatan alergi, Course ordering (Appetizer/Main/Dessert), Split bill per tamu/item, Service charge.

---

### 6. Yayasan & Penggalangan Dana (`nonprofit_donation`)
- **Kategori**: Nonprofit
- **Ikon**: 💚
- **Deskripsi**: Preset berorientasi pada penerimaan donasi, zakat, infaq, pengelolaan kampanye, & penyaluran dana (tanpa HPP).
- **Default Capabilities**: `donation.receipt`, `checkout.basic`, `shift.basic`
- **Menu Navigasi UI**:
  - `💚 Penerimaan Donasi & Kwitansi` (`/donations`)
  - `👥 Data Donatur & Mustahik` (`/customers`)
  - `💰 Penyaluran Dana & Proposal` (`/finances`)
  - `📊 Laporan Dana Masuk-Keluar` (`/reports`)
  - `🔑 Kas & Rekonsiliasi Bank` (`/shift`)
  - `⚙️ Pengaturan` (`/settings`)

**Fitur Operasional Detail**:
- **Penerimaan Donasi**: Donasi tunai, Transfer bank, QRIS, Donasi barang, Donasi rutin & Cetak kuitansi resmi donatur.
- **Program & Penyaluran**: Dana terikat & tidak terikat, Target kampanye, Permohonan pencairan dana, Approval, Data penerima manfaat (Mustahik), Laporan pertanggungjawaban.

---

### 7. Toko Koperasi Anggota (`cooperative_member_store`)
- **Kategori**: Koperasi
- **Ikon**: 🏢
- **Deskripsi**: Preset untuk koperasi karyawan, koperasi sekolah, atau toko khusus anggota.
- **Default Capabilities**: `inventory.basic`, `member.patronage`, `checkout.basic`, `shift.basic`
- **Menu Navigasi UI**:
  - `🛒 POS Kasir Koperasi` (`/pos`)
  - `🏢 Data Anggota & SHU` (`/cooperative`)
  - `🗃️ Produk & Harga Anggota` (`/inventory`)
  - `💰 Simpanan & Potong Gaji` (`/finances`)
  - `📊 Penjualan Anggota vs Umum` (`/reports`)
  - `🔑 Kas Shift` (`/shift`)
  - `⚙️ Pengaturan` (`/settings`)

**Fitur Operasional Detail**:
- **Kasir & Benefit Anggota**: Penjualan tunai & kredit anggota, Harga khusus anggota, Potong saldo, Potong gaji, Batas limit belanja harian/bulanan.
- **Simpanan & SHU**: Simpanan pokok/wajib/sukarela, Perhitungan poin kontribusi belanja & Rekapitulasi pembagian SHU (Sisa Hasil Usaha).

---

### 8. Retribusi / Layanan Publik (`public_service_fee`)
- **Kategori**: Publik
- **Ikon**: 🏛️
- **Deskripsi**: Preset untuk loket retribusi pasar, sampah, terminal, perizinan, atau fasilitas publik pemerintah.
- **Default Capabilities**: `public_service.receipt`, `checkout.no_discount`, `shift.basic`
- **Menu Navigasi UI**:
  - `🏛️ Loket Retribusi & Karcis` (`/retribusi`)
  - `👥 Data Wajib Bayar` (`/customers`)
  - `📊 Laporan Pendapatan Loket` (`/reports`)
  - `🔑 Setoran Petugas & Shift` (`/shift`)
  - `⚙️ Pengaturan Tarif` (`/settings`)

**Fitur Operasional Detail**:
- **Pelayanan Loket**: Registrasi pemohon/wajib bayar, Nomor antrean, Verifikasi berkas, Kalkulasi tarif resmi (tanpa fitur diskon), Cetak bukti pembayaran/karcis.
- **Tagihan & Petugas Lapangan**: Penerbitan tagihan berkala, Tunggakan & Denda, Penagihan mobile oleh petugas lapangan, Rekonsiliasi setoran kas.

---

### 9. Pengeluaran Barang Internal (`internal_issue`)
- **Kategori**: Gudang
- **Ikon**: 📦
- **Deskripsi**: Sistem permintaan & distribusi barang internal (pantry, ATK, suku cadang pabrik) tanpa transaksi penjualan tunai.
- **Default Capabilities**: `inventory.basic`, `internal.issue`
- **Menu Navigasi UI**:
  - `📦 Disposisi & Permintaan Gudang` (`/internal-issue`)
  - `🗃️ Master Barang & Gudang` (`/inventory`)
  - `📊 Pemakaian per Departemen` (`/reports`)
  - `⚙️ Pengaturan Departemen` (`/settings`)

**Fitur Operasional Detail**:
- **Permintaan & Approval**: Form permohonan barang per departemen/cost center/proyek, Workflow persetujuan atasan & manajer gudang.
- **Pengeluaran & Retur**: Picking list barang, Scan barcode serah terima, Bukti pengeluaran barang (Goods Issue Note), Retur sisa/barang rusak ke gudang.

---

### 10. Kantin / Toko Sekolah (`school_campus`)
- **Kategori**: Pendidikan
- **Ikon**: 🏫
- **Deskripsi**: Preset untuk transaksi siswa, guru, kartu RFID sekolah, saldo e-wallet, & pembatasan belanja harian.
- **Default Capabilities**: `inventory.basic`, `student.allowance`, `checkout.basic`, `shift.basic`
- **Menu Navigasi UI**:
  - `🛒 POS Kasir Kantin` (`/pos`)
  - `🏫 Wallet Siswa & Pre-Order` (`/school`)
  - `🗃️ Produk & Menu Kantin` (`/inventory`)
  - `📊 Penjualan per Kelas & Siswa` (`/reports`)
  - `🔑 Kas Shift` (`/shift`)
  - `⚙️ Pengaturan` (`/settings`)

**Fitur Operasional Detail**:
- **Kasir Kantin & Kartu Siswa**: Tap kartu RFID siswa/guru, Pembayaran saldo dompet digital, Limit belanja harian/per transaksi, Peringatan alergi makanan.
- **Top Up & Subsidi**: Top up saldo tunai/transfer, Riwayat mutasi saldo, Program kuota makan gratis/subsidi sekolah.

---

### 11. Parkir & Gate Sistem (`parking`)
- **Kategori**: Parkir
- **Ikon**: 🚗
- **Deskripsi**: Preset untuk sistem pengelola parkir kendaraan, gate palang otomatis, tiket barcode, RFID & kamera pelat nomor (LPR).
- **Default Capabilities**: `parking.gate`, `parking.tariff`, `shift.basic`
- **Menu Navigasi UI**:
  - `🚗 Gate Masuk & Keluar Parkir` (`/parking`)
  - `👥 Pelanggan Langganan` (`/customers`)
  - `📊 Trafik & Pendapatan Parkir` (`/reports`)
  - `🔑 Setoran & Shift Petugas` (`/shift`)
  - `⚙️ Pengaturan Tarif Parkir` (`/settings`)

**Fitur Operasional Detail**:
- **Gate Masuk & Keluar**: Ambil tiket barcode / Scan RFID member, Hitung durasi otomatis, Kalkulasi tarif progresif/flat/inap/grace period, Integrasi palang gate & kamera LPR.
- **Member & Tiket Bermasalah**: Langganan member bulanan, Penanganan tiket hilang/rusak, Manual barrier override log oleh supervisor.

---

### 12. SPBU / EV Charging (`fuel_energy`)
- **Kategori**: Energi
- **Ikon**: ⛽
- **Deskripsi**: Preset untuk stasiun pengisian bahan bakar umum (SPBU), pengisian daya kendaraan listrik (EV Charging), & manajemen fleet pelanggan.
- **Default Capabilities**: `fuel.dispenser`, `fuel.reconciliation`, `checkout.basic`, `shift.basic`
- **Menu Navigasi UI**:
  - `⛽ SPBU & Meter` (`/fuel`)
  - `🛒 Kasir BBM` (`/pos`)
  - `🗃️ Tangki & Stok` (`/inventory`)
  - `📊 Laporan Shift SPBU` (`/reports`)
  - `🔑 Kas & Shift` (`/shift`)
  - `⚙️ Pengaturan Dispenser` (`/settings`)

**Fitur Operasional Detail**:
- **Dispenser & Charging**: Monitor status pompa, nozzle, & connector EV, Transaksi per liter/rupiah/kWh, Catatan plat nomor & odometer armada.
- **Tangki, Meter & Shift**: Tank dipping (stok fisik tangki pencetan), Rekonsiliasi totalizer meter pompa, Monitoring loss/gain BBM per shift operator.

---

## C. Rekomendasi Struktur Sidebar Dinamis & Konfigurasi JSON

Urutan menu sidebar dinamis dalam aplikasi dikelompokkan ke dalam 6 kelompok utama:

1. **Operasional**: Dashboard, POS Kasir, KDS, Denah Meja, Gate Parkir, Dispenser BBM, Permintaan Barang.
2. **Master Data**: Master Produk, Layanan, Tarif, Pelanggan, Member, Donatur, Wajib Bayar, Kartu Siswa.
3. **Persediaan**: Stok Opname, Transfer Gudang, Resep (BOM), Batch & Kedaluwarsa, Tangki Penampungan.
4. **Keuangan**: Kas & Shift, Pembelian, Pengeluaran, Piutang, Simpanan & SHU, Penyaluran Donasi.
5. **Analitik**: Laporan Penjualan, Margin, Food Cost, Audit Trail, Dashboard Manajemen.
6. **Administrasi**: Pengguna, Hak Akses, Profil Outlet, Perangkat (Printer/Gate), Pengaturan Mode Bisnis.

### Contoh Manifest Konfigurasi Preset (`fnb_table_service`):

```json
{
  "preset": "fnb_table_service",
  "name": "Restoran Meja / Fine Dining",
  "category": "F&B",
  "version": 1,
  "default_capabilities": [
    "inventory.basic",
    "recipe.bom",
    "fnb.kds",
    "fnb.table",
    "fnb.modifiers",
    "fnb.split_bill",
    "checkout.basic",
    "shift.basic"
  ],
  "modules": {
    "table_management": true,
    "reservation": true,
    "kitchen_display": true,
    "recipe_bom": true,
    "split_bill": true,
    "serial_number": false,
    "membership": true,
    "accounts_receivable": true
  }
}
```

---

> **Catatan Pengembang**:
> Seluruh preset terdaftar pada registri frontend (`apps/desktop/src/lib/capabilities/presetRegistry.ts`) dan kernel kapabilitas Rust backend (`apps/desktop/src-tauri/src/capabilities.rs`).
