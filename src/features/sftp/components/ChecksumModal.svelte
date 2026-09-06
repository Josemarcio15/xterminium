<script lang="ts">
  import { SftpService, type FileItem } from '../../../core/services';
  import Button from '@/shared/components/Button.svelte';

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
      class="bg-[var(--bg-panel)] border border-[var(--border-panel)] rounded-xl shadow-2xl p-6 flex flex-col gap-5 text-[var(--text-base)] w-auto min-w-[560px] max-w-[95vw] transition-all"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Cabeçalho -->
      <div class="flex items-center justify-between border-b border-[var(--border-subtle)] pb-3">
        <div class="flex items-center gap-2.5">
          <div class="p-2 rounded-lg bg-indigo-500/10 border border-indigo-500/30 text-indigo-500 dark:text-indigo-400">
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path>
              <path d="m9 12 2 2 4-4"></path>
            </svg>
          </div>
          <div>
            <h3 class="font-semibold text-sm text-[var(--text-base)]">Verificação de Integridade (SHA-256)</h3>
            <p class="text-xs text-[var(--text-muted)]">Comparação criptográfica do arquivo local e remoto</p>
          </div>
        </div>

        <button
          onclick={onClose}
          class="text-[var(--text-muted)] hover:text-[var(--text-base)] p-1 rounded-md hover:bg-black/5 dark:hover:bg-white/5 transition-colors cursor-pointer"
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
          ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-600 dark:text-emerald-300'
          : (loadingLocal || loadingRemote
              ? 'bg-blue-500/10 border-blue-500/30 text-blue-600 dark:text-blue-300'
              : 'bg-amber-500/10 border-amber-500/30 text-amber-600 dark:text-amber-300')}">
          {#if loadingLocal || loadingRemote}
            <div class="w-3 h-3 rounded-full border-2 border-blue-500 border-t-transparent animate-spin"></div>
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
          <div class="flex flex-col gap-1.5 bg-[var(--bg-item)] p-3.5 rounded-lg border border-[var(--border-subtle)]">
            <div class="flex items-center justify-between text-xs">
              <span class="text-purple-600 dark:text-purple-300 font-semibold flex items-center gap-1.5">
                <span class="w-2 h-2 rounded-full bg-purple-500"></span>
                Arquivo Local: <span class="text-[var(--text-base)] font-mono">{localFile.name}</span>
              </span>
              {#if localHash}
                <Button
                  size="xs"
                  variant={copiedField === 'local' ? 'success' : 'secondary'}
                  onclick={() => copyToClipboard(localHash!, 'local')}
                  class="gap-1.5 font-medium"
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
                </Button>
              {/if}
            </div>

            <div class="font-mono text-xs text-[var(--text-base)] break-all select-all tracking-wide bg-[var(--bg-item-input)] px-3 py-2 rounded border border-[var(--border-subtle)]">
              {#if loadingLocal}
                <span class="text-[var(--text-muted)] italic animate-pulse">Calculando SHA-256 do arquivo local...</span>
              {:else if errorLocal}
                <span class="text-red-500 dark:text-red-400">{errorLocal}</span>
              {:else if localHash}
                <span class="text-purple-600 dark:text-purple-300 font-medium">{localHash}</span>
              {/if}
            </div>
          </div>
        {/if}

        <!-- Hash Remoto -->
        {#if remoteFile}
          <div class="flex flex-col gap-1.5 bg-[var(--bg-item)] p-3.5 rounded-lg border border-[var(--border-subtle)]">
            <div class="flex items-center justify-between text-xs">
              <span class="text-blue-600 dark:text-blue-300 font-semibold flex items-center gap-1.5">
                <span class="w-2 h-2 rounded-full bg-blue-500"></span>
                Arquivo Remoto (VPS): <span class="text-[var(--text-base)] font-mono">{remoteFile.name}</span>
              </span>
              {#if remoteHash}
                <Button
                  size="xs"
                  variant={copiedField === 'remote' ? 'success' : 'secondary'}
                  onclick={() => copyToClipboard(remoteHash!, 'remote')}
                  class="gap-1.5 font-medium"
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
                </Button>
              {/if}
            </div>

            <div class="font-mono text-xs text-[var(--text-base)] break-all select-all tracking-wide bg-[var(--bg-item-input)] px-3 py-2 rounded border border-[var(--border-subtle)]">
              {#if loadingRemote}
                <span class="text-[var(--text-muted)] italic animate-pulse">Calculando SHA-256 via streaming SFTP...</span>
              {:else if errorRemote}
                <span class="text-red-500 dark:text-red-400">{errorRemote}</span>
              {:else if remoteHash}
                <span class="text-blue-600 dark:text-blue-300 font-medium">{remoteHash}</span>
              {/if}
            </div>
          </div>
        {/if}
      </div>

      <!-- Rodapé -->
      <div class="flex justify-end pt-2 border-t border-[var(--border-subtle)]">
        <Button
          variant="primary"
          size="sm"
          onclick={onClose}
        >
          Fechar
        </Button>
      </div>
    </div>
  </div>
{/if}
