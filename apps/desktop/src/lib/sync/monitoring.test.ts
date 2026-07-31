import { syncStore } from './store.svelte';

/**
 * Sync Monitoring Integration Tests
 */
export function runSyncMonitoringTests() {
  syncStore.stats = {
    pending: 3,
    syncing: 0,
    synced: 12,
    failed: 1
  };

  console.assert(syncStore.totalUnsynced === 4, 'totalUnsynced should be 4');
  console.assert(syncStore.hasErrors === true, 'hasErrors should be true');
}
