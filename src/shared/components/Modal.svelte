<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    show: boolean;
    title: string;
    widthClass?: string;
    onClose: () => void;
    icon?: Snippet;
    actions?: Snippet;
    children: Snippet;
  }

  let { 
    show = false, 
    title, 
    widthClass = 'w-80', 
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
    class="absolute top-9 right-0 {widthClass} bg-[#171926] border border-white/10 rounded-lg shadow-2xl p-3 z-[160]" 
    role="dialog" 
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.key === 'Escape' && onClose()}
  >
    <div class="flex justify-between items-center mb-2.5">
      <div class="flex items-center gap-1.5 text-xs font-semibold text-slate-100">
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
