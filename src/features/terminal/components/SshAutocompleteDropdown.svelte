<script lang="ts">
  import { type SshHost } from '../../../core/types';

  interface Props {
    hosts: SshHost[];
    selectedIndex: number;
    position: { x: number; y: number };
    commandName?: string;
    onSelect: (host: SshHost) => void;
  }

  let { hosts, selectedIndex, position, commandName = 'vps', onSelect }: Props = $props();
</script>

{#if hosts.length > 0}
  <div
    class="fixed z-50 flex flex-col bg-[#161922]/95 backdrop-blur-md border border-sky-500/30 rounded-lg shadow-[0_8px_24px_rgba(0,0,0,0.6)] py-1 min-w-[280px] max-w-[360px] text-xs font-mono select-none pointer-events-auto"
    style="left: {Math.max(10, position.x)}px; top: {Math.max(10, position.y)}px;"
  >
    <!-- Header com indicação do comando detectado -->
    <div class="px-2.5 py-1 text-[10px] uppercase font-bold tracking-wider text-slate-400 border-b border-white/5 flex items-center justify-between">
      <span class="flex items-center gap-1.5">
        <span class="w-1.5 h-1.5 rounded-full bg-sky-400 shadow-[0_0_6px_#38bdf8]"></span>
        Hosts Salvos ({commandName.toUpperCase()})
      </span>
      <span class="text-[9px] text-slate-500 lowercase font-normal">Tab / Enter</span>
    </div>

    <!-- Lista de hosts sugeridos -->
    <div class="max-h-48 overflow-y-auto overflow-x-hidden flex flex-col p-1 gap-0.5">
      {#each hosts as host, idx}
        <button
          type="button"
          tabindex="-1"
          class="w-full text-left px-2 py-1.5 rounded flex items-center justify-between gap-2 border-none cursor-pointer transition-colors {idx === selectedIndex ? 'bg-sky-500/20 text-sky-200 border border-sky-500/40' : 'bg-transparent text-slate-300 hover:bg-white/5'}"
          onmousedown={(e) => e.preventDefault()}
          onclick={() => onSelect(host)}
        >
          <div class="flex flex-col min-w-0">
            <span class="font-bold truncate text-white flex items-center gap-1.5">
              {host.label || host.ip}
              {#if host.port && host.port !== '22'}
                <span class="text-[9px] px-1 py-0.2 bg-white/10 rounded text-slate-300">:{host.port}</span>
              {/if}
            </span>
            <span class="text-[11px] text-slate-400 truncate">
              {host.user}@{host.ip}
            </span>
          </div>

          <div class="text-right shrink-0">
            <span class="text-[10px] text-sky-400/80 bg-sky-950/40 px-1.5 py-0.5 rounded border border-sky-400/20">
              selecionar
            </span>
          </div>
        </button>
      {/each}
    </div>
  </div>
{/if}
