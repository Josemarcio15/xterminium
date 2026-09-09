<script lang="ts">
  import Button from '@/shared/components/Button.svelte';
  import { UpdateService, type UpdateCheckResult } from '@/core/services/update.service';

  interface Props {
    show: boolean;
    info: UpdateCheckResult;
    onClose: () => void;
  }

  let { show = false, info, onClose }: Props = $props();

  let isUpdating = $state(false);
  let isDone = $state(false);
  let errorMessage = $state<string | null>(null);

  async function handleConfirmUpdate() {
    isUpdating = true;
    errorMessage = null;
    try {
      await UpdateService.runUpdate();
      isDone = true;
    } catch (err: any) {
      errorMessage = typeof err === 'string' ? err : (err?.message || 'Falha ao atualizar.');
    } finally {
      isUpdating = false;
    }
  }
</script>

{#if show}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 z-[200] bg-black/60 backdrop-blur-sm flex items-center justify-center p-4"
    role="dialog"
    tabindex="-1"
  >
    <div
      class="w-[420px] max-w-full bg-[var(--bg-panel)] border border-[var(--border-panel)] rounded-2xl shadow-2xl p-6 flex flex-col gap-4 relative overflow-hidden"
    >
      <!-- Glow ambient background -->
      <div class="absolute -top-16 -right-16 w-36 h-36 bg-sky-500/15 rounded-full blur-3xl pointer-events-none"></div>

      <!-- Header -->
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-xl bg-sky-500/10 border border-sky-500/20 flex items-center justify-center text-sky-400 shrink-0">
          {#if isDone}
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
            </svg>
          {:else}
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
            </svg>
          {/if}
        </div>
        <div class="flex-1 min-w-0">
          <h2 class="text-sm font-semibold text-[var(--text-base)]">
            {#if isDone}
              Atualização Concluída
            {:else if isUpdating}
              Atualizando xterminium...
            {:else}
              Nova Atualização Disponível
            {/if}
          </h2>
          <p class="text-xs text-[var(--text-muted)]">
            {#if isDone}
              Pronto para uso
            {:else}
              v{info.currentVersion} &rarr; <span class="text-sky-400 font-medium">v{info.latestVersion}</span>
            {/if}
          </p>
        </div>
      </div>

      <!-- Body / Status View -->
      <div class="py-1">
        {#if isUpdating}
          <div class="flex flex-col items-center justify-center py-6 gap-3">
            <div class="relative w-12 h-12">
              <div class="absolute inset-0 rounded-full border-2 border-sky-500/20"></div>
              <div class="absolute inset-0 rounded-full border-2 border-sky-400 border-t-transparent animate-spin"></div>
            </div>
            <p class="text-xs text-[var(--text-muted)] animate-pulse">Baixando e instalando nova versão...</p>
          </div>
        {:else if isDone}
          <div class="p-3.5 bg-emerald-500/10 border border-emerald-500/20 rounded-xl flex items-start gap-2.5">
            <svg class="text-emerald-400 shrink-0 mt-0.5" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <p class="text-xs text-emerald-300 font-medium leading-relaxed">
              Reinicie o aplicativo para aplicar a atualização.
            </p>
          </div>
        {:else}
          {#if errorMessage}
            <div class="p-3 bg-red-500/10 border border-red-500/20 rounded-xl text-xs text-red-300 mb-2">
              {errorMessage}
            </div>
          {/if}

          <p class="text-xs text-[var(--text-muted)] leading-relaxed">
            Uma nova versão do xterminium está pronta para ser instalada. Deseja realizar a atualização agora?
          </p>

          {#if info.releaseNotes}
            <div class="mt-3 p-3 bg-black/20 rounded-xl border border-[var(--border-subtle)] max-h-32 overflow-y-auto text-[11px] text-[var(--text-muted)] font-mono whitespace-pre-wrap select-text">
              {info.releaseNotes}
            </div>
          {/if}
        {/if}
      </div>

      <!-- Actions -->
      <div class="flex justify-end items-center gap-2 pt-2 border-t border-[var(--border-subtle)]">
        {#if isDone}
          <Button variant="primary" size="sm" onclick={onClose}>
            Fechar
          </Button>
        {:else if isUpdating}
          <span class="text-xs text-[var(--text-muted)] italic">Aguarde a finalização...</span>
        {:else}
          <Button variant="secondary" size="sm" onclick={onClose}>
            Depois
          </Button>
          <Button variant="primary" size="sm" onclick={handleConfirmUpdate}>
            Atualizar Agora
          </Button>
        {/if}
      </div>
    </div>
  </div>
{/if}
