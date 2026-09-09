<script lang="ts">
  import { configStore } from "../../../core/stores/config.svelte";
  import { presetThemes, type AppTheme } from "../../../core/types";
  import Modal from "../../../shared/components/Modal.svelte";
  import Button from "@/shared/components/Button.svelte";
  import ThemePresetsList from "./theme/ThemePresetsList.svelte";
  import ThemeShapesSliders from "./theme/ThemeShapesSliders.svelte";
  import ThemeColorEditors from "./theme/ThemeColorEditors.svelte";

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

  function openSaveAs() {
    nameInputMode = "new";
    nameInputValue =
      draft.name === "Default Dark" ||
      presetThemes.some((p) => p.name === draft.name)
        ? ""
        : draft.name;
    showNameInput = true;
  }

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

  function updateDraftValue(key: keyof AppTheme, value: string | number) {
    (draft as any)[key] = value;
    draft = { ...draft };
    configStore.applyTheme(draft);
  }

  const isDirty = $derived(
    JSON.stringify(draft) !== JSON.stringify(configStore.theme)
  );

  const isCustomTheme = $derived(
    configStore.customThemes.some((t) => t.name === configStore.theme.name)
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
    <ThemePresetsList onSelect={selectPreset} onRename={openRename} />

    <div class="h-px bg-[var(--border-subtle)] mb-3"></div>

    <ThemeShapesSliders {draft} onUpdate={updateDraftValue} />

    <div class="h-px bg-[var(--border-subtle)] mb-3"></div>

    <ThemeColorEditors
      {draft}
      {isCustomTheme}
      {showNameInput}
      bind:nameInputValue
      {nameInputMode}
      {isDirty}
      onUpdateColor={updateDraftValue}
      onOpenSaveAs={openSaveAs}
      onOpenRename={() => openRename(configStore.theme)}
      onConfirmName={confirmNameInput}
      onCancelName={cancelNameInput}
    />
  </div>
</Modal>
