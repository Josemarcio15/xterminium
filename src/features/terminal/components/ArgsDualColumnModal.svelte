<script lang="ts">
  import Button from "@/shared/components/Button.svelte";

  export interface PipelineArgOption {
    label: string;
    value: string;
  }

  interface Props {
    show: boolean;
    activeColumn: "arg1" | "arg2";
    arg1List: PipelineArgOption[];
    arg2List: PipelineArgOption[];
    selectedArg1Index: number;
    selectedArg2Index: number;
    position: { x: number; y: number };
    triggerCmd: string;
    onSelectColumn: (col: "arg1" | "arg2") => void;
    onSelectArg1: (index: number) => void;
    onSelectArg2: (index: number) => void;
    onConfirm: () => void;
    onSkipToVps: () => void;
  }

  let {
    show = false,
    activeColumn = "arg1",
    arg1List,
    arg2List,
    selectedArg1Index,
    selectedArg2Index,
    position,
    triggerCmd = "COMANDO",
    onSelectColumn,
    onSelectArg1,
    onSelectArg2,
    onConfirm,
    onSkipToVps,
  }: Props = $props();

  let arg1Refs: (HTMLButtonElement | null)[] = $state([]);
  let arg2Refs: (HTMLButtonElement | null)[] = $state([]);

  $effect(() => {
    if (activeColumn === "arg1" && selectedArg1Index >= 0 && arg1Refs[selectedArg1Index]) {
      arg1Refs[selectedArg1Index]?.scrollIntoView({ block: "nearest" });
    } else if (activeColumn === "arg2" && selectedArg2Index >= 0 && arg2Refs[selectedArg2Index]) {
      arg2Refs[selectedArg2Index]?.scrollIntoView({ block: "nearest" });
    }
  });
</script>

{#if show}
  <div
    class="fixed z-50 flex flex-col bg-(--bg-panel,#18181b)/95 backdrop-blur-md border border-white/15 rounded-xl shadow-[0_12px_32px_rgba(0,0,0,0.7)] py-2 min-w-[480px] max-w-[620px] text-xs font-mono select-none pointer-events-auto"
    style="left: {Math.max(10, position.x)}px; top: {Math.max(10, position.y)}px;"
  >
    <!-- Header com navegação e instrução -->
    <div
      class="px-3 pb-2 text-[11px] font-bold tracking-wider text-(--text-muted) border-b border-white/10 flex items-center justify-between"
    >
      <div class="flex items-center gap-2">
        <span class="w-2 h-2 rounded-full bg-amber-400 shadow-[0_0_8px_#f59e0b]"></span>
        <span class="text-white uppercase">Pipeline: {triggerCmd}</span>
      </div>
      <div class="flex items-center gap-2 text-[10px] text-zinc-400 font-normal">
        <span><kbd class="px-1 py-0.5 bg-white/10 rounded">← →</kbd> mudar coluna</span>
        <span><kbd class="px-1 py-0.5 bg-white/10 rounded">↑ ↓</kbd> escolher</span>
        <span><kbd class="px-1 py-0.5 bg-white/10 rounded">Enter</kbd> aplicar</span>
        <span><kbd class="px-1 py-0.5 bg-white/10 rounded">Esc</kbd> pular p/ VPS</span>
      </div>
    </div>

    <!-- Colunas Lado a Lado (Args 1 | Args 2) -->
    <div class="grid grid-cols-2 gap-2 p-2.5">
      <!-- Coluna 1: Args 1 (Pré-VPS / Flags) -->
      <div
        class="flex flex-col rounded-lg border transition-all {activeColumn === 'arg1'
          ? 'border-amber-400/50 bg-amber-500/5'
          : 'border-white/5 bg-white/[0.02]'}"
      >
        <button
          type="button"
          class="w-full text-left px-2.5 py-1.5 border-b border-white/5 flex items-center justify-between cursor-pointer bg-transparent"
          onclick={() => onSelectColumn("arg1")}
        >
          <span class="text-[11px] font-bold text-amber-300 flex items-center gap-1.5">
            <span class="w-1.5 h-1.5 rounded-full {activeColumn === 'arg1' ? 'bg-amber-400' : 'bg-zinc-600'}"></span>
            1. Args 1 (Pré-VPS)
          </span>
          {#if activeColumn === "arg1"}
            <span class="text-[9px] px-1.5 py-0.2 rounded bg-amber-400/20 text-amber-200">Ativo</span>
          {/if}
        </button>

        <div class="max-h-48 overflow-y-auto p-1 flex flex-col gap-0.5">
          {#each arg1List as item, idx}
            <button
              bind:this={arg1Refs[idx]}
              type="button"
              class="w-full text-left px-2 py-1.5 rounded flex items-center justify-between text-xs cursor-pointer border-none transition-all {selectedArg1Index === idx
                ? 'bg-amber-500/20 text-amber-200 font-bold border border-amber-500/40'
                : 'bg-transparent text-zinc-300 hover:bg-white/5'}"
              onclick={() => {
                onSelectColumn("arg1");
                onSelectArg1(idx);
              }}
            >
              <span class="truncate">{item.label}</span>
              {#if selectedArg1Index === idx}
                <span class="text-[9px] text-amber-400">✓</span>
              {/if}
            </button>
          {/each}
        </div>
      </div>

      <!-- Coluna 2: Args 2 (Pós-VPS / Destino) -->
      <div
        class="flex flex-col rounded-lg border transition-all {activeColumn === 'arg2'
          ? 'border-emerald-400/50 bg-emerald-500/5'
          : 'border-white/5 bg-white/[0.02]'}"
      >
        <button
          type="button"
          class="w-full text-left px-2.5 py-1.5 border-b border-white/5 flex items-center justify-between cursor-pointer bg-transparent"
          onclick={() => onSelectColumn("arg2")}
        >
          <span class="text-[11px] font-bold text-emerald-300 flex items-center gap-1.5">
            <span class="w-1.5 h-1.5 rounded-full {activeColumn === 'arg2' ? 'bg-emerald-400' : 'bg-zinc-600'}"></span>
            2. Args 2 (Pós-VPS)
          </span>
          {#if activeColumn === "arg2"}
            <span class="text-[9px] px-1.5 py-0.2 rounded bg-emerald-400/20 text-emerald-200">Ativo</span>
          {/if}
        </button>

        <div class="max-h-48 overflow-y-auto p-1 flex flex-col gap-0.5">
          {#each arg2List as item, idx}
            <button
              bind:this={arg2Refs[idx]}
              type="button"
              class="w-full text-left px-2 py-1.5 rounded flex items-center justify-between text-xs cursor-pointer border-none transition-all {selectedArg2Index === idx
                ? 'bg-emerald-500/20 text-emerald-200 font-bold border border-emerald-500/40'
                : 'bg-transparent text-zinc-300 hover:bg-white/5'}"
              onclick={() => {
                onSelectColumn("arg2");
                onSelectArg2(idx);
              }}
            >
              <span class="truncate">{item.label}</span>
              {#if selectedArg2Index === idx}
                <span class="text-[9px] text-emerald-400">✓</span>
              {/if}
            </button>
          {/each}
        </div>
      </div>
    </div>

    <!-- Rodapé: Botões de Ação Rápida -->
    <div class="px-3 pt-2 border-t border-white/10 flex items-center justify-between">
      <Button variant="ghost" size="xs" onclick={onSkipToVps}>
        Pular p/ VPS (sem flags) ⇥
      </Button>
      <div class="flex items-center gap-1.5">
        <Button variant="primary" size="xs" onclick={onConfirm}>
          Confirmar e Escolher VPS ↵
        </Button>
      </div>
    </div>
  </div>
{/if}

