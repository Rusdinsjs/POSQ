<script lang="ts">
  import { fade, fly } from 'svelte/transition';
  import type { Product } from '$lib/types';
  import ProductCard from './ProductCard.svelte';

  interface Props {
    products: Product[];
    selectedCategoryId?: string | null;
    isLoading?: boolean;
    onSelectProduct?: (product: Product) => void;
  }

  let { products, selectedCategoryId = null, isLoading = false, onSelectProduct }: Props = $props();

  // Filter products by selected category if provided
  let filteredProducts = $derived(
    selectedCategoryId && selectedCategoryId !== 'Semua'
      ? products.filter(
          (p) => p.category_name === selectedCategoryId || p.category_id === selectedCategoryId
        )
      : products
  );
</script>

<div class="w-full h-full flex flex-col overflow-hidden">
  {#if isLoading}
    <!-- Skeleton Loading Grid -->
    <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-3.5 p-1 animate-pulse">
      {#each Array(10) as _}
        <div class="h-44 bg-slate-800/60 border border-slate-700/50 rounded-2xl p-3 flex flex-col justify-between">
          <div class="w-full h-24 bg-slate-700/50 rounded-xl"></div>
          <div class="space-y-1.5 pt-2">
            <div class="h-3.5 bg-slate-700/60 rounded w-3/4"></div>
            <div class="h-3 bg-slate-700/40 rounded w-1/2"></div>
          </div>
        </div>
      {/each}
    </div>
  {:else if filteredProducts.length === 0}
    <!-- Empty State -->
    <div
      in:fade={{ duration: 150 }}
      class="flex-1 flex flex-col items-center justify-center text-center p-8 text-slate-500 space-y-3"
    >
      <div class="w-16 h-16 rounded-3xl bg-slate-800/80 border border-slate-700/60 flex items-center justify-center text-3xl shadow-inner">
        📦
      </div>
      <div>
        <h4 class="text-sm font-bold text-slate-300">Tidak Ada Produk</h4>
        <p class="text-xs text-slate-400 mt-1 max-w-xs">
          Tidak ada produk yang tersedia dalam kategori ini atau hasil pencarian kosong.
        </p>
      </div>
    </div>
  {:else}
    <!-- Responsive Product Grid with Smooth Transition -->
    <div
      in:fly={{ y: 8, duration: 150 }}
      out:fade={{ duration: 100 }}
      class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-3.5 p-1 overflow-y-auto custom-scrollbar"
    >
      {#each filteredProducts as product (product.id)}
        <ProductCard {product} onSelect={onSelectProduct} />
      {/each}
    </div>
  {/if}
</div>
