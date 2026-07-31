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
  selectedModifiers?: string[];
  notes?: string;
}

export interface CheckoutItemPayload {
  item_id: string;
  quantity: number;
  modifiers?: string[];
  notes?: string;
}

export interface CheckoutPayload {
  items: CheckoutItemPayload[];
  customer_id?: string;
  payment_method?: string;
  notes?: string;
}
