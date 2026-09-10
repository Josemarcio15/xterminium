import { Terminal } from "@xterm/xterm";
import { type CustomAlias } from "../../../core/types";
import { PtyService } from "../../../core/services";
import { configStore } from "../../../core/stores/config.svelte";

export function useAliasSuggestion(
  getId: () => string,
  getTerm: () => Terminal | null,
  getContainer: () => HTMLDivElement | null,
) {
  let activeSuggestion = $state<CustomAlias | null>(null);
  let matchedPrefix = $state("");
  let position = $state({ x: 100, y: 100 });

  function check(isOtherDropdownOpen: boolean) {
    const term = getTerm();
    if (!term || isOtherDropdownOpen) {
      close();
      return;
    }

    const availableAliases = configStore.aliases;
    if (!availableAliases || availableAliases.length === 0) {
      close();
      return;
    }

    // Lê a linha do cursor
    const buffer = term.buffer.active;
    const cursorY = buffer.cursorY;
    const lineObj = buffer.getLine(buffer.baseY + cursorY);
    let textBeforeCursor = "";
    if (lineObj) {
      const fullLine = lineObj.translateToString(true);
      textBeforeCursor = fullLine.slice(0, buffer.cursorX);
    }

    // Extrai a última palavra digitada antes do cursor
    const match = textBeforeCursor.match(/([a-zA-Z0-9_\-\.\/]+)$/);
    const currentWord = match ? match[1] : "";

    // Sugere a partir de 2 caracteres digitados se coincidir com o início de algum alias
    if (currentWord.length >= 2) {
      const wordLower = currentWord.toLowerCase();
      const matched = availableAliases.find((a) =>
        a.alias.toLowerCase().startsWith(wordLower)
      );

      if (matched) {
        activeSuggestion = matched;
        matchedPrefix = currentWord;
        updatePosition();
        return;
      }
    }

    close();
  }

  function updatePosition() {
    const container = getContainer();
    const term = getTerm();
    if (!container || !term) return;

    const rect = container.getBoundingClientRect();
    const core = (term as any)._core;
    const cellWidth = core?._renderService?.dimensions?.css?.cell?.width || 9;
    const cellHeight =
      core?._renderService?.dimensions?.css?.cell?.height || 17;

    const cursorX = term.buffer.active.cursorX;
    const cursorY = term.buffer.active.cursorY;

    const posX = rect.left + cursorX * cellWidth;
    let posY = rect.top + (cursorY - 1.8) * cellHeight;
    if (posY < rect.top + 5) {
      posY = rect.top + (cursorY + 1.2) * cellHeight;
    }

    position = {
      x: Math.min(posX, window.innerWidth - 320),
      y: Math.max(10, posY),
    };
  }

  let isApplying = $state(false);

  function close() {
    activeSuggestion = null;
    matchedPrefix = "";
  }

  function apply() {
    if (!activeSuggestion || isApplying) return;

    isApplying = true;
    const term = getTerm();
    let wordToErase = matchedPrefix;

    // Recalcula em tempo real a palavra antes do cursor para garantir
    // que se o usuário digitou letras a mais rapidamente antes do Enter,
    // todos os caracteres digitados sejam apagados sem sobrar nada!
    if (term) {
      const buffer = term.buffer.active;
      const cursorY = buffer.cursorY;
      const lineObj = buffer.getLine(buffer.baseY + cursorY);
      if (lineObj) {
        const fullLine = lineObj.translateToString(true);
        const textBeforeCursor = fullLine.slice(0, buffer.cursorX);
        const match = textBeforeCursor.match(/([a-zA-Z0-9_\-\.\/]+)$/);
        if (match && match[1]) {
          wordToErase = match[1];
        }
      }
    }

    // Se mesmo assim wordToErase tiver menos caracteres que matchedPrefix, usa o maior
    const eraseLength = Math.max(wordToErase.length, matchedPrefix.length);
    const backspaces = "\x7f".repeat(eraseLength);
    const commandToInsert = activeSuggestion.command;

    PtyService.writePty(getId(), backspaces + commandToInsert)
      .catch(console.error)
      .finally(() => {
        setTimeout(() => {
          isApplying = false;
        }, 50);
      });

    close();
    if (term) {
      term.focus();
      requestAnimationFrame(() => term.focus());
    }
  }

  return {
    get activeSuggestion() {
      return activeSuggestion;
    },
    get matchedPrefix() {
      return matchedPrefix;
    },
    get position() {
      return position;
    },
    get isApplying() {
      return isApplying;
    },
    check,
    close,
    apply,
  };
}

