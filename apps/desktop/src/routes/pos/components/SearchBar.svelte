<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { cartStore } from '$lib/stores/cart.svelte';
  import { showToast } from '$lib/toast.svelte';
  import type { Product } from '$lib/types';
  import KbdHint from '$lib/components/KbdHint.svelte';

  interface Props {
    onSearchResults: (results: Product[] | null) => void;
    inputElement?: HTMLInputElement | null;
  }

  let { onSearchResults, inputElement = $bindable(null) }: Props = $props();

  let searchQuery = $state<string>('');
  let isSearching = $state<boolean>(false);

  // Barcode Scanner Tracking
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

    // Debounce 300ms for manual typing search
    if (searchTimeout) clearTimeout(searchTimeout);
    if (searchQuery.trim() === '') {
      onSearchResults(null); // Reset search filter
      isSearching = false;
      return;
    }

    isSearching = true;
    searchTimeout = setTimeout(() => {
      executeSearch(searchQuery);
    }, 300);
  }

  async function executeSearch(query: string) {
    if (!query.trim()) {
      onSearchResults(null);
      isSearching = false;
      return;
    }

    try {
      const results = await invoke<Product[]>('search_products', {
        query: query.trim(),
        limit: 25
      });
      onSearchResults(results);
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

      // Check average interval to distinguish barcode scanner (< 80ms) from human typing
      const avgInterval =
        keyIntervals.length > 0
          ? keyIntervals.reduce((a, b) => a + b, 0) / keyIntervals.length
          : 999;

      const isBarcodePattern = trimmed.length >= 8 && (avgInterval < 100 || keyIntervals.length >= 7);

      if (isBarcodePattern) {
        e.preventDefault();
        if (searchTimeout) clearTimeout(searchTimeout);

        try {
          const product = await invoke<Product | null>('get_product_by_barcode', {
            barcode: trimmed
          });

          if (product) {
            cartStore.addItem(product, 1);
            showToast(`Barcode scanned: ${product.name}`, 'success');
          } else {
            showToast(`Produk dengan barcode '${trimmed}' tidak ditemukan`, 'warning');
          }
        } catch (err) {
          console.error('Barcode scan error:', err);
          showToast('Error memproses barcode scanner', 'error');
        } finally {
          searchQuery = '';
          keyIntervals = [];
          onSearchResults(null);
          isSearching = false;
        }
      }
    }
  }

  function handleClear() {
    searchQuery = '';
    keyIntervals = [];
    if (searchTimeout) clearTimeout(searchTimeout);
    onSearchResults(null);
    isSearching = false;
    inputElement?.focus();
  }
</script>

<div class="relative w-full">
  <div class="relative flex items-center">
    <!-- Search Icon -->
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
      aria-label="Cari produk atau barcode"
    />

    <!-- Right Controls (Clear & Hotkey Hint) -->
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
</div>
