<script lang="ts">
  import { type SavedPath } from './types';
  import { ConfigService } from './config';
  import { invoke } from '@tauri-apps/api/core';

  interface Props {
    show: boolean;
    activeTabId: string;
    onClose: () => void;
    onNavigate: (path: string) => void;
  }

  let { show = false, activeTabId, onClose, onNavigate }: Props = $props();

  let paths = $state<SavedPath[]>([]);
  let currentPath = $state('');
  let showAddForm = $state(false);
  let newName = $state('');
  let newPath = $state('');

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
      ConfigService.loadPaths().then((loaded) => {
        paths = loaded;
      });
      updateCurrentPath();
    }
  });

  function handleUseCurrent() {
    if (!currentPath) return;
    newPath = currentPath;
    const segments = currentPath.replace(/\/+$/, '').split('/');
    newName = segments[segments.length - 1] || currentPath;
  }

  async function addPath() {
    if (!newPath.trim()) return;

    const targetPath = newPath.trim();
    const segments = targetPath.replace(/\/+$/, '').split('/');
    const targetName = newName.trim() || segments[segments.length - 1] || targetPath;

    const item: SavedPath = {
      id: crypto.randomUUID(),
      name: targetName,
      path: targetPath,
    };

    paths.push(item);
    await ConfigService.savePaths(paths);

    newName = '';
    newPath = '';
    showAddForm = false;
  }

  async function removePath(id: string, e: MouseEvent) {
    e.stopPropagation();
    paths = paths.filter((p) => p.id !== id);
    await ConfigService.savePaths(paths);
  }
</script>

{#if show}
  <button 
    type="button" 
    class="fixed inset-0 z-[150] bg-transparent border-none cursor-default" 
    onclick={onClose}
    aria-label="Fechar modal"
  ></button>
  <div 
    class="absolute top-9 right-0 w-80 bg-[#171926] border border-white/10 rounded-lg shadow-2xl p-3 z-[160]" 
    role="dialog" 
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.key === 'Escape' && onClose()}
  >
    <div class="flex justify-between items-center mb-2.5">
      <div class="flex items-center gap-2 text-xs font-semibold text-slate-200">
        <svg class="text-sky-400" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
        </svg>
        <span>Diretórios Salvos</span>
      </div>
      <button 
        class="w-[22px] h-[22px] rounded flex items-center justify-center text-xs bg-white/5 border border-white/10 text-slate-400 hover:bg-white/10 hover:text-white transition-all" 
        onclick={() => { showAddForm = !showAddForm; if (showAddForm) handleUseCurrent(); }} 
        title={showAddForm ? 'Fechar formulário' : 'Adicionar Diretório'}
      >
        {showAddForm ? '✕' : '+'}
      </button>
    </div>

    <!-- Formulário Novo Caminho -->
    {#if showAddForm}
      <form class="bg-[#12141f] border border-white/10 rounded-md p-2.5 mb-2.5 flex flex-col gap-2" onsubmit={(e) => { e.preventDefault(); addPath(); }}>
        {#if currentPath}
          <div class="flex items-center gap-1.5 bg-white/[0.03] px-2 py-1 rounded text-[10.5px] border border-white/5">
            <span class="text-sky-400 font-semibold">Atual:</span>
            <span class="text-slate-400 truncate flex-1 font-mono" title={currentPath}>{currentPath}</span>
            <button type="button" class="bg-sky-500/15 border border-sky-400/30 text-sky-400 px-1.5 py-0.5 rounded text-[10px] hover:bg-sky-400 hover:text-[#0f111a] transition-all" onclick={handleUseCurrent} title="Preencher com o atual">Usar</button>
          </div>
        {/if}
        <input class="bg-[#0d0e17] border border-white/10 rounded text-slate-200 px-2 py-1.5 text-xs outline-none focus:border-sky-400 transition-colors" type="text" placeholder="Nome/Apelido (ex: Web, Projetos)" bind:value={newName} />
        <input class="bg-[#0d0e17] border border-white/10 rounded text-slate-200 px-2 py-1.5 text-xs outline-none focus:border-sky-400 transition-colors" type="text" placeholder="Caminho (ex: /var/www)" bind:value={newPath} required />
        <button type="submit" class="bg-sky-400 hover:bg-sky-500 text-slate-950 font-semibold rounded py-1.5 text-xs transition-colors cursor-pointer">Salvar</button>
      </form>
    {/if}

    <!-- Lista de Diretórios Salvos -->
    <div class="max-h-60 overflow-y-auto flex flex-col gap-1">
      {#if paths.length === 0}
        <div class="text-center text-slate-400 text-xs py-4 leading-relaxed">
          Nenhum diretório salvo.<br />
          Clique no <b>+</b> acima para adicionar.
        </div>
      {:else}
        {#each paths as p (p.id)}
          <div 
            class="flex justify-between items-center px-2 py-1.5 rounded bg-white/[0.02] border border-transparent hover:bg-sky-400/10 hover:border-sky-400/20 cursor-pointer transition-all" 
            onclick={() => { onNavigate(p.path); onClose(); }}
            role="button"
            tabindex="0"
            onkeydown={(e) => e.key === 'Enter' && (onNavigate(p.path), onClose())}
          >
            <div class="flex flex-col gap-0.5 overflow-hidden pr-2">
              <span class="text-xs font-medium text-slate-200 truncate">{p.name}</span>
              <span class="text-[10px] text-slate-400 font-mono truncate">{p.path}</span>
            </div>
            <div class="flex items-center gap-1.5 shrink-0">
              <span class="text-[10px] text-sky-400 bg-sky-400/15 px-1.5 py-0.5 rounded">cd ↵</span>
              <button class="text-slate-400 hover:text-red-400 hover:bg-red-400/15 p-0.5 rounded text-xs leading-none transition-all cursor-pointer" onclick={(e) => removePath(p.id, e)} title="Remover">✕</button>
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </div>
{/if}
