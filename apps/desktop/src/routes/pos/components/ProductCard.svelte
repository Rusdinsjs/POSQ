<script lang="ts">
  import type { Product } from '$lib/types';

  interface Props {
    product: Product;
    onSelect: (product: Product) => void;
  }

  let { product, onSelect }: Props = $props();

  let stockCount = $derived(product.qty_on_hand ?? product.stock ?? 0);
  let isOutOfStock = $derived(stockCount <= 0);

  let formattedPrice = $derived(
    new Intl.NumberFormat('id-ID', {
      style: 'currency',
      currency: 'IDR',
      minimumFractionDigits: 0
    }).format(product.price)
  );

  function handleClick() {
    if (!isOutOfStock) {
      onSelect(product);
    }
  }
</script>

<div
  role="button"
  tabindex={isOutOfStock ? -1 : 0}
  onclick={handleClick}
  onkeydown={(e) => e.key === 'Enter' && handleClick()}
  class="relative flex flex-col justify-between bg-slate-800/80 border border-slate-700/70 rounded-2xl p-3 overflow-hidden transition-all duration-200 shadow-sm group select-none {isOutOfStock
    ? 'opacity-60 cursor-not-allowed border-rose-500/20'
    : 'hover:border-blue-500/60 hover:shadow-md hover:bg-slate-800 active:scale-[0.98] cursor-pointer'}"
  aria-disabled={isOutOfStock}
  aria-label="{product.name}, harga {formattedPrice}, {isOutOfStock ? 'stok habis' : `stok ${stockCount}`}"
>
  <!-- Stock Badge (Top Right) -->
  <div class="absolute top-2.5 right-2.5 z-10">
    {#if isOutOfStock}
      <span class="px-2 py-0.5 rounded-lg bg-rose-500/90 text-white font-black text-[10px] tracking-wide shadow">
        HABIS
      </span>
    {:else}
      <span
        class="px-2 py-0.5 rounded-lg text-[10px] font-bold font-mono shadow backdrop-blur-sm {stockCount < 5
          ? 'bg-amber-500/80 text-white'
          : 'bg-slate-900/80 text-slate-300 border border-slate-700/60'}"
      >
        Stok: {stockCount}
      </span>
    {/if}
  </div>

  <!-- Product Image Container -->
  <div class="w-full aspect-[4/3] rounded-xl bg-slate-900 overflow-hidden mb-2.5 border border-slate-700/50 flex items-center justify-center relative">
    {#if product.image_url}
      <img
        src={product.image_url}
        alt={product.name}
        loading="lazy"
        class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
      />
    {:else}
      <div class="text-3xl text-slate-600 font-extrabold select-none">
        {product.name.charAt(0).toUpperCase()}
      </div>
    {/if}

    <!-- Dark Overlay for Out of Stock -->
    {#if isOutOfStock}
      <div class="absolute inset-0 bg-slate-950/80 backdrop-blur-[1px] flex flex-col items-center justify-center z-10">
        <span class="text-rose-400 font-black text-sm tracking-wider uppercase">HABIS</span>
        <span class="text-[10px] text-slate-400 mt-0.5">Stok Kosong</span>
      </div>
    {/if}
  </div>

  <!-- Product Metadata -->
  <div class="space-y-1">
    <h3 class="text-xs font-bold text-slate-100 line-clamp-2 leading-snug group-hover:text-blue-300 transition-colors" title={product.name}>
      {product.name}
    </h3>

    <div class="flex items-center justify-between pt-0.5">
      <span class="text-xs font-black text-blue-400 font-mono">
        {formattedPrice}
      </span>
      {#if product.sku}
        <span class="text-[10px] font-mono text-slate-500 truncate max-w-[60px]">
          {product.sku}
        </span>
      {/if}
    </div>
  </div>
</div>
