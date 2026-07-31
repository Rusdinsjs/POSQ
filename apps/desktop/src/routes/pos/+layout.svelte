<script lang="ts">
  import type { Snippet } from 'svelte';
  import { syncStore } from '$lib/sync/store.svelte';
  import SyncIndicator from '$lib/sync/SyncIndicator.svelte';
  import SyncIssuesPanel from '$lib/sync/SyncIssuesPanel.svelte';

  interface Props {
    children?: Snippet;
  }

  let { children }: Props = $props();

  let showPanel = $state<boolean>(false);

  $effect(() => {
    syncStore.fetchStatus();
    const interval = setInterval(() => {
      syncStore.fetchStatus();
    }, 10000);
    return () => clearInterval(interval);
  });
</script>

<div class="w-full h-full flex flex-col relative">
  <!-- Layout Header Sync Bar -->
  <div class="absolute top-3 right-4 z-40">
    <SyncIndicator onOpenIssues={() => (showPanel = true)} />
  </div>

  {#if children}
    {@render children()}
  {/if}

  <SyncIssuesPanel show={showPanel} onClose={() => (showPanel = false)} />
</div>
