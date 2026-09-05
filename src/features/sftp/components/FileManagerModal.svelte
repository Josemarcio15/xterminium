<script lang="ts">
  import { onMount } from 'svelte';
  import {
    SftpService,
    type FileItem,
    type SftpConnectionConfig,
    ConfigService,
  } from '../../../core/services';
  import { type SshHost } from '../../../core/types';
  import { configStore } from '../../../core/stores/config.svelte';

  interface Props {
    isOpen?: boolean;
    isViewMode?: boolean;
    initialLocalPath?: string;
    currentSshInfo?: SshHost;
    onClose?: () => void;
  }

  let { isOpen = true, isViewMode = false, initialLocalPath, currentSshInfo, onClose }: Props = $props();

  // VPS Salvas
  let savedHosts = $state<SshHost[]>([]);

  // Preferências
  let showHiddenFiles = $state(false);

  // Estado Local
  let localPath = $state('');
  let localFiles = $state<FileItem[]>([]);
  let selectedLocal = $state<FileItem | null>(null);
  let loadingLocal = $state(false);

  let filteredLocalFiles = $derived(
    showHiddenFiles ? localFiles : localFiles.filter((f) => !f.name.startsWith('.'))
  );

  // Estado Remoto
  let remotePath = $state('');
  let remoteFiles = $state<FileItem[]>([]);
  let selectedRemote = $state<FileItem | null>(null);
  let isConnected = $state(false);
  let isConnecting = $state(false);
  let loadingRemote = $state(false);

  let filteredRemoteFiles = $derived(
    showHiddenFiles ? remoteFiles : remoteFiles.filter((f) => !f.name.startsWith('.'))
  );

  // Dados de Conexão SFTP Ativa
  let sshHost = $state('');
  let sshUser = $state('');
  let sshPort = $state(22);
  let sshKey = $state<string | undefined>(undefined);

  $effect(() => {
    if (isOpen) {
      ConfigService.loadSshHosts().then((hosts) => {
        savedHosts = hosts;
      });

      if (initialLocalPath) {
        loadLocal(initialLocalPath);
      } else {
        loadLocal();
      }

      if (currentSshInfo?.ip && currentSshInfo?.user) {
        sshHost = currentSshInfo.ip;
        sshUser = currentSshInfo.user;
        sshPort = currentSshInfo.port ? parseInt(currentSshInfo.port, 10) || 22 : 22;
        sshKey = currentSshInfo.key;
        handleConnect();
      }
    }
  });

  async function connectToHost(host: SshHost) {
    sshHost = host.ip;
    sshUser = host.user;
    sshPort = host.port ? parseInt(host.port, 10) || 22 : 22;
    sshKey = host.key;
    await handleConnect();
  }

  // Status e Feedback de Transferência
  let transferStatus = $state('');
  let isTransferring = $state(false);

  async function loadLocal(path?: string) {
    loadingLocal = true;
    try {
      const items = await SftpService.listLocal(path);
      localFiles = items;
      if (path) {
        localPath = path;
      } else if (items.length > 0) {
        // Pega a pasta pai a partir do primeiro item se necessário
        const first = items[0].path;
        localPath = first.substring(0, first.lastIndexOf('/')) || '/';
      }
      selectedLocal = null;
    } catch (err: any) {
      transferStatus = `Erro local: ${err}`;
    } finally {
      loadingLocal = false;
    }
  }

  function goUpLocal() {
    if (!localPath || localPath === '/') return;
    const parent = localPath.substring(0, localPath.lastIndexOf('/')) || '/';
    loadLocal(parent);
  }

  function handleLocalItemClick(item: FileItem) {
    selectedLocal = item;
  }

  function handleLocalItemDblClick(item: FileItem) {
    if (item.is_dir) {
      loadLocal(item.path);
    }
  }

  async function handleConnect() {
    if (!sshHost || !sshUser) {
      transferStatus = 'Informe Host e Usuário';
      return;
    }

    isConnecting = true;
    transferStatus = `Conectando a ${sshUser}@${sshHost}...`;
    try {
      const config: SftpConnectionConfig = {
        host: sshHost,
        user: sshUser,
        port: sshPort,
        key_path: sshKey,
      };
      const home = await SftpService.connect(config);
      isConnected = true;
      remotePath = home;
      transferStatus = 'Conectado com sucesso!';
      await loadRemote(home);
    } catch (err: any) {
      transferStatus = `Falha na conexão: ${err}`;
      isConnected = false;
    } finally {
      isConnecting = false;
    }
  }

  async function handleDisconnect() {
    try {
      await SftpService.disconnect();
    } catch (_) {}
    isConnected = false;
    remoteFiles = [];
    selectedRemote = null;
    transferStatus = 'Desconectado';
  }

  async function loadRemote(path: string) {
    loadingRemote = true;
    try {
      const items = await SftpService.listRemote(path);
      remoteFiles = items;
      remotePath = path;
      selectedRemote = null;
    } catch (err: any) {
      transferStatus = `Erro remoto: ${err}`;
    } finally {
      loadingRemote = false;
    }
  }

  function goUpRemote() {
    if (!remotePath || remotePath === '/') return;
    const parent = remotePath.substring(0, remotePath.lastIndexOf('/')) || '/';
    loadRemote(parent);
  }

  function handleRemoteItemClick(item: FileItem) {
    selectedRemote = item;
  }

  function handleRemoteItemDblClick(item: FileItem) {
    if (item.is_dir) {
      loadRemote(item.path);
    }
  }

  // Ação: Enviar do Local para o Remoto
  async function handleSend() {
    if (!selectedLocal || !isConnected || !remotePath) return;
    isTransferring = true;
    transferStatus = `Enviando ${selectedLocal.name} para o servidor...`;
    try {
      await SftpService.upload(selectedLocal.path, remotePath);
      transferStatus = `Sucesso: ${selectedLocal.name} enviado!`;
      await loadRemote(remotePath);
    } catch (err: any) {
      transferStatus = `Erro ao enviar: ${err}`;
    } finally {
      isTransferring = false;
    }
  }

  // Ação: Receber do Remoto para o Local
  async function handleReceive() {
    if (!selectedRemote || !localPath) return;
    isTransferring = true;
    transferStatus = `Baixando ${selectedRemote.name} para o seu computador...`;
    try {
      await SftpService.download(selectedRemote.path, localPath);
      transferStatus = `Sucesso: ${selectedRemote.name} baixado!`;
      await loadLocal(localPath);
    } catch (err: any) {
      transferStatus = `Erro ao receber: ${err}`;
    } finally {
      isTransferring = false;
    }
  }

  function handleCloseModal() {
    handleDisconnect();
    onClose?.();
  }
</script>

{#if isOpen}
  {#snippet content()}
    <div class="w-full h-full flex flex-col overflow-hidden bg-[#0f111a] text-sm text-gray-200">
      <!-- Cabeçalho -->
      <div class="flex items-center justify-between px-4 py-2.5 bg-[#13151f] border-b border-white/10 shrink-0 select-none">
        <div class="flex items-center gap-2">
          <svg class="w-4 h-4 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4"
            />
          </svg>
          <span class="font-semibold text-white tracking-wide text-xs">Explorador de Arquivos (SFTP)</span>
        </div>

        <div class="flex items-center gap-4">
          <label class="flex items-center gap-1.5 text-xs text-gray-400 hover:text-gray-200 cursor-pointer select-none">
            <input
              type="checkbox"
              bind:checked={showHiddenFiles}
              class="w-3.5 h-3.5 rounded border-white/20 bg-black/40 text-blue-500 focus:ring-0 cursor-pointer"
            />
            <span>Mostrar ocultos</span>
          </label>

          {#if transferStatus}
            <span class="text-xs text-blue-300 animate-pulse truncate max-w-md">{transferStatus}</span>
          {/if}

          {#if !isViewMode}
            <button
              onclick={handleCloseModal}
              class="px-2.5 py-1 text-xs font-medium text-gray-400 hover:text-white bg-white/5 hover:bg-red-500/20 hover:border-red-500/40 border border-transparent rounded transition-all"
              title="Fechar e encerrar conexão"
            >
              Fechar ✕
            </button>
          {/if}
        </div>
      </div>

      <!-- Área de Conteúdo dos 2 Painéis -->
      <div class="flex-1 flex overflow-hidden p-3 gap-3">
        <!-- PAINEL LOCAL (ESQUERDA) -->
        <div class="flex-1 flex flex-col bg-[#12141d] border border-white/5 rounded-lg overflow-hidden">
          <!-- Topo do Painel Local -->
          <div class="p-2 bg-[#161822] border-b border-white/5 flex items-center justify-between gap-2">
            <div class="flex items-center gap-2 flex-1 min-w-0">
              <span class="text-xs font-semibold px-2 py-0.5 rounded bg-purple-500/20 text-purple-300 border border-purple-500/30">Local</span>
              <span class="text-xs text-gray-400 truncate flex-1 font-mono" title={localPath}>{localPath || 'Carregando...'}</span>
            </div>
            <button
              onclick={goUpLocal}
              class="p-1.5 bg-blue-500/10 hover:bg-blue-500/20 text-blue-400 hover:text-blue-300 rounded border border-blue-500/30 transition-all cursor-pointer flex items-center justify-center"
              title="Subir um nível"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="12" y1="19" x2="12" y2="5"></line>
                <polyline points="5 12 12 5 19 12"></polyline>
              </svg>
            </button>
          </div>

          <!-- Lista de Arquivos Locais -->
          <div class="flex-1 overflow-y-auto p-1 space-y-0.5">
            {#if loadingLocal}
              <div class="p-4 text-center text-xs text-gray-500">Lendo arquivos locais...</div>
            {:else if filteredLocalFiles.length === 0}
              <div class="p-4 text-center text-xs text-gray-500">Pasta vazia</div>
            {:else}
              {#each filteredLocalFiles as item}
                <div
                  role="button"
                  tabindex="0"
                  onclick={() => handleLocalItemClick(item)}
                  ondblclick={() => handleLocalItemDblClick(item)}
                  onkeydown={(e) => {
                    if (e.key === 'Enter') handleLocalItemDblClick(item);
                    else if (e.key === ' ') { e.preventDefault(); handleLocalItemClick(item); }
                  }}
                  class="flex items-center justify-between px-2.5 py-1.5 rounded cursor-pointer transition-colors text-xs {selectedLocal?.path === item.path
                    ? 'bg-blue-600/30 border-b-2 border-blue-500 text-blue-200 font-medium'
                    : 'hover:bg-white/5 text-gray-300'}"
                >
                  <div class="flex items-center gap-2 truncate">
                    {#if item.is_dir}
                      <span class="text-yellow-400">📁</span>
                    {:else}
                      <span class="text-gray-400">📄</span>
                    {/if}
                    <span class="truncate">{item.name}</span>
                  </div>
                  <span class="text-[10px] text-gray-500 font-mono ml-2">
                    {item.is_dir ? 'Pasta' : SftpService.formatFileSize(item.size)}
                  </span>
                </div>
              {/each}
            {/if}
          </div>
        </div>

        <!-- AÇÕES CENTRAIS (BOTOES ENVIAR / RECEBER) -->
        <div class="flex flex-col items-center justify-center gap-3 px-1">
          <button
            onclick={handleSend}
            disabled={!selectedLocal || !isConnected || isTransferring}
            class="flex flex-col items-center justify-center w-14 h-12 rounded-lg border transition-all text-xs font-semibold shadow-lg {selectedLocal && isConnected && !isTransferring
              ? 'bg-blue-600 hover:bg-blue-500 text-white border-blue-400 hover:scale-105'
              : 'bg-white/5 border-white/5 text-gray-600 cursor-not-allowed'}"
            title="Enviar arquivo selecionado para o servidor"
          >
            <span>➔</span>
            <span class="text-[9px] mt-0.5">Enviar</span>
          </button>

          <button
            onclick={handleReceive}
            disabled={!selectedRemote || !isConnected || isTransferring}
            class="flex flex-col items-center justify-center w-14 h-12 rounded-lg border transition-all text-xs font-semibold shadow-lg {selectedRemote && isConnected && !isTransferring
              ? 'bg-emerald-600 hover:bg-emerald-500 text-white border-emerald-400 hover:scale-105'
              : 'bg-white/5 border-white/5 text-gray-600 cursor-not-allowed'}"
            title="Receber arquivo do servidor para a máquina local"
          >
            <span>⬅</span>
            <span class="text-[9px] mt-0.5">Receber</span>
          </button>
        </div>

        <!-- PAINEL REMOTO (DIREITA) -->
        <div class="flex-1 flex flex-col bg-[#12141d] border border-white/5 rounded-lg overflow-hidden">
          {#if !isConnected}
            <!-- Seleção de Servidores Salvos -->
            <div class="flex-1 flex flex-col p-4 overflow-y-auto">
              <div class="flex items-center gap-2 pb-3 border-b border-white/10 mb-4">
                <div class="w-8 h-8 rounded-lg bg-blue-500/10 border border-blue-500/20 flex items-center justify-center text-blue-400">
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <rect x="2" y="2" width="20" height="8" rx="2" ry="2"></rect>
                    <rect x="2" y="14" width="20" height="8" rx="2" ry="2"></rect>
                    <line x1="6" y1="6" x2="6.01" y2="6"></line>
                    <line x1="6" y1="18" x2="6.01" y2="18"></line>
                  </svg>
                </div>
                <div>
                  <h3 class="text-xs font-semibold text-white">Servidores</h3>
                </div>
              </div>

              <!-- Lista de Servidores Salvos com 1 clique -->
              {#if savedHosts.length === 0}
                <div class="flex-1 flex flex-col items-center justify-center p-6 text-center">
                  <p class="text-xs text-gray-400 mb-1">Nenhum servidor SSH salvo no momento.</p>
                  <p class="text-[11px] text-gray-500">Adicione servidores pelo botão de SSH na barra de título.</p>
                </div>
              {:else}
                <div class="grid grid-cols-1 gap-2">
                  {#each savedHosts as host}
                    <button
                      onclick={() => connectToHost(host)}
                      disabled={isConnecting}
                      class="w-full flex items-center justify-between p-3 rounded-lg bg-[#181a26] border border-white/5 hover:border-blue-500 hover:bg-blue-600/15 text-left transition-all cursor-pointer group active:scale-[0.99] {isConnecting && sshHost === host.ip ? 'border-blue-500 bg-blue-600/20 ring-1 ring-blue-500/50' : ''}"
                      title="Conectar a {host.label || host.ip}"
                    >
                      <div class="flex items-center gap-3">
                        <div class="w-8 h-8 rounded-lg bg-blue-500/10 border border-blue-500/30 flex items-center justify-center text-blue-400 group-hover:bg-blue-500 group-hover:text-white transition-colors">
                          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <rect x="2" y="2" width="20" height="8" rx="2" ry="2"></rect>
                            <rect x="2" y="14" width="20" height="8" rx="2" ry="2"></rect>
                            <line x1="6" y1="6" x2="6.01" y2="6"></line>
                            <line x1="6" y1="18" x2="6.01" y2="18"></line>
                          </svg>
                        </div>
                        <div>
                          <div class="text-sm font-semibold text-white group-hover:text-blue-300 transition-colors flex items-center gap-2">
                            <span>{host.label || `${host.user}@${host.ip}`}</span>
                            {#if host.key}
                              <span class="text-[9px] px-1.5 py-0.2 bg-amber-500/20 text-amber-300 border border-amber-500/30 rounded" title="Chave privada configurada: {host.key}">🔑 chave</span>
                            {/if}
                          </div>
                          <div class="text-[11px] text-gray-400 font-mono group-hover:text-gray-300">
                            {host.user}@{host.ip}{host.port ? `:${host.port}` : ''}
                          </div>
                        </div>
                      </div>

                      <div class="flex items-center">
                        <span class="text-xs {isConnecting && sshHost === host.ip ? 'text-blue-400' : 'text-gray-500 group-hover:text-blue-400'} transition-colors">➔</span>
                      </div>
                    </button>
                  {/each}
                </div>
              {/if}
            </div>
          {:else}
            <!-- Topo do Painel Remoto -->
            <div class="p-2 bg-[#161822] border-b border-white/5 flex items-center justify-between gap-2">
              <div class="flex items-center gap-2 flex-1 min-w-0">
                <span class="text-xs font-semibold px-2 py-0.5 rounded bg-blue-500/20 text-blue-300 border border-blue-500/30">Remoto</span>
                <span class="text-xs text-gray-400 truncate flex-1 font-mono" title={remotePath}>{remotePath || '/'}</span>
              </div>
              <div class="flex items-center gap-1.5">
                <button
                  onclick={goUpRemote}
                  class="p-1.5 bg-blue-500/10 hover:bg-blue-500/20 text-blue-400 hover:text-blue-300 rounded border border-blue-500/30 transition-all cursor-pointer flex items-center justify-center"
                  title="Subir um nível"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                    <line x1="12" y1="19" x2="12" y2="5"></line>
                    <polyline points="5 12 12 5 19 12"></polyline>
                  </svg>
                </button>
                <button
                  onclick={handleDisconnect}
                  class="px-2 py-0.5 text-xs bg-red-500/20 text-red-300 hover:bg-red-500/30 rounded border border-red-500/30"
                  title="Desconectar do SFTP"
                >
                  Desconectar
                </button>
              </div>
            </div>

            <!-- Lista de Arquivos Remotos -->
            <div class="flex-1 overflow-y-auto p-1 space-y-0.5">
              {#if loadingRemote}
                <div class="p-4 text-center text-xs text-gray-500">Lendo arquivos remotos...</div>
              {:else if filteredRemoteFiles.length === 0}
                <div class="p-4 text-center text-xs text-gray-500">Pasta vazia</div>
              {:else}
                {#each filteredRemoteFiles as item}
                  <div
                    role="button"
                    tabindex="0"
                    onclick={() => handleRemoteItemClick(item)}
                    ondblclick={() => handleRemoteItemDblClick(item)}
                    onkeydown={(e) => {
                      if (e.key === 'Enter') handleRemoteItemDblClick(item);
                      else if (e.key === ' ') { e.preventDefault(); handleRemoteItemClick(item); }
                    }}
                    class="flex items-center justify-between px-2.5 py-1.5 rounded cursor-pointer transition-colors text-xs {selectedRemote?.path === item.path
                      ? 'bg-blue-600/30 border-b-2 border-blue-500 text-blue-200 font-medium'
                      : 'hover:bg-white/5 text-gray-300'}"
                  >
                    <div class="flex items-center gap-2 truncate">
                      {#if item.is_dir}
                        <span class="text-yellow-400">📁</span>
                      {:else}
                        <span class="text-gray-400">📄</span>
                      {/if}
                      <span class="truncate">{item.name}</span>
                    </div>
                    <span class="text-[10px] text-gray-500 font-mono ml-2">
                      {item.is_dir ? 'Pasta' : SftpService.formatFileSize(item.size)}
                    </span>
                    </div>
                  {/each}
                {/if}
              </div>
            {/if}
          </div>
        </div>
      </div>
  {/snippet}

  {#if isViewMode}
    {@render content()}
  {:else}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4 select-none"
      onclick={(e) => {
        if (e.target === e.currentTarget) handleCloseModal();
      }}
    >
      <div
        class="bg-[#181a24] border border-white/10 rounded-xl shadow-2xl w-[92vw] max-w-5xl h-[80vh] flex flex-col overflow-hidden text-sm text-gray-200"
      >
        {@render content()}
      </div>
    </div>
  {/if}
{/if}
