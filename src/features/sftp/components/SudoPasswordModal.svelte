<script lang="ts">
  interface Props {
    isOpen: boolean;
    title?: string;
    description?: string;
    isSubmitting?: boolean;
    errorMessage?: string;
    onSubmit: (password: string) => void;
    onClose: () => void;
  }

  let {
    isOpen,
    title = 'Acesso Privilegiado (sudo)',
    description = 'Permissão negada. Digite a senha do usuário para executar esta ação como root:',
    isSubmitting = false,
    errorMessage = '',
    onSubmit,
    onClose,
  }: Props = $props();

  let passwordValue = $state('');
  let showPassword = $state(false);
  let passwordInputRef = $state<HTMLInputElement | null>(null);

  $effect(() => {
    if (isOpen) {
      passwordValue = '';
      showPassword = false;
      setTimeout(() => {
        passwordInputRef?.focus();
      }, 50);
    }
  });

  function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    if (!passwordValue || isSubmitting) return;
    onSubmit(passwordValue);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose();
    }
  }
</script>

{#if isOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed inset-0 z-80 flex items-center justify-center bg-black/75 backdrop-blur-xs p-4 select-none animate-in fade-in duration-150"
    onclick={(e) => {
      if (e.target === e.currentTarget && !isSubmitting) onClose();
    }}
    onkeydown={handleKeydown}
  >
    <div
      class="bg-[#161824] border border-amber-500/30 rounded-xl shadow-2xl w-full max-w-sm p-5 text-gray-200 flex flex-col gap-4"
      tabindex="-1"
    >
      <!-- Cabeçalho -->
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="w-9 h-9 rounded-lg bg-amber-500/15 border border-amber-500/30 flex items-center justify-center text-amber-400 shrink-0 shadow-sm">
            <svg class="w-5 h-5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path>
              <rect x="9" y="10" width="6" height="5" rx="1"></rect>
            </svg>
          </div>
          <div>
            <h3 class="text-sm font-semibold text-white tracking-wide">{title}</h3>
            <p class="text-[11px] text-amber-300/80 font-medium">Elevação de Privilégios</p>
          </div>
        </div>
        <button
          type="button"
          onclick={onClose}
          disabled={isSubmitting}
          class="text-gray-400 hover:text-white p-1 rounded hover:bg-white/5 transition-colors cursor-pointer disabled:opacity-50"
          title="Cancelar"
        >
          <svg class="w-4 h-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>

      <p class="text-xs text-gray-300 leading-relaxed bg-amber-500/10 border border-amber-500/20 rounded-lg p-2.5">
        {description}
      </p>

      {#if errorMessage}
        <div class="text-xs text-red-400 bg-red-500/10 border border-red-500/20 rounded-lg p-2.5 flex items-start gap-2 max-h-32 overflow-y-auto">
          <svg class="w-4 h-4 shrink-0 mt-0.5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="12" y1="8" x2="12" y2="12"></line>
            <line x1="12" y1="16" x2="12.01" y2="16"></line>
          </svg>
          <span class="flex-1 break-words font-mono text-[11px] leading-relaxed select-text">{errorMessage}</span>
        </div>
      {/if}

      <form onsubmit={handleSubmit} class="flex flex-col gap-3">
        <div>
          <label for="sudo-modal-password" class="block text-xs font-medium text-gray-300 mb-1.5">
            Senha do usuário (sudo)
          </label>
          <div class="relative flex items-center">
            <input
              id="sudo-modal-password"
              bind:this={passwordInputRef}
              type={showPassword ? 'text' : 'password'}
              bind:value={passwordValue}
              placeholder="Digite a senha sudo..."
              disabled={isSubmitting}
              class="w-full bg-[#10111a] border border-white/10 rounded-lg pl-3 pr-10 py-2 text-xs text-white placeholder-gray-500 focus:outline-none focus:border-amber-500 transition-colors"
            />
            <button
              type="button"
              onclick={() => (showPassword = !showPassword)}
              class="absolute right-2 text-gray-400 hover:text-white p-1 rounded transition-colors cursor-pointer"
              title={showPassword ? 'Ocultar senha' : 'Exibir senha'}
            >
              {#if showPassword}
                <svg class="w-4 h-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"></path>
                  <line x1="1" y1="1" x2="23" y2="23"></line>
                </svg>
              {:else}
                <svg class="w-4 h-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path>
                  <circle cx="12" cy="12" r="3"></circle>
                </svg>
              {/if}
            </button>
          </div>
        </div>

        <div class="flex items-center justify-end gap-2 pt-2 border-t border-white/10">
          <button
            type="button"
            onclick={onClose}
            disabled={isSubmitting}
            class="px-3 py-1.5 rounded-lg text-xs font-medium text-gray-400 hover:text-white bg-white/5 hover:bg-white/10 transition-colors cursor-pointer disabled:opacity-50"
          >
            Cancelar
          </button>
          <button
            type="submit"
            disabled={!passwordValue || isSubmitting}
            class="px-3.5 py-1.5 rounded-lg text-xs font-semibold bg-amber-600 hover:bg-amber-500 text-white shadow-md transition-all cursor-pointer flex items-center gap-1.5 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {#if isSubmitting}
              <svg class="animate-spin -ml-1 mr-1 h-3.5 w-3.5 text-white" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              Executando...
            {:else}
              <svg class="w-3.5 h-3.5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="20 6 9 17 4 12"></polyline>
              </svg>
              Executar com Sudo
            {/if}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
