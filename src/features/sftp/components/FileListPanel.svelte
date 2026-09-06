<script lang="ts">
  import type { FileItem } from '../../../core/services';
  import FileListItem from './FileListItem.svelte';

  interface Props {
    title: string;
    path: string;
    files: FileItem[];
    selectedFile: FileItem | null;
    loading: boolean;
    tagColor?: 'purple' | 'blue';
    isRemote?: boolean;
    onSelect: (item: FileItem) => void;
    onDoubleClick: (item: FileItem) => void;
    onGoUp: () => void;
    onCreateFile?: () => void;
    onCreateFolder?: () => void;
    onRename?: (item: FileItem) => void;
    onDelete?: (item: FileItem) => void;
    activeTransfer?: {
      file_name: string;
      percentage: number;
      transferred_bytes: number;
      total_bytes: number;
      direction: 'upload' | 'download';
    } | null;
    extraActionsSnippet?: import('svelte').Snippet;
  }

  let {
    title,
    path,
    files,
    selectedFile,
    loading,
    tagColor = 'purple',
    isRemote = false,
    onSelect,
    onDoubleClick,
    onGoUp, onCreateFile, onCreateFolder, onRename, onDelete,
    activeTransfer = null, extraActionsSnippet, onNavigate,
  }: Props & { onNavigate?: (newPath: string) => void } = $props();

  let isEditingPath = $state(false);
  let editPathValue = $state('');
  let pathInputElement = $state<HTMLInputElement | null>(null);
  let isMenuOpen = $state(false);

  function startEditingPath() {
    editPathValue = path || '/';
    isEditingPath = true;
    setTimeout(() => { pathInputElement?.focus(); pathInputElement?.select(); }, 50);
  }

  function submitPath() {
    if (!isEditingPath) return;
    const trimmed = editPathValue.trim();
    isEditingPath = false;
    if (trimmed && trimmed !== path && onNavigate) onNavigate(trimmed);
  }

  function cancelPathEdit() {
    isEditingPath = false;
  }
</script>

<svelte:window
  onclick={() => {
    if (isMenuOpen) isMenuOpen = false;
  }}
/>

<div class="flex-1 flex flex-col bg-[#12141d] border border-white/5 rounded-lg overflow-hidden select-none">
  <!-- Cabeçalho do Painel -->
  <div class="p-2 bg-[#161822] border-b border-white/5 flex items-center justify-between gap-2 relative">
    <div class="flex items-center gap-2 flex-1 min-w-0 transition-all">
      <span
        class="text-xs font-semibold px-2 py-0.5 rounded border shrink-0 {tagColor === 'purple'
          ? 'bg-purple-500/20 text-purple-300 border-purple-500/30'
          : 'bg-blue-500/20 text-blue-300 border-blue-500/30'}"
      >
        {title}
      </span>

      <!-- Path: Exibição clicável ou Input Expansível -->
      {#if isEditingPath}
        <form
          onsubmit={(e) => {
            e.preventDefault();
            submitPath();
          }}
          class="flex items-center gap-1 flex-1 min-w-0 animate-in fade-in zoom-in-95 duration-150"
        >
          <input
            bind:this={pathInputElement}
            bind:value={editPathValue}
            onkeydown={(e) => {
              if (e.key === 'Escape') cancelPathEdit();
            }}
            onblur={() => {
              setTimeout(() => {
                if (isEditingPath) isEditingPath = false;
              }, 150);
            }}
            placeholder="Digite o caminho..."
            class="flex-1 min-w-0 px-2 py-0.5 rounded text-xs font-mono focus:outline-none shadow-inner {tagColor === 'purple'
              ? 'bg-purple-950/40 border border-purple-500/40 text-purple-100 focus:border-purple-400 focus:ring-1 focus:ring-purple-500/30'
              : 'bg-blue-950/40 border border-blue-500/40 text-blue-100 focus:border-blue-400 focus:ring-1 focus:ring-blue-500/30'}"
          />
          <button
            type="submit"
            class="p-1 rounded text-white shadow transition-all cursor-pointer flex items-center justify-center shrink-0 {tagColor === 'purple'
              ? 'bg-purple-600 hover:bg-purple-500'
              : 'bg-blue-600 hover:bg-blue-500'}"
            title="Ir para o caminho digitado"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <line x1="5" y1="12" x2="19" y2="12"></line>
              <polyline points="12 5 19 12 12 19"></polyline>
            </svg>
          </button>
        </form>
      {:else}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          onclick={startEditingPath}
          class="text-xs px-2 py-0.5 rounded cursor-pointer truncate flex-1 font-mono transition-all border flex items-center justify-between gap-1 group {tagColor === 'purple'
            ? 'bg-purple-500/10 hover:bg-purple-500/20 text-purple-200/90 hover:text-white border-purple-500/20 hover:border-purple-500/40'
            : 'bg-blue-500/10 hover:bg-blue-500/20 text-blue-200/90 hover:text-white border-blue-500/20 hover:border-blue-500/40'}"
          title="Clique para digitar o caminho manualmente"
        >
          <span class="truncate">{path || (loading ? 'Carregando...' : '/')}</span>
          <svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="opacity-0 group-hover:opacity-80 shrink-0">
            <path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"></path>
          </svg>
        </div>
      {/if}
    </div>

    <!-- Controles de Ação Compactos -->
    <div class="flex items-center gap-1.5 shrink-0 relative">
      <!-- Botão Subir Nível -->
      <button
        onclick={onGoUp}
        class="p-1.5 bg-white/5 hover:bg-white/10 text-gray-300 hover:text-white rounded border border-white/10 transition-all cursor-pointer flex items-center justify-center"
        title="Subir um nível de diretório"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="12" y1="19" x2="12" y2="5"></line>
          <polyline points="5 12 12 5 19 12"></polyline>
        </svg>
      </button>

      <!-- Botão Menu Hambúrguer -->
      <div class="relative">
        <button
          onclick={(e) => {
            e.stopPropagation();
            isMenuOpen = !isMenuOpen;
          }}
          class="p-1.5 rounded border transition-all flex items-center justify-center cursor-pointer {isMenuOpen
            ? 'bg-white/15 text-white border-white/30'
            : 'bg-white/5 hover:bg-white/10 text-gray-300 hover:text-white border-white/10'}"
          title="Opções e Ações"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="4" y1="6" x2="20" y2="6"></line>
            <line x1="4" y1="12" x2="20" y2="12"></line>
            <line x1="4" y1="18" x2="20" y2="18"></line>
          </svg>
        </button>

        <!-- Dropdown Flutuante do Menu Hambúrguer -->
        {#if isMenuOpen}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            onclick={(e) => e.stopPropagation()}
            class="absolute right-0 top-full mt-1.5 w-48 bg-[#181a24] border border-white/15 rounded-lg shadow-2xl py-1 z-50 text-xs flex flex-col gap-0.5 backdrop-blur-md animate-in fade-in zoom-in-95 duration-100"
          >
            <!-- Novo Arquivo (Local ou Remoto) -->
            {#if onCreateFile}
              <button
                onclick={() => {
                  isMenuOpen = false;
                  onCreateFile();
                }}
                class="w-full px-3 py-1.5 text-left flex items-center gap-2 text-gray-300 hover:text-white hover:bg-blue-600/20 transition-colors cursor-pointer"
              >
                <svg xmlns="http://www.w3.org/2000/svg" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-blue-400">
                  <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
                  <polyline points="14 2 14 8 20 8"></polyline>
                  <line x1="12" y1="18" x2="12" y2="12"></line>
                  <line x1="9" y1="15" x2="15" y2="15"></line>
                </svg>
                <span>Novo Arquivo</span>
              </button>
            {/if}

            <!-- Nova Pasta (Local ou Remoto) -->
            {#if onCreateFolder}
              <button
                onclick={() => {
                  isMenuOpen = false;
                  onCreateFolder();
                }}
                class="w-full px-3 py-1.5 text-left flex items-center gap-2 text-gray-300 hover:text-white hover:bg-amber-500/20 transition-colors cursor-pointer"
              >
                <svg xmlns="http://www.w3.org/2000/svg" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-amber-400">
                  <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
                  <line x1="12" y1="11" x2="12" y2="17"></line>
                  <line x1="9" y1="14" x2="15" y2="14"></line>
                </svg>
                <span>Nova Pasta</span>
              </button>
            {/if}

            {#if onRename || onDelete}
              <div class="my-1 border-t border-white/5"></div>
            {/if}

            <!-- Renomear Item Selecionado -->
            {#if onRename}
              <button
                onclick={() => {
                  isMenuOpen = false;
                  if (selectedFile) onRename(selectedFile);
                }}
                disabled={!selectedFile}
                class="w-full px-3 py-1.5 text-left flex items-center gap-2 {selectedFile
                  ? 'text-gray-300 hover:text-white hover:bg-purple-600/20 cursor-pointer'
                  : 'text-gray-600 cursor-not-allowed opacity-50'}"
              >
                <svg xmlns="http://www.w3.org/2000/svg" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-purple-400">
                  <path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"></path>
                </svg>
                <span>Renomear</span>
              </button>
            {/if}

            <!-- Excluir Item Selecionado -->
            {#if onDelete}
              <button
                onclick={() => {
                  isMenuOpen = false;
                  if (selectedFile) onDelete(selectedFile);
                }}
                disabled={!selectedFile}
                class="w-full px-3 py-1.5 text-left flex items-center gap-2 {selectedFile
                  ? 'text-red-400 hover:text-red-200 hover:bg-red-500/20 cursor-pointer'
                  : 'text-gray-600 cursor-not-allowed opacity-50'}"
              >
                <svg xmlns="http://www.w3.org/2000/svg" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-red-400">
                  <polyline points="3 6 5 6 21 6"></polyline>
                  <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                </svg>
                <span>Excluir</span>
              </button>
            {/if}

            {#if extraActionsSnippet}
              <div class="my-1 border-t border-white/5"></div>
              <div class="px-2 py-1 flex items-center">
                {@render extraActionsSnippet()}
              </div>
            {/if}
          </div>
        {/if}
      </div>
    </div>
  </div>

  <!-- Lista de Arquivos -->
  <div class="flex-1 overflow-y-auto p-1 space-y-0.5">
    {#if loading}
      <div class="p-4 text-center text-xs text-gray-500">Lendo arquivos...</div>
    {:else if files.length === 0}
      <div class="p-4 text-center text-xs text-gray-500">Pasta vazia</div>
    {:else}
      {#each files as item (item.path)}
        <FileListItem
          {item}
          isSelected={selectedFile?.path === item.path}
          {activeTransfer}
          {onSelect}
          {onDoubleClick}
        />
      {/each}
    {/if}
  </div>
</div>
