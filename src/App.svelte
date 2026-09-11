<script lang="ts">
  import { onMount, tick } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import Titlebar from "./shared/layout/Titlebar.svelte";
  import ResizeHandles from "./shared/layout/ResizeHandles.svelte";
  import TerminalView from "./features/terminal/components/TerminalView.svelte";
  import FileManagerModal from "./features/sftp/components/FileManagerModal.svelte";
  import MatrixRain from "./shared/components/MatrixRain.svelte";
  import FireEffect from "./shared/components/FireEffect.svelte";
  import IceEffect from "./shared/components/IceEffect.svelte";
  import RainEffect from "./shared/components/RainEffect.svelte";
  import UpdateModal from "./features/settings/modals/UpdateModal.svelte";
  import {
    type SshHost,
    isRainTheme,
    isFireTheme,
    isIceTheme,
    isWaterRainTheme,
    hasBackgroundEffect,
  } from "./core/types";
  import { configStore } from "./core/stores/config.svelte";
  import {
    UpdateService,
    type UpdateCheckResult,
  } from "./core/services/update.service";

  interface TabData {
    id: string;
    title: string;
    type: "local" | "ssh" | "sftp";
    sshInfo?: SshHost;
    initialPath?: string;
    isConnectedSsh?: boolean;
  }

  let tabs = $state<TabData[]>([]);
  let activeTabId = $state<string>("");
  let terminalRefs: Record<string, ReturnType<typeof TerminalView>> = {};
  let currentTerminalCwd = $state("");
  let updateInfo = $state<UpdateCheckResult | null>(null);
  let showUpdateModal = $state(false);

  // Efeitos de animação de fundo
  const hasRainEffect = $derived(isRainTheme(configStore.theme));
  const hasFire = $derived(isFireTheme(configStore.theme));
  const hasIce = $derived(isIceTheme(configStore.theme));
  const hasWaterRain = $derived(isWaterRainTheme(configStore.theme));
  const hasAnyEffect = $derived(hasBackgroundEffect(configStore.theme));

  function createTab(type: "local" | "ssh", sshHost?: SshHost) {
    const id = crypto.randomUUID();
    const title =
      type === "local"
        ? "New Tab"
        : sshHost?.label || `${sshHost?.user}@${sshHost?.ip}`;
    tabs.push({ id, title, type, sshInfo: sshHost });
    activeTabId = id;
  }

  function openSftpTab(sshHost?: SshHost) {
    // Se já houver uma aba SFTP com este servidor (ou genérica), apenas foca nela
    const existing = tabs.find(
      (t) => t.type === "sftp" && (!sshHost || t.sshInfo?.id === sshHost.id),
    );
    if (existing) {
      activeTabId = existing.id;
      return;
    }

    const id = crypto.randomUUID();
    const title = sshHost ? sshHost.label || sshHost.ip : "SFTP";
    tabs.push({
      id,
      title,
      type: "sftp",
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

  function closeTab(id: string, e?: MouseEvent) {
    if (e) e.stopPropagation();
    const index = tabs.findIndex((t) => t.id === id);
    if (index === -1) return;

    delete terminalRefs[id];
    tabs.splice(index, 1);

    if (tabs.length === 0) {
      createTab("local");
    } else if (activeTabId === id) {
      activeTabId = tabs[Math.max(0, index - 1)].id;
      switchTab(activeTabId);
    }
  }

  function executeCommand(cmd: string) {
    if (!activeTabId) return;
    invoke("write_pty", { id: activeTabId, data: cmd }).catch(console.error);
    terminalRefs[activeTabId]?.fitAndFocus();
  }

  let muteTabs = new Set<string>();

  function navigateSilently(path: string) {
    if (!activeTabId || !path) return;
    // Remove quebras de linha, retornos de carro e caracteres de controle ANSI perigosos
    const sanitizedPath = path.replace(/[\r\n\x00-\x1f\x7f]/g, "").trim();
    if (!sanitizedPath) return;

    const tabId = activeTabId;
    muteTabs.add(tabId);
    invoke("write_pty", {
      id: tabId,
      data: `cd ${JSON.stringify(sanitizedPath)}\n`,
    }).catch(console.error);

    setTimeout(() => {
      muteTabs.delete(tabId);
      // Envia Enter simples para redesenhar o prompt limpo na nova pasta
      invoke("write_pty", { id: tabId, data: "\n" }).catch(console.error);
      terminalRefs[tabId]?.fitAndFocus();
    }, 120);
  }

  onMount(() => {
    configStore.init();
    createTab("local");

    let unlistenOut: (() => void) | undefined;
    let unlistenExit: (() => void) | undefined;

    listen<{ id: string; data: string }>("pty-out", (event) => {
      if (muteTabs.has(event.payload.id)) {
        return;
      }
      terminalRefs[event.payload.id]?.write(event.payload.data);
    }).then((fn) => {
      unlistenOut = fn;
    });

    listen<string>("pty-exit", (event) => {
      const exitId = event.payload;
      closeTab(exitId);
    }).then((fn) => {
      unlistenExit = fn;
    });

    const handleResize = () => {
      terminalRefs[activeTabId]?.fitAndFocus();
    };

    window.addEventListener("resize", handleResize);

    interface PtyStatusResponse {
      cwd: string;
      foreground_process?: string;
      cmdline?: string;
      is_ssh: boolean;
    }

    // Atualiza o estado, título e tipo das abas dinamicamente (incluindo se entrou em SSH)
    async function updateTabTitles() {
      for (const tab of tabs) {
        if (
          tab.type === "local" ||
          (tab.type === "ssh" && tab.isConnectedSsh)
        ) {
          try {
            const status = await invoke<PtyStatusResponse>("get_pty_status", {
              id: tab.id,
            });
            if (status) {
              if (status.is_ssh) {
                tab.type = "ssh";
                tab.isConnectedSsh = true;
                if (status.cmdline) {
                  // Extrai destino do comando ssh (ex: "ssh user@ip" -> "user@ip")
                  const parts = status.cmdline.split(/\s+/);
                  const dest = parts.find(
                    (p, i) =>
                      i > 0 &&
                      !p.startsWith("-") &&
                      (p.includes("@") || p.includes(".")),
                  );
                  tab.title = dest || "ssh";
                } else {
                  tab.title = "ssh";
                }
              } else {
                // Se estava em SSH e agora o processo terminou (voltou para shell local)
                if (tab.isConnectedSsh) {
                  tab.type = "local";
                  tab.isConnectedSsh = false;
                }
                if (status.cwd) {
                  const clean = status.cwd.replace(/\/+$/, "");
                  const dirName = clean.split("/").pop() || "/";
                  tab.title = dirName;
                  if (tab.id === activeTabId) {
                    currentTerminalCwd = status.cwd;
                  }
                }
              }
            }
          } catch {}
        }
      }
    }

    const titleInterval = setInterval(updateTabTitles, 800);
    updateTabTitles();

    // Verifica atualizações
    const updateTimeout = setTimeout(async () => {
      const res = await UpdateService.checkForUpdates();
      if (res && res.hasUpdate) {
        updateInfo = res;
        showUpdateModal = true;
      }
    }, 2500);

    return () => {
      window.removeEventListener("resize", handleResize);
      clearInterval(titleInterval);
      clearTimeout(updateTimeout);
      if (unlistenOut) unlistenOut();
      if (unlistenExit) unlistenExit();
    };
  });
</script>

<div
  class="flex flex-col w-full h-full overflow-hidden rounded-[var(--window-radius)] bg-[var(--bg-base)]"
>
  <Titlebar
    {tabs}
    {activeTabId}
    showFileManager={tabs.some(
      (t) => t.type === "sftp" && t.id === activeTabId,
    )}
    onSelectTab={switchTab}
    onCloseTab={closeTab}
    onNewTab={() => createTab("local")}
    onConnectSsh={(host) => createTab("ssh", host)}
    onToggleFileManager={() => {
      const activeTab = tabs.find((t) => t.id === activeTabId);
      openSftpTab(activeTab?.type === "ssh" ? activeTab.sshInfo : undefined);
    }}
    onNavigatePath={(path) => {
      navigateSilently(path);
    }}
  />

  <main
    class="flex-1 min-h-0 relative bg-[var(--bg-base)] {hasAnyEffect
      ? 'matrix-rain-active'
      : ''}"
  >
    {#if hasRainEffect}
      <MatrixRain />
    {:else if hasFire}
      <FireEffect />
    {:else if hasIce}
      <IceEffect />
    {:else if hasWaterRain}
      <RainEffect />
    {/if}
    {#each tabs as tab (tab.id)}
      {#if tab.type === "sftp"}
        <!-- `relative z-1`: sem isso o fundo da aba é pintado abaixo do canvas
             da chuva (absolute z-0) e o painel parece translúcido nos temas
             com efeito de animação. -->
        <div
          class="relative z-[1] w-full h-full"
          style:display={activeTabId === tab.id ? "block" : "none"}
        >
          <FileManagerModal
            isOpen={true}
            isViewMode={true}
            initialLocalPath={tab.initialPath || currentTerminalCwd}
            currentSshInfo={tab.sshInfo}
            onClose={() => {
              const fakeEvent = new MouseEvent("click");
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
          onNewTab={() => createTab("local")}
        />
      {/if}
    {/each}
  </main>
  <ResizeHandles />

  {#if showUpdateModal && updateInfo}
    <UpdateModal
      show={showUpdateModal}
      info={updateInfo}
      onClose={() => {
        showUpdateModal = false;
      }}
    />
  {/if}
</div>
