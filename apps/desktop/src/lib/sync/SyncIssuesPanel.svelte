<script lang="ts">
  import { syncStore } from './store.svelte';

  interface Props {
    show?: boolean;
    onClose?: () => void;
  }

  let { show = false, onClose }: Props = $props();
</script>

{#if show}
  <div class="fixed inset-0 bg-slate-950/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
    <div class="bg-slate-900 border border-slate-800 rounded-3xl p-6 max-w-lg w-full shadow-2xl space-y-4">
      <div class="flex items-center justify-between border-b border-slate-800 pb-3">
        <div class="flex items-center gap-2">
          <span class="text-xl">⚠️</span>
          <h3 class="text-base font-bold text-slate-100">Daftar Kendala Sinkronisasi</h3>
        </div>
        <button
          type="button"
          onclick={onClose}
          class="text-slate-400 hover:text-white px-2 py-1 rounded-lg bg-slate-800 hover:bg-slate-700 text-xs"
        >
          ✕ Tutup
        </button>
      </div>

      <div class="space-y-3">
        <p class="text-xs text-slate-400">
          Aplikasi tetap berjalan normal (Local-First). Data transaksi lokal aman di SQLite.
        </p>

        <div class="bg-slate-950 border border-slate-800 rounded-2xl p-4 space-y-2">
          <div class="flex justify-between text-xs font-mono text-slate-300">
            <span>Pending Outbox:</span>
            <span class="font-bold text-amber-400">{syncStore.stats.pending}</span>
          </div>
          <div class="flex justify-between text-xs font-mono text-slate-300">
            <span>Tersinkronisasi:</span>
            <span class="font-bold text-emerald-400">{syncStore.stats.synced}</span>
          </div>
          <div class="flex justify-between text-xs font-mono text-slate-300">
            <span>Gagal Sinkronisasi:</span>
            <span class="font-bold text-rose-400">{syncStore.stats.failed}</span>
          </div>
        </div>
      </div>

      <div class="flex justify-end gap-2 pt-2">
        <button
          type="button"
          onclick={() => syncStore.triggerManualSync()}
          class="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white font-bold text-xs rounded-xl shadow transition-colors"
        >
          Coba Sinkronkan Ulang
        </button>
      </div>
    </div>
  </div>
{/if}
