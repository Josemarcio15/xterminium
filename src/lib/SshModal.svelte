<script lang="ts">
  import { type SshHost } from './types';
  import { ConfigService } from './config';

  interface Props {
    show: boolean;
    onClose: () => void;
    onConnect: (host: SshHost) => void;
  }

  let { show = false, onClose, onConnect }: Props = $props();

  let hosts = $state<SshHost[]>([]);
  let showAddForm = $state(false);
  let newLabel = $state('');
  let newUser = $state('');
  let newIp = $state('');
  let newPort = $state('22');

  $effect(() => {
    if (show) {
      ConfigService.loadSshHosts().then((loaded) => {
        hosts = loaded;
      });
    }
  });

  async function addHost() {
    if (!newUser.trim() || !newIp.trim()) return;

    const host: SshHost = {
      id: crypto.randomUUID(),
      label: newLabel.trim() || `${newUser.trim()}@${newIp.trim()}`,
      user: newUser.trim(),
      ip: newIp.trim(),
      port: newPort.trim() || '22',
    };

    hosts.push(host);
    await ConfigService.saveSshHosts(hosts);

    newLabel = '';
    newUser = '';
    newIp = '';
    newPort = '22';
    showAddForm = false;
  }

  async function removeHost(id: string, e: MouseEvent) {
    e.stopPropagation();
    hosts = hosts.filter((h) => h.id !== id);
    await ConfigService.saveSshHosts(hosts);
  }
</script>

{#if show}
  <button 
    type="button" 
    class="fixed inset-0 z-[150] bg-transparent border-none cursor-default" 
    onclick={onClose}
    aria-label="Fechar modal"
  ></button>
  <div 
    class="absolute top-9 right-0 w-80 bg-[#171926] border border-white/10 rounded-lg shadow-2xl p-3 z-[160]" 
    role="dialog" 
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.key === 'Escape' && onClose()}
  >
    <div class="flex justify-between items-center mb-2.5">
      <div class="flex items-center gap-1.5 text-xs font-semibold text-slate-100">
        <svg class="text-sky-400" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="2" width="20" height="8" rx="2" ry="2"></rect><rect x="2" y="14" width="20" height="8" rx="2" ry="2"></rect><line x1="6" y1="6" x2="6.01" y2="6"></line><line x1="6" y1="18" x2="6.01" y2="18"></line></svg>
        <span>Conexões SSH</span>
      </div>
      <button 
        class="w-[22px] h-[22px] rounded flex items-center justify-center text-xs bg-white/5 border border-white/10 text-slate-400 hover:bg-white/10 hover:text-white transition-all" 
        onclick={() => (showAddForm = !showAddForm)} 
        title={showAddForm ? 'Fechar formulário' : 'Novo Host'}
      >
        {showAddForm ? '✕' : '+'}
      </button>
    </div>

    <!-- Formulário Novo Host -->
    {#if showAddForm}
      <form class="bg-[#12141f] border border-white/10 rounded-md p-2.5 mb-2.5 flex flex-col gap-2" onsubmit={(e) => { e.preventDefault(); addHost(); }}>
        <input class="bg-[#0d0e17] border border-white/10 rounded text-slate-200 px-2 py-1.5 text-xs outline-none focus:border-sky-400 transition-colors" type="text" placeholder="Nome/Apelido (ex: Servidor Prod)" bind:value={newLabel} />
        <div class="flex gap-2">
          <input class="flex-1 bg-[#0d0e17] border border-white/10 rounded text-slate-200 px-2 py-1.5 text-xs outline-none focus:border-sky-400 transition-colors" type="text" placeholder="Usuário (ex: root)" bind:value={newUser} required />
          <input class="w-16 bg-[#0d0e17] border border-white/10 rounded text-slate-200 px-2 py-1.5 text-xs outline-none focus:border-sky-400 transition-colors" type="text" placeholder="Porta" bind:value={newPort} />
        </div>
        <input class="bg-[#0d0e17] border border-white/10 rounded text-slate-200 px-2 py-1.5 text-xs outline-none focus:border-sky-400 transition-colors" type="text" placeholder="IP / Hostname (ex: 192.168.1.10)" bind:value={newIp} required />
        <button type="submit" class="bg-sky-400 hover:bg-sky-500 text-slate-950 font-semibold rounded py-1.5 text-xs transition-colors cursor-pointer">Salvar</button>
      </form>
    {/if}

    <!-- Lista de Hosts Salvos -->
    <div class="max-h-60 overflow-y-auto flex flex-col gap-1">
      {#if hosts.length === 0}
        <div class="text-center text-slate-400 text-xs py-4 leading-relaxed">
          Nenhum host SSH salvo.<br />
          Clique no <b>+</b> acima para adicionar.
        </div>
      {:else}
        {#each hosts as host (host.id)}
          <div 
            class="flex justify-between items-center px-2 py-1.5 rounded bg-white/[0.02] border border-transparent hover:bg-sky-400/10 hover:border-sky-400/20 cursor-pointer transition-all" 
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
              <span class="text-[10px] text-sky-400 bg-sky-400/15 px-1.5 py-0.5 rounded">Conectar ↵</span>
              <button class="text-slate-400 hover:text-red-400 hover:bg-red-400/15 p-0.5 rounded text-xs leading-none transition-all cursor-pointer" onclick={(e) => removeHost(host.id, e)} title="Remover">✕</button>
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </div>
{/if}
