<script lang="ts">
  import { configStore } from "../../../core/stores/config.svelte";
  import {
    type ShellProfile,
    sameShellPath,
    shellIdFromPath,
  } from "../../../core/types";
  import { PtyService } from "../../../core/services";
  import Modal from "../../../shared/components/Modal.svelte";
  import Button from "@/shared/components/Button.svelte";

  interface Props {
    show: boolean;
    onClose: () => void;
  }

  let { show = false, onClose }: Props = $props();

  let detected = $state<ShellProfile[]>([]);
  let loading = $state(false);
  let loadError = $state<string | null>(null);
  let wasShown = false;

  let showForm = $state(false);
  let formName = $state("");
  let formPath = $state("");
  let formArgs = $state("");

  let argsDraft = $state("");
  let savedFlash = $state(false);

  const current = $derived(configStore.shells.default);

  /** Shells customizados que não duplicam um já detectado. */
  const customOnly = $derived(
    configStore.shells.custom.filter(
      (c) => !detected.some((d) => sameShellPath(d.path, c.path)),
    ),
  );

  /** Divide a string de argumentos respeitando aspas simples/duplas. */
  function parseArgs(raw: string): string[] {
    const matches = raw.match(/"[^"]*"|'[^']*'|\S+/g) ?? [];
    return matches.map((m) => m.replace(/^["']|["']$/g, ""));
  }

  function formatArgs(args: string[]): string {
    return args.map((a) => (/\s/.test(a) ? `"${a}"` : a)).join(" ");
  }

  async function loadShells() {
    loading = true;
    loadError = null;
    try {
      detected = await PtyService.listShells();
    } catch (e) {
      loadError = "Não foi possível detectar os shells instalados.";
      console.error("[shells] falha ao listar shells:", e);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (show && !wasShown) {
      wasShown = true;
      configStore.init();
      loadShells();
      showForm = false;
    } else if (!show) {
      wasShown = false;
      showForm = false;
    }
  });

  // Mantém o campo de argumentos sincronizado com o shell padrão atual
  $effect(() => {
    argsDraft = formatArgs(current?.args ?? []);
  });

  function isSelected(shell: { path: string }): boolean {
    return !!current && sameShellPath(current.path, shell.path);
  }

  function flashSaved() {
    savedFlash = true;
    setTimeout(() => (savedFlash = false), 1400);
  }

  async function selectShell(shell: ShellProfile) {
    await configStore.setDefaultShell({
      name: shell.name,
      path: shell.path,
      args: shell.args ?? [],
      kind: shell.kind,
    });
    flashSaved();
  }

  async function useSystemDefault() {
    await configStore.setDefaultShell(null);
    flashSaved();
  }

  async function saveArgs() {
    if (!current) return;
    await configStore.setDefaultShellArgs(parseArgs(argsDraft));
    argsDraft = formatArgs(parseArgs(argsDraft));
    flashSaved();
  }

  async function addCustom() {
    const path = formPath.trim();
    if (!path) return;
    const name = formName.trim() || path.split(/[\\/]/).pop() || path;
    await configStore.addCustomShell({
      id: shellIdFromPath(path),
      name,
      path,
      args: parseArgs(formArgs),
      kind: "other",
      custom: true,
    });
    showForm = false;
    formName = "";
    formPath = "";
    formArgs = "";
    flashSaved();
  }

  async function removeCustom(id: string) {
    await configStore.removeCustomShell(id);
  }
</script>

<Modal {show} title="Shell do Terminal" widthClass="w-[380px]" {onClose}>
  {#snippet icon()}
    <svg
      class="text-emerald-400"
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <polyline points="4 17 10 11 4 5"></polyline>
      <line x1="12" y1="19" x2="20" y2="19"></line>
    </svg>
  {/snippet}

  {#snippet actions()}
    <div class="flex items-center gap-1.5">
      <button
        class="w-[22px] h-[22px] rounded flex items-center justify-center text-xs bg-white/5 border border-white/10 text-[var(--text-muted)] hover:bg-white/10 hover:text-[var(--text-base)] transition-all cursor-pointer disabled:opacity-50"
        onclick={loadShells}
        disabled={loading}
        title="Detectar novamente"
      >
        <svg
          width="12"
          height="12"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          class={loading ? "animate-spin" : ""}
        >
          <path d="M21 12a9 9 0 1 1-2.64-6.36"></path>
          <polyline points="21 3 21 9 15 9"></polyline>
        </svg>
      </button>
      <button
        class="w-[22px] h-[22px] rounded flex items-center justify-center text-xs bg-white/5 border border-white/10 text-[var(--text-muted)] hover:bg-white/10 hover:text-[var(--text-base)] transition-all cursor-pointer"
        onclick={() => (showForm = !showForm)}
        title={showForm ? "Fechar formulário" : "Adicionar shell manualmente"}
      >
        {showForm ? "✕" : "+"}
      </button>
    </div>
  {/snippet}

  <div class="flex flex-col gap-2.5">
    <p class="text-[10px] text-[var(--text-muted)] leading-relaxed">
      Escolha o shell usado ao abrir uma nova aba local. A alteração vale para
      as próximas abas.
    </p>

    <!-- Formulário de shell customizado -->
    {#if showForm}
      <form
        class="bg-[var(--bg-item)] border border-[var(--border-panel)] rounded-lg p-2.5 flex flex-col gap-2"
        onsubmit={(e) => {
          e.preventDefault();
          addCustom();
        }}
      >
        <div
          class="text-[11px] font-semibold text-emerald-600 dark:text-emerald-400"
        >
          Adicionar shell manualmente
        </div>
        <input
          class="bg-[var(--bg-item-input)] border border-[var(--border-subtle)] rounded text-[var(--text-base)] px-2 py-1.5 text-xs outline-none focus:border-emerald-400 transition-colors"
          type="text"
          placeholder="Nome (ex: Meu Zsh)"
          bind:value={formName}
        />
        <input
          class="bg-[var(--bg-item-input)] border border-[var(--border-subtle)] rounded text-[var(--text-base)] px-2 py-1.5 text-xs outline-none focus:border-emerald-400 transition-colors font-mono"
          type="text"
          placeholder="Caminho do executável (ex: /usr/bin/zsh)"
          bind:value={formPath}
          required
        />
        <input
          class="bg-[var(--bg-item-input)] border border-[var(--border-subtle)] rounded text-[var(--text-base)] px-2 py-1.5 text-xs outline-none focus:border-emerald-400 transition-colors font-mono"
          type="text"
          placeholder="Argumentos (opcional, ex: --login -i)"
          bind:value={formArgs}
        />
        <Button
          type="submit"
          variant="primary"
          size="sm"
          class="w-full justify-center"
        >
          Adicionar
        </Button>
      </form>
    {/if}

    <!-- Argumentos do shell padrão -->
    {#if current}
      <div class="flex flex-col gap-1">
        <label
          class="text-[10px] font-semibold text-[var(--text-muted)]"
          for="shell-args"
        >
          Argumentos de <span class="text-[var(--text-base)]"
            >{current.name}</span
          >
        </label>
        <div class="flex gap-1.5">
          <input
            id="shell-args"
            class="flex-1 bg-[var(--bg-item-input)] border border-[var(--border-subtle)] rounded text-[var(--text-base)] px-2 py-1.5 text-xs outline-none focus:border-emerald-400 transition-colors font-mono"
            type="text"
            placeholder="ex: --login -i"
            bind:value={argsDraft}
            onblur={saveArgs}
            onkeydown={(e) => e.key === "Enter" && saveArgs()}
          />
          <Button
            variant="secondary"
            size="sm"
            onclick={saveArgs}
            title="Salvar argumentos"
          >
            Salvar
          </Button>
        </div>
      </div>
    {/if}

    {#if loadError}
      <div
        class="text-[10px] text-amber-500 bg-amber-500/10 border border-amber-500/25 rounded px-2 py-1.5"
      >
        {loadError}
      </div>
    {/if}

    <!-- Lista de shells -->
    <div class="max-h-64 overflow-y-auto flex flex-col gap-1">
      <!-- Padrão do sistema -->
      <button
        type="button"
        class="flex items-center gap-2 px-2.5 py-2 rounded-lg border text-left transition-all cursor-pointer w-full
          {!current
          ? 'border-emerald-400/60 bg-emerald-500/10'
          : 'bg-[var(--bg-item)] border-[var(--border-subtle)] hover:border-emerald-400/40 hover:bg-emerald-500/5'}"
        onclick={useSystemDefault}
      >
        <span
          class="w-3 h-3 rounded-full border shrink-0 flex items-center justify-center
            {!current
            ? 'border-emerald-400 bg-emerald-400'
            : 'border-[var(--text-faint)]'}"
        ></span>
        <span class="flex flex-col gap-0.5 overflow-hidden flex-1">
          <span class="text-xs font-medium text-[var(--text-base)]">
            Padrão do sistema
          </span>
          <span class="text-[10px] text-[var(--text-muted)] truncate">
            Usa o shell padrão detectado ao iniciar
          </span>
        </span>
      </button>

      {#if detected.length > 0 || customOnly.length > 0}
        <div
          class="text-[10px] font-semibold text-[var(--text-faint)] uppercase tracking-wide mt-1.5 px-0.5"
        >
          Detectados no sistema
        </div>
      {/if}

      {#each detected as shell (shell.id + shell.path)}
        {@const selected = isSelected(shell)}
        <div
          class="flex items-center gap-2 px-2.5 py-2 rounded-lg border transition-all group
            {selected
            ? 'border-emerald-400/60 bg-emerald-500/10'
            : 'bg-[var(--bg-item)] border-[var(--border-subtle)] hover:border-emerald-400/40 hover:bg-emerald-500/5'}"
        >
          <button
            type="button"
            class="flex items-center gap-2 flex-1 overflow-hidden text-left bg-transparent border-none cursor-pointer p-0"
            onclick={() => selectShell(shell)}
            title="Usar este shell como padrão"
          >
            <span
              class="w-3 h-3 rounded-full border shrink-0 flex items-center justify-center
                {selected
                ? 'border-emerald-400 bg-emerald-400'
                : 'border-[var(--text-faint)]'}"
            ></span>
            <span class="flex flex-col gap-0.5 overflow-hidden flex-1">
              <span class="flex items-center gap-1.5">
                <span
                  class="text-xs font-medium text-[var(--text-base)] truncate"
                >
                  {shell.name}
                </span>
                {#if shell.recommended}
                  <span
                    class="text-[9px] text-sky-400 bg-sky-400/15 px-1 py-0.5 rounded shrink-0"
                    title="Shell padrão do sistema operacional"
                  >
                    sistema
                  </span>
                {/if}
                {#if selected}
                  <span class="text-[9px] text-emerald-400 shrink-0"
                    >em uso</span
                  >
                {/if}
              </span>
              <span
                class="text-[10px] text-[var(--text-muted)] font-mono truncate"
              >
                {shell.path}{shell.args?.length
                  ? ` ${formatArgs(shell.args)}`
                  : ""}
              </span>
            </span>
          </button>
        </div>
      {/each}

      {#if customOnly.length > 0}
        <div
          class="text-[10px] font-semibold text-[var(--text-faint)] uppercase tracking-wide mt-1.5 px-0.5"
        >
          Adicionados por você
        </div>
        {#each customOnly as shell (shell.id + shell.path)}
          {@const selected = isSelected(shell)}
          <div
            class="flex items-center gap-2 px-2.5 py-2 rounded-lg border transition-all
              {selected
              ? 'border-emerald-400/60 bg-emerald-500/10'
              : 'bg-[var(--bg-item)] border-[var(--border-subtle)] hover:border-emerald-400/40 hover:bg-emerald-500/5'}"
          >
            <button
              type="button"
              class="flex items-center gap-2 flex-1 overflow-hidden text-left bg-transparent border-none cursor-pointer p-0"
              onclick={() => selectShell(shell)}
              title="Usar este shell como padrão"
            >
              <span
                class="w-3 h-3 rounded-full border shrink-0 flex items-center justify-center
                  {selected
                  ? 'border-emerald-400 bg-emerald-400'
                  : 'border-[var(--text-faint)]'}"
              ></span>
              <span class="flex flex-col gap-0.5 overflow-hidden flex-1">
                <span class="flex items-center gap-1.5">
                  <span
                    class="text-xs font-medium text-[var(--text-base)] truncate"
                  >
                    {shell.name}
                  </span>
                  {#if selected}
                    <span class="text-[9px] text-emerald-400 shrink-0"
                      >em uso</span
                    >
                  {/if}
                </span>
                <span
                  class="text-[10px] text-[var(--text-muted)] font-mono truncate"
                >
                  {shell.path}{shell.args?.length
                    ? ` ${formatArgs(shell.args)}`
                    : ""}
                </span>
              </span>
            </button>
            <button
              class="text-[var(--text-muted)] hover:text-red-400 hover:bg-red-400/15 p-1 rounded text-xs leading-none transition-all cursor-pointer border-none bg-transparent shrink-0"
              onclick={() => removeCustom(shell.id)}
              title="Remover"
            >
              ✕
            </button>
          </div>
        {/each}
      {/if}

      {#if !loading && detected.length === 0 && customOnly.length === 0}
        <div
          class="text-center text-[var(--text-muted)] text-xs py-4 leading-relaxed"
        >
          Nenhum shell detectado.<br />
          Use o <b>+</b> para informar o caminho manualmente.
        </div>
      {/if}
    </div>

    {#if savedFlash}
      <div class="text-[10px] text-emerald-400 text-center">
        Configuração salva ✓
      </div>
    {/if}
  </div>
</Modal>
