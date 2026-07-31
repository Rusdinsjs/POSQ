<script lang="ts">
  import { syncStore } from './store.svelte';

  interface Props {
    onOpenIssues?: () => void;
  }

  let { onOpenIssues }: Props = $props();

  let isOnline = $derived(syncStore.isOnline);
  let pendingCount = $derived(syncStore.stats.pending);
  let failedCount = $derived(syncStore.stats.failed);
  let isSyncing = $derived(syncStore.isSyncing);

  function handleClick() {
    syncStore.triggerManualSync();
    if (failedCount > 0 && onOpenIssues) {
      onOpenIssues();
    }
  }
</script>

<button
  type="button"
  onclick={handleClick}
  class="flex items-center gap-2 px-3 py-1.5 rounded-xl border text-xs font-bold transition-all shadow-sm select-none cursor-pointer {isOnline
    ? failedCount > 0
      ? 'bg-rose-500/20 text-rose-300 border-rose-500/40 hover:bg-rose-500/30'
      : pendingCount > 0
      ? 'bg-amber-500/20 text-amber-300 border-amber-500/40 hover:bg-amber-500/30'
      : 'bg-emerald-500/15 text-emerald-300 border-emerald-500/30 hover:bg-emerald-500/25'
    : 'bg-slate-800 text-slate-400 border-slate-700 hover:bg-slate-700'}"
  title={isOnline ? 'Koneksi Online. Klik untuk sinkronkan' : 'Aplikasi berjalan dalam mode Offline (Local-First)'}
  aria-label="Status Sinkronisasi Data"
>
  <span class="relative flex h-2 w-2">
    {#if isSyncing}
      <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-blue-400 opacity-75"></span>
      <span class="relative inline-flex rounded-full h-2 w-2 bg-blue-500"></span>
    {:else if isOnline}
      <span class="relative inline-flex rounded-full h-2 w-2 {failedCount > 0 ? 'bg-rose-400' : pendingCount > 0 ? 'bg-amber-400' : 'bg-emerald-400'}"></span>
    {:else}
      <span class="relative inline-flex rounded-full h-2 w-2 bg-slate-500"></span>
    {/if}
  </span>

  <span>
    {#if !isOnline}
      Offline
    {:else if isSyncing}
      Menyinkronkan...
    {:else if failedCount > 0}
      {failedCount} Gagal Sync
    {:else if pendingCount > 0}
      {pendingCount} Pending
    {:else}
      Tersinkronisasi
    {/if}
  </span>
</button>
