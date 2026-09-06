<script lang="ts">
  import { configStore } from "../../../core/stores/config.svelte";
  import { presetThemes, type AppTheme } from "../../../core/types";
  import Modal from "../../../shared/components/Modal.svelte";
  import Button from "@/shared/components/Button.svelte";
  import ColorPickerWithOpacity from "@/shared/components/ColorPickerWithOpacity.svelte";

  interface Props {
    show: boolean;
    onClose: () => void;
  }

  let { show = false, onClose }: Props = $props();

  let draft = $state<AppTheme>({ ...configStore.theme });
  let saving = $state(false);
  let saved = $state(false);
  let wasShown = false;

  // --- "Salvar como" / Renomear ---
  let showNameInput = $state(false);
  let nameInputValue = $state("");
  let nameInputMode = $state<"new" | "rename">("new");
  let nameInputRef = $state<HTMLInputElement | null>(null);

  $effect(() => {
    if (show && !wasShown) {
      wasShown = true;
      configStore.init();
      draft = { ...configStore.theme };
      showNameInput = false;
    } else if (!show) {
      wasShown = false;
      showNameInput = false;
    }
  });

  // Foca o input de nome quando ele abre
  $effect(() => {
    if (showNameInput && nameInputRef) {
      nameInputRef.focus();
      nameInputRef.select();
    }
  });

  async function handleSave() {
    saving = true;
    configStore.applyTheme(draft);
    await configStore.saveTheme();
    saving = false;
    saved = true;
    setTimeout(() => (saved = false), 1800);
  }

  async function handleReset() {
    await configStore.resetTheme();
    draft = { ...configStore.theme };
    configStore.applyTheme(draft);
  }

  function handleClose() {
    onClose();
  }

  function selectPreset(preset: AppTheme) {
    draft = { ...preset };
    configStore.applyTheme(draft);
    showNameInput = false;
  }

  // Abre o input para salvar como novo tema
  function openSaveAs() {
    nameInputMode = "new";
    nameInputValue =
      draft.name === "Default Dark" ||
      presetThemes.some((p) => p.name === draft.name)
        ? ""
        : draft.name;
    showNameInput = true;
  }

  // Abre o input para renomear um tema customizado
  function openRename(theme: AppTheme) {
    nameInputMode = "rename";
    nameInputValue = theme.name;
    showNameInput = true;
  }

  async function confirmNameInput() {
    const name = nameInputValue.trim();
    if (!name) return;

    if (nameInputMode === "new") {
      const newTheme: AppTheme = { ...draft, name };
      await configStore.addCustomTheme(newTheme);
      configStore.applyTheme(newTheme);
      await configStore.saveTheme();
    } else {
      // rename: o draft.name é o nome atual sendo renomeado
      await configStore.renameCustomTheme(draft.name, name);
      draft = { ...draft, name };
      configStore.applyTheme(draft);
      await configStore.saveTheme();
    }

    showNameInput = false;
    nameInputValue = "";
    saved = true;
    setTimeout(() => (saved = false), 1800);
  }

  function cancelNameInput() {
    showNameInput = false;
    nameInputValue = "";
  }

  async function deleteCustomTheme(name: string, e: MouseEvent) {
    e.stopPropagation();
    await configStore.deleteCustomTheme(name);
    // Se era o ativo, volta ao default
    if (configStore.theme.name === name) {
      await configStore.resetTheme();
      draft = { ...configStore.theme };
    }
  }

  type ColorField = {
    key: keyof AppTheme;
    label: string;
    group: string;
    allowOpacity?: boolean;
  };

  const colorFields: ColorField[] = [
    { key: "bgBase", label: "Fundo Principal", group: "Backgrounds", allowOpacity: true },
    { key: "bgTitlebar", label: "Titlebar", group: "Backgrounds", allowOpacity: true },
    { key: "bgPanel", label: "Painéis / Modais", group: "Backgrounds", allowOpacity: true },
    { key: "bgItem", label: "Itens de Lista", group: "Backgrounds", allowOpacity: true },
    {
      key: "bgItemHover",
      label: "Hover de Itens/Linhas",
      group: "Backgrounds",
      allowOpacity: true,
    },
    { key: "bgItemInput", label: "Inputs", group: "Backgrounds", allowOpacity: true },
    { key: "bgTabActive", label: "Aba Ativa", group: "Backgrounds", allowOpacity: true },
    { key: "bgTabHover", label: "Hover de Abas", group: "Backgrounds", allowOpacity: true },
    {
      key: "accentPrimary",
      label: "Acento Principal (SSH)",
      group: "Cores de Acento",
      allowOpacity: true,
    },
    { key: "accentSecondary", label: "Acento Local", group: "Cores de Acento", allowOpacity: true },
    { key: "accentSftp", label: "Acento SFTP", group: "Cores de Acento", allowOpacity: true },
    { key: "accentWarn", label: "Aviso / Comandos", group: "Cores de Acento", allowOpacity: true },
    { key: "terminalBg", label: "Fundo do Terminal", group: "Terminal", allowOpacity: true },
    { key: "terminalFg", label: "Texto do Terminal", group: "Terminal", allowOpacity: true },
    { key: "terminalCursorLocal", label: "Cursor Local", group: "Terminal", allowOpacity: true },
    { key: "terminalCursorSsh", label: "Cursor SSH", group: "Terminal", allowOpacity: true },
    { key: "textBase", label: "Texto Principal", group: "Fontes & Textos", allowOpacity: true },
    { key: "textMuted", label: "Texto Secundário", group: "Fontes & Textos", allowOpacity: true },
    { key: "textFaint", label: "Texto Suave", group: "Fontes & Textos", allowOpacity: true },
    { key: "textSpecial", label: "Textos Especiais (Templates)", group: "Fontes & Textos", allowOpacity: true },
    {
      key: "btnPrimaryBg",
      label: "Fundo Botão Primário",
      group: "Botões: Ação Principal",
      allowOpacity: true,
    },
    {
      key: "btnPrimaryText",
      label: "Texto Botão Primário",
      group: "Botões: Ação Principal",
      allowOpacity: true,
    },
    {
      key: "btnPrimaryHover",
      label: "Hover Botão Primário",
      group: "Botões: Ação Principal",
      allowOpacity: true,
    },
    {
      key: "btnSecondaryBg",
      label: "Fundo Botão Secundário",
      group: "Botões: Secundários",
      allowOpacity: true,
    },
    {
      key: "btnSecondaryText",
      label: "Texto Botão Secundário",
      group: "Botões: Secundários",
      allowOpacity: true,
    },
    {
      key: "btnSecondaryHover",
      label: "Hover Botão Secundário",
      group: "Botões: Secundários",
      allowOpacity: true,
    },
    {
      key: "btnDangerBg",
      label: "Fundo Botão Perigo",
      group: "Botões: Perigo & Remoção",
      allowOpacity: true,
    },
    {
      key: "btnDangerText",
      label: "Texto Botão Perigo",
      group: "Botões: Perigo & Remoção",
      allowOpacity: true,
    },
    {
      key: "btnDangerHover",
      label: "Hover Botão Perigo",
      group: "Botões: Perigo & Remoção",
      allowOpacity: true,
    },
    {
      key: "btnSuccessBg",
      label: "Fundo Botão Sucesso",
      group: "Botões: Sucesso",
      allowOpacity: true,
    },
    {
      key: "btnSuccessText",
      label: "Texto Botão Sucesso",
      group: "Botões: Sucesso",
      allowOpacity: true,
    },
    {
      key: "btnGlassBg",
      label: "Fundo Botão Vidro (Glass)",
      group: "Botões: Vidro & Translúcido",
      allowOpacity: true,
    },
    {
      key: "btnGlassText",
      label: "Texto Botão Vidro (Glass)",
      group: "Botões: Vidro & Translúcido",
      allowOpacity: true,
    },
    {
      key: "btnGlassHover",
      label: "Hover Botão Vidro (Glass)",
      group: "Botões: Vidro & Translúcido",
      allowOpacity: true,
    },
    {
      key: "btnBorder",
      label: "Borda Global de Botões",
      group: "Botões: Aparência & Borda",
      allowOpacity: true,
    },
  ];

  const groups = [...new Set(colorFields.map((f) => f.group))];

  function updateDraftColor(key: keyof AppTheme, value: string) {
    draft[key] = value;
    draft = { ...draft };
    configStore.applyTheme(draft);
  }

  // Raio dos botões (numérico para o slider: 0 a 16px)
  const btnRadiusNum = $derived(
    parseInt((draft.btnRadius || '6px').replace('px', ''), 10) || 0
  );

  function updateBtnRadius(val: number) {
    updateDraftColor('btnRadius', `${val}px`);
  }

  // Raio da janela (numérico para o slider: 0 a 20px)
  const windowRadiusNum = $derived(
    parseInt((draft.windowRadius || '10px').replace('px', ''), 10) || 0
  );

  function updateWindowRadius(val: number) {
    updateDraftColor('windowRadius', `${val}px`);
  }

  // Nível de elevação / sombras (0 a 4)
  const elevationNum = $derived(
    typeof draft.elevation === 'number' ? draft.elevation : 1
  );

  function updateElevation(val: number) {
    draft.elevation = val;
    draft = { ...draft };
    configStore.applyTheme(draft);
  }

  // Detecta se o draft está diferente do tema salvo (modificado)
  const isDirty = $derived(
    JSON.stringify(draft) !== JSON.stringify(configStore.theme),
  );

  // Detecta se o tema atual é um customizado (editável/renomeável)
  const isCustomTheme = $derived(
    configStore.customThemes.some((t) => t.name === configStore.theme.name),
  );
</script>

<Modal
  {show}
  title="Temas & Aparência"
  widthClass="w-[440px]"
  maxHClass="max-h-[85vh]"
  onClose={handleClose}
>
  {#snippet icon()}
    <svg
      class="text-violet-400"
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
    >
      <circle cx="13.5" cy="6.5" r="2.5"></circle>
      <circle cx="17.5" cy="10.5" r="2.5"></circle>
      <circle cx="8.5" cy="7.5" r="2.5"></circle>
      <circle cx="6.5" cy="12.5" r="2.5"></circle>
      <path d="M12 22c-4.97 0-9-2.69-9-6 0-1.6 1.4-3.1 3.5-4.2"></path>
      <path d="M16.5 19c1.93-1.11 3.5-2.6 3.5-4.2 0-.82-.29-1.6-.8-2.3"></path>
    </svg>
  {/snippet}

  {#snippet actions()}
    <div class="flex items-center gap-1.5">
      <Button
        variant="secondary"
        size="xs"
        onclick={handleReset}
        title="Restaurar tema padrão"
      >
        Padrão
      </Button>
      <Button
        variant={saved ? "success" : "primary"}
        size="xs"
        onclick={handleSave}
        loading={saving}
      >
        {saved ? "✓ Salvo" : "Salvar"}
      </Button>
    </div>
  {/snippet}

  <!-- Container de scroll único para todo o conteúdo do modal -->
  <div class="flex-1 overflow-y-auto pr-1 flex flex-col [scrollbar-width:thin]">
    <!-- Temas Predefinidos -->
    <div class="mb-3">
    <p
      class="text-[10px] uppercase tracking-wider text-[var(--text-muted)] opacity-90 font-bold mb-2"
    >
      Predefinidos
    </p>
    <div class="grid grid-cols-2 gap-1.5">
      {#each presetThemes as preset}
        {@const isActive = configStore.theme.name === preset.name}
        <button
          type="button"
          class="group relative flex items-center gap-2 px-2.5 py-2 rounded-lg border cursor-pointer transition-all text-left
            {isActive
            ? 'border-violet-500 bg-violet-500/10 shadow-sm'
            : 'border-[var(--border-subtle)] bg-[var(--bg-item)] hover:border-[var(--border-panel)] hover:brightness-95 dark:hover:brightness-110'}"
          onclick={() => selectPreset(preset)}
        >
          <div class="flex gap-0.5 shrink-0">
            <div
              class="w-3 h-5 rounded-sm shadow-xs"
              style="background:{preset.bgBase}; border: 1px solid {preset.accentPrimary}44;"
            ></div>
            <div class="flex flex-col gap-0.5">
              <div
                class="w-3 h-2.5 rounded-sm"
                style="background:{preset.bgTitlebar}; border: 1px solid {preset.accentPrimary}22;"
              ></div>
              <div
                class="w-3 h-2 rounded-sm"
                style="background:{preset.accentPrimary};"
              ></div>
            </div>
          </div>
          <span
            class="text-[11px] font-medium {isActive
              ? 'text-violet-600 dark:text-violet-300 font-bold'
              : 'text-[var(--text-base)]'} truncate"
          >
            {preset.name}
          </span>
          {#if isActive}
            <span
              class="absolute top-1.5 right-1.5 text-violet-600 dark:text-violet-400 text-[9px] font-bold"
              >✓</span
            >
          {/if}
        </button>
      {/each}
    </div>
  </div>

  <!-- Temas Customizados -->
  {#if configStore.customThemes.length > 0}
    <div class="mb-3">
      <p
        class="text-[10px] uppercase tracking-wider text-[var(--text-muted)] opacity-90 font-bold mb-2"
      >
        Meus Temas
      </p>
      <div class="grid grid-cols-2 gap-1.5">
        {#each configStore.customThemes as custom}
          {@const isActive = configStore.theme.name === custom.name}
          <button
            type="button"
            class="group relative flex items-center gap-2 px-2.5 py-2 rounded-lg border cursor-pointer transition-all text-left
              {isActive
              ? 'border-violet-500 bg-violet-500/10 shadow-sm'
              : 'border-[var(--border-subtle)] bg-[var(--bg-item)] hover:border-[var(--border-panel)] hover:brightness-95 dark:hover:brightness-110'}"
            onclick={() => selectPreset(custom)}
          >
            <div class="flex gap-0.5 shrink-0">
              <div
                class="w-3 h-5 rounded-sm shadow-xs"
                style="background:{custom.bgBase}; border: 1px solid {custom.accentPrimary}44;"
              ></div>
              <div class="flex flex-col gap-0.5">
                <div
                  class="w-3 h-2.5 rounded-sm"
                  style="background:{custom.bgTitlebar}; border: 1px solid {custom.accentPrimary}22;"
                ></div>
                <div
                  class="w-3 h-2 rounded-sm"
                  style="background:{custom.accentPrimary};"
                ></div>
              </div>
            </div>
            <span
              class="text-[11px] font-medium {isActive
                ? 'text-violet-600 dark:text-violet-300 font-bold'
                : 'text-[var(--text-base)]'} truncate flex-1 min-w-0"
            >
              {custom.name}
            </span>
            <!-- Botões renomear e deletar -->
            <div
              class="absolute top-1 right-1 hidden group-hover:flex items-center gap-0.5"
            >
              <span
                role="button"
                tabindex="0"
                class="p-0.5 rounded text-[var(--text-muted)] hover:text-sky-400 hover:bg-sky-400/10 transition-all cursor-pointer"
                title="Renomear"
                onclick={(e) => {
                  e.stopPropagation();
                  selectPreset(custom);
                  openRename(custom);
                }}
                onkeydown={(e) =>
                  e.key === "Enter" &&
                  (e.stopPropagation(),
                  selectPreset(custom),
                  openRename(custom))}
              >
                <svg
                  width="10"
                  height="10"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2.5"
                >
                  <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"
                  ></path>
                </svg>
              </span>
              <span
                role="button"
                tabindex="0"
                class="p-0.5 rounded text-[var(--text-muted)] hover:text-red-400 hover:bg-red-400/10 transition-all cursor-pointer"
                title="Excluir"
                onclick={(e) => deleteCustomTheme(custom.name, e)}
                onkeydown={(e) =>
                  e.key === "Enter" &&
                  deleteCustomTheme(custom.name, e as unknown as MouseEvent)}
              >
                <svg
                  width="10"
                  height="10"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2.5"
                >
                  <path d="M3 6h18M19 6l-1 14H6L5 6M10 11v6M14 11v6M9 6V4h6v2"
                  ></path>
                </svg>
              </span>
            </div>
            {#if isActive}
              <span
                class="absolute top-1 right-1.5 text-violet-400 text-[8px] group-hover:hidden"
                >✓</span
              >
            {/if}
          </button>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Separador -->
  <div class="h-px bg-[var(--border-subtle)] mb-3"></div>

  <!-- Formas & Arredondamento (Sliders) -->
  <div class="mb-3.5 p-2.5 rounded-lg bg-[var(--bg-item)] border border-[var(--border-subtle)] flex flex-col gap-2.5">
    <p class="text-[10px] uppercase tracking-wider text-[var(--text-muted)] opacity-90 font-bold">
      Formas & Arredondamento
    </p>

    <!-- Slider: Botoes -->
    <div class="flex flex-col gap-1">
      <div class="flex items-center justify-between text-xs">
        <span class="text-[11px] text-[var(--text-base)] font-medium flex items-center gap-1.5">
          <span>Bordas dos Botões</span>
          <span class="text-[9px] text-[var(--text-faint)] font-mono">({btnRadiusNum === 0 ? 'Quadrado' : btnRadiusNum >= 14 ? 'Pílula' : `${btnRadiusNum}px`})</span>
        </span>
        <div class="w-4 h-4 rounded-[var(--btn-radius)] bg-[var(--btn-primary-bg)] border border-[var(--btn-border)] shrink-0 shadow-xs"></div>
      </div>
      <div class="flex items-center gap-2">
        <span class="text-[10px] text-[var(--text-faint)] font-mono">0px</span>
        <input
          type="range"
          min="0"
          max="16"
          step="1"
          class="flex-1 accent-[var(--accent-primary)] cursor-pointer h-2 bg-slate-300 dark:bg-slate-700 border border-[var(--border-subtle)] rounded-lg appearance-none"
          value={btnRadiusNum}
          oninput={(e) => updateBtnRadius(parseInt((e.target as HTMLInputElement).value, 10))}
        />
        <span class="text-[10px] text-[var(--text-faint)] font-mono">16px</span>
      </div>
    </div>

    <div class="h-px bg-[var(--border-subtle)]"></div>

    <!-- Slider: Janela toda -->
    <div class="flex flex-col gap-1">
      <div class="flex items-center justify-between text-xs">
        <span class="text-[11px] text-[var(--text-base)] font-medium flex items-center gap-1.5">
          <span>Cantos da Janela</span>
          <span class="text-[9px] text-[var(--text-faint)] font-mono">({windowRadiusNum === 0 ? 'Reto' : `${windowRadiusNum}px`})</span>
        </span>
        <div class="w-4 h-4 rounded-[var(--window-radius)] bg-[var(--bg-panel)] border border-[var(--border-panel)] shrink-0 shadow-xs"></div>
      </div>
      <div class="flex items-center gap-2">
        <span class="text-[10px] text-[var(--text-faint)] font-mono">0px</span>
        <input
          type="range"
          min="0"
          max="20"
          step="1"
          class="flex-1 accent-[var(--accent-primary)] cursor-pointer h-2 bg-slate-300 dark:bg-slate-700 border border-[var(--border-subtle)] rounded-lg appearance-none"
          value={windowRadiusNum}
          oninput={(e) => updateWindowRadius(parseInt((e.target as HTMLInputElement).value, 10))}
        />
        <span class="text-[10px] text-[var(--text-faint)] font-mono">20px</span>
      </div>
    </div>

    <div class="h-px bg-[var(--border-subtle)]"></div>

    <!-- Slider: Elevação & Sombras (Elevated Effect) -->
    <div class="flex flex-col gap-1">
      <div class="flex items-center justify-between text-xs">
        <span class="text-[11px] text-[var(--text-base)] font-medium flex items-center gap-1.5">
          <span>Sombras & Elevação (Relevo)</span>
          <span class="text-[9px] text-[var(--text-faint)] font-mono">({
            elevationNum === 0 ? 'Flat (Plano)' :
            elevationNum === 1 ? 'Sutil' :
            elevationNum === 2 ? 'Médio' :
            elevationNum === 3 ? 'Elevado' : 'Alto Relevo'
          })</span>
        </span>
        <div 
          class="w-4 h-4 rounded-md bg-[var(--bg-panel)] border border-[var(--border-panel)] shrink-0 transition-all"
          style="box-shadow: var(--shadow-btn);"
        ></div>
      </div>
      <div class="flex items-center gap-2">
        <span class="text-[10px] text-[var(--text-faint)] font-mono">0</span>
        <input
          type="range"
          min="0"
          max="4"
          step="1"
          class="flex-1 accent-[var(--accent-primary)] cursor-pointer h-2 bg-slate-300 dark:bg-slate-700 border border-[var(--border-subtle)] rounded-lg appearance-none"
          value={elevationNum}
          oninput={(e) => updateElevation(parseInt((e.target as HTMLInputElement).value, 10))}
        />
        <span class="text-[10px] text-[var(--text-faint)] font-mono">4</span>
      </div>
    </div>
  </div>

  <!-- Separador -->
  <div class="h-px bg-[var(--border-subtle)] mb-3"></div>

  <!-- Editor de cores + botão Salvar como -->
  <div class="flex items-center justify-between mb-2">
    <p
      class="text-[10px] uppercase tracking-wider text-[var(--text-muted)] opacity-90 font-bold"
    >
      Personalizar Cores
    </p>
    <div class="flex items-center gap-1.5">
      {#if isCustomTheme}
        <Button
          variant="secondary"
          size="xs"
          onclick={() => openRename(configStore.theme)}
          title="Renomear tema atual"
        >
          ✏ Renomear
        </Button>
      {/if}
      <Button
        variant="primary"
        size="xs"
        onclick={openSaveAs}
        title="Salvar como novo tema"
      >
        + Salvar como...
      </Button>
    </div>
  </div>

  <!-- Input de nome inline -->
  {#if showNameInput}
    <div
      class="flex items-center gap-1.5 mb-3 p-2 rounded-lg bg-[var(--bg-item)] border border-[var(--btn-border)]"
    >
      <span
        class="text-[10px] text-[var(--accent-primary)] shrink-0 font-medium"
      >
        {nameInputMode === "new" ? "Nome do tema:" : "Novo nome:"}
      </span>
      <input
        bind:this={nameInputRef}
        bind:value={nameInputValue}
        type="text"
        placeholder="Ex: Meu Tema"
        class="flex-1 min-w-0 bg-transparent border-b border-[var(--btn-border)] text-[11px] text-[var(--text-base)] outline-none placeholder-[var(--text-faint)] py-0.5"
        onkeydown={(e) => {
          if (e.key === "Enter") confirmNameInput();
          if (e.key === "Escape") cancelNameInput();
        }}
      />
      <Button
        variant="primary"
        size="xs"
        onclick={confirmNameInput}
      >
        ✓ OK
      </Button>
      <Button
        variant="secondary"
        size="xs"
        onclick={cancelNameInput}
      >
        ✕
      </Button>
    </div>
  {/if}

  <!-- Color pickers -->
  <div class="flex flex-col gap-3">
    {#each groups as group}
      <div>
        <p
          class="text-[10px] uppercase tracking-wider text-[var(--text-faint)] font-bold mb-1.5"
        >
          {group}
        </p>
        <div class="flex flex-col gap-1">
          {#each colorFields.filter((f) => f.group === group) as field}
            <div
              class="flex items-center justify-between gap-2 px-2.5 py-1.5 rounded-lg bg-[var(--bg-item)] border border-[var(--border-subtle)]"
            >
              <span class="text-[11px] text-[var(--text-base)] font-medium"
                >{field.label}</span
              >
              <ColorPickerWithOpacity
                value={String(draft[field.key] || '#000000')}
                allowOpacity={field.allowOpacity !== false}
                label={field.label}
                onChange={(newColor) => updateDraftColor(field.key, newColor)}
              />
            </div>
          {/each}
        </div>
      </div>
    {/each}
  </div>

  <!-- Dica: mudanças não salvas -->
  {#if isDirty}
    <p class="text-[10px] text-amber-400/80 mt-2 text-center">
      Cores alteradas — clique em <strong>Salvar</strong> ou
      <strong>+ Salvar como...</strong> para guardar
    </p>
  {/if}
  </div>
</Modal>


