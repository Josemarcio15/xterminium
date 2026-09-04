<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Terminal } from '@xterm/xterm';
  import { FitAddon } from '@xterm/addon-fit';
  import '@xterm/xterm/css/xterm.css';
  import { invoke } from '@tauri-apps/api/core';
  import { type SshHost } from './types';
  import { ConfigService } from './config';
  import { normalizeShortcut, parseKeyboardEvent } from './shortcuts';

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

    // Intercepta atalhos configurados dinamicamente de forma síncrona
    term.attachCustomKeyEventHandler((e) => {
      if (e.type !== 'keydown') return true;

      // Recarrega se o cache foi atualizado
      ConfigService.loadShortcuts().then((s) => {
        shortcuts = s;
      });

      const pressed = normalizeShortcut(parseKeyboardEvent(e));
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
      invoke('spawn_pty', {
        id,
        cols: term.cols,
        rows: term.rows,
        command: 'ssh',
        args: [...portArg, `${sshInfo.user}@${sshInfo.ip}`],
      }).catch(console.error);
    } else {
      invoke('spawn_pty', {
        id,
        cols: term.cols,
        rows: term.rows,
      }).catch(console.error);
    }

    term.onData((data) => {
      invoke('write_pty', { id, data }).catch(console.error);
    });

    setTimeout(() => {
      fitAddon.fit();
      term.focus();
    }, 50);
  });

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

<style>
  :global(.xterm) {
    height: 100%;
  }

  :global(.xterm-viewport) {
    overflow-y: auto !important;
  }
</style>
