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
  let typedFull = $state("");
  let position = $state({ x: 100, y: 100 });
  let isOpen = $state(false);

  const PAGE_SIZE = 10;
  let visibleCount = PAGE_SIZE;
  let allFiltered: string[] = [];
  let lastQueryKey = "";

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

  async function fetchDirectoriesIfNeeded(relativePath?: string): Promise<string[]> {
    if (getType() === "ssh") {
      return [];
    }

    try {
      isFetching = true;
      const res = await PtyService.listSubdirectories(getId(), relativePath);
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

    const fullQuery = (cdMatch[1] || "").trimStart();
    typedFull = fullQuery.trim();

    // Separa "MEGA/sub/pa" em pasta pai ("MEGA/sub/") e o trecho final ("pa").
    // Digitar "/" (ou "\" no Windows) dispara a sugestão do próximo nível.
    const lastSep = Math.max(fullQuery.lastIndexOf("/"), fullQuery.lastIndexOf("\\"));
    const parentPath = lastSep >= 0 ? fullQuery.slice(0, lastSep + 1).replace(/["']/g, "") : "";
    const typedQuery = lastSep >= 0 ? fullQuery.slice(lastSep + 1) : fullQuery.trim();
    matchedQuery = typedQuery;

    // Busca diretórios (relativos à pasta pai, se houver)
    const allDirs = await fetchDirectoriesIfNeeded(parentPath || undefined);
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

    // Se o usuário já digitou algo (ex: "cd ~/" ou "cd ant"), a primeira opção (índice 0)
    // é sempre o que ele digitou (para que Enter execute direto se não apertar Tab),
    // e as pastas começam na segunda opção em diante.
    const hasTypedText = typedFull.length > 0;
    const finalList = hasTypedText ? [typedFull, ...filtered] : filtered;

    totalMatches = finalList.length;
    // Exibe 10 diretórios por vez; ao navegar até o último, carrega mais 10 (ver next())
    const queryKey = parentPath + "\u0000" + typedQuery;
    if (queryKey !== lastQueryKey) {
      lastQueryKey = queryKey;
      visibleCount = PAGE_SIZE;
      selectedIndex = 0;
    }
    allFiltered = finalList;
    directories = finalList.slice(0, visibleCount);

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
    allFiltered = [];
    visibleCount = PAGE_SIZE;
    lastQueryKey = "";
    totalMatches = 0;
    selectedIndex = 0;
    matchedQuery = "";
    typedFull = "";
  }

  function next() {
    if (directories.length === 0) return;
    const isLast = selectedIndex === directories.length - 1;
    if (isLast && directories.length < allFiltered.length) {
      // Chegou ao fim da página: carrega mais 10
      visibleCount += PAGE_SIZE;
      directories = allFiltered.slice(0, visibleCount);
      selectedIndex++;
    } else {
      selectedIndex = (selectedIndex + 1) % directories.length;
    }
  }

  function prev() {
    if (directories.length > 0) {
      selectedIndex =
        (selectedIndex - 1 + directories.length) % directories.length;
    }
  }

  function selectCurrent(): boolean {
    if (activeDirectory) {
      const isTypedOption = selectedIndex === 0 && typedFull.length > 0;
      apply(activeDirectory, isTypedOption);
      // Retorna true se o Enter foi consumido para autocompletar uma pasta (2ª opção em diante).
      // Retorna false se estava na 1ª opção (o que o usuário digitou), permitindo que o Enter execute o comando no terminal!
      return !isTypedOption;
    }
    return false;
  }

  function apply(dirName: string, isTypedOption = false) {
    if (!dirName) return;

    const term = getTerm();

    // Se selecionou a 1ª opção (exatamente o que já foi digitado), não precisa reescrever nada
    // ou executa o Enter direto se foi confirmado!
    // Caso clique ou aperte Enter na 1ª opção que é o próprio texto digitado:
    if (isTypedOption || (typedFull.length > 0 && dirName === typedFull && directories[0] === dirName)) {
      lastAppliedDir = dirName;
      close();
      if (term) {
        term.focus();
        requestAnimationFrame(() => term.focus());
      }
      return;
    }

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
    // Comparação case-sensitive: se o case digitado diferir do real (ex: "ca" vs "CaG"),
    // cai no else e reescreve o nome exatamente como está no sistema.
    if (!dirName.includes(" ") && matchedQuery && dirName.startsWith(matchedQuery)) {
      const remaining = dirName.slice(matchedQuery.length);
      PtyService.writePty(getId(), remaining).catch(console.error);
    } else {
      // Apaga apenas o trecho final digitado (após o último separador, ou após "cd ")
      let charsToErase = matchedQuery.length;
      const match = textBeforeCursor.match(/(?:^|[\s;&|❯$#>%\n\r])cd\s+(.*)$/i);
      if (match && match[1] !== undefined) {
        const typed = match[1];
        const sep = Math.max(typed.lastIndexOf("/"), typed.lastIndexOf("\\"));
        charsToErase = sep >= 0 ? typed.length - sep - 1 : typed.length;
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
    get typedFull() {
      return typedFull;
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
