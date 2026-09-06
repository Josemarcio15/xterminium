<script lang="ts">
  import type { SshHost } from '../types';

  interface Props {
    hosts: SshHost[];
    isConnecting: boolean;
    activeConnectingHost?: string;
    onSelectHost: (host: SshHost) => void;
  }

  let { hosts, isConnecting, activeConnectingHost, onSelectHost }: Props = $props();
</script>

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
      <h3 class="text-xs font-semibold text-white">Servidores SSH Salvos</h3>
      <p class="text-[11px] text-gray-400">Clique para conectar diretamente ao SFTP</p>
    </div>
  </div>

  {#if hosts.length === 0}
    <div class="flex-1 flex flex-col items-center justify-center p-6 text-center">
      <p class="text-xs text-gray-400 mb-1">Nenhum servidor SSH salvo no momento.</p>
      <p class="text-[11px] text-gray-500">Adicione servidores pelo botão de SSH na barra de título.</p>
    </div>
  {:else}
    <div class="grid grid-cols-1 gap-2">
      {#each hosts as host}
        <div
          class="w-full flex items-center justify-between p-3 rounded-lg bg-[#181a26] border border-white/5 hover:border-blue-500/50 hover:bg-blue-600/10 transition-all group {isConnecting && activeConnectingHost === host.ip ? 'border-blue-500 bg-blue-600/20 ring-1 ring-blue-500/50' : ''}"
        >
          <button
            onclick={() => onSelectHost(host)}
            disabled={isConnecting}
            class="flex-1 flex items-center gap-3 text-left cursor-pointer bg-transparent border-none p-0 outline-none"
            title="Conectar a {host.label || host.ip}"
          >
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
                  <span class="text-[9px] px-1.5 py-0.5 bg-amber-500/20 text-amber-300 border border-amber-500/30 rounded flex items-center gap-1" title="Chave privada configurada: {host.key}">
                    <svg class="w-2.5 h-2.5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                      <path d="M21 2l-2 2m-1.5 1.5L14 9a5 5 0 1 0-2.5 2.5l7 7 2-2 1.5-1.5z"></path>
                    </svg>
                    chave
                  </span>
                {/if}
              </div>
              <div class="text-[11px] text-gray-400 font-mono group-hover:text-gray-300">
                {host.user}@{host.ip}{host.port ? `:${host.port}` : ''}
              </div>
            </div>
          </button>
        </div>
      {/each}
    </div>
  {/if}
</div>
