<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';
  import Titlebar from './shared/layout/Titlebar.svelte';
  import ResizeHandles from './shared/layout/ResizeHandles.svelte';
  import TerminalView from './features/terminal/components/TerminalView.svelte';
  import FileManagerModal from './features/sftp/components/FileManagerModal.svelte';
  import { type SshHost } from './core/types';

  interface TabData {
    id: string;
    title: string;
    type: 'local' | 'ssh' | 'sftp';
    sshInfo?: SshHost;
    initialPath?: string;
  }

  let tabs = $state<TabData[]>([]);
  let activeTabId = $state<string>('');
  let terminalRefs: Record<string, ReturnType<typeof TerminalView>> = {};
  let currentTerminalCwd = $state('');

  function createTab(type: 'local' | 'ssh', sshHost?: SshHost) {
    const id = crypto.randomUUID();
    const title = type === 'local' ? 'New Tab' : (sshHost?.label || `${sshHost?.user}@${sshHost?.ip}`);
    tabs.push({ id, title, type, sshInfo: sshHost });
    activeTabId = id;
  }

  function openSftpTab(sshHost?: SshHost) {
    // Se já houver uma aba SFTP com este servidor (ou genérica), apenas foca nela
    const existing = tabs.find(
      (t) => t.type === 'sftp' && (!sshHost || t.sshInfo?.id === sshHost.id)
    );
    if (existing) {
      activeTabId = existing.id;
      return;
    }

    const id = crypto.randomUUID();
    const title = sshHost ? (sshHost.label || sshHost.ip) : 'SFTP';
    tabs.push({
      id,
      title,
      type: 'sftp',
      sshInfo: sshHost,
      initialPath: currentTerminalCwd,
    });
    activeTabId = id;
  }

  function switchTab(id: string) {
    activeTabId = id;
    tick().then(() => {
      terminalRefs[id]?.fitAndFocus();
    });
  }

  function closeTab(id: string, e: MouseEvent) {
    e.stopPropagation();
    const index = tabs.findIndex((t) => t.id === id);
    if (index === -1) return;

    delete terminalRefs[id];
    tabs.splice(index, 1);

    if (tabs.length === 0) {
      createTab('local');
    } else if (activeTabId === id) {
      activeTabId = tabs[Math.max(0, index - 1)].id;
      switchTab(activeTabId);
    }
  }

  function executeCommand(cmd: string) {
    if (!activeTabId) return;
    invoke('write_pty', { id: activeTabId, data: cmd }).catch(console.error);
    terminalRefs[activeTabId]?.fitAndFocus();
  }

  let muteTabs = new Set<string>();

  function navigateSilently(path: string) {
    if (!activeTabId) return;
    const tabId = activeTabId;
    muteTabs.add(tabId);
    invoke('write_pty', { id: tabId, data: `cd ${JSON.stringify(path)}\n` }).catch(console.error);

    setTimeout(() => {
      muteTabs.delete(tabId);
      // Envia Enter simples para redesenhar o prompt limpo na nova pasta
      invoke('write_pty', { id: tabId, data: '\n' }).catch(console.error);
      terminalRefs[tabId]?.fitAndFocus();
    }, 120);
  }

  onMount(() => {
    createTab('local');

    let unlisten: (() => void) | undefined;
    listen<{ id: string; data: string }>('pty-out', (event) => {
      if (muteTabs.has(event.payload.id)) {
        return;
      }
      terminalRefs[event.payload.id]?.write(event.payload.data);
    }).then((fn) => {
      unlisten = fn;
    });

    const handleResize = () => {
      terminalRefs[activeTabId]?.fitAndFocus();
    };

    window.addEventListener('resize', handleResize);

    // Atualiza o título das abas locais dinamicamente com o diretório atual
    async function updateTabTitles() {
      for (const tab of tabs) {
        if (tab.type === 'local') {
          try {
            const cwd = await invoke<string>('get_pty_cwd', { id: tab.id });
            if (cwd) {
              const clean = cwd.replace(/\/+$/, '');
              const dirName = clean.split('/').pop() || '/';
              if (tab.title !== dirName) {
                tab.title = dirName;
              }
              if (tab.id === activeTabId) {
                currentTerminalCwd = cwd;
              }
            }
          } catch {}
        }
      }
    }

    const titleInterval = setInterval(updateTabTitles, 1000);
    updateTabTitles();

    return () => {
      window.removeEventListener('resize', handleResize);
      clearInterval(titleInterval);
      if (unlisten) unlisten();
    };
  });
</script>

<div class="flex flex-col w-screen h-screen overflow-hidden bg-[#0f111a]">
  <Titlebar
    {tabs}
    {activeTabId}
    showFileManager={tabs.some((t) => t.type === 'sftp' && t.id === activeTabId)}
    onSelectTab={switchTab}
    onCloseTab={closeTab}
    onNewTab={() => createTab('local')}
    onConnectSsh={(host) => createTab('ssh', host)}
    onToggleFileManager={() => {
      const activeTab = tabs.find((t) => t.id === activeTabId);
      openSftpTab(activeTab?.type === 'ssh' ? activeTab.sshInfo : undefined);
    }}
    onNavigatePath={(path) => {
      navigateSilently(path);
    }}
  />

  <main class="flex-1 min-h-0 relative bg-[#0f111a]">
    {#each tabs as tab (tab.id)}
      {#if tab.type === 'sftp'}
        <div class="w-full h-full" style:display={activeTabId === tab.id ? 'block' : 'none'}>
          <FileManagerModal
            isOpen={true}
            isViewMode={true}
            initialLocalPath={tab.initialPath || currentTerminalCwd}
            currentSshInfo={tab.sshInfo}
            onClose={() => {
              const fakeEvent = new MouseEvent('click');
              closeTab(tab.id, fakeEvent);
            }}
          />
        </div>
      {:else}
        <TerminalView
          bind:this={terminalRefs[tab.id]}
          id={tab.id}
          type={tab.type}
          sshInfo={tab.sshInfo}
          active={activeTabId === tab.id}
          onNewTab={() => createTab('local')}
        />
      {/if}
    {/each}
  </main>
  <ResizeHandles />
</div>


