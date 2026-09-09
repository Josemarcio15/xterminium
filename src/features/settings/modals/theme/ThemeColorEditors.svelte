<script lang="ts">
  import type { AppTheme } from "../../../../core/types";
  import Button from "@/shared/components/Button.svelte";
  import ColorPickerWithOpacity from "@/shared/components/ColorPickerWithOpacity.svelte";
  import { colorFields, themeColorGroups } from "./themeConstants";

  interface Props {
    draft: AppTheme;
    isCustomTheme: boolean;
    showNameInput: boolean;
    nameInputValue: string;
    nameInputMode: "new" | "rename";
    isDirty: boolean;
    onUpdateColor: (key: keyof AppTheme, color: string) => void;
    onOpenSaveAs: () => void;
    onOpenRename: () => void;
    onConfirmName: () => void;
    onCancelName: () => void;
  }

  let {
    draft,
    isCustomTheme,
    showNameInput,
    nameInputValue = $bindable(),
    nameInputMode,
    isDirty,
    onUpdateColor,
    onOpenSaveAs,
    onOpenRename,
    onConfirmName,
    onCancelName,
  }: Props = $props();

  let nameInputRef = $state<HTMLInputElement | null>(null);

  $effect(() => {
    if (showNameInput && nameInputRef) {
      nameInputRef.focus();
      nameInputRef.select();
    }
  });
</script>

<!-- Header da seção: Personalizar Cores -->
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
        onclick={onOpenRename}
        title="Renomear tema atual"
      >
        ✏ Renomear
      </Button>
    {/if}
    <Button
      variant="primary"
      size="xs"
      onclick={onOpenSaveAs}
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
        if (e.key === "Enter") onConfirmName();
        if (e.key === "Escape") onCancelName();
      }}
    />
    <Button variant="primary" size="xs" onclick={onConfirmName}>
      ✓ OK
    </Button>
    <Button variant="secondary" size="xs" onclick={onCancelName}>
      ✕
    </Button>
  </div>
{/if}

<!-- Lista de seletores de cores agrupados -->
<div class="flex flex-col gap-3">
  {#each themeColorGroups as group}
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
            <span class="text-[11px] text-[var(--text-base)] font-medium">
              {field.label}
            </span>
            <ColorPickerWithOpacity
              value={String(draft[field.key] || "#000000")}
              allowOpacity={field.allowOpacity !== false}
              label={field.label}
              onChange={(newColor) => onUpdateColor(field.key, newColor)}
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
