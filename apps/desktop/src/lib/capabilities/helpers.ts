import type { Capability, CapabilityInfo } from '$lib/types';
import { capabilityStore } from './store.svelte';

export function canUseCapability(cap: Capability): boolean {
  return capabilityStore.has(cap);
}

export function canUseAnyCapability(caps: Capability[]): boolean {
  return capabilityStore.hasAny(caps);
}

export function canUseAllCapabilities(caps: Capability[]): boolean {
  return capabilityStore.hasAll(caps);
}

export const CAPABILITY_METADATA: Record<Capability, CapabilityInfo> = {
  TableManagement: {
    id: 'TableManagement',
    label: 'Manajemen Meja',
    description: 'Atur denah meja, reservasi, dan status meja',
    domain: 'fnb'
  },
  SplitBill: {
    id: 'SplitBill',
    label: 'Split Bill',
    description: 'Pisah tagihan per pelanggan atau item',
    domain: 'fnb'
  },
  RecipeManagement: {
    id: 'RecipeManagement',
    label: 'Manajemen Resep',
    description: 'Hitung HPP otomatis dari resep dan bahan baku',
    domain: 'fnb'
  },
  KitchenDisplay: {
    id: 'KitchenDisplay',
    label: 'Kitchen Display System (KDS)',
    description: 'Kirim pesanan ke layar dapur secara real-time',
    domain: 'fnb'
  },
  DiningSession: {
    id: 'DiningSession',
    label: 'Sesi Makan Di Tempat',
    description: 'Lacak durasi makan dan status sesi meja',
    domain: 'fnb'
  },
  SerialNumberTracking: {
    id: 'SerialNumberTracking',
    label: 'Pelacakan Serial Number / IMEI',
    description: 'Catat nomor seri unik per unit barang',
    domain: 'retail'
  },
  BundleDiscount: {
    id: 'BundleDiscount',
    label: 'Diskon Bundling',
    description: 'Atur paket promosi produk hemat',
    domain: 'retail'
  },
  BarcodePrinting: {
    id: 'BarcodePrinting',
    label: 'Cetak Barcode',
    description: 'Cetak label barcode harga produk',
    domain: 'retail'
  },
  TimeBasedBilling: {
    id: 'TimeBasedBilling',
    label: 'Penagihan Berbasis Durasi',
    description: 'Hitung biaya berdasarkan durasi sewa / layanan',
    domain: 'service'
  },
  DepositManagement: {
    id: 'DepositManagement',
    label: 'Manajemen Deposit / DP',
    description: 'Terima uang muka dan lacak sisa tagihan',
    domain: 'service'
  },
  BookingCalendar: {
    id: 'BookingCalendar',
    label: 'Kalender Pemesanan',
    description: 'Jadwal janji temu dan reservasi pelanggan',
    domain: 'service'
  },
  MultiPayment: {
    id: 'MultiPayment',
    label: 'Multi Payment',
    description: 'Terima pembayaran kombinasi Tunai, QRIS, Kartu',
    domain: 'general'
  },
  CustomerLoyalty: {
    id: 'CustomerLoyalty',
    label: 'Program Poin & Loyalitas',
    description: 'Kumpulkan poin belanja dan diskon member',
    domain: 'general'
  },
  DiscountApproval: {
    id: 'DiscountApproval',
    label: 'Persetujuan Diskon Supervisor',
    description: 'Memerlukan PIN supervisor untuk diskon besar',
    domain: 'general'
  },
  InventoryTransfer: {
    id: 'InventoryTransfer',
    label: 'Transfer Stok Cabang',
    description: 'Kirim dan terima barang antar cabang',
    domain: 'general'
  },
  MultiOutlet: {
    id: 'MultiOutlet',
    label: 'Manajemen Multi Cabang',
    description: 'Kelola inventaris dan laporan banyak toko',
    domain: 'general'
  },
  OfflineMode: {
    id: 'OfflineMode',
    label: 'Operasional Offline-First',
    description: 'Tetap bisa bertransaksi tanpa koneksi internet',
    domain: 'general'
  },
  AuditLog: {
    id: 'AuditLog',
    label: 'Log Audit Keamanan',
    description: 'Pencatatan riwayat aksi sensitif pengguna',
    domain: 'general'
  }
};
