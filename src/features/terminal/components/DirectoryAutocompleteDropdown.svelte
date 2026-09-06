<script lang="ts">
  import { type SavedPath } from '../../../core/types';
  import Button from '@/shared/components/Button.svelte';

  interface Props {
    paths: SavedPath[];
    selectedIndex: number;
    position: { x: number; y: number };
    onSelect: (path: SavedPath) => void;
  }

  let { paths, selectedIndex, position, onSelect }: Props = $props();
</script>

{#if paths.length > 0}
  <div
    class="fixed z-50 flex flex-col bg-[var(--bg-panel)]/95 backdrop-blur-md border border-emerald-500/30 rounded-lg shadow-[0_8px_24px_rgba(0,0,0,0.6)] py-1 min-w-[300px] max-w-[420px] text-xs font-mono select-none pointer-events-auto"
    style="left: {Math.max(10, position.x)}px; top: {Math.max(10, position.y)}px;"
  >
    <!-- Header -->
    <div class="px-2.5 py-1 text-[10px] uppercase font-bold tracking-wider text-[var(--text-muted)] border-b border-[var(--border-subtle)] flex items-center justify-between">
      <span class="flex items-center gap-1.5">
        <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 shadow-[0_0_6px_#34d399]"></span>
        Diretórios Salvos
      </span>
      <span class="text-[9px] text-[var(--text-faint)] lowercase font-normal">Tab ↕ · Enter</span>
    </div>

    <!-- Lista de diretórios -->
    <div class="max-h-48 overflow-y-auto overflow-x-hidden flex flex-col p-1 gap-0.5">
      {#each paths as savedPath, idx}
        <button
          type="button"
          tabindex="-1"
          class="w-full text-left px-2 py-1.5 rounded flex items-center justify-between gap-2 border-none cursor-pointer transition-colors {idx === selectedIndex ? 'bg-emerald-500/20 text-emerald-200 border border-emerald-500/40' : 'bg-transparent text-[var(--text-base)] hover:bg-white/5'}"
          onmousedown={(e) => e.preventDefault()}
          onclick={() => onSelect(savedPath)}
        >
          <!-- Ícone de pasta + info -->
          <div class="flex items-center gap-2 min-w-0">
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="{idx === selectedIndex ? 'text-emerald-400' : 'text-[var(--text-faint)]'} shrink-0">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
            </svg>
            <div class="flex flex-col min-w-0">
              <span class="font-bold truncate text-[var(--text-base)]">{savedPath.name}</span>
              <span class="text-[11px] text-[var(--text-muted)] truncate">{savedPath.path}</span>
            </div>
          </div>

          <div class="text-right shrink-0">
            <Button variant="glass" size="xs" class="font-medium pointer-events-none">
              inserir
            </Button>
          </div>
        </button>
      {/each}
    </div>
  </div>
{/if}
