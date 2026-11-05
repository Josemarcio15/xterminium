<script lang="ts">
  import Button from "@/shared/components/Button.svelte";

  export interface PipelineItem {
    id: string;
    label: string;
    sublabel?: string;
    value: string;
    type: "arg1" | "vps" | "arg2";
    raw?: any;
  }

  interface Props {
    items: PipelineItem[];
    selectedIndex: number;
    position: { x: number; y: number };
    stepTitle: string;
    stepColor?: string;
    onSelect: (item: PipelineItem) => void;
  }

  let {
    items,
    selectedIndex,
    position,
    stepTitle = "Sugestões",
    stepColor = "sky",
    onSelect,
  }: Props = $props();

  let itemRefs: (HTMLButtonElement | null)[] = $state([]);

  $effect(() => {
    if (selectedIndex >= 0 && itemRefs[selectedIndex]) {
      itemRefs[selectedIndex]?.scrollIntoView({ block: "nearest" });
    }
  });

  const colorClasses = $derived(() => {
    if (stepColor === "amber") {
      return {
        badge: "bg-amber-400/10 text-amber-300 border-amber-400/20",
        dot: "bg-amber-400 shadow-[0_0_6px_#f59e0b]",
        selected: "bg-amber-500/20 text-amber-200 border border-amber-500/40",
      };
    }
    if (stepColor === "emerald") {
      return {
        badge: "bg-emerald-400/10 text-emerald-300 border-emerald-400/20",
        dot: "bg-emerald-400 shadow-[0_0_6px_#10b981]",
        selected: "bg-emerald-500/20 text-emerald-200 border border-emerald-500/40",
      };
    }
    return {
      badge: "bg-purple-400/10 text-purple-300 border-purple-400/20",
      dot: "bg-purple-400 shadow-[0_0_6px_#c084fc]",
      selected: "bg-purple-500/20 text-purple-200 border border-purple-500/40",
    };
  });
</script>

{#if items.length > 0}
  <div
    class="fixed z-50 flex flex-col bg-(--bg-panel)/95 backdrop-blur-md border border-white/10 rounded-lg shadow-[0_8px_24px_rgba(0,0,0,0.6)] py-1 min-w-72 max-w-96 text-xs font-mono select-none pointer-events-auto"
    style="left: {Math.max(10, position.x)}px; top: {Math.max(10, position.y)}px;"
  >
    <!-- Header do Passo -->
    <div
      class="px-2.5 py-1 text-[10px] uppercase font-bold tracking-wider text-(--text-muted) border-b border-(--border-subtle) flex items-center justify-between"
    >
      <span class="flex items-center gap-1.5">
        <span class="w-1.5 h-1.5 rounded-full {colorClasses().dot}"></span>
        {stepTitle}
      </span>
      <span class="text-[9px] text-(--text-faint) lowercase font-normal">
        {#if stepColor === "amber"}
          Esc = Pular p/ VPS | Tab/Enter
        {:else}
          Esc = Fechar | Tab/Enter
        {/if}
      </span>
    </div>

    <!-- Lista de Itens -->
    <div class="max-h-52 overflow-y-auto overflow-x-hidden flex flex-col p-1 gap-0.5 scroll-smooth">
      {#each items as item, idx}
        <button
          bind:this={itemRefs[idx]}
          type="button"
          tabindex="-1"
          class="w-full text-left px-2 py-1.5 rounded flex items-center justify-between gap-2 border-none cursor-pointer transition-colors {idx === selectedIndex ? colorClasses().selected : 'bg-transparent text-(--text-base) hover:bg-white/5'}"
          onmousedown={(e) => e.preventDefault()}
          onclick={() => onSelect(item)}
        >
          <div class="flex flex-col min-w-0">
            <span class="font-bold truncate text-(--text-base) flex items-center gap-1.5">
              {item.label}
            </span>
            {#if item.sublabel}
              <span class="text-[11px] text-(--text-muted) truncate">
                {item.sublabel}
              </span>
            {/if}
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

