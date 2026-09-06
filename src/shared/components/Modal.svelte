<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    show: boolean;
    title: string;
    widthClass?: string;
    maxHClass?: string;
    onClose: () => void;
    icon?: Snippet;
    actions?: Snippet;
    children: Snippet;
  }

  let { 
    show = false, 
    title, 
    widthClass = 'w-80', 
    maxHClass = '',
    onClose, 
    icon, 
    actions, 
    children 
  }: Props = $props();
</script>

{#if show}
  <button 
    type="button" 
    class="fixed inset-0 z-[150] bg-transparent border-none cursor-default" 
    onclick={onClose}
    aria-label="Fechar modal"
  ></button>
  <div 
    class="absolute top-9 right-0 {widthClass} {maxHClass} bg-[var(--bg-panel)] border border-[var(--border-panel)] rounded-xl shadow-[var(--shadow-panel)] p-3.5 z-[160] flex flex-col" 
    role="dialog" 
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.key === 'Escape' && onClose()}
  >
    <div class="flex justify-between items-center mb-2.5">
      <div class="flex items-center gap-1.5 text-xs font-semibold text-[var(--text-base)]">
        {#if icon}
          {@render icon()}
        {/if}
        <span>{title}</span>
      </div>
      {#if actions}
        {@render actions()}
      {/if}
    </div>

    {@render children()}
  </div>
{/if}
