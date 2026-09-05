<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import SshModal from '../../features/settings/modals/SshModal.svelte';
  import ShortcutsModal from '../../features/settings/modals/ShortcutsModal.svelte';
  import PathsModal from '../../features/settings/modals/PathsModal.svelte';
  import CommandTriggersModal from '../../features/settings/modals/CommandTriggersModal.svelte';
  import { type SshHost, type TabItem } from '../../core/types';

  const appWindow = getCurrentWindow();

  interface Props {
    tabs: TabItem[];
    activeTabId: string;
    onSelectTab: (id: string) => void;
    onCloseTab: (id: string, e: MouseEvent) => void;
    onNewTab: () => void;
    onConnectSsh: (host: SshHost) => void;
    onNavigatePath: (path: string) => void;
    onToggleFileManager?: () => void;
    showFileManager?: boolean;
  }

  let { tabs, activeTabId, onSelectTab, onCloseTab, onNewTab, onConnectSsh, onNavigatePath, onToggleFileManager, showFileManager }: Props = $props();

  let showMenu = $state(false);
  let showPathsModal = $state(false);
  let showSshModal = $state(false);
  let showCommandsModal = $state(false);
  let showShortcutsModal = $state(false);
  let tabsScrollArea: HTMLDivElement | null = $state(null);

  function closeAllModals() {
    showPathsModal = false;
    showSshModal = false;
    showCommandsModal = false;
    showShortcutsModal = false;
  }

  function handleStartDragging(e: MouseEvent) {
    if (
      e.button === 0 &&
      !(e.target as HTMLElement).closest('button, input, .tab, .modal-content, .actions-area')
    ) {
      appWindow.startDragging();
    }
  }

  function handleTabsWheel(e: WheelEvent) {
    if (!tabsScrollArea) return;
    if (e.deltaY !== 0) {
      tabsScrollArea.scrollLeft += e.deltaY;
      e.preventDefault();
    } else if (e.deltaX !== 0) {
      tabsScrollArea.scrollLeft += e.deltaX;
    }
  }
</script>

<header 
  data-tauri-drag-region
  class="h-[38px] bg-[#13151f] border-b border-white/[0.08] flex items-center justify-between pl-2.5 select-none relative z-[100] flex-nowrap overflow-visible" 
  onmousedown={handleStartDragging} 
  role="toolbar" 
  tabindex="-1"
>
  <button 
    class="flex items-center gap-1.5 mr-3 px-2 py-1 rounded-md bg-transparent border border-transparent border-r-white/[0.08] rounded-r-none cursor-pointer shrink-0 hover:bg-white/[0.06] transition-all group" 
    onclick={onNewTab} 
    title="New Tab"
  >
    <img src="/terminal.svg" class="w-[18px] h-[18px] block shrink-0" alt="xterminium Logo" />
    <span class="text-xs font-semibold text-slate-200 tracking-wide lowercase whitespace-nowrap">xterminium</span>
    <span class="text-[13px] font-semibold text-slate-400 inline-flex items-center justify-center w-4 h-4 rounded ml-0.5 leading-none shrink-0 group-hover:text-sky-400 group-hover:bg-sky-400/15 transition-all">+</span>
  </button>

  <div 
    data-tauri-drag-region
    class="flex-1 min-w-0 overflow-x-auto overflow-y-hidden flex items-center scroll-smooth [scrollbar-width:none] [&::-webkit-scrollbar]:hidden" 
    bind:this={tabsScrollArea} 
    onwheel={handleTabsWheel}
  >
    <div data-tauri-drag-region class="flex items-center gap-1 min-w-full h-full">
      {#each tabs as tab (tab.id)}
        <button
          type="button"
          class="tab flex items-center gap-1.5 px-2.5 py-1 rounded-md text-xs font-medium cursor-pointer border transition-all whitespace-nowrap {activeTabId === tab.id ? 'bg-[#1b1e2c] text-slate-100 border-white/[0.12]' : 'bg-white/[0.03] text-slate-400 border-transparent hover:bg-white/[0.08] hover:text-slate-300'}"
          onclick={() => onSelectTab(tab.id)}
        >
          <span class="w-1.5 h-1.5 rounded-full {tab.type === 'ssh' ? 'bg-sky-400 shadow-[0_0_6px_#38bdf8]' : tab.type === 'sftp' ? 'bg-indigo-400 shadow-[0_0_6px_#818cf8]' : 'bg-[#00e699]'}"></span>
          <span class="max-w-[140px] overflow-hidden text-ellipsis">{tab.title}</span>
          <span 
            class="bg-transparent border-none text-inherit text-sm leading-none px-0.5 rounded cursor-pointer opacity-60 hover:opacity-100 hover:bg-white/15" 
            onclick={(e) => onCloseTab(tab.id, e)} 
            onkeydown={(e) => e.key === 'Enter' && onCloseTab(tab.id, e as unknown as MouseEvent)}
            role="button" 
            tabindex="0" 
            title="Fechar aba"
          >×</span>
        </button>
      {/each}
    </div>
  </div>

  <div class="actions-area flex items-center h-full shrink-0">
    <!-- Botão Diretórios Favoritos / Atalho Rápido -->
    <div class="relative">
      <button 
        class="bg-transparent border-none outline-none p-1.5 rounded-md cursor-pointer flex items-center justify-center transition-all hover:bg-white/10 hover:text-white {showPathsModal ? 'bg-sky-400/15 text-sky-400' : 'text-slate-400'}" 
        onclick={() => { closeAllModals(); showPathsModal = !showPathsModal; }} 
        title="Diretórios favoritos (cd rápido)"
      >
        <!-- Ícone de Atalho / Pasta com seta de atalho -->
        <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M15 3h6v6"></path>
          <path d="M10 14L21 3"></path>
          <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
        </svg>
      </button>

      <PathsModal 
        show={showPathsModal} 
        {activeTabId}
        onClose={() => (showPathsModal = false)} 
        onNavigate={(path) => { showPathsModal = false; onNavigatePath(path); }} 
      />
    </div>

    <!-- Menu Hamburguer com Ferramentas -->
    <div class="relative">
      <button 
        class="bg-transparent border-none outline-none p-1.5 rounded-md cursor-pointer flex items-center justify-center transition-all hover:bg-white/10 hover:text-white {showMenu || showSshModal || showCommandsModal || showShortcutsModal || showFileManager ? 'bg-sky-400/15 text-sky-400' : 'text-slate-400'}" 
        onclick={() => (showMenu = !showMenu)} 
        title="Menu de Ferramentas e Configurações"
      >
        <!-- Ícone Hambúrguer SVG -->
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="4" y1="6" x2="20" y2="6"></line>
          <line x1="4" y1="12" x2="20" y2="12"></line>
          <line x1="4" y1="18" x2="20" y2="18"></line>
        </svg>
      </button>

      <!-- Dropdown do Menu Hambúrguer -->
      {#if showMenu}
        <button 
          type="button" 
          class="fixed inset-0 z-[140] bg-transparent border-none cursor-default" 
          onclick={() => (showMenu = false)}
          aria-label="Fechar menu"
        ></button>
        <div 
          class="absolute top-9 right-0 w-56 bg-[#171926] border border-white/10 rounded-lg shadow-2xl p-1.5 z-[150] flex flex-col gap-0.5 text-xs text-slate-300"
          role="menu"
          tabindex="-1"
        >
          <!-- Item: Conexões SSH -->
          <button 
            type="button"
            class="flex items-center gap-2.5 px-2.5 py-1.5 rounded-md hover:bg-white/10 hover:text-white transition-all text-left w-full cursor-pointer border-none bg-transparent"
            onclick={() => { showMenu = false; closeAllModals(); showSshModal = true; }}
          >
            <svg class="text-sky-400 shrink-0" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <rect x="2" y="2" width="20" height="8" rx="2" ry="2"></rect>
              <rect x="2" y="14" width="20" height="8" rx="2" ry="2"></rect>
              <line x1="6" y1="6" x2="6.01" y2="6"></line>
              <line x1="6" y1="18" x2="6.01" y2="18"></line>
            </svg>
            <span class="flex-1">Conexões SSH</span>
          </button>

          <!-- Item: Comandos & Autocomplete -->
          <button 
            type="button"
            class="flex items-center gap-2.5 px-2.5 py-1.5 rounded-md hover:bg-white/10 hover:text-white transition-all text-left w-full cursor-pointer border-none bg-transparent"
            onclick={() => { showMenu = false; closeAllModals(); showCommandsModal = true; }}
          >
            <svg class="text-amber-400 shrink-0" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="4 17 10 11 4 5"></polyline>
              <line x1="12" y1="19" x2="20" y2="19"></line>
            </svg>
            <span class="flex-1">Comandos de VPS</span>
          </button>

          <!-- Item: SFTP / Explorador Duplo -->
          <button 
            type="button"
            class="flex items-center gap-2.5 px-2.5 py-1.5 rounded-md hover:bg-white/10 hover:text-white transition-all text-left w-full cursor-pointer border-none bg-transparent"
            onclick={() => { showMenu = false; closeAllModals(); onToggleFileManager?.(); }}
          >
            <svg class="text-indigo-400 shrink-0" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4"></path>
            </svg>
            <span class="flex-1">Explorador SFTP</span>
          </button>

          <div class="h-[1px] bg-white/10 my-1"></div>

          <!-- Item: Atalhos de Teclado -->
          <button 
            type="button"
            class="flex items-center gap-2.5 px-2.5 py-1.5 rounded-md hover:bg-white/10 hover:text-white transition-all text-left w-full cursor-pointer border-none bg-transparent"
            onclick={() => { showMenu = false; closeAllModals(); showShortcutsModal = true; }}
          >
            <svg class="text-slate-400 shrink-0" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <rect x="2" y="4" width="20" height="16" rx="2" ry="2"></rect>
              <line x1="7" y1="16" x2="17" y2="16"></line>
            </svg>
            <span class="flex-1">Atalhos de Teclado</span>
          </button>
        </div>
      {/if}

      <!-- Modais Renderizados -->
      <SshModal 
        show={showSshModal} 
        onClose={() => (showSshModal = false)} 
        onConnect={(h) => { showSshModal = false; onConnectSsh(h); }} 
      />

      <CommandTriggersModal 
        show={showCommandsModal} 
        onClose={() => (showCommandsModal = false)} 
      />

      <ShortcutsModal 
        show={showShortcutsModal} 
        onClose={() => (showShortcutsModal = false)} 
      />
    </div>

    <div class="w-[1px] h-3.5 bg-white/10 mx-1.5"></div>

    <button class="bg-transparent border-none outline-none text-slate-400 w-11 h-full flex items-center justify-center cursor-pointer hover:bg-white/10 hover:text-white transition-all" onclick={() => appWindow.minimize()} title="Minimizar">
      <svg width="11" height="11" viewBox="0 0 12 12"><rect fill="currentColor" width="10" height="1" x="1" y="6"></rect></svg>
    </button>

    <button class="bg-transparent border-none outline-none text-slate-400 w-11 h-full flex items-center justify-center cursor-pointer hover:bg-white/10 hover:text-white transition-all" onclick={() => appWindow.toggleMaximize()} title="Maximizar">
      <svg width="11" height="11" viewBox="0 0 12 12"><rect fill="none" stroke="currentColor" width="9" height="9" x="1.5" y="1.5"></rect></svg>
    </button>

    <button class="bg-transparent border-none outline-none text-slate-400 w-11 h-full flex items-center justify-center cursor-pointer hover:bg-[#e81123] hover:text-white transition-all" onclick={() => appWindow.close()} title="Fechar">
      <svg width="11" height="11" viewBox="0 0 12 12"><polygon fill="currentColor" points="11 1.7 10.3 1 6 5.3 1.7 1 1 1.7 5.3 6 1 10.3 1.7 11 6 6.7 10.3 11 11 10.3 6.7 6"></polygon></svg>
    </button>
  </div>
</header>
