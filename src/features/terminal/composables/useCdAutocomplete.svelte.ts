import { Terminal } from "@xterm/xterm";
import { PtyService } from "../../../core/services";

export function useCdAutocomplete(
  getId: () => string,
  getType: () => "local" | "ssh",
  getTerm: () => Terminal | null,
  getContainer: () => HTMLDivElement | null,
) {
  let directories = $state<string[]>([]);
  let totalMatches = $state(0);
  let selectedIndex = $state(0);
  let matchedQuery = $state("");
  let position = $state({ x: 100, y: 100 });
  let isOpen = $state(false);

  // Cache para evitar spam de chamadas assíncronas ao Rust
  let cachedCwd = "";
  let cachedDirs: string[] = [];
  let isFetching = false;
  let lastAppliedDir = "";

  const activeDirectory = $derived(
    directories.length > 0 && selectedIndex < directories.length
      ? directories[selectedIndex]
      : null
  );

  async function fetchDirectoriesIfNeeded(): Promise<string[]> {
    if (getType() === "ssh") {
      return [];
    }

    try {
      isFetching = true;
      const res = await PtyService.listSubdirectories(getId());
      cachedDirs = res || [];
      return cachedDirs;
    } catch (err) {
      console.error("[CD Autocomplete] Erro ao listar subdiretórios via Tauri:", err);
      return [];
    } finally {
      isFetching = false;
    }
  }

  async function check(isOtherDropdownOpen: boolean): Promise<boolean> {
    const term = getTerm();
    if (!term || isOtherDropdownOpen) {
      close();
      return false;
    }

    if (getType() === "ssh") {
      close();
      return false;
    }

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
      return false;
    }

    // Se acabou de aplicar este diretório, aguarda nova interação
    if (lastAppliedDir && textBeforeCursor.endsWith(lastAppliedDir)) {
      close();
      return false;
    }

    // Encontra o comando 'cd': suporta prompts tradicionais (>, $, #) e prompts modernos (Oh-My-Posh / Powerlevel10k / Starship)
    // Ex: " Marcio  ~  git  cd " ou "C:\git> cd mydir"
    const cdMatch = textBeforeCursor.match(/(?:^|[\s;&|❯$#>%\n\r])cd(?:\s+(.*))?$/i);
    if (!cdMatch) {
      close();
      return false;
    }

    const typedQuery = (cdMatch[1] || "").trim();
    matchedQuery = typedQuery;

    // Busca diretórios
    const allDirs = await fetchDirectoriesIfNeeded();
    console.log("[CD Autocomplete] Matched CD!", { textBeforeCursor, typedQuery, allDirsCount: allDirs.length });
    if (!allDirs || allDirs.length === 0) {
      close();
      return true; // é cd, mas não tem pastas para listar
    }

    let filtered = allDirs;
    if (typedQuery.length > 0) {
      const qLower = typedQuery.toLowerCase();
      filtered = allDirs.filter((d) => d.toLowerCase().includes(qLower));
    }

    if (filtered.length === 0) {
      close();
      return true;
    }

    totalMatches = filtered.length;
    // Exibe no máximo 10 diretórios por vez
    directories = filtered.slice(0, 10);

    if (selectedIndex >= directories.length) {
      selectedIndex = 0;
    }

    updatePosition();
    isOpen = true;
    return true;
  }

  function updatePosition() {
    const container = getContainer();
    const term = getTerm();
    if (!container || !term) return;

    const rect = container.getBoundingClientRect();
    const core = (term as any)._core;
    const cellWidth = core?._renderService?.dimensions?.css?.cell?.width || 9;
    const cellHeight = core?._renderService?.dimensions?.css?.cell?.height || 17;

    const cursorX = term.buffer.active.cursorX;
    const cursorY = term.buffer.active.cursorY;

    const posX = rect.left + cursorX * cellWidth;
    const posY = rect.top + (cursorY + 1.2) * cellHeight;
    const finalY =
      posY + 240 > window.innerHeight ? Math.max(10, posY - 250) : posY;

    position = {
      x: Math.min(posX, Math.max(10, window.innerWidth - 380)),
      y: finalY,
    };
  }

  function close() {
    isOpen = false;
    directories = [];
    totalMatches = 0;
    selectedIndex = 0;
    matchedQuery = "";
  }

  function next() {
    if (directories.length > 0) {
      selectedIndex = (selectedIndex + 1) % directories.length;
    }
  }

  function prev() {
    if (directories.length > 0) {
      selectedIndex =
        (selectedIndex - 1 + directories.length) % directories.length;
    }
  }

  function selectCurrent() {
    if (activeDirectory) {
      apply(activeDirectory);
    }
  }

  function apply(dirName: string) {
    if (!dirName) return;

    const term = getTerm();
    let textBeforeCursor = "";
    if (term) {
      const buffer = term.buffer.active;
      const cursorY = buffer.cursorY;
      const lineObj = buffer.getLine(buffer.baseY + cursorY);
      if (lineObj) {
        textBeforeCursor = lineObj.translateToString(true).slice(0, buffer.cursorX);
      }
    }

    // Se o nome da pasta tiver espaços, envolve em aspas
    const formattedDir = dirName.includes(" ") ? `"${dirName}"` : dirName;

    // Se a pasta começar exatamente com o que já foi digitado (sem aspas necessárias):
    // basta inserir apenas o restante (remaining) sem precisar dar backspace!
    if (!dirName.includes(" ") && matchedQuery && dirName.toLowerCase().startsWith(matchedQuery.toLowerCase())) {
      const remaining = dirName.slice(matchedQuery.length);
      PtyService.writePty(getId(), remaining).catch(console.error);
    } else {
      // Se precisar de aspas ou ajuste completo, apaga exatamente o que foi digitado após "cd "
      let charsToErase = matchedQuery.length;
      const match = textBeforeCursor.match(/(?:^|[\s;&|❯$#>%\n\r])cd\s+(.*)$/i);
      if (match && match[1] !== undefined) {
        charsToErase = match[1].length;
      }
      const backspaces = "\x7f".repeat(charsToErase);
      PtyService.writePty(getId(), backspaces + formattedDir).catch(console.error);
    }

    lastAppliedDir = formattedDir;
    close();

    if (term) {
      term.focus();
      requestAnimationFrame(() => term.focus());
    }
  }

  return {
    get directories() {
      return directories;
    },
    get selectedIndex() {
      return selectedIndex;
    },
    get matchedQuery() {
      return matchedQuery;
    },
    get totalMatches() {
      return totalMatches;
    },
    get position() {
      return position;
    },
    get isOpen() {
      return isOpen;
    },
    get activeDirectory() {
      return activeDirectory;
    },
    check,
    close,
    next,
    prev,
    selectCurrent,
    apply,
  };
}
