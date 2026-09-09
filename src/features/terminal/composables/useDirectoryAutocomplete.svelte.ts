import { Terminal } from "@xterm/xterm";
import { type SavedPath } from "../../../core/types";
import { PtyService } from "../../../core/services";
import { configStore } from "../../../core/stores/config.svelte";

export function useDirectoryAutocomplete(
  getId: () => string,
  getType: () => "local" | "ssh",
  getTerm: () => Terminal | null,
  getContainer: () => HTMLDivElement | null,
) {
  let availablePaths = $state<SavedPath[]>([]);
  let showDirDropdown = $state(false);
  let filteredPaths = $state<SavedPath[]>([]);
  let selectedDirIndex = $state(0);
  let dirDropdownPosition = $state({ x: 100, y: 100 });
  let currentDirMatchedQuery = "";

  function trigger() {
    if (getType() !== "local") return;

    const term = getTerm();
    availablePaths = configStore.paths;

    if (!availablePaths || availablePaths.length === 0 || !term) return;

    const buffer = term.buffer.active;
    const cursorY = buffer.cursorY;
    const lineObj = buffer.getLine(buffer.baseY + cursorY);
    let textBeforeCursor = "";
    if (lineObj) {
      const fullLine = lineObj.translateToString(true);
      textBeforeCursor = fullLine.slice(0, buffer.cursorX);
    }

    const match = textBeforeCursor.match(/([^\s]+)$/);
    const query = match ? match[1] : "";

    if (query.length > 0) {
      const q = query.toLowerCase();
      filteredPaths = availablePaths.filter(
        (p) =>
          p.name.toLowerCase().includes(q) || p.path.toLowerCase().includes(q),
      );
      currentDirMatchedQuery = query;
    } else {
      filteredPaths = [...availablePaths];
      currentDirMatchedQuery = "";
    }

    if (filteredPaths.length > 0) {
      selectedDirIndex = 0;
      updatePosition();
      showDirDropdown = true;
    } else {
      close();
    }
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
    const posY = rect.top + (cursorY + 1.2) * cellHeight;
    const finalY =
      posY + 200 > window.innerHeight ? Math.max(10, posY - 220) : posY;

    dirDropdownPosition = {
      x: Math.min(posX, window.innerWidth - 340),
      y: finalY,
    };
  }

  function close() {
    showDirDropdown = false;
    filteredPaths = [];
    selectedDirIndex = 0;
    currentDirMatchedQuery = "";
  }

  function next() {
    if (filteredPaths.length > 0) {
      selectedDirIndex = (selectedDirIndex + 1) % filteredPaths.length;
    }
  }

  function prev() {
    if (filteredPaths.length > 0) {
      selectedDirIndex =
        (selectedDirIndex - 1 + filteredPaths.length) % filteredPaths.length;
    }
  }

  function selectCurrent() {
    const selected = filteredPaths[selectedDirIndex];
    if (selected) {
      apply(selected);
    }
  }

  function apply(savedPath: SavedPath) {
    if (!savedPath) return;

    const backspaces = "\x7f".repeat(currentDirMatchedQuery.length);
    PtyService.writePty(getId(), backspaces + savedPath.path).catch(console.error);

    close();
    const term = getTerm();
    if (term) {
      term.focus();
      requestAnimationFrame(() => term.focus());
    }
  }

  return {
    get showDirDropdown() {
      return showDirDropdown;
    },
    get filteredPaths() {
      return filteredPaths;
    },
    get selectedDirIndex() {
      return selectedDirIndex;
    },
    get dirDropdownPosition() {
      return dirDropdownPosition;
    },
    trigger,
    close,
    next,
    prev,
    selectCurrent,
    apply,
  };
}
