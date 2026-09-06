<script lang="ts">
  import { configStore } from '../../../core/stores/config.svelte';
  import { parseKeyboardEvent } from '../../terminal/utils/shortcuts';
  import Modal from '../../../shared/components/Modal.svelte';
  import Button from '@/shared/components/Button.svelte';

  interface Props {
    show: boolean;
    onClose: () => void;
  }

  let { show = false, onClose }: Props = $props();
  let recordingActionId = $state<string | null>(null);

  $effect(() => {
    if (show) {
      configStore.init();
    }
  });

  function handleRecordKeyDown(e: KeyboardEvent, actionId: string) {
    e.preventDefault();
    e.stopPropagation();

    if (e.key === 'Escape') {
      recordingActionId = null;
      return;
    }

    const shortcutStr = parseKeyboardEvent(e);
    const hasMainKey = !['Control', 'Alt', 'Shift', 'Meta'].includes(e.key);
    if (hasMainKey && shortcutStr) {
      configStore.updateShortcut(actionId, shortcutStr);
      recordingActionId = null;
    }
  }

  function handleClose() {
    recordingActionId = null;
    onClose();
  }
</script>

<Modal {show} title="Atalhos de Teclado" widthClass="w-[340px]" onClose={handleClose}>
  {#snippet icon()}
    <svg class="text-sky-400" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="2" y="4" width="20" height="16" rx="2" ry="2"></rect>
      <line x1="7" y1="16" x2="17" y2="16"></line>
    </svg>
  {/snippet}

  {#snippet actions()}
    <Button
      variant="secondary"
      size="xs"
      onclick={() => configStore.resetShortcuts()}
      title="Restaurar padrão"
    >
      Padrão
    </Button>
  {/snippet}

  <p class="text-[11px] text-[var(--text-muted)] mb-2.5">Salvo em <code class="bg-[var(--bg-item)] border border-[var(--border-subtle)] text-sky-600 dark:text-sky-400 px-1 py-0.5 rounded font-mono">~/.config/xterminium/shortcuts.json</code></p>

  <div class="flex flex-col gap-1.5">
    {#each [
      { id: 'copy', label: 'Copiar', defaultKey: configStore.shortcuts.copy },
      { id: 'paste', label: 'Colar', defaultKey: configStore.shortcuts.paste },
      { id: 'selectAll', label: 'Selecionar Tudo', defaultKey: configStore.shortcuts.selectAll || 'Ctrl+Shift+A' },
      { id: 'autocomplete', label: 'Autocomplete VPS', defaultKey: configStore.shortcuts.autocomplete || 'Ctrl+Space' },
      { id: 'stop', label: 'Parar Terminal (SIGINT)', defaultKey: configStore.shortcuts.stop },
      { id: 'newTab', label: 'Nova Aba', defaultKey: configStore.shortcuts.newTab },
      { id: 'newWindow', label: 'Nova Janela', defaultKey: configStore.shortcuts.newWindow },
    ] as item}
      <div class="flex justify-between items-center px-2.5 py-1.5 rounded-lg bg-[var(--bg-item)] border border-[var(--border-subtle)] {recordingActionId === item.id ? 'ring-1 ring-sky-400' : ''}">
        <span class="text-xs font-medium text-[var(--text-base)]">{item.label}</span>
        <div class="flex items-center gap-1.5">
          <Button 
            variant="glass"
            size="xs"
            class="font-mono text-[11px] {recordingActionId === item.id ? '!bg-sky-400 !text-slate-950 font-bold animate-pulse' : ''}" 
            onclick={() => (recordingActionId = item.id)}
            onkeydown={(e) => recordingActionId === item.id && handleRecordKeyDown(e, item.id)}
          >
            {recordingActionId === item.id ? 'Pressione teclas...' : item.defaultKey}
          </Button>
          <!-- Botão Lápis SVG para editar atalho -->
          <button
            type="button"
            class="text-[var(--text-muted)] hover:text-sky-300 hover:bg-sky-400/15 p-1 rounded text-xs transition-all cursor-pointer border-none bg-transparent flex items-center justify-center"
            onclick={() => (recordingActionId = item.id)}
            title="Alterar atalho"
          >
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"></path>
              <path d="m15 5 4 4"></path>
            </svg>
          </button>
        </div>
      </div>
    {/each}
  </div>
</Modal>
