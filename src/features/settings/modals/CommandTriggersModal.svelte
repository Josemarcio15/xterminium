<script lang="ts">
  import { configStore } from '../../../core/stores/config.svelte';
  import Modal from '../../../shared/components/Modal.svelte';
  import Button from '@/shared/components/Button.svelte';
  import IconButton from '@/shared/components/IconButton.svelte';

  interface Props {
    show: boolean;
    onClose: () => void;
  }

  let { show = false, onClose }: Props = $props();

  let activeTab = $state<'all' | 'triggers' | 'prefix' | 'suffix'>('all');

  let newTrigger = $state('');
  let newPrefix = $state('');
  let newSuffix = $state('');

  $effect(() => {
    if (show) {
      configStore.init();
    }
  });

  async function handleAddTrigger() {
    if (!newTrigger.trim()) return;
    await configStore.addPipelineTrigger(newTrigger);
    newTrigger = '';
  }

  async function handleAddPrefix() {
    if (!newPrefix.trim()) return;
    await configStore.addPipelinePrefixArg(newPrefix);
    newPrefix = '';
  }

  async function handleAddSuffix() {
    if (!newSuffix.trim()) return;
    await configStore.addPipelineSuffixArg(newSuffix);
    newSuffix = '';
  }
</script>

<Modal {show} title="Comandos & Autocomplete VPS" widthClass="w-[620px]" {onClose}>
  {#snippet icon()}
    <svg class="text-sky-400" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <polyline points="4 17 10 11 4 5"></polyline>
      <line x1="12" y1="19" x2="20" y2="19"></line>
    </svg>
  {/snippet}

  {#snippet actions()}
    <Button
      variant="secondary"
      size="xs"
      onclick={() => configStore.resetVpsPipeline()}
      title="Restaurar padrões"
    >
      Restaurar Padrões
    </Button>
  {/snippet}

  <!-- Pipeline Visual Desacoplado -->
  <div class="bg-black/30 p-2.5 rounded-md border border-white/5 flex flex-col gap-1.5 mb-3">
    <div class="text-[9px] uppercase tracking-wider text-(--text-faint) font-semibold flex items-center justify-between">
      <span>Pipeline de Blocos Independentes (Qualquer comando combina com qualquer argumento)</span>
    </div>
    <div class="flex items-center gap-1.5 font-mono text-xs flex-wrap">
      <span class="text-sky-400 font-bold bg-sky-400/10 px-2 py-0.5 rounded border border-sky-400/20">
        1. Comandos Gatilho
      </span>
      <span class="text-(--text-faint)">➔</span>
      <span class="text-amber-300 bg-amber-400/10 px-2 py-0.5 rounded border border-amber-400/20">
        2. Args 1 (Pré-VPS)
      </span>
      <span class="text-(--text-faint)">➔</span>
      <span class="text-purple-300 bg-purple-400/15 px-2 py-0.5 rounded border border-purple-400/30 font-semibold flex items-center gap-1">
        <span class="w-1.5 h-1.5 rounded-full bg-purple-400 animate-pulse"></span>
        3. Objeto VPS
      </span>
      <span class="text-(--text-faint)">➔</span>
      <span class="text-emerald-400 bg-emerald-400/10 px-2 py-0.5 rounded border border-emerald-400/20">
        4. Args 2 (Pós-VPS)
      </span>
    </div>
  </div>

  <!-- 3 Painéis Independentes em Colunas -->
  <div class="grid grid-cols-3 gap-2.5">
    <!-- COLUNA 1: Comandos (Gatilhos) -->
    <div class="flex flex-col bg-(--bg-item) border border-white/10 rounded-md p-2 min-h-[300px]">
      <div class="flex items-center justify-between pb-1.5 mb-1.5 border-b border-white/5">
        <span class="text-[11px] font-bold text-sky-400">1. Comandos</span>
        <span class="text-[9px] text-(--text-faint) font-mono">{configStore.vpsPipeline.triggers.length} itens</span>
      </div>

      <!-- Input Adicionar Comando -->
      <form class="flex gap-1 mb-2" onsubmit={(e) => { e.preventDefault(); handleAddTrigger(); }}>
        <input 
          class="flex-1 bg-(--bg-item-input) border border-white/10 rounded text-(--text-base) px-1.5 py-1 text-xs outline-none focus:border-sky-400 transition-colors font-mono" 
          type="text" 
          placeholder="ex: ssh, rsync" 
          bind:value={newTrigger} 
        />
        <IconButton size="xs" variant="primary" type="submit" title="Adicionar comando">+</IconButton>
      </form>

      <!-- Lista de Comandos -->
      <div class="flex-1 overflow-y-auto flex flex-col gap-1 max-h-56 scrollbar-none [&::-webkit-scrollbar]:hidden">
        {#each configStore.vpsPipeline.triggers as trigger}
          <div class="flex items-center justify-between px-2 py-1 rounded bg-white/3 hover:bg-white/6 border border-white/5 font-mono text-xs group">
            <span class="text-sky-400 font-bold">{trigger}</span>
            <button 
              type="button" 
              class="opacity-0 group-hover:opacity-100 bg-transparent border-none text-(--text-faint) hover:text-red-400 cursor-pointer p-0.5 text-xs transition-opacity" 
              onclick={() => configStore.removePipelineTrigger(trigger)}
              title="Remover"
            >
              ✕
            </button>
          </div>
        {:else}
          <div class="text-[10px] text-(--text-faint) text-center py-4 italic">Nenhum comando</div>
        {/each}
      </div>
    </div>

    <!-- COLUNA 2: Args 1 (Pré-VPS) -->
    <div class="flex flex-col bg-(--bg-item) border border-white/10 rounded-md p-2 min-h-[300px]">
      <div class="flex items-center justify-between pb-1.5 mb-1.5 border-b border-white/5">
        <span class="text-[11px] font-bold text-amber-300">2. Args 1 (Pré)</span>
        <span class="text-[9px] text-(--text-faint) font-mono">{configStore.vpsPipeline.prefixArgs.length} itens</span>
      </div>

      <!-- Input Adicionar Args 1 -->
      <form class="flex gap-1 mb-2" onsubmit={(e) => { e.preventDefault(); handleAddPrefix(); }}>
        <input 
          class="flex-1 bg-(--bg-item-input) border border-white/10 rounded text-(--text-base) px-1.5 py-1 text-xs outline-none focus:border-amber-400 transition-colors font-mono" 
          type="text" 
          placeholder="ex: -p 8080" 
          bind:value={newPrefix} 
        />
        <IconButton size="xs" variant="secondary" type="submit" title="Adicionar argumento">+</IconButton>
      </form>

      <!-- Lista de Args 1 -->
      <div class="flex-1 overflow-y-auto flex flex-col gap-1 max-h-56 scrollbar-none [&::-webkit-scrollbar]:hidden">
        {#each configStore.vpsPipeline.prefixArgs as arg}
          <div class="flex items-center justify-between px-2 py-1 rounded bg-white/3 hover:bg-white/6 border border-white/5 font-mono text-xs group">
            <span class="text-amber-300">{arg}</span>
            <button 
              type="button" 
              class="opacity-0 group-hover:opacity-100 bg-transparent border-none text-(--text-faint) hover:text-red-400 cursor-pointer p-0.5 text-xs transition-opacity" 
              onclick={() => configStore.removePipelinePrefixArg(arg)}
              title="Remover"
            >
              ✕
            </button>
          </div>
        {:else}
          <div class="text-[10px] text-(--text-faint) text-center py-4 italic">Nenhum argumento</div>
        {/each}
      </div>
    </div>

    <!-- COLUNA 3: Args 2 (Pós-VPS) -->
    <div class="flex flex-col bg-(--bg-item) border border-white/10 rounded-md p-2 min-h-[300px]">
      <div class="flex items-center justify-between pb-1.5 mb-1.5 border-b border-white/5">
        <span class="text-[11px] font-bold text-emerald-400">3. Args 2 (Pós)</span>
        <span class="text-[9px] text-(--text-faint) font-mono">{configStore.vpsPipeline.suffixArgs.length} itens</span>
      </div>

      <!-- Input Adicionar Args 2 -->
      <form class="flex gap-1 mb-2" onsubmit={(e) => { e.preventDefault(); handleAddSuffix(); }}>
        <input 
          class="flex-1 bg-(--bg-item-input) border border-white/10 rounded text-(--text-base) px-1.5 py-1 text-xs outline-none focus:border-emerald-400 transition-colors font-mono" 
          type="text" 
          placeholder="ex: :~/" 
          bind:value={newSuffix} 
        />
        <IconButton size="xs" variant="secondary" type="submit" title="Adicionar sufixo">+</IconButton>
      </form>

      <!-- Lista de Args 2 -->
      <div class="flex-1 overflow-y-auto flex flex-col gap-1 max-h-56 scrollbar-none [&::-webkit-scrollbar]:hidden">
        {#each configStore.vpsPipeline.suffixArgs as arg}
          <div class="flex items-center justify-between px-2 py-1 rounded bg-white/3 hover:bg-white/6 border border-white/5 font-mono text-xs group">
            <span class="text-emerald-400">{arg}</span>
            <button 
              type="button" 
              class="opacity-0 group-hover:opacity-100 bg-transparent border-none text-(--text-faint) hover:text-red-400 cursor-pointer p-0.5 text-xs transition-opacity" 
              onclick={() => configStore.removePipelineSuffixArg(arg)}
              title="Remover"
            >
              ✕
            </button>
          </div>
        {:else}
          <div class="text-[10px] text-(--text-faint) text-center py-4 italic">Nenhum sufixo</div>
        {/each}
      </div>
    </div>
  </div>
</Modal>
