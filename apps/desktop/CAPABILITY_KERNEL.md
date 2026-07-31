# 🛡️ Capability Kernel Architecture & Guide

POSQ menggunakan pendekatan **Capability Kernel** di mana aplikasi POSQ merupakan SATU sistem modular dengan kapabilitas terkonfigurasi per-outlet. Dilarang keras menggunakan percabangan berbasis mode bisnis seperti `if (outlet.mode === 'fnb')`.

---

## 🎯 Daftar Kapabilitas Terdaftar

| Kapabilitas | Domain | Deskripsi Human-Readable |
| :--- | :--- | :--- |
| `TableManagement` | F&B | Manajemen Meja & Denah Restoran |
| `SplitBill` | F&B | Pemisahan Tagihan (Split Bill) |
| `RecipeManagement` | F&B | Manajemen Resep & HPP Bahan Baku |
| `KitchenDisplay` | F&B | Layar Dapur / KDS Ticket |
| `DiningSession` | F&B | Sesi Makan Di Tempat |
| `SerialNumberTracking` | Retail | Pelacakan Nomor Seri & IMEI |
| `BundleDiscount` | Retail | Diskon Paket / Bundling Produk |
| `BarcodePrinting` | Retail | Cetak Label Barcode |
| `TimeBasedBilling` | Service | Penagihan Berbasis Durasi/Waktu |
| `DepositManagement` | Service | Manajemen Uang Muka / Deposit |
| `BookingCalendar` | Service | Kalender Pemesanan & Jadwal |
| `MultiPayment` | General | Pembayaran Ganda / Split Payment |
| `CustomerLoyalty` | General | Poin Pelanggan & Program Loyalitas |
| `DiscountApproval` | General | Persetujuan Diskon Supervisor |
| `InventoryTransfer` | General | Transfer Stok Antar Cabang |
| `MultiOutlet` | General | Manajemen Banyak Cabang |
| `OfflineMode` | General | Operasional Offline-First |
| `AuditLog` | General | Log Audit & Jejak Aktivitas |

---

## 🚀 Cara Menambahkan Kapabilitas Baru

### 1. Sisi Backend (Rust)
Tambahkan varian baru pada enum `Capability` di `apps/desktop/src-tauri/src/capabilities.rs`:
```rust
pub enum Capability {
    // ...
    NewFeature,
}
```
Lengkapi method `description()`, `domain()`, `Display`, dan `FromStr` pada enum `Capability`.

### 2. Sisi Frontend (TypeScript & Svelte)
Tambahkan nama kapabilitas ke union type `Capability` di `apps/desktop/src/lib/types.ts`:
```typescript
export type Capability = 
  | 'TableManagement' 
  // ...
  | 'NewFeature';
```
Tambahkan metadata visual di `CAPABILITY_METADATA` pada `apps/desktop/src/lib/capabilities/helpers.ts`.

---

## 🔒 Guard Enforcement Pattern (Rust & UI)

### Sisi Backend (Tauri Command)
Panggil `require_capability` di awal fungsi sensitif:
```rust
#[tauri::command]
pub async fn execute_sensitive_action(
    pool: State<'_, SqlitePool>,
    outlet_id: String,
) -> Result<(), String> {
    crate::capabilities::require_capability(&outlet_id, crate::capabilities::Capability::NewFeature, pool.inner()).await?;
    
    // Logika utama
    Ok(())
}
```

### Sisi Frontend (Svelte 5 UI)
Bungkus elemen UI dengan `<CapabilityGuard>`:
```svelte
<script lang="ts">
  import CapabilityGuard from '$lib/capabilities/CapabilityGuard.svelte';
</script>

<CapabilityGuard capability="SplitBill" fallback="disable">
  <button onclick={handleSplit}>Split Bill</button>
</CapabilityGuard>
```
