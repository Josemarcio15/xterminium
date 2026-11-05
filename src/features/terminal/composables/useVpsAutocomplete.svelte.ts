import { Terminal } from "@xterm/xterm";
import { PtyService } from "../../../core/services";
import { configStore } from "../../../core/stores/config.svelte";
import type { PipelineItem } from "../components/VpsPipelineDropdown.svelte";
import type { PipelineArgOption } from "../components/ArgsDualColumnModal.svelte";

export function useVpsAutocomplete(
  getId: () => string,
  getType: () => "local" | "ssh",
  getTerm: () => Terminal | null,
  getContainer: () => HTMLDivElement | null,
) {
  // Modal de Colunas Duplas (Args 1 & Args 2)
  let showDualModal = $state(false);
  let activeColumn = $state<"arg1" | "arg2">("arg1");
  let arg1List = $state<PipelineArgOption[]>([]);
  let arg2List = $state<PipelineArgOption[]>([]);
  let selectedArg1Index = $state(0);
  let selectedArg2Index = $state(0);

  // Dropdown da VPS
  let showDropdown = $state(false);
  let activeStep = $state<"dual" | "vps">("dual");
  let items = $state<PipelineItem[]>([]);
  let selectedIndex = $state(0);
  let dropdownPosition = $state({ x: 100, y: 100 });
  let activeTrigger = $state("");
  let stepTitle = $state("");
  let stepColor = $state<"amber" | "purple" | "emerald">("purple");
  let eraseLength = 0;

  // Guarda o sufixo escolhido (Args 2) para ser anexado após a VPS
  let pendingSuffix = "";
  let dismissedTrigger = "";

  function close() {
    showDualModal = false;
    showDropdown = false;
    items = [];
    selectedIndex = 0;
    activeTrigger = "";
  }

  function dismiss() {
    dismissedTrigger = activeTrigger;
    close();
  }

  function openVpsStep() {
    showDualModal = false;
    const hosts = configStore.hosts || [];
    if (hosts.length === 0) {
      close();
      return;
    }
    activeStep = "vps";
    stepTitle = `Selecionar VPS para ${activeTrigger.toUpperCase() || "COMANDO"}`;
    stepColor = "purple";
    eraseLength = 0;

    items = hosts.map((h) => ({
      id: `vps-${h.id}`,
      label: h.label || h.ip,
      sublabel: `${h.user}@${h.ip}${h.port && h.port !== "22" ? ` :${h.port}` : ""}`,
      value: h.label ? h.label.trim().replace(/\s+/g, "_") : `${h.user}@${h.ip}`,
      type: "vps" as const,
      raw: h,
    }));
    selectedIndex = 0;
    updatePosition();
    showDropdown = true;
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
      posY + 260 > window.innerHeight ? Math.max(10, posY - 280) : posY;

    dropdownPosition = {
      x: Math.min(posX, Math.max(10, window.innerWidth - 500)),
      y: finalY,
    };
  }

  function check(isOtherOpen: boolean) {
    const term = getTerm();
    if (!term || isOtherOpen) {
      if (showDualModal || showDropdown) close();
      return;
    }

    const buffer = term.buffer.active;
    const cursorY = buffer.cursorY;
    const lineObj = buffer.getLine(buffer.baseY + cursorY);
    let textBeforeCursor = "";
    if (lineObj) {
      textBeforeCursor = lineObj.translateToString(true).slice(0, buffer.cursorX);
    }

    if (!textBeforeCursor || textBeforeCursor.trim().length === 0) {
      dismissedTrigger = "";
      if (showDualModal || showDropdown) close();
      return;
    }

    // Comandos gatilho configurados (ex: 'ssh', 'sshfs', 'rsync', 'scp', 'ping')
    const triggers = [
      ...(configStore.vpsPipeline?.triggers || []),
      ...(configStore.commands || []).map((c) => c.command),
    ];

    // Procura se a linha começa ou contém um dos gatilhos seguido de espaço: ex: "ssh ", "rsync "
    const match = textBeforeCursor.match(/(?:^|[\s;&|❯$#>%\n\r])([a-zA-Z0-9_\-]+)\s+(.*)$/);
    if (!match) {
      dismissedTrigger = "";
      if (showDualModal || showDropdown) close();
      return;
    }

    const typedCmd = match[1].toLowerCase();
    if (!triggers.includes(typedCmd)) {
      dismissedTrigger = "";
      if (showDualModal || showDropdown) close();
      return;
    }

    // Se o usuário já dispensou para esse comando na linha atual
    if (dismissedTrigger === typedCmd) {
      if (showDualModal || showDropdown) close();
      return;
    }

    activeTrigger = typedCmd;
    const afterCmd = match[2]; // o que foi digitado após o comando e o espaço

    const tokens = afterCmd.trim().split(/\s+/).filter(Boolean);
    const hosts = configStore.hosts || [];

    // Se já foi digitada uma VPS completa na linha, não precisa de modal
    const hasHostSelected = hosts.some((h) => {
      const alias = (h.label || "").trim().toLowerCase();
      const ip = h.ip.toLowerCase();
      return tokens.some((t) => t.toLowerCase() === alias || t.toLowerCase() === ip || t.toLowerCase().includes(ip));
    });

    if (hasHostSelected) {
      if (showDualModal || showDropdown) close();
      return;
    }

    // Se ainda estamos no início após o comando (apenas deu espaço ou começou a digitar flags)
    const isJustAfterCommand = tokens.length === 0 || (tokens.length === 1 && !afterCmd.endsWith(" ") && afterCmd.startsWith("-"));

    if (isJustAfterCommand && !showDropdown) {
      // Carrega listas de Args 1 e Args 2
      const prefixArgs = configStore.vpsPipeline?.prefixArgs || [];
      const suffixArgs = configStore.vpsPipeline?.suffixArgs || [];

      // Monta lista de Arg 1
      arg1List = [
        { label: "Nenhum (padrão)", value: "" },
        ...prefixArgs.map((p) => ({ label: p, value: p })),
      ];

      // Monta lista de Arg 2
      arg2List = [
        { label: "Nenhum (padrão)", value: "" },
        ...suffixArgs.map((s) => ({ label: s, value: s })),
      ];

      updatePosition();
      showDualModal = true;
      activeStep = "dual";
      return;
    }
  }

  // Navegação no Modal de Colunas Duplas (Args 1 & Args 2)
  function dualNext() {
    if (activeColumn === "arg1") {
      selectedArg1Index = (selectedArg1Index + 1) % arg1List.length;
    } else {
      selectedArg2Index = (selectedArg2Index + 1) % arg2List.length;
    }
  }

  function dualPrev() {
    if (activeColumn === "arg1") {
      selectedArg1Index = (selectedArg1Index - 1 + arg1List.length) % arg1List.length;
    } else {
      selectedArg2Index = (selectedArg2Index - 1 + arg2List.length) % arg2List.length;
    }
  }

  function dualToggleColumn() {
    activeColumn = activeColumn === "arg1" ? "arg2" : "arg1";
  }

  function dualConfirm() {
    const chosenArg1 = arg1List[selectedArg1Index]?.value || "";
    const chosenArg2 = arg2List[selectedArg2Index]?.value || "";

    pendingSuffix = chosenArg2;

    // Se selecionou Arg 1, insere no terminal após o comando
    if (chosenArg1) {
      PtyService.writePty(getId(), `${chosenArg1} `).catch(console.error);
    }

    showDualModal = false;
    setTimeout(() => openVpsStep(), 30);
  }

  // Navegação no Dropdown de VPS
  function next() {
    if (showDualModal) {
      dualNext();
    } else if (items.length > 0) {
      selectedIndex = (selectedIndex + 1) % items.length;
    }
  }

  function prev() {
    if (showDualModal) {
      dualPrev();
    } else if (items.length > 0) {
      selectedIndex = (selectedIndex - 1 + items.length) % items.length;
    }
  }

  function selectCurrent() {
    if (showDualModal) {
      dualConfirm();
    } else {
      const selected = items[selectedIndex];
      if (selected) {
        apply(selected);
      }
    }
  }

  function apply(item: PipelineItem) {
    const term = getTerm();
    let toInsert = item.value;

    // Se havia um Args 2 selecionado (ex: ':~/')
    if (pendingSuffix) {
      if (pendingSuffix.startsWith(":")) {
        toInsert = `${toInsert}${pendingSuffix}`;
      } else {
        toInsert = `${toInsert} ${pendingSuffix}`;
      }
      pendingSuffix = "";
    }

    PtyService.writePty(getId(), toInsert).catch(console.error);

    showDropdown = false;
    items = [];
    selectedIndex = 0;

    if (term) {
      term.focus();
      requestAnimationFrame(() => term.focus());
    }
  }

  function trigger() {
    check(false);
  }

  return {
    get showDualModal() {
      return showDualModal;
    },
    get showDropdown() {
      return showDropdown;
    },
    get activeColumn() {
      return activeColumn;
    },
    get arg1List() {
      return arg1List;
    },
    get arg2List() {
      return arg2List;
    },
    get selectedArg1Index() {
      return selectedArg1Index;
    },
    get selectedArg2Index() {
      return selectedArg2Index;
    },
    get items() {
      return items;
    },
    get selectedIndex() {
      return selectedIndex;
    },
    get dropdownPosition() {
      return dropdownPosition;
    },
    get stepTitle() {
      return stepTitle;
    },
    get stepColor() {
      return stepColor;
    },
    get activeTrigger() {
      return activeTrigger;
    },
    check,
    trigger,
    close,
    dismiss,
    next,
    prev,
    dualToggleColumn,
    dualConfirm,
    openVpsStep,
    selectCurrent,
    apply,
    setColumn: (col: "arg1" | "arg2") => (activeColumn = col),
    setArg1Index: (idx: number) => (selectedArg1Index = idx),
    setArg2Index: (idx: number) => (selectedArg2Index = idx),
  };
}
