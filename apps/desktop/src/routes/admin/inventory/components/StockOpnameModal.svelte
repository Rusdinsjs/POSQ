<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type { InventoryProduct } from '$lib/types';
  import { showToast } from '$lib/toast.svelte';

  interface Props {
    product: InventoryProduct | null;
    onClose: () => void;
    onSuccess: () => void;
  }

  let { product, onClose, onSuccess }: Props = $props();

  let physicalQty = $state<number>(product ? product.qty_on_hand : 0);
  let notes = $state<string>('');
  let isSubmitting = $state<boolean>(false);

  $effect(() => {
    if (product) {
      physicalQty = product.qty_on_hand;
    }
  });

  let delta = $derived(physicalQty - (product ? product.qty_on_hand : 0));
  let isNegativeAdjustment = $derived(delta < 0);

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    if (!product) return;

    if (isNegativeAdjustment && !notes.trim()) {
      showToast('Catatan wajib diisi untuk penyesuaian pengurangan stok', 'warning');
      return;
    }

    isSubmitting = true;
    try {
      await invoke('record_stock_adjustment', {
        productId: product.id,
        newPhysicalQty: Number(physicalQty),
        notes: notes.trim() || null,
        userId: 'admin_user'
      });

      showToast(`Stock Opname berhasil disimpan! (Selisih: ${delta > 0 ? '+' : ''}${delta})`, 'success');
      onSuccess();
      onClose();
    } catch (err: any) {
      showToast(`Gagal menyimpan opname: ${err}`, 'error');
    } finally {
      isSubmitting = false;
    }
  }
</script>

{#if product}
  <div class="fixed inset-0 bg-slate-950/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
    <div class="bg-slate-900 border border-slate-800 rounded-3xl p-6 max-w-md w-full shadow-2xl space-y-4">
      <div class="flex items-center justify-between border-b border-slate-800 pb-3">
        <h3 class="font-bold text-slate-100 text-base">Stock Opname: {product.name}</h3>
        <button
          type="button"
          onclick={onClose}
          class="text-slate-400 hover:text-white px-2 py-1 rounded-lg bg-slate-800 hover:bg-slate-700 text-xs"
        >
          ✕
        </button>
      </div>

      <form onsubmit={handleSubmit} class="space-y-4">
        <div class="bg-slate-950 p-4 rounded-2xl border border-slate-800 space-y-2 text-xs">
          <div class="flex justify-between text-slate-400">
            <span>SKU:</span>
            <span class="font-mono text-slate-200">{product.sku}</span>
          </div>
          <div class="flex justify-between text-slate-400">
            <span>Stok Sistem (Saat Ini):</span>
            <span class="font-bold text-slate-200">{product.qty_on_hand}</span>
          </div>
        </div>

        <div class="space-y-1.5">
          <label for="physicalQty" class="block text-xs font-bold text-slate-300">Stok Fisik Hasil Perhitungan</label>
          <input
            id="physicalQty"
            type="number"
            step="0.01"
            bind:value={physicalQty}
            required
            class="w-full bg-slate-950 border border-slate-800 rounded-xl px-4 py-2.5 text-sm text-slate-100 font-mono focus:border-indigo-500 focus:outline-none"
          />
        </div>

        <div class="p-3 rounded-xl border text-xs font-semibold flex items-center justify-between {delta === 0 ? 'bg-slate-800/40 border-slate-700 text-slate-400' : delta > 0 ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-400' : 'bg-rose-500/10 border-rose-500/30 text-rose-400'}">
          <span>Hasil Selisih Adjustment:</span>
          <span class="font-mono font-bold text-sm">
            {delta > 0 ? `+${delta}` : delta}
          </span>
        </div>

        <div class="space-y-1.5">
          <label for="notes" class="block text-xs font-bold text-slate-300">
            Catatan / Alasan {isNegativeAdjustment ? '(Wajib diisi)' : '(Opsional)'}
          </label>
          <textarea
            id="notes"
            bind:value={notes}
            placeholder={isNegativeAdjustment ? 'Alasan pengurangan (misal: rusaknya produk/waste)' : 'Keterangan opname...'}
            rows="2"
            class="w-full bg-slate-950 border border-slate-800 rounded-xl p-3 text-xs text-slate-100 focus:border-indigo-500 focus:outline-none"
          ></textarea>
        </div>

        <div class="flex justify-end gap-2 pt-2 border-t border-slate-800">
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
            {isSubmitting ? 'Menyimpan...' : 'Simpan & Sinkronkan'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
