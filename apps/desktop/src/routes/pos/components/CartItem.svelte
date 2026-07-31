<script lang="ts">
  import type { CartItem as CartItemType } from '$lib/types';

  interface Props {
    item: CartItemType;
    onUpdateQty: (productId: string, newQty: number) => void;
    onRemove: (productId: string) => void;
  }

  let { item, onUpdateQty, onRemove }: Props = $props();

  // Svelte 5 derived state for formatted currency values
  let formattedItemPrice = $derived(
    new Intl.NumberFormat('id-ID', { style: 'currency', currency: 'IDR', minimumFractionDigits: 0 }).format(
      item.product.price
    )
  );

  let formattedLineTotal = $derived(
    new Intl.NumberFormat('id-ID', { style: 'currency', currency: 'IDR', minimumFractionDigits: 0 }).format(
      item.product.price * item.quantity
    )
  );
</script>

<div
  class="flex items-center justify-between p-3 bg-slate-800/80 border border-slate-700/70 rounded-2xl hover:border-slate-600 transition-colors group shadow-sm"
>
  <!-- Product Info & Image -->
  <div class="flex items-center gap-3 min-w-0 flex-1">
    {#if item.product.image_url}
      <img
        src={item.product.image_url}
        alt={item.product.name}
        class="w-12 h-12 object-cover rounded-xl border border-slate-700 bg-slate-900 shrink-0"
      />
    {:else}
      <div
        class="w-12 h-12 rounded-xl border border-slate-700/80 bg-slate-900/80 flex items-center justify-center text-slate-400 font-extrabold text-lg shrink-0 select-none"
      >
        {item.product.name.charAt(0).toUpperCase()}
      </div>
    {/if}

    <div class="min-w-0 flex-1">
      <h4 class="text-sm font-semibold text-slate-100 truncate" title={item.product.name}>
        {item.product.name}
      </h4>
      <p class="text-xs text-slate-400 font-mono mt-0.5">
        {formattedItemPrice} <span class="text-slate-500">/ item</span>
      </p>
      {#if item.notes}
        <p class="text-[11px] text-amber-400/90 italic mt-0.5 truncate">
          Catatan: {item.notes}
        </p>
      {/if}
    </div>
  </div>

  <!-- Actions & Quantity (Touch-friendly 44x44px controls) -->
  <div class="flex items-center gap-2 shrink-0 ml-2">
    <!-- Touch-Friendly Quantity Selector -->
    <div class="flex items-center bg-slate-900/90 border border-slate-700 rounded-xl p-1 shadow-inner">
      <button
        type="button"
        onclick={() => onUpdateQty(item.product.id, item.quantity - 1)}
        class="min-w-[44px] min-h-[44px] flex items-center justify-center rounded-lg text-slate-200 hover:text-white bg-slate-800 hover:bg-slate-700 active:scale-95 transition-all text-base font-bold select-none cursor-pointer"
        aria-label="Kurangi kuantitas {item.product.name}"
      >
        -
      </button>

      <span class="w-8 text-center text-sm font-extrabold text-white font-mono select-none">
        {item.quantity}
      </span>

      <button
        type="button"
        onclick={() => onUpdateQty(item.product.id, item.quantity + 1)}
        class="min-w-[44px] min-h-[44px] flex items-center justify-center rounded-lg text-slate-200 hover:text-white bg-slate-800 hover:bg-slate-700 active:scale-95 transition-all text-base font-bold select-none cursor-pointer"
        aria-label="Tambah kuantitas {item.product.name}"
      >
        +
      </button>
    </div>

    <!-- Line Total & Trash Icon Delete Button -->
    <div class="flex flex-col items-end min-w-[70px]">
      <span class="block text-xs font-bold text-blue-400 font-mono mb-1">
        {formattedLineTotal}
      </span>

      <button
        type="button"
        onclick={() => onRemove(item.product.id)}
        class="min-w-[44px] min-h-[44px] flex items-center justify-center rounded-xl bg-rose-500/10 hover:bg-rose-500/20 text-rose-400 hover:text-rose-300 transition-colors cursor-pointer border border-rose-500/20"
        aria-label="Hapus {item.product.name} dari keranjang"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="w-5 h-5"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
          stroke-width="2"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
          />
        </svg>
      </button>
    </div>
  </div>
</div>
