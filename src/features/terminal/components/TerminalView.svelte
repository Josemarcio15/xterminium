<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Terminal } from '@xterm/xterm';
  import { FitAddon } from '@xterm/addon-fit';
  import '@xterm/xterm/css/xterm.css';
  import { invoke } from '@tauri-apps/api/core';
  import { type SshHost, type CustomCommand, type SavedPath } from '../../../core/types';
  import { ConfigService, PtyService } from '../../../core/services';
  import { normalizeShortcut, parseKeyboardEvent } from '../utils/shortcuts';
  import SshAutocompleteDropdown from './SshAutocompleteDropdown.svelte';
  import DirectoryAutocompleteDropdown from './DirectoryAutocompleteDropdown.svelte';
  import { configStore } from '../../../core/stores/config.svelte';

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
      allowTransparency: true,
      cursorBlink: true,
      fontFamily: '"JetBrains Mono", "Fira Code", monospace',
      fontSize: 14,
      lineHeight: 1.2,
      theme: {
        background: configStore.theme.terminalBg,
        foreground: configStore.theme.terminalFg,
        cursor: type === 'ssh' ? configStore.theme.terminalCursorSsh : configStore.theme.terminalCursorLocal,
        selectionBackground: configStore.theme.terminalSelection,
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
            if (text) PtyService.writePty(id, text).catch(console.error);
          })
          .catch(console.error);
      },
      selectAll: () => {
        term.selectAll();
      },
      stop: () => {
        PtyService.writePty(id, '\x03').catch(console.error);
      },
      newTab: () => {
        onNewTab();
      },
      newWindow: () => {
        invoke('open_new_window').catch(console.error);
      },
      clear: () => {
        term.clear();
      },
    };

    // Intercepta atalhos configurados dinamicamente de forma síncrona
    term.attachCustomKeyEventHandler((e) => {
      if (e.type !== 'keydown') return true;

      const shortcuts = configStore.shortcuts;

      // Atalho para disparar autocomplete de VPS manualmente (configurável, padrão Ctrl+Space)
      const pressed = normalizeShortcut(parseKeyboardEvent(e));
      const autoShortcut = normalizeShortcut(shortcuts.autocomplete || 'Ctrl+Space');
      if (pressed && pressed === autoShortcut) {
        triggerManualAutocomplete();
        return false;
      }

      // Atalho para disparar autocomplete de diretórios (configurável, padrão Shift+Space)
      const dirShortcut = normalizeShortcut(shortcuts.directoryAutocomplete || 'Shift+Space');
      if (pressed && pressed === dirShortcut) {
        triggerDirectoryAutocomplete();
        return false;
      }

      // Se o dropdown de autocomplete VPS estiver ativo, capturar setas, Tab, Enter e Esc
      if (showDropdown && filteredHosts.length > 0) {
        if (e.key === 'ArrowDown' || (e.key === 'Tab' && !e.shiftKey)) {
          e.preventDefault();
          selectedHostIndex = (selectedHostIndex + 1) % filteredHosts.length;
          return false;
        }
        if (e.key === 'ArrowUp' || (e.key === 'Tab' && e.shiftKey)) {
          e.preventDefault();
          selectedHostIndex = (selectedHostIndex - 1 + filteredHosts.length) % filteredHosts.length;
          return false;
        }
        if (e.key === 'Enter') {
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

      // Se o dropdown de diretórios estiver ativo, capturar setas, Tab, Enter e Esc
      if (showDirDropdown && filteredPaths.length > 0) {
        if (e.key === 'ArrowDown' || (e.key === 'Tab' && !e.shiftKey)) {
          e.preventDefault();
          selectedDirIndex = (selectedDirIndex + 1) % filteredPaths.length;
          return false;
        }
        if (e.key === 'ArrowUp' || (e.key === 'Tab' && e.shiftKey)) {
          e.preventDefault();
          selectedDirIndex = (selectedDirIndex - 1 + filteredPaths.length) % filteredPaths.length;
          return false;
        }
        if (e.key === 'Enter') {
          e.preventDefault();
          e.stopPropagation();
          const selected = filteredPaths[selectedDirIndex];
          if (selected) {
            applyDirectoryAutocomplete(selected);
          }
          return false;
        }
        if (e.key === 'Escape') {
          e.preventDefault();
          e.stopPropagation();
          closeDirAutocomplete();
          return false;
        }
      }

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
      PtyService.spawnPty({
        id,
        cols: term.cols,
        rows: term.rows,
        command: 'ssh',
        args: [...keyArg, ...portArg, `${sshInfo.user}@${sshInfo.ip}`],
      }).catch(console.error);
    } else {
      PtyService.spawnPty({
        id,
        cols: term.cols,
        rows: term.rows,
      }).catch(console.error);
    }

    term.onData((data) => {
      // Fecha os dropdowns se o usuário der Enter ou Ctrl+C
      if (showDropdown && (data.includes('\r') || data.includes('\n') || data === '\x03' || data === '\x15')) {
        closeAutocomplete();
      }
      if (showDirDropdown && (data.includes('\r') || data.includes('\n') || data === '\x03' || data === '\x15')) {
        closeDirAutocomplete();
      }
      PtyService.writePty(id, data).catch(console.error);
    });

    setTimeout(() => {
      fitAddon.fit();
      term.focus();
    }, 50);
  });

  // Estado do Autocomplete de VPS
  let availableSshHosts = $state<SshHost[]>([]);
  let availableCustomCommands = $state<CustomCommand[]>([]);
  let showDropdown = $state(false);
  let filteredHosts = $state<SshHost[]>([]);
  let selectedHostIndex = $state(0);
  let dropdownPosition = $state({ x: 100, y: 100 });
  let activeMatchedCommand = $state<CustomCommand | null>(null);
  let currentMatchedQuery = '';

  // Estado do Autocomplete de Diretórios
  let availablePaths = $state<SavedPath[]>([]);
  let showDirDropdown = $state(false);
  let filteredPaths = $state<SavedPath[]>([]);
  let selectedDirIndex = $state(0);
  let dirDropdownPosition = $state({ x: 100, y: 100 });
  let currentDirMatchedQuery = '';

  async function triggerManualAutocomplete() {
    if (type !== 'local') return;

    availableSshHosts = configStore.hosts;
    availableCustomCommands = configStore.commands;

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

    // Identifica se algum dos comandos configurados está presente na linha antes do cursor
    let matchedCmd: CustomCommand | null = null;
    for (const cmd of availableCustomCommands) {
      const regex = new RegExp(`(?:^|[;&|\\s])${cmd.command}(?:\\s+|$)`, 'i');
      if (regex.test(textBeforeCursor)) {
        matchedCmd = cmd;
        break;
      }
    }

    // Se nenhum comando configurado foi detectado, usa o primeiro comando ou fallback
    if (!matchedCmd && availableCustomCommands.length > 0) {
      matchedCmd = availableCustomCommands[0];
    }
    activeMatchedCommand = matchedCmd;

    // Verifica se o usuário já começou a digitar algum prefixo do host antes do cursor
    const match = textBeforeCursor.match(/([a-zA-Z0-9_\-\.]+)$/);
    const query = match ? match[1] : '';

    // Se a query for o próprio comando configurado ou flag ("-r", etc.), não filtra por esse termo
    const knownCommands = availableCustomCommands.map((c) => c.command.toLowerCase());
    const isCommandWord = knownCommands.includes(query.toLowerCase());
    const validQuery = (isCommandWord || query.startsWith('-')) ? '' : query;

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

  // --- Autocomplete de Diretórios ---

  async function triggerDirectoryAutocomplete() {
    if (type !== 'local') return;

    availablePaths = configStore.paths;

    if (!availablePaths || availablePaths.length === 0 || !term) return;

    // Obtém o texto antes do cursor
    const buffer = term.buffer.active;
    const cursorY = buffer.cursorY;
    const lineObj = buffer.getLine(buffer.baseY + cursorY);
    let textBeforeCursor = '';
    if (lineObj) {
      const fullLine = lineObj.translateToString(true);
      textBeforeCursor = fullLine.slice(0, buffer.cursorX);
    }

    // Tenta pegar o último token como query de filtro
    const match = textBeforeCursor.match(/([^\s]+)$/);
    const query = match ? match[1] : '';

    if (query.length > 0) {
      const q = query.toLowerCase();
      filteredPaths = availablePaths.filter((p) =>
        p.name.toLowerCase().includes(q) || p.path.toLowerCase().includes(q)
      );
      currentDirMatchedQuery = query;
    } else {
      filteredPaths = [...availablePaths];
      currentDirMatchedQuery = '';
    }

    if (filteredPaths.length > 0) {
      selectedDirIndex = 0;
      updateDirDropdownPosition();
      showDirDropdown = true;
    } else {
      closeDirAutocomplete();
    }
  }

  function updateDirDropdownPosition() {
    if (!container || !term) return;
    const rect = container.getBoundingClientRect();
    const core = (term as any)._core;
    const cellWidth = core?._renderService?.dimensions?.css?.cell?.width || 9;
    const cellHeight = core?._renderService?.dimensions?.css?.cell?.height || 17;

    const cursorX = term.buffer.active.cursorX;
    const cursorY = term.buffer.active.cursorY;

    const posX = rect.left + cursorX * cellWidth;
    const posY = rect.top + (cursorY + 1.2) * cellHeight;
    const finalY = posY + 200 > window.innerHeight ? Math.max(10, posY - 220) : posY;

    dirDropdownPosition = {
      x: Math.min(posX, window.innerWidth - 340),
      y: finalY,
    };
  }

  function closeDirAutocomplete() {
    showDirDropdown = false;
    filteredPaths = [];
    selectedDirIndex = 0;
    currentDirMatchedQuery = '';
  }

  function applyDirectoryAutocomplete(savedPath: SavedPath) {
    if (!savedPath) return;

    // Apaga o prefixo digitado (se houver) e insere apenas o path
    const backspaces = '\x7f'.repeat(currentDirMatchedQuery.length);
    PtyService.writePty(id, backspaces + savedPath.path).catch(console.error);

    closeDirAutocomplete();
    term.focus();
    requestAnimationFrame(() => {
      term.focus();
    });
  }

  function applyAutocomplete(host: SshHost) {
    if (!host) return;

    // Quantidade de caracteres que o usuário já digitou e precisam ser apagados
    const backspaces = '\x7f'.repeat(currentMatchedQuery.length);

    // Substitui placeholders no template e nos args
    const port = host.port || '22';
    const key = host.key || '';
    const user = host.user || '';
    const ip = host.ip || '';
    const label = host.label || '';

    const formatString = (str?: string) => {
      if (!str) return '';
      return str
        .replace(/\{user\}/g, user)
        .replace(/\{ip\}/g, ip)
        .replace(/\{port\}/g, port)
        .replace(/\{key\}/g, key)
        .replace(/\{label\}/g, label);
    };

    let replacement = '';
    if (activeMatchedCommand) {
      const templateStr = formatString(activeMatchedCommand.template);
      const suffixStr = formatString(activeMatchedCommand.suffixArgs);
      replacement = `${templateStr}${suffixStr}`;
    } else {
      replacement = `${user}@${ip}`;
    }

    // Envia ao PTY os backspaces para remover a query digitada (se houver) e insere o texto formatado
    PtyService.writePty(id, backspaces + replacement).catch(console.error);

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
      PtyService.resizePty(id, term.cols, term.rows).catch(console.error);
    }
  }

  // Atualiza o tema do terminal reativamente quando o store muda
  $effect(() => {
    if (!term) return;
    const t = configStore.theme;
    term.options.theme = {
      background: t.terminalBg,
      foreground: t.terminalFg,
      cursor: type === 'ssh' ? t.terminalCursorSsh : t.terminalCursorLocal,
      selectionBackground: t.terminalSelection,
    };
  });

  onDestroy(() => {
    PtyService.closePty(id).catch(console.error);
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
    commandName={activeMatchedCommand?.command || 'vps'}
    onSelect={applyAutocomplete}
  />
{/if}

{#if showDirDropdown && active}
  <DirectoryAutocompleteDropdown
    paths={filteredPaths}
    selectedIndex={selectedDirIndex}
    position={dirDropdownPosition}
    onSelect={applyDirectoryAutocomplete}
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
