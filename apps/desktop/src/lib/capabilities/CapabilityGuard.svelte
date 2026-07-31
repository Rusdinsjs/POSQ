<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { Capability } from '$lib/types';
  import { capabilityStore } from './store.svelte';

  interface Props {
    capability?: Capability;
    capabilities?: Capability[];
    mode?: 'all' | 'any';
    fallback?: 'hide' | 'disable' | 'message';
    children?: Snippet;
  }

  let {
    capability,
    capabilities = [],
    mode = 'all',
    fallback = 'hide',
    children
  }: Props = $props();

  let isAllowed = $derived.by(() => {
    const list: Capability[] = [];
    if (capability) list.push(capability);
    if (capabilities && capabilities.length > 0) list.push(...capabilities);

    if (list.length === 0) return true;

    if (mode === 'any') {
      return capabilityStore.hasAny(list);
    } else {
      return capabilityStore.hasAll(list);
    }
  });
</script>

{#if isAllowed}
  {#if children}
    {@render children()}
  {/if}
{:else if fallback === 'disable'}
  <div class="opacity-50 pointer-events-none select-none filter grayscale cursor-not-allowed inline-block w-full" aria-disabled="true">
    {#if children}
      {@render children()}
    {/if}
  </div>
{:else if fallback === 'message'}
  <div class="p-3 bg-amber-500/10 border border-amber-500/30 rounded-xl text-amber-300 text-xs flex items-center gap-2">
    <span class="text-sm">🔒</span>
    <span>Fitur ini tidak diaktifkan untuk outlet Anda.</span>
  </div>
{/if}
