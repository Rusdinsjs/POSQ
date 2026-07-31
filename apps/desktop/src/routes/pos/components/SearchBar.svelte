<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { cartStore } from '$lib/stores/cart.svelte';
  import { showToast } from '$lib/toast.svelte';
  import type { Product } from '$lib/types';
  import KbdHint from '$lib/components/KbdHint.svelte';

  interface Props {
    selectedCategory?: string | null;
    onSearchResults?: (results: Product[] | null) => void;
    inputElement?: HTMLInputElement | null;
  }

  let { selectedCategory = null, onSearchResults, inputElement = $bindable(null) }: Props = $props();

  let searchQuery = $state<string>('');
  let searchResults = $state<Product[]>([]);
  let isSearching = $state<boolean>(false);
  let showDropdown = $state<boolean>(false);

  // Barcode Scanner Timing Tracking
  let lastKeyTime = 0;
  let keyIntervals: number[] = [];
  let searchTimeout: ReturnType<typeof setTimeout> | null = null;

  function handleInput(e: Event) {
    const target = e.target as HTMLInputElement;
    searchQuery = target.value;

    const now = Date.now();
    if (lastKeyTime > 0) {
      keyIntervals.push(now - lastKeyTime);
      if (keyIntervals.length > 10) keyIntervals.shift();
    }
    lastKeyTime = now;

    if (searchTimeout) clearTimeout(searchTimeout);

    if (searchQuery.trim() === '') {
      searchResults = [];
      showDropdown = false;
      isSearching = false;
      if (onSearchResults) onSearchResults(null);
      return;
    }

    isSearching = true;
    showDropdown = true;

    // Debounce 300ms for manual text search
    searchTimeout = setTimeout(() => {
      executeSearch(searchQuery);
    }, 300);
  }

  async function executeSearch(query: string) {
    if (!query.trim()) {
      searchResults = [];
      showDropdown = false;
      isSearching = false;
      if (onSearchResults) onSearchResults(null);
      return;
    }

    try {
      const results = await invoke<Product[]>('search_products', {
        query: query.trim(),
        category_id: selectedCategory,
        limit: 50
      });

      searchResults = results;
      if (onSearchResults) onSearchResults(results);
    } catch (err) {
      console.error('Search error:', err);
      showToast('Gagal mencari produk', 'error');
    } finally {
      isSearching = false;
    }
  }

  async function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      const trimmed = searchQuery.trim();
      if (!trimmed) return;

      const avgInterval =
        keyIntervals.length > 0
          ? keyIntervals.reduce((a, b) => a + b, 0) / keyIntervals.length
          : 999;

      const isBarcodePattern = trimmed.length >= 8 && (avgInterval < 100 || keyIntervals.length >= 6);

      if (isBarcodePattern) {
        e.preventDefault();
        if (searchTimeout) clearTimeout(searchTimeout);

        try {
          const product = await invoke<Product | null>('get_product_by_barcode', {
            barcode: trimmed
          });

          if (product) {
            cartStore.addItem(product, 1);
            showToast(`Produk ditambahkan via scan: ${product.name}`, 'success');
          } else {
            showToast(`Produk dengan barcode '${trimmed}' tidak ditemukan`, 'warning');
          }
        } catch (err) {
          console.error('Barcode scan error:', err);
          showToast('Gagal memproses barcode scanner', 'error');
        } finally {
          searchQuery = '';
          searchResults = [];
          showDropdown = false;
          keyIntervals = [];
          isSearching = false;
          if (onSearchResults) onSearchResults(null);
        }
      }
    }
  }

  function handleSelectProduct(product: Product) {
    if ((product.qty_on_hand ?? product.stock ?? 0) <= 0) {
      showToast(`Stok ${product.name} habis`, 'warning');
      return;
    }
    cartStore.addItem(product, 1);
    handleClear();
  }

  function handleClear() {
    searchQuery = '';
    searchResults = [];
    showDropdown = false;
    keyIntervals = [];
    if (searchTimeout) clearTimeout(searchTimeout);
    if (onSearchResults) onSearchResults(null);
    isSearching = false;
    inputElement?.focus();
  }

  function highlightText(text: string, query: string): string {
    if (!query) return text;
    const regex = new RegExp(`(${query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'gi');
    return text.replace(regex, '<mark class="bg-blue-500/30 text-blue-300 font-bold px-0.5 rounded">$1</mark>');
  }
</script>

<div class="relative w-full">
  <div class="relative flex items-center">
    <!-- Search Icon / Spinner -->
    <div class="absolute left-3.5 text-slate-400 pointer-events-none flex items-center justify-center">
      {#if isSearching}
        <span class="inline-block w-4 h-4 border-2 border-blue-400 border-t-transparent rounded-full animate-spin"></span>
      {:else}
        <span class="text-base">🔍</span>
      {/if}
    </div>

    <!-- Main Search Input -->
    <input
      bind:this={inputElement}
      type="text"
      value={searchQuery}
      oninput={handleInput}
      onkeydown={handleKeyDown}
      placeholder="Cari nama produk, SKU, atau pindai Barcode..."
      class="w-full bg-slate-800/90 border border-slate-700/80 hover:border-slate-600 focus:border-blue-500 focus:outline-none rounded-2xl py-3 pl-10 pr-24 text-sm text-slate-100 placeholder-slate-400 font-medium transition-all shadow-inner"
      aria-label="Cari produk atau scan barcode"
    />

    <!-- Right Controls -->
    <div class="absolute right-3 flex items-center gap-2">
      {#if searchQuery}
        <button
          type="button"
          onclick={handleClear}
          class="text-xs text-slate-400 hover:text-white px-1.5 py-0.5 rounded bg-slate-700/60 hover:bg-slate-700 transition-colors"
          aria-label="Bersihkan pencarian"
        >
          ✕
        </button>
      {/if}

      <KbdHint keys={['F2']} class="hidden sm:inline-flex" />
    </div>
  </div>

  <!-- Vertical List Search Results Dropdown -->
  {#if showDropdown && searchQuery.trim() !== ''}
    <div
      class="absolute top-full left-0 right-0 mt-2 bg-slate-900 border border-slate-800 rounded-2xl shadow-2xl overflow-hidden z-40 max-h-80 overflow-y-auto custom-scrollbar"
    >
      {#if isSearching}
        <div class="p-4 text-center text-xs text-slate-400">Memuat hasil pencarian...</div>
      {:else if searchResults.length === 0}
        <div class="p-4 text-center text-xs text-slate-400">Tidak ada produk yang cocok</div>
      {:else}
        <div class="divide-y divide-slate-800">
          {#each searchResults as product}
            <button
              type="button"
              onclick={() => handleSelectProduct(product)}
              class="w-full text-left p-3 hover:bg-slate-800/80 flex items-center justify-between transition-colors group cursor-pointer"
            >
              <div class="flex items-center gap-3 min-w-0 flex-1">
                {#if product.image_url}
                  <img src={product.image_url} alt={product.name} class="w-9 h-9 object-cover rounded-lg bg-slate-950 shrink-0" />
                {:else}
                  <div class="w-9 h-9 rounded-lg bg-slate-950 flex items-center justify-center font-bold text-slate-500 text-xs shrink-0">
                    {product.name.charAt(0)}
                  </div>
                {/if}

                <div class="min-w-0 flex-1">
                  <p class="text-xs font-bold text-slate-200 group-hover:text-blue-300 truncate">
                    {@html highlightText(product.name, searchQuery)}
                  </p>
                  <p class="text-[10px] text-slate-500 font-mono">
                    SKU: {@html highlightText(product.sku || '-', searchQuery)}
                  </p>
                </div>
              </div>

              <div class="text-right shrink-0 ml-2">
                <span class="text-xs font-black text-blue-400 font-mono block">
                  Rp {product.price.toLocaleString('id-ID')}
                </span>
                <span class="text-[10px] text-slate-500 font-mono">
                  Stok: {product.qty_on_hand ?? product.stock ?? 0}
                </span>
              </div>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>
