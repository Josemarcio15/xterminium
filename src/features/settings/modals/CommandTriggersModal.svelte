<script lang="ts">
  import { type CustomCommand } from '../../../core/types';
  import { configStore } from '../../../core/stores/config.svelte';
  import Modal from '../../../shared/components/Modal.svelte';

  interface Props {
    show: boolean;
    onClose: () => void;
  }

  let { show = false, onClose }: Props = $props();

  let showForm = $state(false);
  let editingId = $state<string | null>(null);

  let formCommand = $state('');
  let formPrefixArgs = $state('');
  let formTemplate = $state('{user}@{ip}');
  let formSuffixArgs = $state('');

  $effect(() => {
    if (show) {
      configStore.init();
    }
  });

  function openNewForm() {
    editingId = null;
    formCommand = '';
    formPrefixArgs = '';
    formTemplate = '{user}@{ip}';
    formSuffixArgs = '';
    showForm = !showForm;
  }

  function startEdit(cmd: CustomCommand, e: MouseEvent) {
    e.stopPropagation();
    editingId = cmd.id;
    formCommand = cmd.command;
    formPrefixArgs = cmd.prefixArgs || '';
    formTemplate = cmd.template;
    formSuffixArgs = cmd.suffixArgs || '';
    showForm = true;
  }

  async function saveCommand() {
    if (!formCommand.trim()) return;

    if (editingId) {
      const updated: CustomCommand = {
        id: editingId,
        command: formCommand.trim().toLowerCase(),
        prefixArgs: formPrefixArgs.trim(),
        template: formTemplate.trim() || '{user}@{ip}',
        suffixArgs: formSuffixArgs.trim(),
      };
      await configStore.updateCommand(updated);
    } else {
      const cmd: CustomCommand = {
        id: crypto.randomUUID(),
        command: formCommand.trim().toLowerCase(),
        prefixArgs: formPrefixArgs.trim(),
        template: formTemplate.trim() || '{user}@{ip}',
        suffixArgs: formSuffixArgs.trim(),
      };
      await configStore.addCommand(cmd);
    }

    formCommand = '';
    formPrefixArgs = '';
    formTemplate = '{user}@{ip}';
    formSuffixArgs = '';
    editingId = null;
    showForm = false;
  }

  async function removeCommand(id: string, e: MouseEvent) {
    e.stopPropagation();
    if (editingId === id) {
      showForm = false;
      editingId = null;
    }
    await configStore.removeCommand(id);
  }
</script>

<Modal {show} title="Comandos & Autocomplete VPS" widthClass="w-96" {onClose}>
  {#snippet icon()}
    <svg class="text-sky-400" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <polyline points="4 17 10 11 4 5"></polyline>
      <line x1="12" y1="19" x2="20" y2="19"></line>
    </svg>
  {/snippet}

  {#snippet actions()}
    <div class="flex items-center gap-1">
      <button 
        type="button"
        class="px-1.5 py-0.5 rounded text-[11px] bg-white/5 border border-white/10 text-slate-400 hover:bg-white/10 hover:text-white transition-all cursor-pointer"
        onclick={() => configStore.resetCommands()}
        title="Restaurar comandos padrões"
      >
        Padrões
      </button>
      <button 
        type="button"
        class="w-[22px] h-[22px] rounded flex items-center justify-center text-xs bg-white/5 border border-white/10 text-slate-400 hover:bg-white/10 hover:text-white transition-all cursor-pointer" 
        onclick={openNewForm} 
        title={showForm ? 'Fechar formulário' : 'Adicionar novo comando'}
      >
        {showForm ? '✕' : '+'}
      </button>
    </div>
  {/snippet}

  <!-- Formulário Novo / Editar Comando -->
  {#if showForm}
    <form class="bg-[#12141f] border border-white/10 rounded-md p-2.5 mb-2.5 flex flex-col gap-2" onsubmit={(e) => { e.preventDefault(); saveCommand(); }}>
      <div class="text-[11px] font-semibold text-sky-400 flex items-center justify-between">
        <span>{editingId ? 'Editar Comando' : 'Novo Comando'}</span>
        {#if editingId}
          <button type="button" class="text-slate-500 hover:text-slate-300 text-[10px] bg-transparent border-none cursor-pointer" onclick={() => { showForm = false; editingId = null; }}>Cancelar</button>
        {/if}
      </div>

      <div class="flex gap-2">
        <input 
          class="w-1/3 bg-[#0d0e17] border border-white/10 rounded text-slate-200 px-2 py-1.5 text-xs outline-none focus:border-sky-400 transition-colors font-mono" 
          type="text" 
          placeholder="Comando (ex: rsync)" 
          bind:value={formCommand} 
          required 
        />
        <input 
          class="flex-1 bg-[#0d0e17] border border-white/10 rounded text-slate-200 px-2 py-1.5 text-xs outline-none focus:border-sky-400 transition-colors font-mono" 
          type="text" 
          placeholder="Args (ex: -avz)" 
          bind:value={formPrefixArgs} 
        />
      </div>

      <div class="flex gap-2">
        <input 
          class="flex-1 bg-[#0d0e17] border border-white/10 rounded text-slate-200 px-2 py-1.5 text-xs outline-none focus:border-sky-400 transition-colors font-mono" 
          type="text" 
          placeholder="Template VPS (ex: {`{user}@{ip}`})" 
          bind:value={formTemplate} 
          required 
        />
        <input 
          class="w-1/2 bg-[#0d0e17] border border-white/10 rounded text-slate-200 px-2 py-1.5 text-xs outline-none focus:border-sky-400 transition-colors font-mono" 
          type="text" 
          placeholder="Args (ex: :~/)" 
          bind:value={formSuffixArgs} 
        />
      </div>

      <div class="text-[10px] text-slate-400 bg-white/[0.02] p-1.5 rounded border border-white/5">
        Variáveis disponíveis: <code class="text-sky-300">{"{user}"}</code>, <code class="text-sky-300">{"{ip}"}</code>, <code class="text-sky-300">{"{port}"}</code>, <code class="text-sky-300">{"{key}"}</code>.
      </div>

      <button type="submit" class="bg-sky-400 hover:bg-sky-500 text-slate-950 font-semibold rounded py-1.5 text-xs transition-colors cursor-pointer">
        {editingId ? 'Atualizar Comando' : 'Salvar Comando'}
      </button>
    </form>
  {/if}

  <!-- Lista de Comandos -->
  <div class="max-h-72 overflow-y-auto flex flex-col gap-1.5 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
    {#each configStore.commands as cmd (cmd.id)}
      <div class="flex items-center justify-between p-2 rounded bg-white/[0.03] hover:bg-white/[0.06] border border-white/5 transition-all text-xs {editingId === cmd.id ? 'border-sky-400/40 bg-sky-400/10' : ''}">
        <div class="flex flex-col min-w-0 pr-2">
          <div class="flex items-center gap-1.5 font-mono">
            <span class="text-sky-400 font-bold">{cmd.command}</span>
            {#if cmd.prefixArgs}
              <span class="text-slate-400 text-[11px]">{cmd.prefixArgs}</span>
            {/if}
            <span class="text-amber-300 text-[11px] bg-amber-500/10 px-1 py-0.5 rounded">{cmd.template}</span>
            {#if cmd.suffixArgs}
              <span class="text-emerald-400 font-mono text-[11px] bg-emerald-500/10 px-1 py-0.5 rounded">{cmd.suffixArgs}</span>
            {/if}
          </div>
          <div class="text-[10px] text-slate-500 mt-0.5 font-mono">
            ex: {cmd.command} {cmd.prefixArgs ? cmd.prefixArgs + ' ' : ''}{cmd.template.replace('{user}', 'root').replace('{ip}', '1.2.3.4').replace('{port}', '22')}{cmd.suffixArgs || ''}
          </div>
        </div>
        <div class="flex items-center gap-1 shrink-0">
          <!-- Botão de Editar (Lápis SVG) -->
          <button 
            type="button" 
            class="text-slate-400 hover:text-sky-300 hover:bg-sky-400/15 p-1 rounded text-xs transition-all cursor-pointer border-none bg-transparent flex items-center justify-center" 
            onclick={(e) => startEdit(cmd, e)} 
            title="Editar comando"
          >
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"></path>
              <path d="m15 5 4 4"></path>
            </svg>
          </button>
          <button 
            type="button" 
            class="bg-transparent border-none text-slate-500 hover:text-red-400 p-1 cursor-pointer transition-colors" 
            onclick={(e) => removeCommand(cmd.id, e)} 
            title="Remover comando"
          >
            ✕
          </button>
        </div>
      </div>
    {:else}
      <div class="text-center py-4 text-xs text-slate-500">
        Nenhum comando configurado.
      </div>
    {/each}
  </div>
</Modal>
