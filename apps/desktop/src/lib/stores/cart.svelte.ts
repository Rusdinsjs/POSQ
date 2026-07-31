import { showToast } from '$lib/toast.svelte';
import type { Product, CartItem, CheckoutItemPayload } from '$lib/types';

class CartStore {
  // Svelte 5 Runes for reactive cart state
  items = $state<CartItem[]>([]);

  // Derived subtotal estimation for UI
  subtotal = $derived(
    this.items.reduce((acc, item) => acc + (item.product.price || 0) * item.quantity, 0)
  );

  // Derived total item count for UI
  totalItems = $derived(
    this.items.reduce((acc, item) => acc + item.quantity, 0)
  );

  // Derived estimated tax (PPN 11%)
  estimatedTax = $derived(
    Math.round(this.subtotal * 0.11)
  );

  // Derived estimated total for UI display
  estimatedTotal = $derived(
    this.subtotal + this.estimatedTax
  );

  /**
   * Add product to cart or increment quantity if duplicate.
   */
  addItem(product: Product, qty: number = 1) {
    if (qty <= 0) return;

    const availableStock = product.qty_on_hand ?? product.stock;
    const existingIndex = this.items.findIndex((item) => item.product.id === product.id);
    const currentQty = existingIndex !== -1 ? this.items[existingIndex].quantity : 0;
    const targetQty = currentQty + qty;

    // Stock availability validation
    if (availableStock !== undefined && availableStock !== null && targetQty > availableStock) {
      showToast(`Stok tidak mencukupi! Tersedia: ${availableStock}`, 'warning');
      return;
    }

    if (existingIndex !== -1) {
      this.items[existingIndex].quantity = targetQty;
    } else {
      this.items.push({
        id: product.id,
        product,
        quantity: qty,
        modifier_ids: [],
        notes: ''
      });
    }

    showToast(`"${product.name}" ditambahkan ke keranjang`, 'success');
  }

  /**
   * Update item quantity directly
   */
  updateQty(itemId: string, newQty: number) {
    if (newQty <= 0) {
      this.removeItem(itemId);
      return;
    }

    const item = this.items.find((i) => i.product.id === itemId);
    if (!item) return;

    const availableStock = item.product.qty_on_hand ?? item.product.stock;
    if (availableStock !== undefined && availableStock !== null && newQty > availableStock) {
      showToast(`Stok tidak mencukupi! Tersedia: ${availableStock}`, 'warning');
      return;
    }

    item.quantity = newQty;
  }

  /**
   * Update item quantity by delta (+1 or -1)
   */
  updateQuantity(itemId: string, delta: number) {
    const item = this.items.find((i) => i.product.id === itemId);
    if (!item) return;
    this.updateQty(itemId, item.quantity + delta);
  }

  /**
   * Remove item from cart
   */
  removeItem(itemId: string) {
    const index = this.items.findIndex((i) => i.product.id === itemId);
    if (index !== -1) {
      const removed = this.items.splice(index, 1)[0];
      showToast(`"${removed.product.name}" dihapus dari keranjang`, 'info');
    }
  }

  /**
   * Clear all items in cart
   */
  clearCart() {
    this.items = [];
  }

  /**
   * Prepare secure checkout payload for Rust backend (Tauri invoke).
   * EXCLUDES frontend prices, subtotals, and totals for financial security.
   */
  prepareCheckoutPayload(): CheckoutItemPayload[] {
    return this.items.map((item) => ({
      item_id: item.product.id,
      quantity: item.quantity,
      modifier_ids: item.modifier_ids && item.modifier_ids.length > 0 ? item.modifier_ids : undefined
    }));
  }
}

export const cartStore = new CartStore();
