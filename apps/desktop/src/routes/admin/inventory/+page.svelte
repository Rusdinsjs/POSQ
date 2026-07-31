<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type { InventoryProduct } from '$lib/types';
  import ProductForm from './components/ProductForm.svelte';
  import StockOpnameModal from './components/StockOpnameModal.svelte';
  import { showToast } from '$lib/toast.svelte';

  let products = $state<InventoryProduct[]>([]);
  let isLoading = $state<boolean>(true);
  let searchQuery = $state<string>('');

  let showForm = $state<boolean>(false);
  let selectedOpnameProduct = $state<InventoryProduct | null>(null);

  let filteredProducts = $derived(
    products.filter(
      (p) =>
        p.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        p.sku.toLowerCase().includes(searchQuery.toLowerCase())
    )
  );

  async function loadProducts() {
    isLoading = true;
    try {
      const res: InventoryProduct[] = await invoke('get_all_products');
      products = res || [];
    } catch (err: any) {
      showToast(`Gagal memuat katalog produk: ${err}`, 'error');
    } finally {
      isLoading = false;
    }
  }

  $effect(() => {
    loadProducts();
  });
</script>

<div class="p-6 space-y-6 max-w-7xl mx-auto text-slate-100">
  <!-- Header & Actions -->
  <div class="flex flex-col md:flex-row md:items-center justify-between gap-4 border-b border-slate-800 pb-4">
    <div>
      <h1 class="text-2xl font-bold tracking-tight">Katalog Produk & Inventaris</h1>
      <p class="text-xs text-slate-400">Master Data Edge-Originated & Ledger Stok POSQ</p>
    </div>

    <div class="flex items-center gap-3">
      <input
        type="text"
        bind:value={searchQuery}
        placeholder="Cari Produk / SKU..."
        class="bg-slate-900 border border-slate-800 rounded-xl px-4 py-2 text-xs w-64 focus:border-indigo-500 focus:outline-none"
      />
      <button
        type="button"
        onclick={() => (showForm = true)}
        class="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 font-bold text-xs rounded-xl shadow transition-colors"
      >
        + Tambah Produk
      </button>
    </div>
  </div>

  <!-- Table -->
  <div class="bg-slate-900 border border-slate-800 rounded-3xl overflow-hidden shadow-xl">
    {#if isLoading}
      <div class="p-12 text-center text-xs text-slate-400">Memuat data produk...</div>
    {:else if filteredProducts.length === 0}
      <div class="p-12 text-center text-xs text-slate-400">Tidak ada produk yang ditemukan.</div>
    {:else}
      <div class="overflow-x-auto">
        <table class="w-full text-left text-xs">
          <thead class="bg-slate-950/60 text-slate-400 uppercase tracking-wider font-bold border-b border-slate-800">
            <tr>
              <th class="py-3.5 px-4">Nama Produk</th>
              <th class="py-3.5 px-4">SKU</th>
              <th class="py-3.5 px-4">Tipe</th>
              <th class="py-3.5 px-4 text-right">Harga Jual</th>
              <th class="py-3.5 px-4 text-right">Stok Saat Ini</th>
              <th class="py-3.5 px-4">Status Sync</th>
              <th class="py-3.5 px-4 text-center">Aksi</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-slate-800/60">
            {#each filteredProducts as prod (prod.id)}
              <tr class="hover:bg-slate-800/40 transition-colors">
                <td class="py-3.5 px-4 font-semibold text-slate-200">
                  {prod.name}
                  {#if prod.is_ingredient}
                    <span class="ml-1.5 px-1.5 py-0.5 rounded bg-amber-500/20 text-amber-300 text-[10px]">Bahan Baku</span>
                  {/if}
                </td>
                <td class="py-3.5 px-4 font-mono text-slate-400">{prod.sku}</td>
                <td class="py-3.5 px-4 text-slate-400">
                  {prod.track_stock ? 'Fisik (Tracked)' : 'F&B / Non-Track'}
                </td>
                <td class="py-3.5 px-4 text-right font-mono font-bold text-slate-200">
                  Rp {prod.price.toLocaleString()}
                </td>
                <td class="py-3.5 px-4 text-right font-mono font-bold {prod.qty_on_hand <= 0 ? 'text-rose-400' : 'text-emerald-400'}">
                  {prod.qty_on_hand}
                </td>
                <td class="py-3.5 px-4">
                  {#if prod.erp_item_id}
                    <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-bold bg-emerald-500/15 text-emerald-400 border border-emerald-500/30">
                      ✓ Synced
                    </span>
                  {:else}
                    <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-bold bg-amber-500/15 text-amber-400 border border-amber-500/30">
                      ⏳ Pending
                    </span>
                  {/if}
                </td>
                <td class="py-3.5 px-4 text-center">
                  <button
                    type="button"
                    onclick={() => (selectedOpnameProduct = prod)}
                    class="px-3 py-1 bg-slate-800 hover:bg-slate-700 text-slate-200 text-[11px] font-bold rounded-lg transition-colors"
                  >
                    Opname
                  </button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </div>
</div>

<ProductForm
  show={showForm}
  onClose={() => (showForm = false)}
  onSuccess={loadProducts}
/>

<StockOpnameModal
  product={selectedOpnameProduct}
  onClose={() => (selectedOpnameProduct = null)}
  onSuccess={loadProducts}
/>
