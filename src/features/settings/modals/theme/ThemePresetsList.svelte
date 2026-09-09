<script lang="ts">
  import { configStore } from "../../../../core/stores/config.svelte";
  import { presetThemes, type AppTheme } from "../../../../core/types";

  interface Props {
    onSelect: (theme: AppTheme) => void;
    onRename: (theme: AppTheme) => void;
  }

  let { onSelect, onRename }: Props = $props();

  async function deleteCustomTheme(name: string, e: MouseEvent) {
    e.stopPropagation();
    await configStore.deleteCustomTheme(name);
    if (configStore.theme.name === name) {
      await configStore.resetTheme();
    }
  }
</script>

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
        onclick={() => onSelect(preset)}
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
          >
            ✓
          </span>
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
          onclick={() => onSelect(custom)}
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
                onSelect(custom);
                onRename(custom);
              }}
              onkeydown={(e) =>
                e.key === "Enter" &&
                (e.stopPropagation(), onSelect(custom), onRename(custom))}
            >
              <svg
                width="10"
                height="10"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.5"
              >
                <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"></path>
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
                <path d="M3 6h18M19 6l-1 14H6L5 6M10 11v6M14 11v6M9 6V4h6v2"></path>
              </svg>
            </span>
          </div>
          {#if isActive}
            <span
              class="absolute top-1 right-1.5 text-violet-400 text-[8px] group-hover:hidden"
            >
              ✓
            </span>
          {/if}
        </button>
      {/each}
    </div>
  </div>
{/if}
