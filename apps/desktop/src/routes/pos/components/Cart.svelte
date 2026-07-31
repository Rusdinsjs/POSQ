<script lang="ts">
  import { cartStore } from '$lib/stores/cart.svelte';
  import CartItem from './CartItem.svelte';

  interface Props {
    onCheckout?: () => void;
    onClearCart?: () => void;
  }

  let { onCheckout, onClearCart }: Props = $props();

  // Svelte 5 derived currency formatters for UI estimations
  let formattedSubtotal = $derived(
    new Intl.NumberFormat('id-ID', { style: 'currency', currency: 'IDR', minimumFractionDigits: 0 }).format(
      cartStore.subtotal
    )
  );

  let formattedTax = $derived(
    new Intl.NumberFormat('id-ID', { style: 'currency', currency: 'IDR', minimumFractionDigits: 0 }).format(
      cartStore.estimatedTax
    )
  );

  let formattedTotal = $derived(
    new Intl.NumberFormat('id-ID', { style: 'currency', currency: 'IDR', minimumFractionDigits: 0 }).format(
      cartStore.estimatedTotal
    )
  );

  function handleClear() {
    cartStore.clearCart();
    if (onClearCart) onClearCart();
  }
</script>

<aside
  class="flex flex-col h-full bg-slate-900/90 border-l border-slate-800 overflow-hidden relative"
  aria-label="Keranjang Belanja Kasir"
>
  <!-- Cart Header -->
  <header class="flex items-center justify-between p-4 border-b border-slate-800 bg-slate-900/60 backdrop-blur shrink-0">
    <div class="flex items-center gap-2.5">
      <div
        class="w-9 h-9 rounded-xl bg-blue-600/20 border border-blue-500/30 flex items-center justify-center text-blue-400 font-bold text-base shadow-sm"
      >
        🛒
      </div>
      <div>
        <h2 class="text-base font-bold text-white tracking-tight">Keranjang Pesanan</h2>
        <p class="text-xs text-slate-400">
          {cartStore.totalItems} {cartStore.totalItems === 1 ? 'item' : 'items'} terpilih
        </p>
      </div>
    </div>

    {#if cartStore.items.length > 0}
      <button
        type="button"
        onclick={handleClear}
        class="text-xs text-rose-400 hover:text-rose-300 font-semibold px-3 py-1.5 rounded-xl bg-rose-500/10 hover:bg-rose-500/20 border border-rose-500/20 transition-all active:scale-95 cursor-pointer"
        aria-label="Kosongkan seluruh isi keranjang"
      >
        Kosongkan
      </button>
    {/if}
  </header>

  <!-- Cart Item List with Vertical Scroll -->
  <div class="flex-1 overflow-y-auto p-4 space-y-3 custom-scrollbar">
    {#if cartStore.items.length === 0}
      <div class="h-full flex flex-col items-center justify-center text-center p-6 text-slate-500 space-y-3">
        <div class="w-16 h-16 rounded-3xl bg-slate-800/80 border border-slate-700/60 flex items-center justify-center text-3xl shadow-inner">
          🛍️
        </div>
        <div>
          <p class="text-sm font-semibold text-slate-300">Keranjang Masih Kosong</p>
          <p class="text-xs text-slate-400 mt-1 max-w-[220px]">
            Pilih atau pindai produk untuk mulai menambah pesanan.
          </p>
        </div>
      </div>
    {:else}
      {#each cartStore.items as item (item.product.id)}
        <CartItem
          {item}
          onUpdateQty={(id, newQty) => cartStore.updateQty(id, newQty)}
          onRemove={(id) => cartStore.removeItem(id)}
        />
      {/each}
    {/if}
  </div>

  <!-- Sticky Footer Summary & Checkout Button -->
  <footer class="p-4 border-t border-slate-800 bg-slate-900/95 backdrop-blur shadow-2xl space-y-3 sticky bottom-0 z-10 shrink-0">
    <div class="space-y-1.5 text-xs">
      <div class="flex justify-between text-slate-400">
        <span>Subtotal (Estimasi)</span>
        <span class="font-mono text-slate-200 font-medium">{formattedSubtotal}</span>
      </div>

      <div class="flex justify-between text-slate-400">
        <span>PPN 11% (Estimasi)</span>
        <span class="font-mono text-slate-200 font-medium">{formattedTax}</span>
      </div>

      <div class="border-t border-slate-800/80 pt-2 mt-1 flex justify-between items-baseline">
        <span class="text-sm font-bold text-white">Total Estimasi</span>
        <span class="text-lg font-black text-blue-400 font-mono tracking-tight">{formattedTotal}</span>
      </div>
    </div>

    <button
      type="button"
      onclick={() => onCheckout?.()}
      disabled={cartStore.totalItems === 0}
      class="w-full py-4 px-4 rounded-xl bg-blue-600 hover:bg-blue-500 active:scale-[0.98] disabled:opacity-50 disabled:pointer-events-none disabled:active:scale-100 text-white font-extrabold text-sm tracking-wide shadow-lg shadow-blue-600/30 transition-all flex items-center justify-center gap-2 cursor-pointer min-h-[48px]"
      aria-label="Proses Pembayaran dengan {cartStore.totalItems} item"
    >
      <span>BAYAR</span>
      <span class="text-blue-200 font-normal">({cartStore.totalItems} item)</span>
    </button>
  </footer>
</aside>
