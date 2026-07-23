# Panduan Preset Bisnis POSQ (A - K)

Dokumen ini menjelaskan daftar **11 Preset DNA Bisnis** yang didukung secara penuh oleh aplikasi POSQ, beserta tata letak menu navigasi dan kapabilitas bawaan masing-masing.

---

## Cara Memilih & Mengubah Preset Bisnis

1. Buka aplikasi POSQ Desktop.
2. Navigasi ke menu **Pengaturan -> Mode Bisnis** (`/settings/business-mode`).
3. Pilih salah satu kartu dari 11 Preset DNA Bisnis yang tersedia.
4. Klik **"Aktifkan Preset Ini"**.
5. Sistem akan menyimpan konfigurasi ke database SQLite lokal dan memperbarui bilah navigasi utama secara otomatis.

---

## Daftar Preset & Spesifikasi Menu Navigasi (A - K)

### A. Mode Umum & Fleksibel (`general_flexible`)
- **Deskripsi**: Modul POS universal dengan kapabilitas fleksibel untuk segala jenis usaha umum.
- **Menu Navigasi**:
  1. `🛒 Kasir POS` (`/pos`)
  2. `🗃️ Produk & Kategori` (`/inventory`)
  3. `👥 Pelanggan & Supplier` (`/customers`)
  4. `💰 Kas & Pengeluaran` (`/finances`)
  5. `📊 Laporan Penjualan` (`/reports`)
  6. `🔑 Kas & Shift` (`/shift`)
  7. `⚙️ Pengaturan` (`/settings`)

### B. Toko Retail / Minimarket (`retail_standard`)
- **Deskripsi**: Toko kelontong, minimarket, barcode & harga grosir/ecer.
- **Menu Navigasi**:
  1. `🛒 POS Kasir Retail` (`/pos`)
  2. `🗃️ Produk, Barcode & Opname` (`/inventory`)
  3. `👥 Pelanggan & Member` (`/customers`)
  4. `💰 Pembelian & Retur` (`/finances`)
  5. `📊 Laporan Margin & Best Seller` (`/reports`)
  6. `🔑 Kas Shift` (`/shift`)
  7. `⚙️ Pengaturan` (`/settings`)

### C. Retail Berserial / Elektronik (`retail_serialized`)
- **Deskripsi**: Toko HP, laptop, unit ber-IMEI, garansi & servis perbaikan.
- **Menu Navigasi**:
  1. `🛒 POS Kasir Serial` (`/pos`)
  2. `📱 Stok IMEI & Garansi` (`/inventory`)
  3. `🔧 Servis & Garansi` (`/repairs`)
  4. `👥 Pelanggan & Cicilan` (`/customers`)
  5. `📊 Laporan Serial & Sales` (`/reports`)
  6. `🔑 Kas Shift` (`/shift`)
  7. `⚙️ Pengaturan` (`/settings`)

### D. F&B Stand / Kedai Kopi (`fnb_quick_service`)
- **Deskripsi**: Kedai cepat saji, kopi, antrian & tampilan dapur/bar.
- **Menu Navigasi**:
  1. `🛒 POS Cepat F&B` (`/pos`)
  2. `🍳 Tampilan Dapur / Bar (KDS)` (`/kds`)
  3. `🗃️ Resep & Bahan Baku` (`/inventory`)
  4. `📊 Penjualan per Menu & Jam Ramai` (`/reports`)
  5. `🔑 Kas Shift` (`/shift`)
  6. `⚙️ Pengaturan` (`/settings`)

### E. Restoran Meja / Fine Dining (`fnb_table_service`)
- **Deskripsi**: Restoran meja, reservasi, split bill & KDS dapur.
- **Menu Navigasi**:
  1. `🛒 POS Order Meja` (`/pos`)
  2. `🍽️ Peta Meja & Reservasi` (`/tables`)
  3. `🍳 Kitchen Display (KDS)` (`/kds`)
  4. `🗃️ Resep & Purchasing Bahan` (`/inventory`)
  5. `📊 Laporan per Meja & Pelayan` (`/reports`)
  6. `🔑 Kas Shift` (`/shift`)
  7. `⚙️ Pengaturan` (`/settings`)

### F. Yayasan & Penggalangan Dana (`nonprofit_donation`)
- **Deskripsi**: Penerimaan donasi, zakat, infaq, campaign & penerima manfaat (tanpa HPP).
- **Menu Navigasi**:
  1. `💚 Penerimaan Donasi & Kwitansi` (`/donations`)
  2. `👥 Data Donatur & Mustahik` (`/customers`)
  3. `💰 Penyaluran Dana & Proposal` (`/finances`)
  4. `📊 Laporan Dana Masuk-Keluar` (`/reports`)
  5. `🔑 Kas & Rekonsiliasi Bank` (`/shift`)
  6. `⚙️ Pengaturan` (`/settings`)

### G. Toko Koperasi Anggota (`cooperative_member_store`)
- **Deskripsi**: Toko koperasi, data anggota, simpanan, potong gaji & SHU.
- **Menu Navigasi**:
  1. `🛒 POS Kasir Koperasi` (`/pos`)
  2. `🏢 Data Anggota & SHU` (`/cooperative`)
  3. `🗃️ Produk & Harga Anggota` (`/inventory`)
  4. `💰 Simpanan & Potong Gaji` (`/finances`)
  5. `📊 Penjualan Anggota vs Umum` (`/reports`)
  6. `🔑 Kas Shift` (`/shift`)
  7. `⚙️ Pengaturan` (`/settings`)

### H. Retribusi / Layanan Publik (`public_service_fee`)
- **Deskripsi**: Loket retribusi resmi, nomor antrean, karcis & larangan diskon.
- **Menu Navigasi**:
  1. `🏛️ Loket Retribusi & Karcis` (`/retribusi`)
  2. `👥 Data Wajib Bayar` (`/customers`)
  3. `📊 Laporan Pendapatan Loket` (`/reports`)
  4. `🔑 Setoran Petugas & Shift` (`/shift`)
  5. `⚙️ Pengaturan Tarif` (`/settings`)

### I. Pengeluaran Barang Internal (`internal_issue`)
- **Deskripsi**: Disposisi gudang internal, pantry, cost center & transfer.
- **Menu Navigasi**:
  1. `📦 Disposisi & Permintaan Gudang` (`/internal-issue`)
  2. `🗃️ Master Barang & Gudang` (`/inventory`)
  3. `📊 Pemakaian per Departemen` (`/reports`)
  4. `⚙️ Pengaturan Departemen` (`/settings`)

### J. Kantin / Toko Sekolah (`school_campus`)
- **Deskripsi**: Kantin sekolah, saldo wallet RFID siswa, limit harian & pre-order.
- **Menu Navigasi**:
  1. `🛒 POS Kasir Kantin` (`/pos`)
  2. `🏫 Wallet Siswa & Pre-Order` (`/school`)
  3. `🗃️ Produk & Menu Kantin` (`/inventory`)
  4. `📊 Penjualan per Kelas & Siswa` (`/reports`)
  5. `🔑 Kas Shift` (`/shift`)
  6. `⚙️ Pengaturan` (`/settings`)

### K. Parkir & Gate Sistem (`parking`)
- **Deskripsi**: Gate tiket masuk/keluar, tarif progresif, lost ticket & langganan.
- **Menu Navigasi**:
  1. `🚗 Gate Masuk & Keluar Parkir` (`/parking`)
  2. `👥 Pelanggan Langganan` (`/customers`)
  3. `📊 Trafik & Pendapatan Parkir` (`/reports`)
  4. `🔑 Setoran & Shift Petugas` (`/shift`)
  5. `⚙️ Pengaturan Tarif Parkir` (`/settings`)
