<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Terminal } from '@xterm/xterm';
  import { FitAddon } from '@xterm/addon-fit';
  import '@xterm/xterm/css/xterm.css';
  import { invoke } from '@tauri-apps/api/core';
  import { type SshHost } from './types';
  import { ConfigService } from './config';
  import { normalizeShortcut, parseKeyboardEvent } from './shortcuts';
  import SshAutocompleteDropdown from './SshAutocompleteDropdown.svelte';

  interface Props {
    id: string;
    type: 'local' | 'ssh';
    sshInfo?: SshHost;
    active: boolean;
    onNewTab: () => void;
  }

  let { id, type, sshInfo, active, onNewTab }: Props = $props();

  let container: HTMLDivElement;
  let term: Terminal;
  let fitAddon: FitAddon;

  onMount(async () => {
    term = new Terminal({
      cursorBlink: true,
      fontFamily: '"JetBrains Mono", "Fira Code", monospace',
      fontSize: 14,
      lineHeight: 1.2,
      theme: {
        background: '#0f111a',
        foreground: '#e6e6e6',
        cursor: type === 'ssh' ? '#38bdf8' : '#00e699',
        selectionBackground: 'rgba(255, 255, 255, 0.2)',
      },
    });

    fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.open(container);
    fitAddon.fit();

    // Tabela genérica de comandos do terminal
    const commands: Record<string, () => void> = {
      copy: () => {
        if (term.hasSelection()) {
          const text = term.getSelection();
          invoke('write_clipboard', { text }).catch(() => {
            navigator.clipboard.writeText(text).catch(() => {});
          });
        }
      },
      paste: () => {
        invoke<string>('read_clipboard')
          .then((text) => (!text ? navigator.clipboard.readText() : text))
          .then((text) => {
            if (text) invoke('write_pty', { id, data: text }).catch(console.error);
          })
          .catch(() => {});
      },
      selectAll: () => {
        term.selectAll();
      },
      stop: () => {
        invoke('write_pty', { id, data: '\x03' }).catch(console.error);
      },
      newTab: () => {
        onNewTab();
      },
      newWindow: () => {
        invoke('new_window').catch(console.error);
      },
    };

    // Pré-carrega atalhos em cache
    let shortcuts = await ConfigService.loadShortcuts();

    // Carrega hosts SSH disponíveis para autocomplete
    ConfigService.loadSshHosts().then((hosts) => {
      availableSshHosts = hosts;
    });

    // Intercepta atalhos configurados dinamicamente de forma síncrona
    term.attachCustomKeyEventHandler((e) => {
      if (e.type !== 'keydown') return true;

      // Atalho para disparar autocomplete de VPS manualmente (configurável, padrão Ctrl+Space)
      const pressed = normalizeShortcut(parseKeyboardEvent(e));
      const autoShortcut = normalizeShortcut(shortcuts.autocomplete || 'Ctrl+Space');
      if (pressed && pressed === autoShortcut) {
        triggerManualAutocomplete();
        return false;
      }

      // Se o dropdown de autocomplete estiver ativo, capturar setas, Tab, Enter e Esc
      if (showDropdown && filteredHosts.length > 0) {
        if (e.key === 'ArrowDown') {
          selectedHostIndex = (selectedHostIndex + 1) % filteredHosts.length;
          return false;
        }
        if (e.key === 'ArrowUp') {
          selectedHostIndex = (selectedHostIndex - 1 + filteredHosts.length) % filteredHosts.length;
          return false;
        }
        if (e.key === 'Tab' || e.key === 'Enter') {
          e.preventDefault();
          e.stopPropagation();
          const selected = filteredHosts[selectedHostIndex];
          if (selected) {
            applyAutocomplete(selected);
          }
          return false;
        }
        if (e.key === 'Escape') {
          e.preventDefault();
          e.stopPropagation();
          closeAutocomplete();
          return false;
        }
      }

      // Recarrega se o cache foi atualizado
      ConfigService.loadShortcuts().then((s) => {
        shortcuts = s;
      });
      ConfigService.loadSshHosts().then((hosts) => {
        availableSshHosts = hosts;
      });

      if (!pressed) return true;

      // Tratamento inteligente para Ctrl+C (Copiar se tiver seleção, senão SIGINT)
      const isCtrlC = pressed === 'ctrl+c';
      const hasSelection = term.hasSelection();
      const copyKey = normalizeShortcut(shortcuts.copy || '');

      if (pressed === copyKey || (isCtrlC && hasSelection && normalizeShortcut(shortcuts.stop || '') !== 'ctrl+c')) {
        commands.copy();
        return false;
      }

      // Procura se alguma ação registrada coincide com a combinação pressionada
      for (const [action, combo] of Object.entries(shortcuts)) {
        if (combo && normalizeShortcut(combo) === pressed) {
          const handler = commands[action];
          if (handler) {
            handler();
            return false;
          }
        }
      }

      return true;
    });

    // Inicia PTY
    if (type === 'ssh' && sshInfo) {
      const portArg = sshInfo.port && sshInfo.port !== '22' ? ['-p', sshInfo.port] : [];
      const keyArg = sshInfo.key ? ['-i', sshInfo.key] : [];
      invoke('spawn_pty', {
        id,
        cols: term.cols,
        rows: term.rows,
        command: 'ssh',
        args: [...keyArg, ...portArg, `${sshInfo.user}@${sshInfo.ip}`],
      }).catch(console.error);
    } else {
      invoke('spawn_pty', {
        id,
        cols: term.cols,
        rows: term.rows,
      }).catch(console.error);
    }

    term.onData((data) => {
      // Fecha o dropdown se o usuário der Enter ou Ctrl+C
      if (showDropdown && (data.includes('\r') || data.includes('\n') || data === '\x03' || data === '\x15')) {
        closeAutocomplete();
      }
      invoke('write_pty', { id, data }).catch(console.error);
    });

    setTimeout(() => {
      fitAddon.fit();
      term.focus();
    }, 50);
  });

  // Estado do Autocomplete
  let availableSshHosts = $state<SshHost[]>([]);
  let showDropdown = $state(false);
  let filteredHosts = $state<SshHost[]>([]);
  let selectedHostIndex = $state(0);
  let dropdownPosition = $state({ x: 100, y: 100 });
  let detectedCommandType = $state<'scp' | 'ssh'>('scp');
  let currentMatchedQuery = '';

  async function triggerManualAutocomplete() {
    if (type !== 'local') return;

    // Recarrega hosts atualizados
    try {
      availableSshHosts = await ConfigService.loadSshHosts();
    } catch {
      // Ignora erro
    }

    if (!availableSshHosts || availableSshHosts.length === 0 || !term) {
      return;
    }

    // Obtém a linha onde está o cursor
    const buffer = term.buffer.active;
    const cursorY = buffer.cursorY;
    const lineObj = buffer.getLine(buffer.baseY + cursorY);
    let textBeforeCursor = '';
    if (lineObj) {
      const fullLine = lineObj.translateToString(true);
      textBeforeCursor = fullLine.slice(0, buffer.cursorX);
    }

    // Identifica o comando no contexto: scp ou ssh
    const isScp = /(?:^|[;&|\s])scp(?:\s+|$)/.test(textBeforeCursor);
    const isSsh = /(?:^|[;&|\s])ssh(?:\s+|$)/.test(textBeforeCursor);

    // Se nenhum comando específico foi digitado, assume 'scp' ou lista as VPS para conectar
    const cmd: 'scp' | 'ssh' = isScp ? 'scp' : (isSsh ? 'ssh' : 'scp');

    // Verifica se o usuário já começou a digitar algum prefixo do host antes do cursor
    // Ex: "scp arquivo.tar Deb" -> "Deb"
    // Ex: "scp arquivo.tar " -> ""
    const match = textBeforeCursor.match(/([a-zA-Z0-9_\-\.]+)$/);
    const query = match ? match[1] : '';

    // Se a query for o próprio comando ("scp" ou "ssh") ou flag ("-r"), não filtra por esse termo
    const validQuery = (query === 'scp' || query === 'ssh' || query.startsWith('-')) ? '' : query;

    if (validQuery.length > 0) {
      const q = validQuery.toLowerCase();
      filteredHosts = availableSshHosts.filter((h) => {
        const labelMatch = h.label && h.label.toLowerCase().includes(q);
        const ipMatch = h.ip.toLowerCase().includes(q);
        const userMatch = h.user.toLowerCase().includes(q);
        return labelMatch || ipMatch || userMatch;
      });
      currentMatchedQuery = validQuery;
    } else {
      // Se não digitou nada ou deu espaço, exibe TODAS as VPS salvas
      filteredHosts = [...availableSshHosts];
      currentMatchedQuery = '';
    }

    if (filteredHosts.length > 0) {
      selectedHostIndex = 0;
      detectedCommandType = cmd;
      updateDropdownPosition();
      showDropdown = true;
    } else {
      closeAutocomplete();
    }
  }

  function updateDropdownPosition() {
    if (!container || !term) return;
    const rect = container.getBoundingClientRect();
    
    // Tenta obter dimensões exatas de célula do xterm.js
    const core = (term as any)._core;
    const cellWidth = core?._renderService?.dimensions?.css?.cell?.width || 9;
    const cellHeight = core?._renderService?.dimensions?.css?.cell?.height || 17;

    const cursorX = term.buffer.active.cursorX;
    const cursorY = term.buffer.active.cursorY;

    const posX = rect.left + cursorX * cellWidth;
    const posY = rect.top + (cursorY + 1.2) * cellHeight;

    // Se estiver muito próximo da borda inferior da tela, joga para cima do cursor
    const finalY = posY + 200 > window.innerHeight ? Math.max(10, posY - 220) : posY;

    dropdownPosition = {
      x: Math.min(posX, window.innerWidth - 300),
      y: finalY,
    };
  }

  function closeAutocomplete() {
    showDropdown = false;
    filteredHosts = [];
    selectedHostIndex = 0;
    currentMatchedQuery = '';
  }

  function applyAutocomplete(host: SshHost) {
    if (!host) return;

    // Quantidade de caracteres que o usuário já digitou e precisam ser apagados
    const backspaces = '\x7f'.repeat(currentMatchedQuery.length);

    let replacement = '';
    if (detectedCommandType === 'scp') {
      // Formata como usuario@ip:~/
      replacement = `${host.user}@${host.ip}:~/`;
    } else {
      // ssh usuario@ip (ou adiciona a porta se não for 22)
      if (host.port && host.port !== '22') {
        replacement = `-p ${host.port} ${host.user}@${host.ip}`;
      } else {
        replacement = `${host.user}@${host.ip}`;
      }
    }

    // Envia ao PTY os backspaces para remover a query digitada (se houver) e insere o texto completo
    invoke('write_pty', { id, data: backspaces + replacement }).catch(console.error);

    closeAutocomplete();
    term.focus();
    requestAnimationFrame(() => {
      term.focus();
    });
  }

  export function write(data: string) {
    if (term) term.write(data);
  }

  export function clear() {
    if (term) term.clear();
  }

  export function fitAndFocus() {
    if (fitAddon && term) {
      fitAddon.fit();
      term.focus();
      invoke('resize_pty', {
        id,
        cols: term.cols,
        rows: term.rows,
      }).catch(console.error);
    }
  }

  onDestroy(() => {
    invoke('close_pty', { id }).catch(console.error);
    if (term) term.dispose();
  });
</script>

<div 
  bind:this={container} 
  class="absolute inset-0 px-[10px] py-2 box-border {active ? 'visible pointer-events-auto z-[2]' : 'invisible pointer-events-none z-[1]'}"
></div>

{#if showDropdown && active}
  <SshAutocompleteDropdown
    hosts={filteredHosts}
    selectedIndex={selectedHostIndex}
    position={dropdownPosition}
    commandType={detectedCommandType}
    onSelect={applyAutocomplete}
  />
{/if}

<style>
  :global(.xterm) {
    height: 100%;
  }

  :global(.xterm-viewport) {
    overflow-y: auto !important;
  }
</style>
