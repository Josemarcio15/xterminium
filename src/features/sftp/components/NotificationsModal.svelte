<script lang="ts">
  import type { SftpNotificationItem } from '../composables/useSftpNotifications.svelte';
  import Button from '@/shared/components/Button.svelte';

  interface Props {
    isOpen: boolean;
    notifications: SftpNotificationItem[];
    onClose: () => void;
  }

  let { isOpen, notifications, onClose }: Props = $props();
</script>

{#if isOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed inset-0 z-[70] flex items-center justify-center bg-black/75 backdrop-blur-md p-4 select-none"
    onclick={onClose}
  >
    <div
      class="bg-[var(--bg-panel)] border border-[var(--border-panel)] rounded-xl shadow-2xl p-5 flex flex-col gap-4 text-[var(--text-base)] w-full max-w-md max-h-[70vh] transition-all animate-in fade-in zoom-in-95 duration-150"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Cabeçalho -->
      <div class="flex items-center justify-between border-b border-[var(--border-subtle)] pb-3">
        <div class="flex items-center gap-2.5">
          <div class="p-2 rounded-lg bg-amber-500/10 border border-amber-500/30 text-amber-500 dark:text-amber-400">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9"></path>
              <path d="M10.3 21a1.94 1.94 0 0 0 3.4 0"></path>
            </svg>
          </div>
          <div>
            <h3 class="font-semibold text-sm text-[var(--text-base)]">Notificações</h3>
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

      <!-- Lista de Notificações -->
      <div class="flex-1 overflow-y-auto space-y-2 pr-1 min-h-[120px] max-h-[40vh]">
        {#if notifications.length === 0}
          <div class="p-6 text-center text-xs text-[var(--text-muted)] flex flex-col items-center gap-2">
            <span>Nenhuma notificação registrada</span>
          </div>
        {:else}
          {#each notifications as item (item.id)}
            <div class="p-2.5 rounded-lg bg-[var(--bg-item)] border border-[var(--border-subtle)] flex flex-col gap-1 text-xs">
              <div class="flex items-center justify-between text-[10px] text-[var(--text-muted)] font-mono">
                <span class="flex items-center gap-1 text-blue-500 dark:text-blue-400">
                  <span class="w-1.5 h-1.5 rounded-full bg-blue-500"></span>
                  Evento
                </span>
                <span>{item.timestamp}</span>
              </div>
              <p class="text-[var(--text-base)] font-sans break-words">{item.message}</p>
            </div>
          {/each}
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
