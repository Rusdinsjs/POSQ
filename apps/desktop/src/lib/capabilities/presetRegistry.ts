export interface SubModuleItem {
  id: string;
  label: string;
  href: string;
  icon: string;
  requiredCapability?: string;
  allowedRoles?: string[];
  description?: string;
}

export interface PresetConfig {
  code: string;
  name: string;
  category: string;
  icon: string;
  description: string;
  defaultCapabilities: string[];
  menuItems: SubModuleItem[];
}

export const PRESET_REGISTRY: Record<string, PresetConfig> = {
  general_flexible: {
    code: 'general_flexible',
    name: 'Mode Umum & Fleksibel',
    category: 'Umum',
    icon: '⚡',
    description: 'A. Modul POS universal dengan kapabilitas fleksibel',
    defaultCapabilities: ['inventory.basic', 'checkout.basic', 'shift.basic'],
    menuItems: [
      { id: 'pos', label: '🛒 Kasir POS', href: '/pos', icon: 'shopping-cart', requiredCapability: 'checkout.basic' },
      { id: 'inventory', label: '🗃️ Produk & Kategori', href: '/inventory', icon: 'package', requiredCapability: 'inventory.basic' },
      { id: 'customers', label: '👥 Pelanggan & Supplier', href: '/customers', icon: 'users' },
      { id: 'finances', label: '💰 Kas & Pengeluaran', href: '/finances', icon: 'credit-card' },
      { id: 'reports', label: '📊 Laporan Penjualan', href: '/reports', icon: 'bar-chart-2' },
      { id: 'shift', label: '🔑 Kas & Shift', href: '/shift', icon: 'clock', requiredCapability: 'shift.basic' },
      { id: 'settings', label: '⚙️ Pengaturan', href: '/settings', icon: 'settings' },
    ],
  },

  retail_standard: {
    code: 'retail_standard',
    name: 'Toko Retail / Minimarket',
    category: 'Retail',
    icon: '🛍️',
    description: 'B. Toko kelontong, minimarket, barcode & harga grosir',
    defaultCapabilities: ['inventory.basic', 'inventory.barcode', 'checkout.basic', 'checkout.refund', 'shift.basic'],
    menuItems: [
      { id: 'pos', label: '🛒 POS Kasir Retail', href: '/pos', icon: 'shopping-cart', requiredCapability: 'checkout.basic' },
      { id: 'inventory', label: '🗃️ Produk, Barcode & Opname', href: '/inventory', icon: 'package', requiredCapability: 'inventory.basic' },
      { id: 'customers', label: '👥 Pelanggan & Member', href: '/customers', icon: 'users' },
      { id: 'finances', label: '💰 Pembelian & Retur', href: '/finances', icon: 'credit-card' },
      { id: 'reports', label: '📊 Laporan Margin & Best Seller', href: '/reports', icon: 'bar-chart-2' },
      { id: 'shift', label: '🔑 Kas Shift', href: '/shift', icon: 'clock', requiredCapability: 'shift.basic' },
      { id: 'settings', label: '⚙️ Pengaturan', href: '/settings', icon: 'settings' },
    ],
  },

  retail_serialized: {
    code: 'retail_serialized',
    name: 'Retail Berserial / Elektronik',
    category: 'Retail',
    icon: '📱',
    description: 'C. Toko HP, laptop, unit ber-IMEI, garansi & servis',
    defaultCapabilities: ['inventory.basic', 'inventory.serial', 'checkout.basic', 'checkout.trade_in', 'warranty.tracking', 'repair.tickets', 'shift.basic'],
    menuItems: [
      { id: 'pos', label: '🛒 POS Kasir Serial', href: '/pos', icon: 'shopping-cart', requiredCapability: 'checkout.basic' },
      { id: 'inventory', label: '📱 Stok IMEI & Garansi', href: '/inventory', icon: 'package', requiredCapability: 'inventory.basic' },
      { id: 'repairs', label: '🔧 Servis & Garansi', href: '/repairs', icon: 'wrench', requiredCapability: 'repair.tickets' },
      { id: 'customers', label: '👥 Pelanggan & Cicilan', href: '/customers', icon: 'users' },
      { id: 'reports', label: '📊 Laporan Serial & Sales', href: '/reports', icon: 'bar-chart-2' },
      { id: 'shift', label: '🔑 Kas Shift', href: '/shift', icon: 'clock', requiredCapability: 'shift.basic' },
      { id: 'settings', label: '⚙️ Pengaturan', href: '/settings', icon: 'settings' },
    ],
  },

  fnb_quick_service: {
    code: 'fnb_quick_service',
    name: 'F&B Stand / Kedai Kopi',
    category: 'F&B',
    icon: '☕',
    description: 'D. Kedai cepat saji, kopi, antrian & tampilan dapur/bar',
    defaultCapabilities: ['inventory.basic', 'recipe.bom', 'fnb.kds', 'fnb.modifiers', 'checkout.basic', 'shift.basic'],
    menuItems: [
      { id: 'pos', label: '🛒 POS Cepat F&B', href: '/pos', icon: 'shopping-cart', requiredCapability: 'checkout.basic' },
      { id: 'kds', label: '🍳 Tampilan Dapur / Bar (KDS)', href: '/kds', icon: 'utensils', requiredCapability: 'fnb.kds' },
      { id: 'inventory', label: '🗃️ Resep & Bahan Baku', href: '/inventory', icon: 'package', requiredCapability: 'inventory.basic' },
      { id: 'reports', label: '📊 Penjualan per Menu & Jam Ramai', href: '/reports', icon: 'bar-chart-2' },
      { id: 'shift', label: '🔑 Kas Shift', href: '/shift', icon: 'clock', requiredCapability: 'shift.basic' },
      { id: 'settings', label: '⚙️ Pengaturan', href: '/settings', icon: 'settings' },
    ],
  },

  fnb_table_service: {
    code: 'fnb_table_service',
    name: 'Restoran Meja / Fine Dining',
    category: 'F&B',
    icon: '🍔',
    description: 'E. Restoran meja, reservasi, split bill & KDS dapur',
    defaultCapabilities: ['inventory.basic', 'recipe.bom', 'fnb.kds', 'fnb.table', 'fnb.modifiers', 'fnb.split_bill', 'checkout.basic', 'shift.basic'],
    menuItems: [
      { id: 'pos', label: '🛒 POS Order Meja', href: '/pos', icon: 'shopping-cart', requiredCapability: 'checkout.basic' },
      { id: 'tables', label: '🍽️ Peta Meja & Reservasi', href: '/tables', icon: 'grid', requiredCapability: 'fnb.table' },
      { id: 'kds', label: '🍳 Kitchen Display (KDS)', href: '/kds', icon: 'utensils', requiredCapability: 'fnb.kds' },
      { id: 'inventory', label: '🗃️ Resep & Purchasing Bahan', href: '/inventory', icon: 'package', requiredCapability: 'inventory.basic' },
      { id: 'reports', label: '📊 Laporan per Meja & Pelayan', href: '/reports', icon: 'bar-chart-2' },
      { id: 'shift', label: '🔑 Kas Shift', href: '/shift', icon: 'clock', requiredCapability: 'shift.basic' },
      { id: 'settings', label: '⚙️ Pengaturan', href: '/settings', icon: 'settings' },
    ],
  },

  nonprofit_donation: {
    code: 'nonprofit_donation',
    name: 'Yayasan & Penggalangan Dana',
    category: 'Nonprofit',
    icon: '💚',
    description: 'F. Penerimaan donasi, zakat, infaq, campaign & penerima manfaat',
    defaultCapabilities: ['donation.receipt', 'checkout.basic', 'shift.basic'],
    menuItems: [
      { id: 'donations', label: '💚 Penerimaan Donasi & Kwitansi', href: '/donations', icon: 'heart', requiredCapability: 'donation.receipt' },
      { id: 'customers', label: '👥 Data Donatur & Mustahik', href: '/customers', icon: 'users' },
      { id: 'finances', label: '💰 Penyaluran Dana & Proposal', href: '/finances', icon: 'credit-card' },
      { id: 'reports', label: '📊 Laporan Dana Masuk-Keluar', href: '/reports', icon: 'bar-chart-2' },
      { id: 'shift', label: '🔑 Kas & Rekonsiliasi Bank', href: '/shift', icon: 'clock', requiredCapability: 'shift.basic' },
      { id: 'settings', label: '⚙️ Pengaturan', href: '/settings', icon: 'settings' },
    ],
  },

  cooperative_member_store: {
    code: 'cooperative_member_store',
    name: 'Toko Koperasi Anggota',
    category: 'Koperasi',
    icon: '🏢',
    description: 'G. Toko koperasi, data anggota, simpanan, potong gaji & SHU',
    defaultCapabilities: ['inventory.basic', 'member.patronage', 'checkout.basic', 'shift.basic'],
    menuItems: [
      { id: 'pos', label: '🛒 POS Kasir Koperasi', href: '/pos', icon: 'shopping-cart', requiredCapability: 'checkout.basic' },
      { id: 'cooperative', label: '🏢 Data Anggota & SHU', href: '/cooperative', icon: 'users', requiredCapability: 'member.patronage' },
      { id: 'inventory', label: '🗃️ Produk & Harga Anggota', href: '/inventory', icon: 'package', requiredCapability: 'inventory.basic' },
      { id: 'finances', label: '💰 Simpanan & Potong Gaji', href: '/finances', icon: 'credit-card' },
      { id: 'reports', label: '📊 Penjualan Anggota vs Umum', href: '/reports', icon: 'bar-chart-2' },
      { id: 'shift', label: '🔑 Kas Shift', href: '/shift', icon: 'clock', requiredCapability: 'shift.basic' },
      { id: 'settings', label: '⚙️ Pengaturan', href: '/settings', icon: 'settings' },
    ],
  },

  public_service_fee: {
    code: 'public_service_fee',
    name: 'Retribusi / Layanan Publik',
    category: 'Publik',
    icon: '🏛️',
    description: 'H. Loket retribusi resmi, nomor antrean, karcis & larangan diskon',
    defaultCapabilities: ['public_service.receipt', 'checkout.no_discount', 'shift.basic'],
    menuItems: [
      { id: 'retribusi', label: '🏛️ Loket Retribusi & Karcis', href: '/retribusi', icon: 'file-text', requiredCapability: 'public_service.receipt' },
      { id: 'customers', label: '👥 Data Wajib Bayar', href: '/customers', icon: 'users' },
      { id: 'reports', label: '📊 Laporan Pendapatan Loket', href: '/reports', icon: 'bar-chart-2' },
      { id: 'shift', label: '🔑 Setoran Petugas & Shift', href: '/shift', icon: 'clock', requiredCapability: 'shift.basic' },
      { id: 'settings', label: '⚙️ Pengaturan Tarif', href: '/settings', icon: 'settings' },
    ],
  },

  internal_issue: {
    code: 'internal_issue',
    name: 'Pengeluaran Barang Internal',
    category: 'Gudang',
    icon: '📦',
    description: 'I. Disposisi gudang internal, pantry, cost center & transfer',
    defaultCapabilities: ['inventory.basic', 'internal.issue'],
    menuItems: [
      { id: 'internal_issue', label: '📦 Disposisi & Permintaan Gudang', href: '/internal-issue', icon: 'archive', requiredCapability: 'internal.issue' },
      { id: 'inventory', label: '🗃️ Master Barang & Gudang', href: '/inventory', icon: 'package', requiredCapability: 'inventory.basic' },
      { id: 'reports', label: '📊 Pemakaian per Departemen', href: '/reports', icon: 'bar-chart-2' },
      { id: 'settings', label: '⚙️ Pengaturan Departemen', href: '/settings', icon: 'settings' },
    ],
  },

  school_campus: {
    code: 'school_campus',
    name: 'Kantin / Toko Sekolah',
    category: 'Pendidikan',
    icon: '🏫',
    description: 'J. Kantin sekolah, saldo wallet RFID siswa, limit harian & pre-order',
    defaultCapabilities: ['inventory.basic', 'student.allowance', 'checkout.basic', 'shift.basic'],
    menuItems: [
      { id: 'pos', label: '🛒 POS Kasir Kantin', href: '/pos', icon: 'shopping-cart', requiredCapability: 'checkout.basic' },
      { id: 'school', label: '🏫 Wallet Siswa & Pre-Order', href: '/school', icon: 'user-check', requiredCapability: 'student.allowance' },
      { id: 'inventory', label: '🗃️ Produk & Menu Kantin', href: '/inventory', icon: 'package', requiredCapability: 'inventory.basic' },
      { id: 'reports', label: '📊 Penjualan per Kelas & Siswa', href: '/reports', icon: 'bar-chart-2' },
      { id: 'shift', label: '🔑 Kas Shift', href: '/shift', icon: 'clock', requiredCapability: 'shift.basic' },
      { id: 'settings', label: '⚙️ Pengaturan', href: '/settings', icon: 'settings' },
    ],
  },

  parking: {
    code: 'parking',
    name: 'Parkir & Gate Sistem',
    category: 'Parkir',
    icon: '🚗',
    description: 'K. Gate tiket masuk/keluar, tarif progresif, lost ticket & langganan',
    defaultCapabilities: ['parking.gate', 'parking.tariff', 'shift.basic'],
    menuItems: [
      { id: 'parking', label: '🚗 Gate Masuk & Keluar Parkir', href: '/parking', icon: 'car', requiredCapability: 'parking.gate' },
      { id: 'customers', label: '👥 Pelanggan Langganan', href: '/customers', icon: 'users' },
      { id: 'reports', label: '📊 Trafik & Pendapatan Parkir', href: '/reports', icon: 'bar-chart-2' },
      { id: 'shift', label: '🔑 Setoran & Shift Petugas', href: '/shift', icon: 'clock', requiredCapability: 'shift.basic' },
      { id: 'settings', label: '⚙️ Pengaturan Tarif Parkir', href: '/settings', icon: 'settings' },
    ],
  },

  fuel_energy: {
    code: 'fuel_energy',
    name: 'SPBU / EV Charging',
    category: 'Energi',
    icon: '⛽',
    description: 'Stasiun pengisian bahan bakar / listrik, pencatatan meter dispenser',
    defaultCapabilities: ['fuel.dispenser', 'fuel.reconciliation', 'checkout.basic', 'shift.basic'],
    menuItems: [
      { id: 'fuel', label: '⛽ SPBU & Meter', href: '/fuel', icon: 'fuel', requiredCapability: 'fuel.dispenser' },
      { id: 'pos', label: '🛒 Kasir BBM', href: '/pos', icon: 'shopping-cart', requiredCapability: 'checkout.basic' },
      { id: 'inventory', label: '🗃️ Tangki & Stok', href: '/inventory', icon: 'package', requiredCapability: 'inventory.basic' },
      { id: 'reports', label: '📊 Laporan Shift SPBU', href: '/reports', icon: 'bar-chart-2' },
      { id: 'shift', label: '🔑 Kas & Shift', href: '/shift', icon: 'clock', requiredCapability: 'shift.basic' },
      { id: 'settings', label: '⚙️ Pengaturan Dispenser', href: '/settings', icon: 'settings' },
    ],
  },
};

export function getPresetConfig(presetCode: string): PresetConfig {
  return PRESET_REGISTRY[presetCode] || PRESET_REGISTRY.general_flexible;
}
