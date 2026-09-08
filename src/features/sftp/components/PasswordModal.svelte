<script lang="ts">
  import Button from '@/shared/components/Button.svelte';

  interface Props {
    isOpen: boolean;
    sshUser: string;
    sshHost: string;
    hasKey: boolean;
    initialPassphraseOnly?: boolean;
    requireVpsPassword?: boolean;
    errorMessage?: string;
    isConnecting: boolean;
    onSubmit: (data: { passphrase?: string; password?: string }) => Promise<boolean | void> | boolean | void;
    onClose: () => void;
  }

  let {
    isOpen,
    sshUser,
    sshHost,
    hasKey,
    initialPassphraseOnly = false,
    requireVpsPassword = false,
    errorMessage = '',
    isConnecting,
    onSubmit,
    onClose,
  }: Props = $props();

  let keyPassphrase = $state('');
  let vpsPassword = $state('');
  let showPassphraseText = $state(false);
  let showPasswordText = $state(false);
  let localError = $state('');
  let isPassphraseValidated = $state(false);

  let passphraseInputRef = $state<HTMLInputElement | null>(null);
  let passwordInputRef = $state<HTMLInputElement | null>(null);

  $effect(() => {
    if (isOpen) {
      if (!isPassphraseValidated) {
        keyPassphrase = '';
      }
      vpsPassword = '';
      showPassphraseText = false;
      showPasswordText = false;
      localError = errorMessage || '';

      if (requireVpsPassword) {
        isPassphraseValidated = true;
      } else if (initialPassphraseOnly) {
        isPassphraseValidated = false;
      }

      setTimeout(() => {
        if (!isPassphraseValidated && hasKey) {
          passphraseInputRef?.focus();
        } else {
          passwordInputRef?.focus();
        }
      }, 60);
    } else {
      isPassphraseValidated = false;
      keyPassphrase = '';
      vpsPassword = '';
      localError = '';
    }
  });

  $effect(() => {
    if (errorMessage) {
      localError = errorMessage;
    }
    if (requireVpsPassword) {
      isPassphraseValidated = true;
      setTimeout(() => passwordInputRef?.focus(), 60);
    }
  });

  async function handlePassphraseOk() {
    if (!keyPassphrase) {
      localError = 'Digite o passphrase da chave';
      return;
    }
    localError = '';
    
    // Tenta conectar enviando a passphrase
    const success = await onSubmit({ passphrase: keyPassphrase });
    // Se não concluiu (precisa da senha da VPS em seguida), o useSftpConnection atualiza requireVpsPassword
  }

  function handleVpsPasswordSubmit(e?: Event) {
    if (e) e.preventDefault();
    if (!vpsPassword) {
      localError = 'Digite a senha da VPS';
      return;
    }
    localError = '';
    onSubmit({
      passphrase: keyPassphrase || undefined,
      password: vpsPassword,
    });
  }
</script>

{#if isOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed inset-0 z-60 flex items-center justify-center bg-black/70 backdrop-blur-xs p-4 select-none"
    onclick={(e) => {
      if (e.target === e.currentTarget) onClose();
    }}
  >
    <div
      class="bg-[var(--bg-panel)] border border-[var(--border-panel)] rounded-xl shadow-2xl w-full max-w-sm p-5 text-[var(--text-base)] flex flex-col gap-4 animate-in fade-in zoom-in-95 duration-150"
    >
      <!-- Cabeçalho do Mini Modal -->
      <div class="flex items-center justify-between pb-2 border-b border-[var(--border-subtle)]">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-lg bg-indigo-500/10 border border-indigo-500/20 flex items-center justify-center text-indigo-500 dark:text-indigo-400">
            <svg class="w-4 h-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <rect width="18" height="11" x="3" y="11" rx="2" ry="2"></rect>
              <path d="M7 11V7a5 5 0 0 1 10 0v4"></path>
            </svg>
          </div>
          <div>
            <h3 class="text-sm font-semibold text-[var(--text-base)]">Autenticação SFTP</h3>
            <p class="text-[11px] text-[var(--text-muted)] font-mono">{sshUser}@{sshHost}</p>
          </div>
        </div>
        <button
          type="button"
          onclick={onClose}
          class="text-[var(--text-muted)] hover:text-[var(--text-base)] p-1 rounded hover:bg-black/5 dark:hover:bg-white/5 transition-colors cursor-pointer"
          title="Cancelar"
        >
          <svg class="w-4 h-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>

      <div class="flex flex-col gap-3.5">
        <!-- 1. CAMPO DE PASSPHRASE DA CHAVE (Se o host usar chave ou se passphrase for exigida) -->
        {#if hasKey || initialPassphraseOnly}
          <div class="flex flex-col gap-1.5 p-2.5 rounded-lg border transition-all {isPassphraseValidated ? 'bg-emerald-500/5 border-emerald-500/20' : 'bg-black/10 dark:bg-white/[0.03] border-[var(--border-subtle)]'}">
            <div class="flex items-center justify-between">
              <label for="sftp-key-passphrase" class="text-xs font-medium text-[var(--text-base)] flex items-center gap-1.5">
                <svg class="w-3.5 h-3.5 text-indigo-400" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M21 2l-2 2m-1.5 1.5L14 9a5 5 0 1 1-3.5-3.5l3.5-3.5 2 2z"></path>
                </svg>
                Passphrase da Chave SSH
              </label>
              {#if isPassphraseValidated}
                <span class="text-[10px] text-emerald-400 font-semibold flex items-center gap-1">
                  ✓ Validada
                </span>
              {/if}
            </div>

            <div class="relative flex items-center gap-1.5">
              <div class="relative flex-1 flex items-center">
                <input
                  id="sftp-key-passphrase"
                  bind:this={passphraseInputRef}
                  type={showPassphraseText ? 'text' : 'password'}
                  bind:value={keyPassphrase}
                  disabled={isConnecting || isPassphraseValidated}
                  onkeydown={(e) => {
                    if (e.key === 'Enter') {
                      e.preventDefault();
                      handlePassphraseOk();
                    }
                  }}
                  placeholder="Digite a passphrase da chave..."
                  class="w-full bg-[var(--bg-item-input)] border border-[var(--border-subtle)] rounded-lg px-3 py-2 pr-9 text-xs text-[var(--text-base)] placeholder-[var(--text-faint)] focus:outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500/50 disabled:opacity-60 transition-all"
                  autocomplete="off"
                />
                <button
                  type="button"
                  tabindex="-1"
                  onclick={() => (showPassphraseText = !showPassphraseText)}
                  class="absolute right-2 text-[var(--text-muted)] hover:text-[var(--text-base)] transition-colors cursor-pointer p-1"
                  title={showPassphraseText ? 'Ocultar' : 'Ver'}
                >
                  {#if showPassphraseText}
                    <svg class="w-3.5 h-3.5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                      <path d="M9.88 9.88a3 3 0 1 0 4.24 4.24"></path>
                      <path d="M10.73 5.08A10.43 10.43 0 0 1 12 5c7 0 10 7 10 7a13.16 13.16 0 0 1-1.67 2.68"></path>
                      <path d="M6.61 6.61A13.526 13.526 0 0 0 2 12s3 7 10 7a9.74 9.74 0 0 0 5.39-1.61"></path>
                      <line x1="2" y1="2" x2="22" y2="22"></line>
                    </svg>
                  {:else}
                    <svg class="w-3.5 h-3.5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                      <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"></path>
                      <circle cx="12" cy="12" r="3"></circle>
                    </svg>
                  {/if}
                </button>
              </div>

              {#if !isPassphraseValidated}
                <Button
                  variant="primary"
                  size="sm"
                  loading={isConnecting}
                  onclick={handlePassphraseOk}
                >
                  OK
                </Button>
              {/if}
            </div>
          </div>
        {/if}

        <!-- 2. CAMPO DA SENHA DA VPS (Aparece se não tiver chave OU se a VPS exigir senha após validação) -->
        {#if !hasKey || isPassphraseValidated || requireVpsPassword}
          <form onsubmit={handleVpsPasswordSubmit} class="flex flex-col gap-2.5 animate-in fade-in slide-in-from-top-2 duration-150">
            <div>
              <label for="sftp-auth-password" class="block text-xs font-medium text-[var(--text-base)] mb-1.5 flex items-center gap-1.5">
                <svg class="w-3.5 h-3.5 text-sky-400" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <rect width="20" height="8" x="2" y="2" rx="2" ry="2"></rect>
                  <rect width="20" height="8" x="2" y="14" rx="2" ry="2"></rect>
                  <line x1="6" y1="6" x2="6.01" y2="6"></line>
                  <line x1="6" y1="18" x2="6.01" y2="18"></line>
                </svg>
                Senha SSH da VPS
              </label>
              <div class="relative flex items-center">
                <input
                  id="sftp-auth-password"
                  bind:this={passwordInputRef}
                  type={showPasswordText ? 'text' : 'password'}
                  bind:value={vpsPassword}
                  disabled={isConnecting}
                  placeholder="Digite a senha do usuário na VPS..."
                  class="w-full bg-[var(--bg-item-input)] border border-[var(--border-subtle)] rounded-lg px-3 py-2 pr-10 text-xs text-[var(--text-base)] placeholder-[var(--text-faint)] focus:outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500/50 transition-all"
                  autocomplete="current-password"
                />
                <button
                  type="button"
                  tabindex="-1"
                  onclick={() => (showPasswordText = !showPasswordText)}
                  class="absolute right-2.5 text-[var(--text-muted)] hover:text-[var(--text-base)] transition-colors cursor-pointer p-1"
                  title={showPasswordText ? 'Ocultar senha' : 'Ver senha'}
                >
                  {#if showPasswordText}
                    <svg class="w-4 h-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                      <path d="M9.88 9.88a3 3 0 1 0 4.24 4.24"></path>
                      <path d="M10.73 5.08A10.43 10.43 0 0 1 12 5c7 0 10 7 10 7a13.16 13.16 0 0 1-1.67 2.68"></path>
                      <path d="M6.61 6.61A13.526 13.526 0 0 0 2 12s3 7 10 7a9.74 9.74 0 0 0 5.39-1.61"></path>
                      <line x1="2" y1="2" x2="22" y2="22"></line>
                    </svg>
                  {:else}
                    <svg class="w-4 h-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                      <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"></path>
                      <circle cx="12" cy="12" r="3"></circle>
                    </svg>
                  {/if}
                </button>
              </div>
            </div>

            <div class="flex items-center justify-end gap-2 pt-2 border-t border-[var(--border-subtle)]">
              <Button
                variant="secondary"
                size="sm"
                onclick={onClose}
              >
                Cancelar
              </Button>
              <Button
                type="submit"
                variant="primary"
                size="sm"
                loading={isConnecting}
              >
                Autenticar VPS
              </Button>
            </div>
          </form>
        {/if}

        {#if localError}
          <p class="text-[11px] text-amber-500 dark:text-amber-400 font-medium animate-in fade-in">
            {localError}
          </p>
        {/if}

        <!-- Botão Cancelar quando apenas o campo de passphrase estiver visível -->
        {#if (hasKey || initialPassphraseOnly) && !isPassphraseValidated && !requireVpsPassword}
          <div class="flex items-center justify-end pt-1">
            <Button
              variant="secondary"
              size="sm"
              onclick={onClose}
            >
              Cancelar
            </Button>
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}
