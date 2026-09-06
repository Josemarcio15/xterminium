<script lang="ts">
  import { type SavedPath } from '../../../core/types';
  import { configStore } from '../../../core/stores/config.svelte';
  import Modal from '../../../shared/components/Modal.svelte';
  import Button from '@/shared/components/Button.svelte';
  import IconButton from '@/shared/components/IconButton.svelte';
  import { invoke } from '@tauri-apps/api/core';

  interface Props {
    show: boolean;
    activeTabId: string;
    onClose: () => void;
    onNavigate: (path: string) => void;
  }

  let { show = false, activeTabId, onClose, onNavigate }: Props = $props();

  let currentPath = $state('');
  let showForm = $state(false);
  let editingId = $state<string | null>(null);
  let formName = $state('');
  let formPath = $state('');

  async function updateCurrentPath() {
    if (!activeTabId) return;
    try {
      const cwd = await invoke<string>('get_pty_cwd', { id: activeTabId });
      if (cwd) {
        currentPath = cwd;
      }
    } catch {}
  }

  $effect(() => {
    if (show) {
      configStore.init();
      updateCurrentPath();
    }
  });

  function openNewForm() {
    editingId = null;
    formName = '';
    formPath = '';
    showForm = !showForm;
    if (showForm) handleUseCurrent();
  }

  function startEdit(p: SavedPath, e: MouseEvent) {
    e.stopPropagation();
    editingId = p.id;
    formName = p.name;
    formPath = p.path;
    showForm = true;
  }

  function handleUseCurrent() {
    if (!currentPath) return;
    formPath = currentPath;
    const segments = currentPath.replace(/\/+$/, '').split('/');
    formName = segments[segments.length - 1] || currentPath;
  }

  async function savePath() {
    if (!formPath.trim()) return;

    const targetPath = formPath.trim();
    const segments = targetPath.replace(/\/+$/, '').split('/');
    const targetName = formName.trim() || segments[segments.length - 1] || targetPath;

    if (editingId) {
      const updated: SavedPath = {
        id: editingId,
        name: targetName,
        path: targetPath,
      };
      await configStore.updatePath(updated);
    } else {
      const item: SavedPath = {
        id: crypto.randomUUID(),
        name: targetName,
        path: targetPath,
      };
      await configStore.addPath(item);
    }

    formName = '';
    formPath = '';
    editingId = null;
    showForm = false;
  }

  async function removePath(id: string, e: MouseEvent) {
    e.stopPropagation();
    if (editingId === id) {
      showForm = false;
      editingId = null;
    }
    await configStore.removePath(id);
  }
</script>

<Modal {show} title="Diretórios Salvos" {onClose}>
  {#snippet icon()}
    <svg class="text-sky-400" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
    </svg>
  {/snippet}

  {#snippet actions()}
    <IconButton 
      size="xs"
      variant="secondary"
      onclick={openNewForm} 
      title={showForm ? 'Fechar formulário' : 'Adicionar Diretório'}
    >
      {showForm ? '✕' : '+'}
    </IconButton>
  {/snippet}

  <!-- Formulário Novo / Editar Caminho -->
  {#if showForm}
    <form class="bg-[var(--bg-item)] border border-[var(--border-panel)] rounded-lg p-2.5 mb-2.5 flex flex-col gap-2" onsubmit={(e) => { e.preventDefault(); savePath(); }}>
      <div class="text-[11px] font-semibold text-sky-600 dark:text-sky-400 flex items-center justify-between">
        <span>{editingId ? 'Editar Diretório' : 'Novo Diretório'}</span>
        {#if editingId}
          <button type="button" class="text-[var(--text-faint)] hover:text-[var(--text-muted)] text-[10px] bg-transparent border-none cursor-pointer" onclick={() => { showForm = false; editingId = null; }}>Cancelar</button>
        {/if}
      </div>

      {#if currentPath}
        <div class="flex items-center gap-1.5 bg-black/5 dark:bg-white/[0.03] px-2 py-1 rounded text-[10.5px] border border-[var(--border-subtle)]">
          <span class="text-[var(--accent-primary)] font-semibold">Atual:</span>
          <span class="text-[var(--text-muted)] truncate flex-1 font-mono" title={currentPath}>{currentPath}</span>
          <Button variant="secondary" size="xs" onclick={handleUseCurrent} title="Preencher com o atual">Usar</Button>
        </div>
      {/if}
      <input class="bg-[var(--bg-item-input)] border border-[var(--border-subtle)] rounded text-[var(--text-base)] px-2 py-1.5 text-xs outline-none focus:border-sky-400 transition-colors" type="text" placeholder="Nome/Apelido (ex: Web, Projetos)" bind:value={formName} />
      <input class="bg-[var(--bg-item-input)] border border-[var(--border-subtle)] rounded text-[var(--text-base)] px-2 py-1.5 text-xs outline-none focus:border-sky-400 transition-colors" type="text" placeholder="Caminho (ex: /var/www)" bind:value={formPath} required />
      <Button type="submit" variant="primary" size="sm" class="w-full justify-center">
        {editingId ? 'Atualizar Diretório' : 'Salvar'}
      </Button>
    </form>
  {/if}

  <!-- Lista de Diretórios Salvos -->
  <div class="max-h-60 overflow-y-auto flex flex-col gap-1">
    {#if configStore.paths.length === 0}
      <div class="text-center text-[var(--text-muted)] text-xs py-4 leading-relaxed">
        Nenhum diretório salvo.<br />
        Clique no <b>+</b> acima para adicionar.
      </div>
    {:else}
      {#each configStore.paths as p (p.id)}
        <div 
          class="flex justify-between items-center px-2.5 py-2 rounded-lg bg-[var(--bg-item)] border border-[var(--border-subtle)] hover:border-sky-400/50 hover:bg-sky-500/5 cursor-pointer transition-all group {editingId === p.id ? 'border-sky-400 bg-sky-500/10' : ''}" 
          onclick={() => { onNavigate(p.path); onClose(); }}
          role="button"
          tabindex="0"
          onkeydown={(e) => e.key === 'Enter' && (onNavigate(p.path), onClose())}
        >
          <div class="flex flex-col gap-0.5 overflow-hidden pr-2">
            <span class="text-xs font-medium text-[var(--text-base)] truncate">{p.name}</span>
            <span class="text-[10px] text-[var(--text-muted)] font-mono truncate">{p.path}</span>
          </div>
          <div class="flex items-center gap-1.5 shrink-0">
            <!-- Botão de Editar (Lápis SVG) -->
            <button 
              type="button" 
              class="text-[var(--text-muted)] hover:text-sky-300 hover:bg-sky-400/15 p-1 rounded text-xs transition-all cursor-pointer border-none bg-transparent flex items-center justify-center" 
              onclick={(e) => startEdit(p, e)} 
              title="Editar diretório"
            >
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"></path>
                <path d="m15 5 4 4"></path>
              </svg>
            </button>
            <span class="text-[10px] text-sky-400 bg-sky-400/15 px-1.5 py-0.5 rounded">cd ↵</span>
            <button class="text-[var(--text-muted)] hover:text-red-400 hover:bg-red-400/15 p-1 rounded text-xs leading-none transition-all cursor-pointer border-none bg-transparent" onclick={(e) => removePath(p.id, e)} title="Remover">✕</button>
          </div>
        </div>
      {/each}
    {/if}
  </div>
</Modal>
