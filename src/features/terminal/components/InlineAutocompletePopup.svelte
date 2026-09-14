<script lang="ts">
  import type { CustomAlias } from "../../../core/types";

  interface Props {
    suggestions: CustomAlias[];
    selectedIndex: number;
    matchedPrefix: string;
    position: { x: number; y: number };
    onSelect?: (alias: CustomAlias) => void;
  }

  let {
    suggestions,
    selectedIndex,
    matchedPrefix,
    position,
    onSelect,
  }: Props = $props();

  let listContainer: HTMLDivElement | null = $state(null);
  let itemRefs: (HTMLDivElement | null)[] = $state([]);

  $effect(() => {
    if (selectedIndex >= 0 && itemRefs[selectedIndex]) {
      itemRefs[selectedIndex]?.scrollIntoView({ block: "nearest" });
    }
  });
</script>

{#if suggestions.length > 0}
  <div
    class="fixed z-50 pointer-events-auto select-none animate-in fade-in zoom-in-95 duration-100 ease-out"
    style="left: {position.x}px; top: {position.y}px;"
  >
    <!-- Card Flutuante Estilo Inline / Floating Pill -->
    <div
      class="flex flex-col rounded-lg bg-(--bg-card,#18181b)/95 backdrop-blur-md border border-(--border-base,rgba(255,255,255,0.15)) shadow-2xl shadow-black/50 text-xs font-mono overflow-hidden max-w-[500px]"
    >
      <!-- Se houver mais de 1 sugestão, exibe barra superior com dica do Tab -->
      {#if suggestions.length > 1}
        <div
          class="flex items-center justify-between px-2.5 py-1 bg-white/5 border-b border-white/10 text-[10px] text-(--text-muted) gap-3"
        >
          <span class="flex items-center gap-1 font-semibold text-amber-400">
            <span>⚡</span>
            <span>Sugestões ({suggestions.length})</span>
          </span>
          <div class="flex items-center gap-1 text-[9px] text-zinc-400">
            <kbd class="px-1 py-0.5 bg-white/10 rounded border border-white/15 text-zinc-300 font-semibold">Tab ⇥</kbd>
            <span class="text-zinc-500">alternar</span>
            <kbd class="px-1 py-0.5 bg-white/10 rounded border border-white/15 text-zinc-300 font-semibold ml-1">↵ Enter</kbd>
          </div>
        </div>
      {/if}

      <!-- Lista de sugestões inline -->
      <div
        bind:this={listContainer}
        class="flex flex-col p-1 gap-0.5 max-h-[180px] overflow-y-auto overflow-x-hidden scroll-smooth"
      >
        {#each suggestions as s, idx}
          {@const isSelected = idx === selectedIndex}
          {@const remaining = s.alias.toLowerCase().startsWith(matchedPrefix.toLowerCase())
            ? s.alias.slice(matchedPrefix.length)
            : s.alias}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            bind:this={itemRefs[idx]}
            class="flex items-center justify-between gap-3 px-2 py-1 rounded cursor-pointer transition-all {isSelected
              ? 'bg-amber-500/15 border border-amber-500/30 text-zinc-100'
              : 'text-zinc-400 border border-transparent hover:bg-white/5 hover:text-zinc-200'}"
            onclick={() => onSelect?.(s)}
          >
            <div class="flex items-baseline gap-1.5 whitespace-nowrap min-w-0">
              {#if isSelected}
                <span class="text-[10px] text-amber-400 font-bold shrink-0">⚡</span>
              {:else}
                <span class="w-2.5 shrink-0"></span>
              {/if}

              <!-- Texto do Alias com o prefixo digitado em destaque (sem whitespace entre as tags para nao separar as letras) -->
              <span class="text-xs font-semibold tracking-wide shrink-0 font-mono"><span
                  class={isSelected
                    ? "underline decoration-amber-400 decoration-2 underline-offset-4 text-amber-300 font-bold"
                    : "text-zinc-300 font-medium"}
                >{matchedPrefix}</span><span class={isSelected ? "text-zinc-100" : "text-zinc-400"}>{remaining}</span></span>

              <!-- Seta indicadora -->
              <span class="text-zinc-500 text-[10px] shrink-0">➔</span>

              <!-- Comando que será executado -->
              <span
                class="font-medium max-w-[220px] truncate text-[11px] px-1.5 py-0.5 rounded border {isSelected
                  ? 'text-emerald-400 bg-emerald-950/50 border-emerald-500/30'
                  : 'text-emerald-500/80 bg-emerald-950/20 border-emerald-500/10'}"
                title={s.command}
              >
                {s.command}
              </span>
            </div>

            {#if isSelected && suggestions.length === 1}
              <!-- Badge Enter se houver apenas 1 sugestão -->
              <div class="flex items-center gap-1 pl-1 border-l border-zinc-700/60 ml-1 shrink-0">
                <kbd
                  class="px-1.5 py-0.5 text-[10px] font-semibold text-zinc-300 bg-zinc-800/80 rounded border border-zinc-600/60 shadow-sm"
                >
                  ↵ Enter
                </kbd>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  </div>
{/if}
