import { invoke } from "@tauri-apps/api/core";
import { readText, writeText } from "@tauri-apps/plugin-clipboard-manager";
import { Terminal } from "@xterm/xterm";
import { PtyService } from "../../../core/services";
import { normalizeShortcut, parseKeyboardEvent } from "../utils/shortcuts";
import { configStore } from "../../../core/stores/config.svelte";

export interface ShortcutHandlers {
  onNewTab: () => void;
  // Aliases
  hasActiveAlias: () => boolean;
  applyAlias: () => void;
  closeAlias: () => void;
  // Autocomplete VPS
  hasActiveVps: () => boolean;
  onVpsNext: () => void;
  onVpsPrev: () => void;
  onVpsSelect: () => void;
  onVpsClose: () => void;
  triggerVpsManual: () => void;
  // Autocomplete Diretórios
  hasActiveDir: () => boolean;
  onDirNext: () => void;
  onDirPrev: () => void;
  onDirSelect: () => void;
  onDirClose: () => void;
  triggerDirManual: () => void;
}

export function createTerminalKeyHandler(
  term: Terminal,
  ptyId: string,
  handlers: ShortcutHandlers,
) {
  let lastPasteTime = 0;

  // Comandos genéricos executados pelo terminal
  const actions: Record<string, () => void> = {
    copy: async () => {
      if (term.hasSelection()) {
        const text = term.getSelection();
        try {
          await writeText(text);
        } catch {
          if (navigator.clipboard && navigator.clipboard.writeText) {
            navigator.clipboard.writeText(text).catch(() => {
              invoke("write_clipboard", { text }).catch(console.error);
            });
          } else {
            invoke("write_clipboard", { text }).catch(console.error);
          }
        }
      }
    },
    paste: async () => {
      const now = Date.now();
      if (now - lastPasteTime < 150) {
        return; // Evita disparo duplicado
      }
      lastPasteTime = now;

      let text = "";
      // 1. Leitura robusta oficial do Tauri (Cross-platform nativo no Rust: Windows, Linux, macOS)
      try {
        text = await readText();
      } catch {}

      // 2. Fallback pelo comando Rust nativo invoke('read_clipboard')
      if (!text) {
        try {
          text = await invoke<string>("read_clipboard");
        } catch {}
      }

      // 3. Fallback pelo navegador / WebView
      if (!text) {
        try {
          if (navigator.clipboard && navigator.clipboard.readText) {
            text = await navigator.clipboard.readText();
          }
        } catch {}
      }

      if (text) {
        PtyService.writePty(ptyId, text).catch(console.error);
      }
    },
    selectAll: () => {
      term.selectAll();
    },
    stop: () => {
      PtyService.writePty(ptyId, "\x03").catch(console.error);
    },
    newTab: () => {
      handlers.onNewTab();
    },
    newWindow: () => {
      invoke("new_window").catch(console.error);
    },
    clear: () => {
      term.clear();
    },
  };

  return (e: KeyboardEvent): boolean => {
    if (e.type !== "keydown") return true;

    const shortcuts = configStore.shortcuts;
    const pressed = normalizeShortcut(parseKeyboardEvent(e));

    // 1. Atalho para disparar autocomplete de VPS manualmente (ex: Ctrl+Space)
    const autoShortcut = normalizeShortcut(
      shortcuts.autocomplete || "Ctrl+Space",
    );
    if (pressed && pressed === autoShortcut) {
      e.preventDefault();
      e.stopPropagation();
      handlers.triggerVpsManual();
      return false;
    }

    // 2. Atalho para disparar autocomplete de diretórios manualmente (ex: Shift+Space)
    const dirShortcut = normalizeShortcut(
      shortcuts.directoryAutocomplete || "Shift+Space",
    );
    if (pressed && pressed === dirShortcut) {
      e.preventDefault();
      e.stopPropagation();
      handlers.triggerDirManual();
      return false;
    }

    // 3. Intercepta Enter ou Escape para sugestão de Alias (Estilo iOS)
    if (handlers.hasActiveAlias()) {
      if (e.key === "Enter") {
        e.preventDefault();
        e.stopPropagation();
        handlers.applyAlias();
        return false;
      }
      if (e.key === "Escape") {
        e.preventDefault();
        e.stopPropagation();
        handlers.closeAlias();
        return false;
      }
    }

    // 4. Se algum dropdown de autocomplete (VPS ou Dir) estiver ativo
    const isVpsActive = handlers.hasActiveVps();
    const isDirActive = handlers.hasActiveDir();

    if (isVpsActive || isDirActive) {
      const isTabKey =
        e.key === "Tab" ||
        e.key === "Backtab" ||
        e.code === "Tab" ||
        e.keyCode === 9;
      const isShift = e.shiftKey || e.key === "Backtab";
      const isBackTab = isTabKey && isShift;
      const isNextTab = isTabKey && !isShift;

      if (isVpsActive) {
        if (e.key === "ArrowDown" || isNextTab) {
          e.preventDefault();
          e.stopPropagation();
          handlers.onVpsNext();
          return false;
        }
        if (e.key === "ArrowUp" || isBackTab) {
          e.preventDefault();
          e.stopPropagation();
          handlers.onVpsPrev();
          return false;
        }
        if (e.key === "Enter") {
          e.preventDefault();
          e.stopPropagation();
          handlers.onVpsSelect();
          return false;
        }
        if (e.key === "Escape") {
          e.preventDefault();
          e.stopPropagation();
          handlers.onVpsClose();
          return false;
        }
      }

      if (isDirActive) {
        if (e.key === "ArrowDown" || isNextTab) {
          e.preventDefault();
          e.stopPropagation();
          handlers.onDirNext();
          return false;
        }
        if (e.key === "ArrowUp" || isBackTab) {
          e.preventDefault();
          e.stopPropagation();
          handlers.onDirPrev();
          return false;
        }
        if (e.key === "Enter") {
          e.preventDefault();
          e.stopPropagation();
          handlers.onDirSelect();
          return false;
        }
        if (e.key === "Escape") {
          e.preventDefault();
          e.stopPropagation();
          handlers.onDirClose();
          return false;
        }
      }
    }

    if (!pressed) return true;

    // 5. Tratamento inteligente para Copiar:
    // Se coincidir com o atalho de cópia cadastrado (ex: Ctrl+Shift+C ou Ctrl+C)
    // OU se o usuário apertou Ctrl+C e tem texto selecionado (comportamento nativo de terminal):
    const isCtrlC = pressed === "ctrl+c";
    const hasSelection = term.hasSelection();
    const copyKey = normalizeShortcut(shortcuts.copy || "");

    if (pressed === copyKey || (isCtrlC && hasSelection)) {
      e.preventDefault();
      e.stopPropagation();
      actions.copy();
      return false;
    }

    // 6. Procura se alguma ação registrada coincide com a combinação pressionada
    for (const [action, combo] of Object.entries(shortcuts)) {
      if (combo && normalizeShortcut(combo) === pressed) {
        const handler = actions[action];
        if (handler) {
          e.preventDefault();
          e.stopPropagation();
          handler();
          return false;
        }
      }
    }

    return true;
  };
}
