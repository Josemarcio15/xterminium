<script lang="ts">
  import type { CustomAlias } from '../../../core/types';

  interface Props {
    suggestion: CustomAlias;
    matchedPrefix: string;
    position: { x: number; y: number };
  }

  let { suggestion, matchedPrefix, position }: Props = $props();

  // Divide o alias na parte que o usuário já digitou e o restante que falta
  let remainingAlias = $derived(
    suggestion.alias.toLowerCase().startsWith(matchedPrefix.toLowerCase())
      ? suggestion.alias.slice(matchedPrefix.length)
      : suggestion.alias
  );
</script>

<div
  class="fixed z-50 pointer-events-none select-none animate-in fade-in zoom-in-95 duration-100 ease-out"
  style="left: {position.x}px; top: {position.y}px;"
>
  <!-- Card Flutuante Estilo iOS / Floating Pill -->
  <div class="flex items-center gap-2 px-2.5 py-1.5 rounded-lg bg-[var(--bg-card,#18181b)]/95 backdrop-blur-md border border-[var(--border-base,rgba(255,255,255,0.15))] shadow-xl shadow-black/40 text-xs font-mono">
    <!-- Ícone ou Indicador de Sugestão -->
    <span class="flex items-center justify-center w-4 h-4 rounded bg-amber-500/20 text-amber-400 text-[10px] font-bold shrink-0">
      ⚡
    </span>

    <!-- Visualização do Alias com parte digitada vs restante -->
    <div class="flex items-baseline gap-1.5 whitespace-nowrap">
      <span class="text-sm font-semibold tracking-wide text-zinc-100">
        <span class="underline decoration-amber-400 decoration-2 underline-offset-4 text-amber-300 font-bold">{matchedPrefix}</span><span>{remainingAlias}</span>
      </span>

      <!-- Seta indicadora -->
      <span class="text-zinc-500 text-[11px]">➔</span>

      <!-- Comando Expandido -->
      <span class="text-emerald-400 font-medium max-w-[280px] truncate text-[11px] bg-emerald-950/40 px-1.5 py-0.5 rounded border border-emerald-500/20">
        {suggestion.command}
      </span>
    </div>

    <!-- Badge de Tecla de Aceite (Enter) -->
    <div class="flex items-center gap-1 pl-1 border-l border-zinc-700/60 ml-1">
      <kbd class="px-1.5 py-0.5 text-[10px] font-semibold text-zinc-300 bg-zinc-800/80 rounded border border-zinc-600/60 shadow-sm flex items-center gap-0.5">
        <span>↵</span>
        <span>Enter</span>
      </kbd>
    </div>
  </div>
</div>
