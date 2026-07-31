<script lang="ts">
  import type { Capability } from '$lib/types';
  import { CAPABILITY_METADATA } from './helpers';
  import { capabilityStore } from './store.svelte';

  interface Props {
    capability: Capability;
    showStatus?: boolean;
  }

  let { capability, showStatus = true }: Props = $props();

  let meta = $derived(CAPABILITY_METADATA[capability]);
  let isEnabled = $derived(capabilityStore.has(capability));

  let domainColor = $derived.by(() => {
    switch (meta?.domain) {
      case 'fnb':
        return 'bg-amber-500/15 text-amber-300 border-amber-500/30';
      case 'retail':
        return 'bg-emerald-500/15 text-emerald-300 border-emerald-500/30';
      case 'service':
        return 'bg-purple-500/15 text-purple-300 border-purple-500/30';
      default:
        return 'bg-blue-500/15 text-blue-300 border-blue-500/30';
    }
  });
</script>

{#if meta}
  <span
    class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-xs font-semibold border backdrop-blur-sm transition-all {domainColor}"
    title={meta.description}
  >
    {#if showStatus}
      <span class="w-1.5 h-1.5 rounded-full {isEnabled ? 'bg-emerald-400 animate-pulse' : 'bg-slate-500'}"></span>
    {/if}
    <span>{meta.label}</span>
  </span>
{/if}
