<script lang="ts">
  interface Props {
    isOpen: boolean;
    title: string;
    message: string;
    confirmText?: string;
    cancelText?: string;
    variant?: 'danger' | 'warning' | 'info' | 'primary';
    inputMode?: boolean;
    inputLabel?: string;
    inputValue?: string;
    inputPlaceholder?: string;
    onConfirm: (inputValue?: string) => void;
    onClose: () => void;
  }

  let {
    isOpen,
    title,
    message,
    confirmText = 'Confirmar',
    cancelText = 'Cancelar',
    variant = 'primary',
    inputMode = false,
    inputLabel = '',
    inputValue = '',
    inputPlaceholder = '',
    onConfirm,
    onClose,
  }: Props = $props();

  let textValue = $state('');
  let inputElement = $state<HTMLInputElement | null>(null);

  $effect(() => {
    if (isOpen) {
      textValue = inputValue || '';
      if (inputMode) {
        setTimeout(() => {
          inputElement?.focus();
          inputElement?.select();
        }, 50);
      }
    }
  });

  function handleConfirm() {
    if (inputMode && !textValue.trim()) return;
    onConfirm(inputMode ? textValue.trim() : undefined);
    onClose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose();
    } else if (e.key === 'Enter') {
      e.preventDefault();
      handleConfirm();
    }
  }
</script>

{#if isOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed inset-0 z-70 flex items-center justify-center bg-black/70 backdrop-blur-xs p-4 select-none"
    onclick={(e) => {
      if (e.target === e.currentTarget) onClose();
    }}
    onkeydown={handleKeydown}
  >
    <div
      class="bg-[#181a26] border border-white/10 rounded-xl shadow-2xl w-full max-w-sm p-5 text-gray-200 flex flex-col gap-4 animate-in fade-in zoom-in-95 duration-150"
      tabindex="-1"
    >
      <!-- Cabeçalho -->
      <div class="flex items-center gap-3">
        {#if variant === 'danger'}
          <div class="w-9 h-9 rounded-lg bg-red-500/10 border border-red-500/20 flex items-center justify-center text-red-400 shrink-0">
            <svg class="w-5 h-5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"></path>
              <line x1="12" y1="9" x2="12" y2="13"></line>
              <line x1="12" y1="17" x2="12.01" y2="17"></line>
            </svg>
          </div>
        {:else if variant === 'warning'}
          <div class="w-9 h-9 rounded-lg bg-amber-500/10 border border-amber-500/20 flex items-center justify-center text-amber-400 shrink-0">
            <svg class="w-5 h-5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="12" y1="8" x2="12" y2="12"></line>
              <line x1="12" y1="16" x2="12.01" y2="16"></line>
            </svg>
          </div>
        {:else}
          <div class="w-9 h-9 rounded-lg bg-blue-500/10 border border-blue-500/20 flex items-center justify-center text-blue-400 shrink-0">
            <svg class="w-5 h-5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="12" y1="16" x2="12" y2="12"></line>
              <line x1="12" y1="8" x2="12.01" y2="8"></line>
            </svg>
          </div>
        {/if}

        <div class="min-w-0 flex-1">
          <h3 class="text-sm font-semibold text-white tracking-wide">{title}</h3>
          <p class="text-xs text-gray-400 mt-0.5 leading-relaxed">{message}</p>
        </div>
      </div>

      <!-- Input opcional se estiver em modo prompt (criar ou renomear) -->
      {#if inputMode}
        <div class="flex flex-col gap-1.5">
          {#if inputLabel}
            <label class="text-[11px] font-medium text-gray-400" for="confirm-modal-input">{inputLabel}</label>
          {/if}
          <input
            id="confirm-modal-input"
            type="text"
            bind:this={inputElement}
            bind:value={textValue}
            placeholder={inputPlaceholder}
            class="w-full bg-[#11121a] border border-white/10 rounded-lg px-3 py-2 text-xs text-white placeholder-gray-500 focus:outline-none focus:border-blue-500 transition-colors"
          />
        </div>
      {/if}

      <!-- Botões de Ação -->
      <div class="flex items-center justify-end gap-2 pt-2 border-t border-white/10">
        <button
          type="button"
          onclick={onClose}
          class="px-3 py-1.5 rounded-lg text-xs font-medium text-gray-400 hover:text-white bg-white/5 hover:bg-white/10 transition-colors cursor-pointer"
        >
          {cancelText}
        </button>

        <button
          type="button"
          onclick={handleConfirm}
          disabled={inputMode && !textValue.trim()}
          class="px-3.5 py-1.5 rounded-lg text-xs font-semibold shadow-md transition-all cursor-pointer flex items-center gap-1.5 {variant === 'danger'
            ? 'bg-red-600 hover:bg-red-500 text-white'
            : variant === 'warning'
            ? 'bg-amber-600 hover:bg-amber-500 text-white'
            : 'bg-blue-600 hover:bg-blue-500 text-white'} {inputMode && !textValue.trim() ? 'opacity-50 cursor-not-allowed' : ''}"
        >
          {confirmText}
        </button>
      </div>
    </div>
  </div>
{/if}
