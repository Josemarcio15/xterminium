<script lang="ts">
  import { defaultShortcuts } from './types';
  import { ConfigService } from './config';
  import { parseKeyboardEvent } from './shortcuts';

  interface Props {
    show: boolean;
    onClose: () => void;
  }

  let { show = false, onClose }: Props = $props();

  let shortcuts = $state<Record<string, string>>({ ...defaultShortcuts });
  let recordingActionId = $state<string | null>(null);

  $effect(() => {
    if (show) {
      ConfigService.loadShortcuts().then((loaded) => {
        shortcuts = loaded;
      });
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
      shortcuts[actionId] = shortcutStr;
      ConfigService.saveShortcuts(shortcuts);
      recordingActionId = null;
    }
  }

  async function resetDefaults() {
    shortcuts = { ...defaultShortcuts };
    await ConfigService.saveShortcuts(shortcuts);
  }
</script>

{#if show}
  <button 
    type="button" 
    class="fixed inset-0 z-[150] bg-transparent border-none cursor-default" 
    onclick={() => { onClose(); recordingActionId = null; }}
    aria-label="Fechar modal de atalhos"
  ></button>
  <div 
    class="absolute top-9 right-0 w-[340px] bg-[#171926] border border-white/10 rounded-lg shadow-2xl p-3 z-[160]" 
    role="dialog" 
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.key === 'Escape' && onClose()}
  >
    <div class="flex justify-between items-center mb-2.5">
      <div class="flex items-center gap-1.5 text-xs font-semibold text-slate-100">
        <svg class="text-sky-400" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="4" width="20" height="16" rx="2" ry="2"></rect><line x1="7" y1="16" x2="17" y2="16"></line></svg>
        <span>Atalhos de Teclado</span>
      </div>
      <button 
        class="bg-transparent border border-white/10 text-slate-400 px-2 py-0.5 rounded text-[11px] hover:bg-white/10 hover:text-white cursor-pointer transition-all" 
        onclick={resetDefaults}
        title="Restaurar padrão"
      >
        Padrão
      </button>
    </div>

    <p class="text-[11px] text-slate-400 mb-2.5">Salvo em <code class="bg-white/10 text-sky-400 px-1 py-0.5 rounded font-mono">~/.config/xterminium/shortcuts.json</code></p>

    <div class="flex flex-col gap-1.5">
      <!-- Copiar -->
      <div class="flex justify-between items-center px-2.5 py-1.5 rounded-md bg-[#1d2030]">
        <span class="text-xs font-medium text-slate-200">Copiar</span>
        <button 
          class="bg-[#10121c] border border-white/15 text-sky-400 px-2 py-1 rounded text-[11px] font-mono cursor-pointer hover:border-sky-400 hover:bg-sky-400/10 transition-all {recordingActionId === 'copy' ? '!bg-sky-400 !text-slate-950 font-bold animate-pulse' : ''}" 
          onclick={() => (recordingActionId = 'copy')}
          onkeydown={(e) => recordingActionId === 'copy' && handleRecordKeyDown(e, 'copy')}
        >
          {recordingActionId === 'copy' ? 'Pressione teclas...' : shortcuts.copy}
        </button>
      </div>

      <!-- Colar -->
      <div class="flex justify-between items-center px-2.5 py-1.5 rounded-md bg-[#1d2030]">
        <span class="text-xs font-medium text-slate-200">Colar</span>
        <button 
          class="bg-[#10121c] border border-white/15 text-sky-400 px-2 py-1 rounded text-[11px] font-mono cursor-pointer hover:border-sky-400 hover:bg-sky-400/10 transition-all {recordingActionId === 'paste' ? '!bg-sky-400 !text-slate-950 font-bold animate-pulse' : ''}" 
          onclick={() => (recordingActionId = 'paste')}
          onkeydown={(e) => recordingActionId === 'paste' && handleRecordKeyDown(e, 'paste')}
        >
          {recordingActionId === 'paste' ? 'Pressione teclas...' : shortcuts.paste}
        </button>
      </div>

      <!-- Selecionar Tudo -->
      <div class="flex justify-between items-center px-2.5 py-1.5 rounded-md bg-[#1d2030]">
        <span class="text-xs font-medium text-slate-200">Selecionar Tudo</span>
        <button 
          class="bg-[#10121c] border border-white/15 text-sky-400 px-2 py-1 rounded text-[11px] font-mono cursor-pointer hover:border-sky-400 hover:bg-sky-400/10 transition-all {recordingActionId === 'selectAll' ? '!bg-sky-400 !text-slate-950 font-bold animate-pulse' : ''}" 
          onclick={() => (recordingActionId = 'selectAll')}
          onkeydown={(e) => recordingActionId === 'selectAll' && handleRecordKeyDown(e, 'selectAll')}
        >
          {recordingActionId === 'selectAll' ? 'Pressione teclas...' : (shortcuts.selectAll || 'Ctrl+Shift+A')}
        </button>
      </div>

      <!-- Autocomplete de VPS -->
      <div class="flex justify-between items-center px-2.5 py-1.5 rounded-md bg-[#1d2030]">
        <span class="text-xs font-medium text-slate-200">Autocomplete VPS</span>
        <button 
          class="bg-[#10121c] border border-white/15 text-sky-400 px-2 py-1 rounded text-[11px] font-mono cursor-pointer hover:border-sky-400 hover:bg-sky-400/10 transition-all {recordingActionId === 'autocomplete' ? '!bg-sky-400 !text-slate-950 font-bold animate-pulse' : ''}" 
          onclick={() => (recordingActionId = 'autocomplete')}
          onkeydown={(e) => recordingActionId === 'autocomplete' && handleRecordKeyDown(e, 'autocomplete')}
        >
          {recordingActionId === 'autocomplete' ? 'Pressione teclas...' : (shortcuts.autocomplete || 'Ctrl+Space')}
        </button>
      </div>

      <!-- Parar terminal -->
      <div class="flex justify-between items-center px-2.5 py-1.5 rounded-md bg-[#1d2030]">
        <span class="text-xs font-medium text-slate-200">Parar Terminal (SIGINT)</span>
        <button 
          class="bg-[#10121c] border border-white/15 text-sky-400 px-2 py-1 rounded text-[11px] font-mono cursor-pointer hover:border-sky-400 hover:bg-sky-400/10 transition-all {recordingActionId === 'stop' ? '!bg-sky-400 !text-slate-950 font-bold animate-pulse' : ''}" 
          onclick={() => (recordingActionId = 'stop')}
          onkeydown={(e) => recordingActionId === 'stop' && handleRecordKeyDown(e, 'stop')}
        >
          {recordingActionId === 'stop' ? 'Pressione teclas...' : shortcuts.stop}
        </button>
      </div>

      <!-- Nova Aba -->
      <div class="flex justify-between items-center px-2.5 py-1.5 rounded-md bg-[#1d2030]">
        <span class="text-xs font-medium text-slate-200">Nova Aba</span>
        <button 
          class="bg-[#10121c] border border-white/15 text-sky-400 px-2 py-1 rounded text-[11px] font-mono cursor-pointer hover:border-sky-400 hover:bg-sky-400/10 transition-all {recordingActionId === 'newTab' ? '!bg-sky-400 !text-slate-950 font-bold animate-pulse' : ''}" 
          onclick={() => (recordingActionId = 'newTab')}
          onkeydown={(e) => recordingActionId === 'newTab' && handleRecordKeyDown(e, 'newTab')}
        >
          {recordingActionId === 'newTab' ? 'Pressione teclas...' : shortcuts.newTab}
        </button>
      </div>

      <!-- Nova Janela -->
      <div class="flex justify-between items-center px-2.5 py-1.5 rounded-md bg-[#1d2030]">
        <span class="text-xs font-medium text-slate-200">Nova Janela</span>
        <button 
          class="bg-[#10121c] border border-white/15 text-sky-400 px-2 py-1 rounded text-[11px] font-mono cursor-pointer hover:border-sky-400 hover:bg-sky-400/10 transition-all {recordingActionId === 'newWindow' ? '!bg-sky-400 !text-slate-950 font-bold animate-pulse' : ''}" 
          onclick={() => (recordingActionId = 'newWindow')}
          onkeydown={(e) => recordingActionId === 'newWindow' && handleRecordKeyDown(e, 'newWindow')}
        >
          {recordingActionId === 'newWindow' ? 'Pressione teclas...' : shortcuts.newWindow}
        </button>
      </div>
    </div>
  </div>
{/if}
