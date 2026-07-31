import { invoke } from '@tauri-apps/api/core';
import type { SyncStats, SyncOutboxEntry } from '$lib/types';

class SyncStore {
  stats = $state<SyncStats>({
    pending: 0,
    syncing: 0,
    synced: 0,
    failed: 0
  });

  isSyncing = $state<boolean>(false);
  isOnline = $state<boolean>(typeof navigator !== 'undefined' ? navigator.onLine : true);
  failedEntries = $state<SyncOutboxEntry[]>([]);

  // Derived state
  totalUnsynced = $derived(this.stats.pending + this.stats.failed);
  hasErrors = $derived(this.stats.failed > 0);

  constructor() {
    if (typeof window !== 'undefined') {
      window.addEventListener('online', () => {
        this.isOnline = true;
        this.fetchStatus();
      });
      window.addEventListener('offline', () => {
        this.isOnline = false;
      });
    }
  }

  async fetchStatus() {
    try {
      const res: any = await invoke('get_sync_status_cmd');
      if (res) {
        this.stats = {
          pending: res.pending_outbox_count || 0,
          syncing: 0,
          synced: res.pushed_outbox_count || 0,
          failed: res.failed_outbox_count || 0
        };
      }
    } catch (err) {
      console.warn('Failed to fetch sync status from backend:', err);
    }
  }

  async triggerManualSync() {
    if (this.isSyncing) return;
    this.isSyncing = true;
    try {
      await this.fetchStatus();
    } finally {
      this.isSyncing = false;
    }
  }
}

export const syncStore = new SyncStore();
