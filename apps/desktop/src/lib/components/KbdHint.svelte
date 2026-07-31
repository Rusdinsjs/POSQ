<script lang="ts">
  interface Props {
    keys: string[];
    label?: string;
    class?: string;
  }

  let { keys, label = '', class: className = '' }: Props = $props();
</script>

<span class="inline-flex items-center gap-1 text-[10px] font-mono font-medium text-slate-400 select-none {className}">
  {#if label}
    <span class="mr-0.5">{label}</span>
  {/if}
  {#each keys as k, i}
    <kbd
      class="px-1.5 py-0.5 rounded bg-slate-800 border border-slate-700/80 text-slate-300 shadow-sm font-bold tracking-tight text-[10px]"
    >
      {k}
    </kbd>
    {#if i < keys.length - 1}
      <span class="text-slate-500 font-sans text-[9px]">+</span>
    {/if}
  {/each}
</span>
