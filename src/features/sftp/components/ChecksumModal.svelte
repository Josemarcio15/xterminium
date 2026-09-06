<script lang="ts">
  import { SftpService, type FileItem } from '../../../core/services';

  interface Props {
    isOpen: boolean;
    localFile: FileItem | null;
    remoteFile: FileItem | null;
    onClose: () => void;
  }

  let { isOpen, localFile, remoteFile, onClose }: Props = $props();

  let localHash = $state<string | null>(null);
  let remoteHash = $state<string | null>(null);
  let loadingLocal = $state(false);
  let loadingRemote = $state(false);
  let errorLocal = $state<string | null>(null);
  let errorRemote = $state<string | null>(null);
  let copiedField = $state<string | null>(null);

  $effect(() => {
    if (isOpen) {
      localHash = null;
      remoteHash = null;
      errorLocal = null;
      errorRemote = null;

      if (localFile && !localFile.is_dir) {
        loadingLocal = true;
        SftpService.calculateLocalHash(localFile.path)
          .then((hash) => {
            localHash = hash;
          })
          .catch((err) => {
            errorLocal = String(err);
          })
          .finally(() => {
            loadingLocal = false;
          });
      }

      if (remoteFile && !remoteFile.is_dir) {
        loadingRemote = true;
        SftpService.calculateRemoteHash(remoteFile.path)
          .then((hash) => {
            remoteHash = hash;
          })
          .catch((err) => {
            errorRemote = String(err);
          })
          .finally(() => {
            loadingRemote = false;
          });
      }
    }
  });

  let hashesMatch = $derived(
    localHash && remoteHash && localHash.toLowerCase() === remoteHash.toLowerCase()
  );

  async function copyToClipboard(text: string, field: string) {
    try {
      await navigator.clipboard.writeText(text);
      copiedField = field;
      setTimeout(() => {
        if (copiedField === field) copiedField = null;
      }, 2000);
    } catch {
      // fallback
    }
  }
</script>

{#if isOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed inset-0 z-[60] flex items-center justify-center bg-black/75 backdrop-blur-md p-4 select-none"
    onclick={onClose}
  >
    <!-- Modal container que expande até o tamanho do hash sem cortar (min-w e max-w dinâmico) -->
    <div
      class="bg-[#12141d] border border-white/10 rounded-xl shadow-2xl p-6 flex flex-col gap-5 text-gray-200 w-auto min-w-[560px] max-w-[95vw] transition-all"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Cabeçalho -->
      <div class="flex items-center justify-between border-b border-white/10 pb-3">
        <div class="flex items-center gap-2.5">
          <div class="p-2 rounded-lg bg-indigo-500/10 border border-indigo-500/30 text-indigo-400">
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path>
              <path d="m9 12 2 2 4-4"></path>
            </svg>
          </div>
          <div>
            <h3 class="font-semibold text-sm text-white">Verificação de Integridade (SHA-256)</h3>
            <p class="text-xs text-gray-400">Comparação criptográfica do arquivo local e remoto</p>
          </div>
        </div>

        <button
          onclick={onClose}
          class="text-gray-400 hover:text-white p-1 rounded-md hover:bg-white/5 transition-colors cursor-pointer"
          title="Fechar"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>

      <!-- Status de Correspondência (quando ambos forem verificados) -->
      {#if localFile && remoteFile}
        <div class="flex items-center gap-2 px-3 py-2 rounded-lg border text-xs {hashesMatch
          ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-300'
          : (loadingLocal || loadingRemote
              ? 'bg-blue-500/10 border-blue-500/30 text-blue-300'
              : 'bg-amber-500/10 border-amber-500/30 text-amber-300')}">
          {#if loadingLocal || loadingRemote}
            <div class="w-3 h-3 rounded-full border-2 border-blue-400 border-t-transparent animate-spin"></div>
            <span>Calculando hashes em paralelo...</span>
          {:else if hashesMatch}
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="20 6 9 17 4 12"></polyline>
            </svg>
            <span class="font-semibold">Os arquivos são 100% idênticos! (Hashes conferem perfeitamente)</span>
          {:else}
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="12" y1="8" x2="12" y2="12"></line>
              <line x1="12" y1="16" x2="12.01" y2="16"></line>
            </svg>
            <span class="font-semibold">Atenção: Os hashes NÃO coincidem ou houve alteração de integridade.</span>
          {/if}
        </div>
      {/if}

      <!-- Painel de Hashes -->
      <div class="flex flex-col gap-4">
        <!-- Hash Local -->
        {#if localFile}
          <div class="flex flex-col gap-1.5 bg-black/40 p-3.5 rounded-lg border border-white/5">
            <div class="flex items-center justify-between text-xs">
              <span class="text-purple-300 font-semibold flex items-center gap-1.5">
                <span class="w-2 h-2 rounded-full bg-purple-400"></span>
                Arquivo Local: <span class="text-white font-mono">{localFile.name}</span>
              </span>
              {#if localHash}
                <button
                  onclick={() => copyToClipboard(localHash!, 'local')}
                  class="text-[11px] px-2.5 py-1 rounded-md border transition-all cursor-pointer flex items-center gap-1.5 font-medium {copiedField === 'local'
                    ? 'bg-emerald-500/20 text-emerald-300 border-emerald-500/30'
                    : 'bg-purple-600/20 hover:bg-purple-600/30 text-purple-300 hover:text-white border-purple-500/30 active:scale-95'}"
                  title="Copiar hash SHA-256 local"
                >
                  {#if copiedField === 'local'}
                    <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                      <polyline points="20 6 9 17 4 12"></polyline>
                    </svg>
                    <span>Copiado!</span>
                  {:else}
                    <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                      <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                      <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                    </svg>
                    <span>Copiar Hash</span>
                  {/if}
                </button>
              {/if}
            </div>

            <div class="font-mono text-xs text-gray-300 break-all select-all tracking-wide bg-[#0c0e14] px-3 py-2 rounded border border-white/5">
              {#if loadingLocal}
                <span class="text-gray-500 italic animate-pulse">Calculando SHA-256 do arquivo local...</span>
              {:else if errorLocal}
                <span class="text-red-400">{errorLocal}</span>
              {:else if localHash}
                <span class="text-purple-200 font-medium">{localHash}</span>
              {/if}
            </div>
          </div>
        {/if}

        <!-- Hash Remoto -->
        {#if remoteFile}
          <div class="flex flex-col gap-1.5 bg-black/40 p-3.5 rounded-lg border border-white/5">
            <div class="flex items-center justify-between text-xs">
              <span class="text-blue-300 font-semibold flex items-center gap-1.5">
                <span class="w-2 h-2 rounded-full bg-blue-400"></span>
                Arquivo Remoto (VPS): <span class="text-white font-mono">{remoteFile.name}</span>
              </span>
              {#if remoteHash}
                <button
                  onclick={() => copyToClipboard(remoteHash!, 'remote')}
                  class="text-[11px] px-2.5 py-1 rounded-md border transition-all cursor-pointer flex items-center gap-1.5 font-medium {copiedField === 'remote'
                    ? 'bg-emerald-500/20 text-emerald-300 border-emerald-500/30'
                    : 'bg-purple-600/20 hover:bg-purple-600/30 text-purple-300 hover:text-white border-purple-500/30 active:scale-95'}"
                  title="Copiar hash SHA-256 remoto"
                >
                  {#if copiedField === 'remote'}
                    <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                      <polyline points="20 6 9 17 4 12"></polyline>
                    </svg>
                    <span>Copiado!</span>
                  {:else}
                    <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                      <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                      <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                    </svg>
                    <span>Copiar Hash</span>
                  {/if}
                </button>
              {/if}
            </div>

            <div class="font-mono text-xs text-gray-300 break-all select-all tracking-wide bg-[#0c0e14] px-3 py-2 rounded border border-white/5">
              {#if loadingRemote}
                <span class="text-gray-500 italic animate-pulse">Calculando SHA-256 via streaming SFTP...</span>
              {:else if errorRemote}
                <span class="text-red-400">{errorRemote}</span>
              {:else if remoteHash}
                <span class="text-blue-200 font-medium">{remoteHash}</span>
              {/if}
            </div>
          </div>
        {/if}
      </div>

      <!-- Rodapé -->
      <div class="flex justify-end pt-2 border-t border-white/5">
        <button
          onclick={onClose}
          class="px-4 py-1.5 rounded-lg bg-indigo-600/20 hover:bg-indigo-600/30 text-indigo-300 hover:text-white border border-indigo-500/30 text-xs font-semibold shadow-sm transition-all cursor-pointer active:scale-95 flex items-center gap-1.5"
        >
          <span>Fechar</span>
        </button>
      </div>
    </div>
  </div>
{/if}
