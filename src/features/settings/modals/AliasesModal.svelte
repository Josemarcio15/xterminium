<script lang="ts">
  import { type CustomAlias } from '../../../core/types';
  import { configStore } from '../../../core/stores/config.svelte';
  import Modal from '../../../shared/components/Modal.svelte';
  import Button from '@/shared/components/Button.svelte';
  import IconButton from '@/shared/components/IconButton.svelte';

  interface Props {
    show: boolean;
    onClose: () => void;
  }

  let { show = false, onClose }: Props = $props();

  let showForm = $state(false);
  let editingId = $state<string | null>(null);

  let formAlias = $state('');
  let formCommand = $state('');
  let formDescription = $state('');

  $effect(() => {
    if (show) {
      configStore.init();
    }
  });

  function openNewForm() {
    editingId = null;
    formAlias = '';
    formCommand = '';
    formDescription = '';
    showForm = !showForm;
  }

  function startEdit(item: CustomAlias, e: MouseEvent) {
    e.stopPropagation();
    editingId = item.id;
    formAlias = item.alias;
    formCommand = item.command;
    formDescription = item.description || '';
    showForm = true;
  }

  async function saveAlias() {
    if (!formAlias.trim() || !formCommand.trim()) return;

    if (editingId) {
      const updated: CustomAlias = {
        id: editingId,
        alias: formAlias.trim().toLowerCase(),
        command: formCommand.trim(),
        description: formDescription.trim() || undefined,
      };
      await configStore.updateAlias(updated);
    } else {
      const newAlias: CustomAlias = {
        id: `alias-${Date.now()}`,
        alias: formAlias.trim().toLowerCase(),
        command: formCommand.trim(),
        description: formDescription.trim() || undefined,
      };
      await configStore.addAlias(newAlias);
    }

    formAlias = '';
    formCommand = '';
    formDescription = '';
    editingId = null;
    showForm = false;
  }

  async function removeAlias(id: string, e: MouseEvent) {
    e.stopPropagation();
    if (editingId === id) {
      showForm = false;
      editingId = null;
    }
    await configStore.removeAlias(id);
  }
</script>

<Modal {show} title="Aliases de Comandos" widthClass="w-96" {onClose}>
  {#snippet icon()}
    <svg class="text-amber-400" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon>
    </svg>
  {/snippet}

  {#snippet actions()}
    <div class="flex items-center gap-1">
      <Button
        variant="secondary"
        size="xs"
        onclick={() => configStore.resetAliases()}
        title="Restaurar aliases padrões"
      >
        Padrões
      </Button>
      <IconButton 
        size="xs"
        variant="secondary"
        onclick={openNewForm} 
        title={showForm ? 'Fechar formulário' : 'Adicionar novo alias'}
      >
        {showForm ? '✕' : '+'}
      </IconButton>
    </div>
  {/snippet}

  <!-- Formulário Novo / Editar Alias -->
  {#if showForm}
    <form class="bg-[var(--bg-item)] border border-white/10 rounded-md p-2.5 mb-2.5 flex flex-col gap-2" onsubmit={(e) => { e.preventDefault(); saveAlias(); }}>
      <div class="text-[11px] font-semibold text-amber-400 flex items-center justify-between">
        <span>{editingId ? 'Editar Alias' : 'Novo Alias'}</span>
        {#if editingId}
          <button type="button" class="text-[var(--text-faint)] hover:text-[var(--text-muted)] text-[10px] bg-transparent border-none cursor-pointer" onclick={() => { showForm = false; editingId = null; }}>Cancelar</button>
        {/if}
      </div>

      <div class="flex gap-2">
        <input 
          class="w-1/3 bg-[var(--bg-item-input)] border border-white/10 rounded text-[var(--text-base)] px-2 py-1.5 text-xs outline-none focus:border-amber-400 transition-colors font-mono" 
          type="text" 
          placeholder="Alias (ex: aptupdate)" 
          bind:value={formAlias} 
          required 
        />
        <input 
          class="flex-1 bg-[var(--bg-item-input)] border border-white/10 rounded text-[var(--text-base)] px-2 py-1.5 text-xs outline-none focus:border-amber-400 transition-colors font-mono" 
          type="text" 
          placeholder="Comando referente (ex: apt update && apt upgrade -y)" 
          bind:value={formCommand} 
          required
        />
      </div>

      <div>
        <input 
          class="w-full bg-[var(--bg-item-input)] border border-white/10 rounded text-[var(--text-base)] px-2 py-1.5 text-xs outline-none focus:border-amber-400 transition-colors" 
          type="text" 
          placeholder="Descrição opcional (ex: Atualizar repositórios do sistema)" 
          bind:value={formDescription} 
        />
      </div>

      <Button type="submit" variant="primary" size="sm" class="w-full justify-center">
        {editingId ? 'Atualizar Alias' : 'Salvar Alias'}
      </Button>
    </form>
  {/if}

  <!-- Lista de Aliases Cadastrados -->
  <div class="max-h-72 overflow-y-auto flex flex-col gap-1.5 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
    {#each configStore.aliases as item (item.id)}
      <div class="flex items-center justify-between p-2 rounded bg-white/[0.03] hover:bg-white/[0.06] border border-white/5 transition-all text-xs {editingId === item.id ? 'border-amber-400/40 bg-amber-400/10' : ''}">
        <div class="flex flex-col min-w-0 pr-2">
          <div class="flex items-center gap-1.5 font-mono">
            <span class="text-amber-400 font-bold">{item.alias}</span>
            <span class="text-[10px] text-zinc-500">➔</span>
            <span class="text-emerald-400 text-[11px] bg-emerald-500/10 px-1.5 py-0.5 rounded truncate max-w-[200px]" title={item.command}>
              {item.command}
            </span>
          </div>
          {#if item.description}
            <div class="text-[10px] text-[var(--text-muted)] truncate mt-0.5">
              {item.description}
            </div>
          {/if}
        </div>

        <div class="flex items-center gap-1 shrink-0">
          <!-- Botão de Editar (Lápis SVG padronizado) -->
          <button 
            type="button" 
            class="text-[var(--text-muted)] hover:text-amber-300 hover:bg-amber-400/15 p-1 rounded text-xs transition-all cursor-pointer border-none bg-transparent flex items-center justify-center" 
            onclick={(e) => startEdit(item, e)} 
            title="Editar alias"
          >
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"></path>
              <path d="m15 5 4 4"></path>
            </svg>
          </button>
          <!-- Botão de Excluir (✕ padronizado) -->
          <button 
            type="button" 
            class="bg-transparent border-none text-[var(--text-faint)] hover:text-red-400 p-1 cursor-pointer transition-colors" 
            onclick={(e) => removeAlias(item.id, e)} 
            title="Remover alias"
          >
            ✕
          </button>
        </div>
      </div>
    {:else}
      <div class="text-center py-4 text-xs text-[var(--text-faint)]">
        Nenhum alias configurado.
      </div>
    {/each}
  </div>
</Modal>
