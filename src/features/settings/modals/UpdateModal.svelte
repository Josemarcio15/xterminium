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

  // Controle de senha sudo (Linux)
  let requireSudo = $state(false);
  let sudoPassword = $state('');
  let showPassword = $state(false);
  let passwordInputRef = $state<HTMLInputElement | null>(null);

  $effect(() => {
    if (show) {
      isUpdating = false;
      isDone = false;
      errorMessage = null;
      requireSudo = false;
      sudoPassword = '';
    }
  });

  $effect(() => {
    if (requireSudo && passwordInputRef) {
      setTimeout(() => passwordInputRef?.focus(), 60);
    }
  });

  function startUpdateProcess() {
    errorMessage = null;
    // No Linux (.deb), solicita a senha de administrador antes de disparar
    const isWindows = typeof navigator !== 'undefined' && navigator.userAgent.toLowerCase().includes('windows');
    if (!isWindows) {
      requireSudo = true;
    } else {
      executeInstallation();
    }
  }

  async function executeInstallation() {
    isUpdating = true;
    errorMessage = null;
    try {
      await UpdateService.runUpdate(sudoPassword || undefined);
      isDone = true;
      requireSudo = false;
    } catch (err: any) {
      errorMessage = typeof err === 'string' ? err : (err?.message || 'Falha ao atualizar.');
      // Se falhou por senha incorreta, mantém no prompt para redigitar
      if (requireSudo) {
        sudoPassword = '';
        setTimeout(() => passwordInputRef?.focus(), 60);
      }
    } finally {
      isUpdating = false;
    }
  }

  function handleCancelSudo() {
    requireSudo = false;
    sudoPassword = '';
    errorMessage = null;
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
      class="w-[440px] max-w-full bg-[var(--bg-panel)] border border-[var(--border-panel)] rounded-2xl shadow-2xl p-6 flex flex-col gap-4 relative overflow-hidden"
    >
      <!-- Glow ambient background -->
      <div class="absolute -top-16 -right-16 w-36 h-36 bg-sky-500/15 rounded-full blur-3xl pointer-events-none"></div>

      <!-- Header -->
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-xl {requireSudo ? 'bg-amber-500/10 border-amber-500/20 text-amber-400' : 'bg-sky-500/10 border-sky-500/20 text-sky-400'} border flex items-center justify-center shrink-0">
          {#if isDone}
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
            </svg>
          {:else if requireSudo}
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path>
              <rect x="9" y="10" width="6" height="5" rx="1"></rect>
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
            {:else if requireSudo}
              Autenticação de Administrador
            {:else}
              Nova Atualização Disponível
            {/if}
          </h2>
          <p class="text-xs text-[var(--text-muted)]">
            {#if isDone}
              Pronto para uso
            {:else if requireSudo}
              Permissão necessária para instalar o pacote .deb
            {:else}
              v{info.currentVersion} &rarr; <span class="text-sky-400 font-medium">v{info.latestVersion}</span>
            {/if}
          </p>
        </div>
      </div>

      <!-- Body / Status View -->
      <div class="py-1 flex flex-col gap-2.5">
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
              Instalação finalizada com sucesso! Reinicie o aplicativo para aplicar a nova versão.
            </p>
          </div>
        {:else if requireSudo}
          {#if errorMessage}
            <div class="p-3 bg-red-500/10 border border-red-500/20 rounded-xl text-xs text-red-300 leading-relaxed">
              {errorMessage}
            </div>
          {/if}

          <p class="text-xs text-[var(--text-muted)] leading-relaxed">
            Digite sua senha de usuário (<code class="text-amber-400 font-mono">sudo</code>) para que o sistema possa atualizar o pacote <strong class="text-[var(--text-base)]">xterminium</strong>:
          </p>

          <form onsubmit={(e) => { e.preventDefault(); executeInstallation(); }} class="flex flex-col gap-3">
            <div class="relative flex items-center">
              <input
                bind:this={passwordInputRef}
                type={showPassword ? 'text' : 'password'}
                bind:value={sudoPassword}
                placeholder="Digite a senha sudo..."
                class="w-full bg-[var(--bg-item-input)] border border-[var(--border-subtle)] rounded-lg pl-3 pr-10 py-2 text-xs text-[var(--text-base)] placeholder-[var(--text-faint)] focus:outline-none focus:border-amber-500 transition-colors"
              />
              <button
                type="button"
                onclick={() => (showPassword = !showPassword)}
                class="absolute right-2 text-[var(--text-muted)] hover:text-[var(--text-base)] p-1 rounded transition-colors cursor-pointer"
                title={showPassword ? 'Ocultar senha' : 'Exibir senha'}
              >
                {#if showPassword}
                  <svg class="w-4 h-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"></path>
                    <line x1="1" y1="1" x2="23" y2="23"></line>
                  </svg>
                {:else}
                  <svg class="w-4 h-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path>
                    <circle cx="12" cy="12" r="3"></circle>
                  </svg>
                {/if}
              </button>
            </div>

            <div class="flex justify-end items-center gap-2 pt-1">
              <Button variant="secondary" size="sm" onclick={handleCancelSudo}>
                Voltar
              </Button>
              <Button variant="primary" size="sm" type="submit" disabled={!sudoPassword}>
                Confirmar e Instalar
              </Button>
            </div>
          </form>
        {:else}
          {#if errorMessage}
            <div class="p-3 bg-red-500/10 border border-red-500/20 rounded-xl text-xs text-red-300">
              {errorMessage}
            </div>
          {/if}

          <p class="text-xs text-[var(--text-muted)] leading-relaxed">
            Uma nova versão do xterminium está disponível. Deseja realizar a atualização automática?
          </p>

          {#if info.releaseNotes}
            <div class="p-3 bg-black/20 rounded-xl border border-[var(--border-subtle)] max-h-36 overflow-y-auto text-[11px] text-[var(--text-muted)] font-mono whitespace-pre-wrap select-text">
              {info.releaseNotes}
            </div>
          {/if}
        {/if}
      </div>

      <!-- Actions (apenas quando não estiver no step de sudo, pois o form de sudo tem seus próprios botões) -->
      {#if !requireSudo}
        <div class="flex justify-end items-center gap-2 pt-2 border-t border-[var(--border-subtle)]">
          {#if isDone}
            <Button variant="primary" size="sm" onclick={onClose}>
              Fechar
            </Button>
          {:else if isUpdating}
            <span class="text-xs text-[var(--text-muted)] italic">Instalando...</span>
          {:else}
            <Button variant="secondary" size="sm" onclick={onClose}>
              Depois
            </Button>
            <Button variant="primary" size="sm" onclick={startUpdateProcess}>
              Atualizar Agora
            </Button>
          {/if}
        </div>
      {/if}
    </div>
  </div>
{/if}
