<script lang="ts">
  import { type SshHost } from '../../../core/types';
  import { configStore } from '../../../core/stores/config.svelte';
  import Modal from '../../../shared/components/Modal.svelte';

  interface Props {
    show: boolean;
    onClose: () => void;
    onConnect: (host: SshHost) => void;
  }

  let { show = false, onClose, onConnect }: Props = $props();

  let showForm = $state(false);
  let editingId = $state<string | null>(null);

  let formLabel = $state('');
  let formUser = $state('');
  let formIp = $state('');
  let formPort = $state('22');
  let formKey = $state('');

  $effect(() => {
    if (show) {
      configStore.init();
    }
  });

  function openNewForm() {
    editingId = null;
    formLabel = '';
    formUser = '';
    formIp = '';
    formPort = '22';
    formKey = '';
    showForm = !showForm;
  }

  function startEdit(host: SshHost, e: MouseEvent) {
    e.stopPropagation();
    editingId = host.id;
    formLabel = host.label || '';
    formUser = host.user;
    formIp = host.ip;
    formPort = host.port || '22';
    formKey = host.key || '';
    showForm = true;
  }

  async function saveHost() {
    if (!formUser.trim() || !formIp.trim()) return;

    if (editingId) {
      const updated: SshHost = {
        id: editingId,
        label: formLabel.trim() || `${formUser.trim()}@${formIp.trim()}`,
        user: formUser.trim(),
        ip: formIp.trim(),
        port: formPort.trim() || '22',
        key: formKey.trim() || undefined,
      };
      await configStore.updateHost(updated);
    } else {
      const host: SshHost = {
        id: crypto.randomUUID(),
        label: formLabel.trim() || `${formUser.trim()}@${formIp.trim()}`,
        user: formUser.trim(),
        ip: formIp.trim(),
        port: formPort.trim() || '22',
        key: formKey.trim() || undefined,
      };
      await configStore.addHost(host);
    }

    formLabel = '';
    formUser = '';
    formIp = '';
    formPort = '22';
    formKey = '';
    editingId = null;
    showForm = false;
  }

  async function removeHost(id: string, e: MouseEvent) {
    e.stopPropagation();
    if (editingId === id) {
      showForm = false;
      editingId = null;
    }
    await configStore.removeHost(id);
  }
</script>

<Modal {show} title="Conexões SSH" {onClose}>
  {#snippet icon()}
    <svg class="text-sky-400" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="2" y="2" width="20" height="8" rx="2" ry="2"></rect>
      <rect x="2" y="14" width="20" height="8" rx="2" ry="2"></rect>
      <line x1="6" y1="6" x2="6.01" y2="6"></line>
      <line x1="6" y1="18" x2="6.01" y2="18"></line>
    </svg>
  {/snippet}

  {#snippet actions()}
    <button 
      class="w-[22px] h-[22px] rounded flex items-center justify-center text-xs bg-white/5 border border-white/10 text-slate-400 hover:bg-white/10 hover:text-white transition-all cursor-pointer" 
      onclick={openNewForm} 
      title={showForm ? 'Fechar formulário' : 'Novo Host'}
    >
      {showForm ? '✕' : '+'}
    </button>
  {/snippet}

  <!-- Formulário Novo / Editar Host -->
  {#if showForm}
    <form class="bg-[#12141f] border border-white/10 rounded-md p-2.5 mb-2.5 flex flex-col gap-2" onsubmit={(e) => { e.preventDefault(); saveHost(); }}>
      <div class="text-[11px] font-semibold text-sky-400 flex items-center justify-between">
        <span>{editingId ? 'Editar Conexão SSH' : 'Nova Conexão SSH'}</span>
        {#if editingId}
          <button type="button" class="text-slate-500 hover:text-slate-300 text-[10px] bg-transparent border-none cursor-pointer" onclick={() => { showForm = false; editingId = null; }}>Cancelar</button>
        {/if}
      </div>
      <input class="bg-[#0d0e17] border border-white/10 rounded text-slate-200 px-2 py-1.5 text-xs outline-none focus:border-sky-400 transition-colors" type="text" placeholder="Nome/Apelido (ex: Servidor Prod)" bind:value={formLabel} />
      <div class="flex gap-2">
        <input class="flex-1 bg-[#0d0e17] border border-white/10 rounded text-slate-200 px-2 py-1.5 text-xs outline-none focus:border-sky-400 transition-colors" type="text" placeholder="Usuário (ex: root)" bind:value={formUser} required />
        <input class="w-16 bg-[#0d0e17] border border-white/10 rounded text-slate-200 px-2 py-1.5 text-xs outline-none focus:border-sky-400 transition-colors" type="text" placeholder="Porta" bind:value={formPort} />
      </div>
      <input class="bg-[#0d0e17] border border-white/10 rounded text-slate-200 px-2 py-1.5 text-xs outline-none focus:border-sky-400 transition-colors" type="text" placeholder="IP / Hostname (ex: 192.168.1.10)" bind:value={formIp} required />
      <input class="bg-[#0d0e17] border border-white/10 rounded text-slate-200 px-2 py-1.5 text-xs outline-none focus:border-sky-400 transition-colors" type="text" placeholder="Chave Privada (ex: ~/.ssh/id_rsa - opcional)" bind:value={formKey} />
      <button type="submit" class="bg-sky-400 hover:bg-sky-500 text-slate-950 font-semibold rounded py-1.5 text-xs transition-colors cursor-pointer">
        {editingId ? 'Atualizar Conexão' : 'Salvar'}
      </button>
    </form>
  {/if}

  <!-- Lista de Hosts Salvos -->
  <div class="max-h-60 overflow-y-auto flex flex-col gap-1">
    {#if configStore.hosts.length === 0}
      <div class="text-center text-slate-400 text-xs py-4 leading-relaxed">
        Nenhum host SSH salvo.<br />
        Clique no <b>+</b> acima para adicionar.
      </div>
    {:else}
      {#each configStore.hosts as host (host.id)}
        <div 
          class="flex justify-between items-center px-2 py-1.5 rounded bg-white/[0.02] border border-transparent hover:bg-sky-400/10 hover:border-sky-400/20 cursor-pointer transition-all group {editingId === host.id ? 'border-sky-400/40 bg-sky-400/10' : ''}" 
          onclick={() => onConnect(host)}
          role="button"
          tabindex="0"
          onkeydown={(e) => e.key === 'Enter' && onConnect(host)}
        >
          <div class="flex flex-col gap-0.5 overflow-hidden pr-2">
            <span class="text-xs font-medium text-slate-200 truncate">{host.label}</span>
            <span class="text-[10px] text-slate-400 font-mono truncate">{host.user}@{host.ip}{host.port && host.port !== '22' ? `:${host.port}` : ''}</span>
          </div>
          <div class="flex items-center gap-1.5 shrink-0">
            <!-- Botão de Editar (Lápis SVG) -->
            <button 
              type="button" 
              class="text-slate-400 hover:text-sky-300 hover:bg-sky-400/15 p-1 rounded text-xs transition-all cursor-pointer border-none bg-transparent flex items-center justify-center" 
              onclick={(e) => startEdit(host, e)} 
              title="Editar dados"
            >
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"></path>
                <path d="m15 5 4 4"></path>
              </svg>
            </button>
            <span class="text-[10px] text-sky-400 bg-sky-400/15 px-1.5 py-0.5 rounded">Conectar ↵</span>
            <button class="text-slate-400 hover:text-red-400 hover:bg-red-400/15 p-1 rounded text-xs leading-none transition-all cursor-pointer border-none bg-transparent" onclick={(e) => removeHost(host.id, e)} title="Remover">✕</button>
          </div>
        </div>
      {/each}
    {/if}
  </div>
</Modal>
