<script lang="ts">
  import type { CartItem as CartItemType } from '$lib/types';

  interface Props {
    item: CartItemType;
    onUpdateQuantity: (productId: string, delta: number) => void;
    onRemove: (productId: string) => void;
  }

  let { item, onUpdateQuantity, onRemove }: Props = $props();

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
  class="flex items-center justify-between p-3 bg-slate-800/70 border border-slate-700/60 rounded-2xl hover:border-slate-600 transition-colors group shadow-sm"
>
  <!-- Product Info & Image -->
  <div class="flex items-center gap-3 min-w-0 flex-1">
    {#if item.product.image_url}
      <img
        src={item.product.image_url}
        alt={item.product.name}
        class="w-11 h-11 object-cover rounded-xl border border-slate-700 bg-slate-900 shrink-0"
      />
    {:else}
      <div
        class="w-11 h-11 rounded-xl border border-slate-700/80 bg-slate-900/80 flex items-center justify-center text-slate-400 font-extrabold text-base shrink-0 select-none"
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

  <!-- Actions & Quantity -->
  <div class="flex items-center gap-3 shrink-0 ml-2">
    <!-- Quantity Selector -->
    <div class="flex items-center bg-slate-900/90 border border-slate-700 rounded-xl p-1 shadow-inner">
      <button
        type="button"
        onclick={() => onUpdateQuantity(item.product.id, -1)}
        class="w-7 h-7 flex items-center justify-center rounded-lg text-slate-300 hover:text-white hover:bg-slate-700/60 active:scale-95 transition-all text-sm font-bold select-none cursor-pointer"
        aria-label="Kurangi kuantitas {item.product.name}"
      >
        -
      </button>

      <span class="w-7 text-center text-xs font-extrabold text-white font-mono select-none">
        {item.quantity}
      </span>

      <button
        type="button"
        onclick={() => onUpdateQuantity(item.product.id, 1)}
        class="w-7 h-7 flex items-center justify-center rounded-lg text-slate-300 hover:text-white hover:bg-slate-700/60 active:scale-95 transition-all text-sm font-bold select-none cursor-pointer"
        aria-label="Tambah kuantitas {item.product.name}"
      >
        +
      </button>
    </div>

    <!-- Line Total & Delete Button -->
    <div class="text-right min-w-[75px]">
      <span class="block text-xs font-bold text-blue-400 font-mono">
        {formattedLineTotal}
      </span>
      <button
        type="button"
        onclick={() => onRemove(item.product.id)}
        class="text-[11px] text-rose-400 hover:text-rose-300 underline mt-0.5 transition-colors opacity-80 group-hover:opacity-100 cursor-pointer"
        aria-label="Hapus {item.product.name} dari keranjang"
      >
        Hapus
      </button>
    </div>
  </div>
</div>
