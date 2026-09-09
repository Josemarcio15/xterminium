<script lang="ts">
  import type { AppTheme } from "../../../../core/types";

  interface Props {
    draft: AppTheme;
    onUpdate: (key: keyof AppTheme, value: string | number) => void;
  }

  let { draft, onUpdate }: Props = $props();

  const btnRadiusNum = $derived(
    parseInt((draft.btnRadius || "6px").replace("px", ""), 10) || 0
  );

  const windowRadiusNum = $derived(
    parseInt((draft.windowRadius || "10px").replace("px", ""), 10) || 0
  );

  const elevationNum = $derived(
    typeof draft.elevation === "number" ? draft.elevation : 1
  );
</script>

<div
  class="mb-3.5 p-2.5 rounded-lg bg-[var(--bg-item)] border border-[var(--border-subtle)] flex flex-col gap-2.5"
>
  <p
    class="text-[10px] uppercase tracking-wider text-[var(--text-muted)] opacity-90 font-bold"
  >
    Formas & Arredondamento
  </p>

  <!-- Slider: Botões -->
  <div class="flex flex-col gap-1">
    <div class="flex items-center justify-between text-xs">
      <span
        class="text-[11px] text-[var(--text-base)] font-medium flex items-center gap-1.5"
      >
        <span>Bordas dos Botões</span>
        <span class="text-[9px] text-[var(--text-faint)] font-mono">
          ({btnRadiusNum === 0
            ? "Quadrado"
            : btnRadiusNum >= 14
              ? "Pílula"
              : `${btnRadiusNum}px`})
        </span>
      </span>
      <div
        class="w-4 h-4 rounded-[var(--btn-radius)] bg-[var(--btn-primary-bg)] border border-[var(--btn-border)] shrink-0 shadow-xs"
      ></div>
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
        oninput={(e) =>
          onUpdate(
            "btnRadius",
            `${parseInt((e.target as HTMLInputElement).value, 10)}px`
          )}
      />
      <span class="text-[10px] text-[var(--text-faint)] font-mono">16px</span>
    </div>
  </div>

  <div class="h-px bg-[var(--border-subtle)]"></div>

  <!-- Slider: Janela toda -->
  <div class="flex flex-col gap-1">
    <div class="flex items-center justify-between text-xs">
      <span
        class="text-[11px] text-[var(--text-base)] font-medium flex items-center gap-1.5"
      >
        <span>Cantos da Janela</span>
        <span class="text-[9px] text-[var(--text-faint)] font-mono">
          ({windowRadiusNum === 0 ? "Reto" : `${windowRadiusNum}px`})
        </span>
      </span>
      <div
        class="w-4 h-4 rounded-[var(--window-radius)] bg-[var(--bg-panel)] border border-[var(--border-panel)] shrink-0 shadow-xs"
      ></div>
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
        oninput={(e) =>
          onUpdate(
            "windowRadius",
            `${parseInt((e.target as HTMLInputElement).value, 10)}px`
          )}
      />
      <span class="text-[10px] text-[var(--text-faint)] font-mono">20px</span>
    </div>
  </div>

  <div class="h-px bg-[var(--border-subtle)]"></div>

  <!-- Slider: Elevação & Sombras -->
  <div class="flex flex-col gap-1">
    <div class="flex items-center justify-between text-xs">
      <span
        class="text-[11px] text-[var(--text-base)] font-medium flex items-center gap-1.5"
      >
        <span>Sombras & Elevação (Relevo)</span>
        <span class="text-[9px] text-[var(--text-faint)] font-mono">
          ({elevationNum === 0
            ? "Flat (Plano)"
            : elevationNum === 1
              ? "Sutil"
              : elevationNum === 2
                ? "Médio"
                : elevationNum === 3
                  ? "Elevado"
                  : "Alto Relevo"})
        </span>
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
        oninput={(e) =>
          onUpdate(
            "elevation",
            parseInt((e.target as HTMLInputElement).value, 10)
          )}
      />
      <span class="text-[10px] text-[var(--text-faint)] font-mono">4</span>
    </div>
  </div>
</div>
