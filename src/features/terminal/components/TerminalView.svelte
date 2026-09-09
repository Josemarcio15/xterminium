<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Terminal } from "@xterm/xterm";
  import { FitAddon } from "@xterm/addon-fit";
  import "@xterm/xterm/css/xterm.css";
  import { type SshHost } from "../../../core/types";
  import { PtyService } from "../../../core/services";
  import { configStore } from "../../../core/stores/config.svelte";
  import SshAutocompleteDropdown from "./SshAutocompleteDropdown.svelte";
  import DirectoryAutocompleteDropdown from "./DirectoryAutocompleteDropdown.svelte";
  import AliasSuggestionPopup from "./AliasSuggestionPopup.svelte";
  import { useAliasSuggestion } from "../composables/useAliasSuggestion.svelte";
  import { useVpsAutocomplete } from "../composables/useVpsAutocomplete.svelte";
  import { useDirectoryAutocomplete } from "../composables/useDirectoryAutocomplete.svelte";
  import { createTerminalKeyHandler } from "../composables/useTerminalShortcuts";

  interface Props {
    id: string;
    type: "local" | "ssh";
    sshInfo?: SshHost;
    active: boolean;
    onNewTab: () => void;
  }

  let { id, type, sshInfo, active, onNewTab }: Props = $props();

  let container = $state<HTMLDivElement | null>(null);
  let term: Terminal | null = null;
  let fitAddon: FitAddon | null = null;

  // Composables modulares de Autocomplete e Sugestões
  const aliasSug = useAliasSuggestion(
    () => id,
    () => term,
    () => container,
  );

  const vpsAuto = useVpsAutocomplete(
    () => id,
    () => type,
    () => term,
    () => container,
  );

  const dirAuto = useDirectoryAutocomplete(
    () => id,
    () => type,
    () => term,
    () => container,
  );

  onMount(async () => {
    if (!container) return;

    term = new Terminal({
      allowTransparency: true,
      cursorBlink: true,
      fontFamily: '"JetBrains Mono", "Fira Code", monospace',
      fontSize: 14,
      lineHeight: 1.2,
      theme: {
        background: configStore.theme.terminalBg,
        foreground: configStore.theme.terminalFg,
        cursor:
          type === "ssh"
            ? configStore.theme.terminalCursorSsh
            : configStore.theme.terminalCursorLocal,
        selectionBackground: configStore.theme.terminalSelection,
      },
    });

    fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.open(container);
    fitAddon.fit();

    // Interceptador modular de atalhos de teclado
    const keyHandler = createTerminalKeyHandler(term, id, {
      onNewTab,
      hasActiveAlias: () => !!aliasSug.activeSuggestion && !!aliasSug.matchedPrefix,
      applyAlias: () => aliasSug.apply(),
      closeAlias: () => aliasSug.close(),

      hasActiveVps: () => vpsAuto.showDropdown && vpsAuto.filteredHosts.length > 0,
      onVpsNext: () => vpsAuto.next(),
      onVpsPrev: () => vpsAuto.prev(),
      onVpsSelect: () => vpsAuto.selectCurrent(),
      onVpsClose: () => vpsAuto.close(),
      triggerVpsManual: () => vpsAuto.trigger(),

      hasActiveDir: () => dirAuto.showDirDropdown && dirAuto.filteredPaths.length > 0,
      onDirNext: () => dirAuto.next(),
      onDirPrev: () => dirAuto.prev(),
      onDirSelect: () => dirAuto.selectCurrent(),
      onDirClose: () => dirAuto.close(),
      triggerDirManual: () => dirAuto.trigger(),
    });

    term.attachCustomKeyEventHandler(keyHandler);

    // Inicia PTY
    if (type === "ssh" && sshInfo) {
      const portArg =
        sshInfo.port && sshInfo.port !== "22" ? ["-p", sshInfo.port] : [];
      const keyArg = sshInfo.key ? ["-i", sshInfo.key] : [];
      PtyService.spawnPty({
        id,
        cols: term.cols,
        rows: term.rows,
        command: "ssh",
        args: [...keyArg, ...portArg, `${sshInfo.user}@${sshInfo.ip}`],
      }).catch(console.error);
    } else {
      PtyService.spawnPty({
        id,
        cols: term.cols,
        rows: term.rows,
      }).catch(console.error);
    }

    // Fluxo de dados e digitação
    term.onData((data) => {
      // Ignora teclas de navegação vazadas para o terminal caso algum dropdown esteja aberto
      if (
        (vpsAuto.showDropdown || dirAuto.showDirDropdown) &&
        (data === "\t" || data === "\x1b[Z")
      ) {
        return;
      }

      // Fecha dropdowns se o usuário der Enter ou sinais de cancelamento
      const isCancelOrEnter =
        data.includes("\r") ||
        data.includes("\n") ||
        data === "\x03" ||
        data === "\x15";

      // Se a substituição de alias estiver sendo processada no PTY, ignora inputs concorrentes temporariamente
      if (aliasSug.isApplying) {
        return;
      }

      // Se a sugestão de alias estiver ativa e o usuário apertou Enter (\r),
      // o evento já foi tratado pelo attachCustomKeyEventHandler para aplicar o alias.
      // Bloqueia o envio do caractere de quebra de linha para o PTY para não duplicar nem engolir letras!
      if (aliasSug.activeSuggestion && (data.includes("\r") || data.includes("\n"))) {
        return;
      }


      if (vpsAuto.showDropdown && isCancelOrEnter) vpsAuto.close();
      if (dirAuto.showDirDropdown && isCancelOrEnter) dirAuto.close();
      if (aliasSug.activeSuggestion && isCancelOrEnter) aliasSug.close();

      PtyService.writePty(id, data).catch(console.error);

      // Verificação não-intrusiva de sugestão de aliases
      setTimeout(() => {
        aliasSug.check(vpsAuto.showDropdown || dirAuto.showDirDropdown);
      }, 10);
    });

    setTimeout(() => {
      fitAddon?.fit();
      term?.focus();
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
      PtyService.resizePty(id, term.cols, term.rows).catch(console.error);
    }
  }

  // Atualiza tema do terminal reativamente quando a configuração muda
  $effect(() => {
    if (!term) return;
    const t = configStore.theme;
    term.options.theme = {
      background: t.terminalBg,
      foreground: t.terminalFg,
      cursor: type === "ssh" ? t.terminalCursorSsh : t.terminalCursorLocal,
      selectionBackground: t.terminalSelection,
    };
  });

  onDestroy(() => {
    PtyService.closePty(id).catch(console.error);
    if (term) term.dispose();
  });
</script>

<div
  bind:this={container}
  class="absolute inset-0 px-[10px] py-2 box-border {active
    ? 'visible pointer-events-auto z-[2]'
    : 'invisible pointer-events-none z-[1]'}"
></div>

{#if vpsAuto.showDropdown && active}
  <SshAutocompleteDropdown
    hosts={vpsAuto.filteredHosts}
    selectedIndex={vpsAuto.selectedHostIndex}
    position={vpsAuto.dropdownPosition}
    commandName={vpsAuto.activeMatchedCommand?.command || "vps"}
    onSelect={vpsAuto.apply}
  />
{/if}

{#if dirAuto.showDirDropdown && active}
  <DirectoryAutocompleteDropdown
    paths={dirAuto.filteredPaths}
    selectedIndex={dirAuto.selectedDirIndex}
    position={dirAuto.dirDropdownPosition}
    onSelect={dirAuto.apply}
  />
{/if}

{#if aliasSug.activeSuggestion && active}
  <AliasSuggestionPopup
    suggestion={aliasSug.activeSuggestion}
    matchedPrefix={aliasSug.matchedPrefix}
    position={aliasSug.position}
  />
{/if}

<style>
  :global(.xterm) {
    height: 100%;
  }

  :global(.xterm-viewport) {
    overflow-y: auto !important;
  }
</style>
