/**
 * POSQ Data Type Definitions
 */

export interface Product {
  id: string;
  name: string;
  sku: string;
  price: number;
  qty_on_hand?: number;
  stock?: number;
  image_url?: string | null;
  category_name?: string | null;
  category_id?: string | null;
}

export interface CartItem {
  id: string;
  product: Product;
  quantity: number;
  modifier_ids?: string[];
  notes?: string;
}

export interface CheckoutItemPayload {
  item_id: string;
  quantity: number;
  modifier_ids?: string[];
}

export interface CheckoutPayload {
  items: CheckoutItemPayload[];
  customer_id?: string;
  payment_method?: string;
  notes?: string;
}

export type PaymentMethod = 'cash' | 'card' | 'qris' | 'ewallet';

export interface PaymentLine {
  method: PaymentMethod;
  amount: number;
}

export interface CheckoutRequest {
  cart_payload: CheckoutItemPayload[];
  payments: PaymentLine[];
}

export type CapabilityDomain = 'fnb' | 'retail' | 'service' | 'general';

export type Capability =
  | 'TableManagement'
  | 'SplitBill'
  | 'RecipeManagement'
  | 'KitchenDisplay'
  | 'DiningSession'
  | 'SerialNumberTracking'
  | 'BundleDiscount'
  | 'BarcodePrinting'
  | 'TimeBasedBilling'
  | 'DepositManagement'
  | 'BookingCalendar'
  | 'MultiPayment'
  | 'CustomerLoyalty'
  | 'DiscountApproval'
  | 'InventoryTransfer'
  | 'MultiOutlet'
  | 'OfflineMode'
  | 'AuditLog';

export interface BusinessPreset {
  code?: string;
  name: string;
  description: string;
  capabilities: Capability[];
}

export interface CapabilityInfo {
  id: Capability;
  label: string;
  description: string;
  domain: CapabilityDomain;
}

export type SyncStatus = 'PENDING' | 'SYNCING' | 'SYNCED' | 'FAILED';

export interface SyncStats {
  pending: number;
  syncing: number;
  synced: number;
  failed: number;
}

export interface SyncOutboxEntry {
  id: string;
  entity_type: string;
  entity_id: string;
  idempotency_key: string;
  payload: string;
  status: SyncStatus;
  retry_count: number;
  max_retries: number;
  last_error: string | null;
  created_at: string;
  updated_at: string;
  synced_at: string | null;
}

export interface InventoryProduct {
  id: string;
  merchant_id: string;
  category_id?: string | null;
  sku: string;
  name: string;
  price: number;
  cost: number;
  qty_on_hand: number;
  track_stock: boolean;
  is_ingredient: boolean;
  erp_item_id?: string | null;
}

export interface StockMovement {
  id: string;
  product_id: string;
  movement_type: string;
  qty_delta: number;
  reason?: string | null;
  reference_number?: string | null;
  erp_synced: boolean;
  created_at: string;
}


