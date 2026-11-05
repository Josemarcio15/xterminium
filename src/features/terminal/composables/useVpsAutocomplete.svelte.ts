import { Terminal } from "@xterm/xterm";
import { PtyService } from "../../../core/services";
import { configStore } from "../../../core/stores/config.svelte";
import type { PipelineItem } from "../components/VpsPipelineDropdown.svelte";

export function useVpsAutocomplete(
  getId: () => string,
  getType: () => "local" | "ssh",
  getTerm: () => Terminal | null,
  getContainer: () => HTMLDivElement | null,
) {
  let showDropdown = $state(false);
  let activeStep = $state<"arg1" | "vps" | "arg2">("arg1");
  let items = $state<PipelineItem[]>([]);
  let selectedIndex = $state(0);
  let dropdownPosition = $state({ x: 100, y: 100 });
  let activeTrigger = $state("");
  let stepTitle = $state("");
  let stepColor = $state<"amber" | "purple" | "emerald">("amber");
  let eraseLength = 0;
  let lastAppliedSuffix = "";

  function close() {
    // Se estava na etapa de Args 1 e apertou Esc, avança diretamente para a etapa da VPS!
    if (showDropdown && activeStep === "arg1") {
      openVpsStep();
      return;
    }

    showDropdown = false;
    items = [];
    selectedIndex = 0;
    activeTrigger = "";
  }

  function openVpsStep() {
    const hosts = configStore.hosts || [];
    if (hosts.length === 0) {
      showDropdown = false;
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
      value: (h.label ? h.label.trim().replace(/\s+/g, "_") : `${h.user}@${h.ip}`),
      type: "vps" as const,
      raw: h,
    }));
    selectedIndex = 0;
    updatePosition();
    showDropdown = true;
  }

  function openSuffixStep() {
    const suffixArgs = configStore.vpsPipeline?.suffixArgs || [];
    if (suffixArgs.length === 0) {
      showDropdown = false;
      return;
    }
    activeStep = "arg2";
    stepTitle = `Args 2 (Pós-VPS / Destino)`;
    stepColor = "emerald";
    eraseLength = 0;

    items = suffixArgs.map((suf) => ({
      id: `arg2-${suf}`,
      label: suf,
      sublabel: "Caminho remoto pós-VPS",
      value: suf,
      type: "arg2" as const,
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
      posY + 220 > window.innerHeight ? Math.max(10, posY - 240) : posY;

    dropdownPosition = {
      x: Math.min(posX, Math.max(10, window.innerWidth - 380)),
      y: finalY,
    };
  }

  // Avaliação automática chamada em tempo real durante a digitação
  function check(isOtherOpen: boolean) {
    const term = getTerm();
    if (!term || isOtherOpen) {
      if (showDropdown) close();
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
      if (showDropdown) close();
      return;
    }

    // Comandos gatilho configurados (ex: 'ssh', 'sshfs', 'rsync', 'scp', 'ping')
    const triggers = [
      ...(configStore.vpsPipeline?.triggers || []),
      ...(configStore.commands || []).map((c) => c.command),
    ];

    // Procura se a linha começa ou contém um dos gatilhos seguido de espaço: ex: "ssh ", "rsync -avz "
    const match = textBeforeCursor.match(/(?:^|[\s;&|❯$#>%\n\r])([a-zA-Z0-9_\-]+)\s+(.*)$/);
    if (!match) {
      if (showDropdown) close();
      return;
    }

    const typedCmd = match[1].toLowerCase();
    if (!triggers.includes(typedCmd)) {
      if (showDropdown) close();
      return;
    }

    activeTrigger = typedCmd;
    const afterCmd = match[2]; // o que foi digitado após o comando e o espaço

    // Identifica tokens após o comando
    const tokens = afterCmd.trim().split(/\s+/).filter(Boolean);
    const lastWord = afterCmd.endsWith(" ") ? "" : (tokens[tokens.length - 1] || "");

    const prefixArgs = configStore.vpsPipeline?.prefixArgs || [];
    const suffixArgs = configStore.vpsPipeline?.suffixArgs || [];
    const hosts = configStore.hosts || [];

    // FASE 1: O usuário acabou de digitar o comando e está no primeiro argumento (ex: "ssh " ou "ssh -")
    // Se digitou uma flag (ex: "-p 8080") OU se apertou Esc para pular, passamos para a Fase 2 (VPS)
    const isAtFirstArg = tokens.length === 0 || (tokens.length === 1 && !afterCmd.endsWith(" ") && lastWord.startsWith("-"));

    // Se houver prefixArgs configurados e estiver no primeiro argumento e NÃO tiver pulado explicitamente com Esc
    if (prefixArgs.length > 0 && isAtFirstArg && activeStep !== "vps") {
      activeStep = "arg1";
      stepTitle = `Args 1 (Pré-VPS) para ${typedCmd.toUpperCase()}`;
      stepColor = "amber";
      eraseLength = lastWord.length;

      const q = lastWord.toLowerCase();
      const filteredPrefix = prefixArgs.filter((a) => a.toLowerCase().includes(q));

      if (filteredPrefix.length > 0) {
        items = filteredPrefix.map((arg) => ({
          id: `arg1-${arg}`,
          label: arg,
          sublabel: "Argumento pré-vps",
          value: `${arg} `,
          type: "arg1" as const,
        }));
        if (selectedIndex >= items.length) selectedIndex = 0;
        updatePosition();
        showDropdown = true;
        return;
      }
    }

    // FASE 2: Busca da VPS
    // Acontece se:
    // 1) O usuário apertou Esc na fase 1 (activeStep === "vps")
    // 2) O usuário já digitou/escolheu uma flag (ex: "ssh -p 8080 ")
    // 3) Não há prefixArgs cadastrados
    // 4) O usuário começou a digitar o nome da VPS (não começa com "-")
    const hasHostSelected = hosts.some((h) => {
      const alias = (h.label || "").trim().toLowerCase();
      const ip = h.ip.toLowerCase();
      return tokens.some((t) => t.toLowerCase() === alias || t.toLowerCase() === ip || t.toLowerCase().includes(ip));
    });

    if (!hasHostSelected) {
      activeStep = "vps";
      stepTitle = `Selecionar VPS para ${typedCmd.toUpperCase()}`;
      stepColor = "purple";
      eraseLength = lastWord.length;

      const q = lastWord.toLowerCase();
      const filteredHosts = hosts.filter((h) => {
        const lMatch = h.label && h.label.toLowerCase().includes(q);
        const ipMatch = h.ip.toLowerCase().includes(q);
        const uMatch = h.user.toLowerCase().includes(q);
        return lMatch || ipMatch || uMatch;
      });

      if (filteredHosts.length > 0) {
        items = filteredHosts.map((h) => ({
          id: `vps-${h.id}`,
          label: h.label || h.ip,
          sublabel: `${h.user}@${h.ip}${h.port && h.port !== "22" ? ` :${h.port}` : ""}`,
          value: (h.label ? h.label.trim().replace(/\s+/g, "_") : `${h.user}@${h.ip}`),
          type: "vps" as const,
          raw: h,
        }));
        if (selectedIndex >= items.length) selectedIndex = 0;
        updatePosition();
        showDropdown = true;
        return;
      }
    }

    // FASE 3: Se a VPS já foi inserida de verdade na linha, sugere Args 2 logo após a VPS
    if (hasHostSelected && suffixArgs.length > 0) {
      // Se a linha termina exatamente com o sufixo que acabou de ser aplicado, NÃO sugere novamente
      if (lastAppliedSuffix && textBeforeCursor.endsWith(lastAppliedSuffix)) {
        close();
        return;
      }

      // Se a linha não termina mais com o sufixo aplicado (o usuário apagou), reseta para permitir sugerir de novo
      if (lastAppliedSuffix && !textBeforeCursor.endsWith(lastAppliedSuffix)) {
        lastAppliedSuffix = "";
      }

      // Procura se tem algum sufixo já digitado por completo na linha
      const alreadyHasSuffix = suffixArgs.some((suf) => textBeforeCursor.endsWith(suf));
      if (alreadyHasSuffix) {
        close();
        return;
      }

      // Encontra a VPS que está na linha
      const matchedHost = hosts.find((h) => {
        const alias = (h.label || "").trim().toLowerCase();
        const ip = h.ip.toLowerCase();
        return tokens.some((t) => t.toLowerCase() === alias || t.toLowerCase().startsWith(alias) || t.toLowerCase() === ip || t.toLowerCase().startsWith(ip));
      });

      // Calcula o que está sendo digitado logo após o identificador da VPS
      // Exemplo: se a VPS é "local", e a linha é "rsync local:", o fragmento digitado após a VPS é ":"
      let queryAfterVps = "";
      if (matchedHost) {
        const alias = (matchedHost.label || "").trim();
        const hostToken = alias || `${matchedHost.user}@${matchedHost.ip}`;
        const idx = textBeforeCursor.lastIndexOf(hostToken);
        if (idx !== -1) {
          queryAfterVps = textBeforeCursor.slice(idx + hostToken.length);
        }
      }

      const q = queryAfterVps.toLowerCase();
      const filteredSuffix = suffixArgs.filter((s) => s.toLowerCase().startsWith(q) || s.toLowerCase().includes(q));

      if (filteredSuffix.length > 0) {
        activeStep = "arg2";
        stepTitle = `Args 2 (Pós-VPS / Destino)`;
        stepColor = "emerald";
        eraseLength = queryAfterVps.length;

        items = filteredSuffix.map((suf) => ({
          id: `arg2-${suf}`,
          label: suf,
          sublabel: "Caminho remoto pós-VPS",
          value: suf,
          type: "arg2" as const,
        }));
        if (selectedIndex >= items.length) selectedIndex = 0;
        updatePosition();
        showDropdown = true;
        return;
      }
    }

    close();
  }

  function next() {
    if (items.length > 0) {
      selectedIndex = (selectedIndex + 1) % items.length;
    }
  }

  function prev() {
    if (items.length > 0) {
      selectedIndex = (selectedIndex - 1 + items.length) % items.length;
    }
  }

  function selectCurrent() {
    const selected = items[selectedIndex];
    if (selected) {
      apply(selected);
    }
  }

  function apply(item: PipelineItem) {
    const term = getTerm();
    const backspaces = "\x7f".repeat(eraseLength);
    const toInsert = item.value;

    PtyService.writePty(getId(), backspaces + toInsert).catch(console.error);

    const prevStep = activeStep;
    if (prevStep === "arg2") {
      lastAppliedSuffix = toInsert;
    }

    showDropdown = false;
    items = [];
    selectedIndex = 0;

    if (term) {
      term.focus();
      requestAnimationFrame(() => term.focus());
    }

    // Transição automática para a próxima etapa:
    // Se selecionou Arg 1 -> abre a lista de VPS
    if (prevStep === "arg1") {
      setTimeout(() => openVpsStep(), 30);
    }
    // Se selecionou a VPS -> abre a lista de Args 2 (se houver cadastrados)
    else if (prevStep === "vps") {
      const suffixArgs = configStore.vpsPipeline?.suffixArgs || [];
      if (suffixArgs.length > 0) {
        setTimeout(() => openSuffixStep(), 30);
      }
    }
  }

  // Mantido para compatibilidade com atalho manual
  function trigger() {
    check(false);
  }

  return {
    get showDropdown() {
      return showDropdown;
    },
    get activeStep() {
      return activeStep;
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
    check,
    trigger,
    close,
    next,
    prev,
    selectCurrent,
    apply,
  };
}
