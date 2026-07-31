<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { showToast } from '$lib/toast.svelte';

  interface Props {
    show?: boolean;
    onClose: () => void;
    onSuccess: () => void;
  }

  let { show = false, onClose, onSuccess }: Props = $props();

  let name = $state<string>('');
  let sku = $state<string>('');
  let price = $state<number>(0);
  let cost = $state<number>(0);
  let productType = $state<'Physical' | 'Recipe' | 'Bundle'>('Physical');
  let initialQty = $state<number>(0);
  let isIngredient = $state<boolean>(false);
  let isSubmitting = $state<boolean>(false);

  function resetForm() {
    name = '';
    sku = '';
    price = 0;
    cost = 0;
    productType = 'Physical';
    initialQty = 0;
    isIngredient = false;
  }

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    if (!name.trim() || !sku.trim()) {
      showToast('Nama dan SKU produk wajib diisi', 'warning');
      return;
    }
    if (price <= 0) {
      showToast('Harga jual produk harus lebih besar dari 0', 'warning');
      return;
    }

    isSubmitting = true;
    try {
      await invoke('create_product', {
        name: name.trim(),
        sku: sku.trim(),
        price: Number(price),
        cost: Number(cost),
        categoryId: null,
        trackStock: productType === 'Physical',
        initialQty: Number(initialQty),
        imageUrl: null,
        isIngredient: isIngredient || productType === 'Recipe',
        minStockFactor: 0.0,
        bufferStock: 0.0,
        leadTimeDays: 0
      });

      showToast('Produk dibuat secara lokal. Menunggu sinkronisasi ke pusat...', 'info');
      resetForm();
      onSuccess();
      onClose();
    } catch (err: any) {
      showToast(`Gagal membuat produk: ${err}`, 'error');
    } finally {
      isSubmitting = false;
    }
  }
</script>

{#if show}
  <div class="fixed inset-0 bg-slate-950/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
    <div class="bg-slate-900 border border-slate-800 rounded-3xl p-6 max-w-lg w-full shadow-2xl space-y-4">
      <div class="flex items-center justify-between border-b border-slate-800 pb-3">
        <h3 class="font-bold text-slate-100 text-base">Tambah Produk Baru (Edge-Originated)</h3>
        <button
          type="button"
          onclick={onClose}
          class="text-slate-400 hover:text-white px-2 py-1 rounded-lg bg-slate-800 hover:bg-slate-700 text-xs"
        >
          ✕
        </button>
      </div>

      <form onsubmit={handleSubmit} class="space-y-3 text-xs">
        <div class="grid grid-cols-2 gap-3">
          <div class="space-y-1">
            <label for="prodName" class="block font-bold text-slate-300">Nama Produk</label>
            <input
              id="prodName"
              type="text"
              bind:value={name}
              placeholder="Contoh: Kopi Susu Aren"
              required
              class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-slate-100 focus:border-indigo-500 focus:outline-none"
            />
          </div>

          <div class="space-y-1">
            <label for="prodSku" class="block font-bold text-slate-300">SKU / Kode Unik</label>
            <input
              id="prodSku"
              type="text"
              bind:value={sku}
              placeholder="SKU-001"
              required
              class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-slate-100 font-mono focus:border-indigo-500 focus:outline-none"
            />
          </div>
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div class="space-y-1">
            <label for="prodPrice" class="block font-bold text-slate-300">Harga Jual (Rp)</label>
            <input
              id="prodPrice"
              type="number"
              bind:value={price}
              min="1"
              required
              class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-slate-100 font-mono focus:border-indigo-500 focus:outline-none"
            />
          </div>

          <div class="space-y-1">
            <label for="prodCost" class="block font-bold text-slate-300">HPP / Modal (Rp)</label>
            <input
              id="prodCost"
              type="number"
              bind:value={cost}
              min="0"
              class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-slate-100 font-mono focus:border-indigo-500 focus:outline-none"
            />
          </div>
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div class="space-y-1">
            <label for="prodType" class="block font-bold text-slate-300">Tipe Produk</label>
            <select
              id="prodType"
              bind:value={productType}
              class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-slate-100 focus:border-indigo-500 focus:outline-none"
            >
              <option value="Physical">Fisik / Ritel (Track Stock)</option>
              <option value="Recipe">F&B / Resep (Explosion)</option>
              <option value="Bundle">Bundle / Paket</option>
            </select>
          </div>

          {#if productType === 'Physical'}
            <div class="space-y-1">
              <label for="initialQty" class="block font-bold text-slate-300">Stok Awal</label>
              <input
                id="initialQty"
                type="number"
                step="0.01"
                bind:value={initialQty}
                class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-slate-100 font-mono focus:border-indigo-500 focus:outline-none"
              />
            </div>
          {/if}
        </div>

        <div class="flex items-center gap-2 pt-1">
          <input
            id="isIngredient"
            type="checkbox"
            bind:checked={isIngredient}
            class="rounded border-slate-700 bg-slate-950 text-indigo-600 focus:ring-indigo-500"
          />
          <label for="isIngredient" class="text-xs text-slate-400">Jadikan sebagai Bahan Baku (Ingredient)</label>
        </div>

        <div class="flex justify-end gap-2 pt-3 border-t border-slate-800">
          <button
            type="button"
            onclick={onClose}
            class="px-4 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-bold transition-colors"
          >
            Batal
          </button>
          <button
            type="submit"
            disabled={isSubmitting}
            class="px-4 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-bold transition-colors shadow disabled:opacity-50"
          >
            {isSubmitting ? 'Menyimpan...' : 'Buat Produk'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
