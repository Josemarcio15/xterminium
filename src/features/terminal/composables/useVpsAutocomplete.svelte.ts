import { Terminal } from "@xterm/xterm";
import { type SshHost, type CustomCommand } from "../../../core/types";
import { PtyService } from "../../../core/services";
import { configStore } from "../../../core/stores/config.svelte";

export function useVpsAutocomplete(
  getId: () => string,
  getType: () => "local" | "ssh",
  getTerm: () => Terminal | null,
  getContainer: () => HTMLDivElement | null,
) {
  let availableSshHosts = $state<SshHost[]>([]);
  let availableCustomCommands = $state<CustomCommand[]>([]);
  let showDropdown = $state(false);
  let filteredHosts = $state<SshHost[]>([]);
  let selectedHostIndex = $state(0);
  let dropdownPosition = $state({ x: 100, y: 100 });
  let activeMatchedCommand = $state<CustomCommand | null>(null);
  let currentMatchedQuery = "";

  function trigger() {
    const term = getTerm();
    availableSshHosts = configStore.hosts;
    availableCustomCommands = configStore.commands;

    if (!availableSshHosts || availableSshHosts.length === 0 || !term) {
      return;
    }

    // Obtém a linha onde está o cursor
    const buffer = term.buffer.active;
    const cursorY = buffer.cursorY;
    const lineObj = buffer.getLine(buffer.baseY + cursorY);
    let textBeforeCursor = "";
    if (lineObj) {
      const fullLine = lineObj.translateToString(true);
      textBeforeCursor = fullLine.slice(0, buffer.cursorX);
    }

    // Identifica se algum dos comandos configurados está presente na linha
    let matchedCmd: CustomCommand | null = null;
    for (const cmd of availableCustomCommands) {
      const regex = new RegExp(`(?:^|[;&|\\s])${cmd.command}(?:\\s+|$)`, "i");
      if (regex.test(textBeforeCursor)) {
        matchedCmd = cmd;
        break;
      }
    }

    if (!matchedCmd && availableCustomCommands.length > 0) {
      matchedCmd = availableCustomCommands[0];
    }
    activeMatchedCommand = matchedCmd;

    // Prefixo digitado antes do cursor
    const match = textBeforeCursor.match(/([a-zA-Z0-9_\-\.]+)$/);
    const query = match ? match[1] : "";

    const knownCommands = availableCustomCommands.map((c) =>
      c.command.toLowerCase(),
    );
    const isCommandWord = knownCommands.includes(query.toLowerCase());
    const validQuery = isCommandWord || query.startsWith("-") ? "" : query;

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
      filteredHosts = [...availableSshHosts];
      currentMatchedQuery = "";
    }

    if (filteredHosts.length > 0) {
      selectedHostIndex = 0;
      updatePosition();
      showDropdown = true;
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

    dropdownPosition = {
      x: Math.min(posX, window.innerWidth - 300),
      y: finalY,
    };
  }

  function close() {
    showDropdown = false;
    filteredHosts = [];
    selectedHostIndex = 0;
    currentMatchedQuery = "";
  }

  function next() {
    if (filteredHosts.length > 0) {
      selectedHostIndex = (selectedHostIndex + 1) % filteredHosts.length;
    }
  }

  function prev() {
    if (filteredHosts.length > 0) {
      selectedHostIndex =
        (selectedHostIndex - 1 + filteredHosts.length) % filteredHosts.length;
    }
  }

  function selectCurrent() {
    const selected = filteredHosts[selectedHostIndex];
    if (selected) {
      apply(selected);
    }
  }

  function apply(host: SshHost) {
    if (!host) return;

    const term = getTerm();
    let textBeforeCursor = "";
    if (term) {
      const buffer = term.buffer.active;
      const cursorY = buffer.cursorY;
      const lineObj = buffer.getLine(buffer.baseY + cursorY);
      if (lineObj) {
        textBeforeCursor = lineObj
          .translateToString(true)
          .slice(0, buffer.cursorX);
      }
    }

    const backspaces = "\x7f".repeat(currentMatchedQuery.length);
    const port = host.port || "22";
    const key = host.key || "";
    const user = host.user || "";
    const ip = host.ip || "";
    const label = host.label || "";

    const formatString = (str?: string) => {
      if (!str) return "";
      return str
        .replace(/\{user\}/g, user)
        .replace(/\{ip\}/g, ip)
        .replace(/\{port\}/g, port)
        .replace(/\{key\}/g, key)
        .replace(/\{label\}/g, label);
    };

    let replacement = "";
    const alias = label ? label.trim().replace(/\s+/g, "_") : "";

    if (activeMatchedCommand) {
      const cmdName = (activeMatchedCommand.command || "").toLowerCase();
      const isSshBased =
        cmdName === "ssh" ||
        cmdName === "scp" ||
        cmdName === "rsync" ||
        cmdName === "sftp";

      // Verifica se o comando tem prefixArgs configurado (ex: '-c 4' para ping, ou '-avz' para rsync)
      // e verifica se o usuário já não digitou esses argumentos na linha antes do cursor
      let prefixToInsert = "";
      if (
        activeMatchedCommand.prefixArgs &&
        activeMatchedCommand.prefixArgs.trim()
      ) {
        const configuredPrefix = activeMatchedCommand.prefixArgs.trim();
        if (!textBeforeCursor.includes(configuredPrefix)) {
          prefixToInsert = `${configuredPrefix} `;
        }
      }

      if (alias && isSshBased) {
        // Objeto VPS: usa o alias direto configurado no ~/.ssh/config pelo ssh.json
        // Linha curta, limpa e com portas/chaves aplicadas de forma transparente!
        const suffixStr = formatString(activeMatchedCommand.suffixArgs);
        if (cmdName === "ssh") {
          replacement = `${prefixToInsert}${alias}`;
        } else if (cmdName === "scp" || cmdName === "rsync") {
          replacement = `${prefixToInsert}${alias}${suffixStr || ":~/"}`;
        } else if (cmdName === "sftp") {
          replacement = `${prefixToInsert}${alias}`;
        } else {
          replacement = `${prefixToInsert}${alias}${suffixStr}`;
        }
      } else {
        // Fallback para comando customizado que use template explícito (como ping {ip}) ou hosts sem alias
        const templateStr = formatString(activeMatchedCommand.template);
        const suffixStr = formatString(activeMatchedCommand.suffixArgs);

        const extraFlags: string[] = [];
        if (key && isSshBased) {
          extraFlags.push(`-i "${key}"`);
        }
        if (port && port !== "22") {
          if (cmdName === "ssh" || cmdName === "sftp") {
            extraFlags.push(`-p ${port}`);
          } else if (cmdName === "scp") {
            extraFlags.push(`-P ${port}`);
          } else if (cmdName === "rsync") {
            extraFlags.push(`-e "ssh -p ${port}"`);
          }
        }

        if (extraFlags.length > 0) {
          replacement = `${prefixToInsert}${extraFlags.join(" ")} ${templateStr}${suffixStr}`;
        } else {
          replacement = `${prefixToInsert}${templateStr}${suffixStr}`;
        }
      }
    } else {
      if (alias) {
        replacement = alias;
      } else if (port && port !== "22") {
        replacement = `-p ${port} ${user}@${ip}`;
      } else {
        replacement = `${user}@${ip}`;
      }
    }

    PtyService.writePty(getId(), backspaces + replacement).catch(console.error);

    close();
    if (term) {
      term.focus();
      requestAnimationFrame(() => term.focus());
    }
  }

  return {
    get showDropdown() {
      return showDropdown;
    },
    get filteredHosts() {
      return filteredHosts;
    },
    get selectedHostIndex() {
      return selectedHostIndex;
    },
    get dropdownPosition() {
      return dropdownPosition;
    },
    get activeMatchedCommand() {
      return activeMatchedCommand;
    },
    trigger,
    close,
    next,
    prev,
    selectCurrent,
    apply,
  };
}
