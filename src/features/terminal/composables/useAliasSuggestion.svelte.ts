import { Terminal } from "@xterm/xterm";
import { type CustomAlias } from "../../../core/types";
import { PtyService } from "../../../core/services";
import { configStore } from "../../../core/stores/config.svelte";

export function useAliasSuggestion(
  getId: () => string,
  getTerm: () => Terminal | null,
  getContainer: () => HTMLDivElement | null,
) {
  let suggestions = $state<CustomAlias[]>([]);
  let selectedIndex = $state(0);
  let matchedPrefix = $state("");
  let position = $state({ x: 100, y: 100 });
  let isApplying = $state(false);

  // Evita que o eco do comando recém-aplicado reabra a sugestão imediatamente
  let justApplied = false;
  let justAppliedTimer: ReturnType<typeof setTimeout> | null = null;
  let lastAppliedCommand: string = "";

  const activeSuggestion = $derived(
    suggestions.length > 0 && selectedIndex < suggestions.length
      ? suggestions[selectedIndex]
      : null
  );

  function check(isOtherDropdownOpen: boolean) {
    const term = getTerm();
    if (!term || isOtherDropdownOpen || isApplying || justApplied) {
      close();
      return;
    }

    const availableAliases = configStore.aliases;
    if (!availableAliases || availableAliases.length === 0) {
      close();
      return;
    }

    // Lê a linha onde está o cursor no buffer do xterm
    const buffer = term.buffer.active;
    const cursorY = buffer.cursorY;
    const lineObj = buffer.getLine(buffer.baseY + cursorY);
    let textBeforeCursor = "";
    if (lineObj) {
      const fullLine = lineObj.translateToString(true);
      textBeforeCursor = fullLine.slice(0, buffer.cursorX);
    }

    if (!textBeforeCursor || textBeforeCursor.trim().length === 0) {
      close();
      return;
    }

    // Se a linha termina exatamente com o comando que acabou de ser aplicado pelo autocomplete, não sugere
    if (lastAppliedCommand && textBeforeCursor.toLowerCase().endsWith(lastAppliedCommand.toLowerCase())) {
      close();
      return;
    }

    const textLower = textBeforeCursor.toLowerCase();

    // Delimitadores que separam comandos independentes (pipes, encadeamento, redirecionamentos ou prompts)
    const delimiters = [";", "&", "|", ">", "$", "#", "❯", "\n", "\r"];
    let lastDelimIdx = -1;
    for (const d of delimiters) {
      const idx = textBeforeCursor.lastIndexOf(d);
      if (idx > lastDelimIdx) lastDelimIdx = idx;
    }

    const commandSegment = lastDelimIdx >= 0 ? textBeforeCursor.slice(lastDelimIdx + 1) : textBeforeCursor;
    // Remove espaços iniciais logo após o delimitador (ex: "$   npm ")
    const leadingSpaces = commandSegment.match(/^\s*/)?.[0]?.length || 0;
    const trimmedCommandStart = (lastDelimIdx >= 0 ? lastDelimIdx + 1 : 0) + leadingSpaces;
    const typedCommand = textBeforeCursor.slice(trimmedCommandStart);

    // Também consideramos as palavras no final (ex: "sudo npm i" -> "npm i", "i")
    const wordsWithIndices: { text: string; startIdx: number }[] = [];
    wordsWithIndices.push({ text: typedCommand, startIdx: trimmedCommandStart });

    // Encontra as palavras intermediárias para o caso do comando ser precedido por "sudo", "npx", etc.
    const spaceIndices: number[] = [];
    for (let i = 0; i < typedCommand.length; i++) {
      if (typedCommand[i] === " ") spaceIndices.push(i);
    }
    for (const spIdx of spaceIndices) {
      const sub = typedCommand.slice(spIdx + 1);
      if (sub.length >= 2) {
        wordsWithIndices.push({
          text: sub,
          startIdx: trimmedCommandStart + spIdx + 1,
        });
      }
    }

    let matchedItems: { alias: CustomAlias; prefix: string; priority: number }[] = [];

    for (const { text: candidateText, startIdx } of wordsWithIndices) {
      if (!candidateText || candidateText.length < 2) continue;
      const candidateLower = candidateText.toLowerCase();

      const items: { alias: CustomAlias; prefix: string; priority: number }[] = [];

      for (const a of availableAliases) {
        const aliasLower = a.alias.toLowerCase();

        // Se o alias já foi digitado por completo na íntegra, não precisa sugerir
        if (textLower.endsWith(aliasLower) && candidateLower === aliasLower) {
          continue;
        }

        // O alias tem que começar exatamente com o que foi digitado!
        // Se digitei "npm ", aliases como "npm run dev", "npm install", "npm start" continuam ativos!
        // Se digitei "npm inst", aliases como "npm install" permanecem, e os outros somem.
        if (aliasLower.startsWith(candidateLower)) {
          items.push({
            alias: a,
            prefix: textBeforeCursor.slice(startIdx),
            priority: candidateLower.length,
          });
        }
      }

      if (items.length > 0) {
        matchedItems = items;
        break; // Dá preferência ao casamento mais específico / mais longo
      }
    }

    if (matchedItems.length > 0) {
      // Ordena alfabeticamente
      matchedItems.sort((a, b) => a.alias.alias.localeCompare(b.alias.alias));

      const newSuggestions = matchedItems.map((m) => m.alias);
      suggestions = newSuggestions;
      matchedPrefix = matchedItems[0].prefix;

      // Mantém o índice selecionado dentro dos limites válidos
      if (selectedIndex >= newSuggestions.length) {
        selectedIndex = 0;
      }

      updatePosition();
      return;
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
    
    // Calcula altura aproximada para não cobrir a linha do cursor
    const estimatedHeight =
      suggestions.length > 1 ? Math.min(180, suggestions.length * 30 + 34) : 38;
    
    let posY = rect.top + cursorY * cellHeight - estimatedHeight - 6;
    if (posY < rect.top + 10) {
      posY = rect.top + (cursorY + 1.2) * cellHeight;
    }

    position = {
      x: Math.min(posX, Math.max(10, window.innerWidth - 380)),
      y: Math.max(10, posY),
    };
  }

  function next() {
    if (suggestions.length > 0) {
      selectedIndex = (selectedIndex + 1) % suggestions.length;
    }
  }

  function prev() {
    if (suggestions.length > 0) {
      selectedIndex =
        (selectedIndex - 1 + suggestions.length) % suggestions.length;
    }
  }

  function close() {
    suggestions = [];
    selectedIndex = 0;
    matchedPrefix = "";
  }

  function onUserInput() {
    lastAppliedCommand = "";
    if (justApplied) {
      justApplied = false;
      if (justAppliedTimer) {
        clearTimeout(justAppliedTimer);
        justAppliedTimer = null;
      }
    }
  }

  function apply(targetAlias?: CustomAlias) {
    const aliasToApply = targetAlias || activeSuggestion;
    if (!aliasToApply || isApplying) return;

    isApplying = true;
    justApplied = true;
    lastAppliedCommand = aliasToApply.command;
    if (justAppliedTimer) clearTimeout(justAppliedTimer);
    justAppliedTimer = setTimeout(() => {
      isApplying = false;
      justApplied = false;
    }, 1000);

    const term = getTerm();
    let wordToErase = matchedPrefix;

    if (term) {
      const buffer = term.buffer.active;
      const cursorY = buffer.cursorY;
      const lineObj = buffer.getLine(buffer.baseY + cursorY);
      if (lineObj) {
        const fullLine = lineObj.translateToString(true);
        const textBeforeCursor = fullLine.slice(0, buffer.cursorX);
        const aliasLower = aliasToApply.alias.toLowerCase();
        
        // Encontra exatamente quantos caracteres daquele alias estavam no final da linha
        for (let len = aliasLower.length; len >= 1; len--) {
          const candidate = aliasLower.slice(0, len);
          if (textBeforeCursor.toLowerCase().endsWith(candidate)) {
            wordToErase = textBeforeCursor.slice(textBeforeCursor.length - len);
            break;
          }
        }
      }
    }

    const eraseLength = Math.max(wordToErase.length, matchedPrefix.length);
    const backspaces = "\x7f".repeat(eraseLength);
    const commandToInsert = aliasToApply.command;

    // Fecha o popup imediatamente
    close();

    PtyService.writePty(getId(), backspaces + commandToInsert)
      .catch(console.error)
      .finally(() => {
        isApplying = false;
      });

    if (term) {
      term.focus();
      requestAnimationFrame(() => term.focus());
    }
  }

  return {
    get suggestions() {
      return suggestions;
    },
    get selectedIndex() {
      return selectedIndex;
    },
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
    next,
    prev,
    close,
    apply,
    onUserInput,
  };
}
