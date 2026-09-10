<script lang="ts">
  import {
    type AppTheme,
    isRainTheme,
    isFireTheme,
    isIceTheme,
    RAIN_DEFAULTS,
  } from "../../../../core/types";

  interface Props {
    draft: AppTheme;
    onUpdate: (key: keyof AppTheme, value: string | number) => void;
  }

  let { draft, onUpdate }: Props = $props();

  const isRain = $derived(isRainTheme(draft));
  const isFire = $derived(isFireTheme(draft));
  const isIce = $derived(isIceTheme(draft));
  const hasEffect = $derived(isRain || isFire || isIce);

  const density = $derived(draft.rainDensity ?? RAIN_DEFAULTS.density);
  const opacity = $derived(draft.rainOpacity ?? RAIN_DEFAULTS.opacity);
  const speed = $derived(draft.rainSpeed ?? RAIN_DEFAULTS.speed);

  const densityLabel = $derived(
    density <= 40
      ? "Escassa"
      : density < 75
        ? "Leve"
        : density <= 130
          ? "Padrão"
          : density <= 170
            ? "Densa"
            : "Intensa",
  );

  const opacityLabel = $derived(
    opacity <= 20
      ? "Quase invisível"
      : opacity < 50
        ? "Discreta"
        : opacity <= 80
          ? "Média"
          : opacity <= 96
            ? "Padrão"
            : "Total",
  );

  const speedLabel = $derived(
    speed <= 45
      ? "Lenta"
      : speed < 90
        ? "Calma"
        : speed <= 130
          ? "Padrão"
          : speed <= 220
            ? "Rápida"
            : "Turbo",
  );
</script>

{#if hasEffect}
  <div class="h-px bg-[var(--border-subtle)] mb-3"></div>

  <div
    class="mb-3.5 p-2.5 rounded-lg bg-[var(--bg-item)] border border-[var(--border-subtle)] flex flex-col gap-2.5"
  >
    <p
      class="text-[10px] uppercase tracking-wider text-[var(--text-muted)] opacity-90 font-bold"
    >
      {#if isFire}
        Efeito de Animação (Chamas & Brasas)
      {:else if isIce}
        Efeito de Animação (Nevasca & Geada)
      {:else}
        Efeito de Animação (Chuva de Binários)
      {/if}
    </p>

    <!-- Quantidade / Intensidade -->
    <div class="flex flex-col gap-1">
      <div class="flex items-center justify-between text-xs">
        <span
          class="text-[11px] text-[var(--text-base)] font-medium flex items-center gap-1.5"
        >
          <span>
            {#if isFire}
              Intensidade das Chamas
            {:else if isIce}
              Densidade da Nevasca & Geada
            {:else}
              Quantidade de Chuva
            {/if}
          </span>
          <span class="text-[9px] text-[var(--text-faint)] font-mono">
            ({densityLabel})
          </span>
        </span>
        <span class="text-[10px] text-[var(--text-faint)] font-mono">
          {density}%
        </span>
      </div>
      <div class="flex items-center gap-2">
        <span class="text-[10px] text-[var(--text-faint)] font-mono">25</span>
        <input
          type="range"
          min="25"
          max="200"
          step="5"
          class="flex-1 accent-[var(--accent-primary)] cursor-pointer h-2 bg-slate-300 dark:bg-slate-700 border border-[var(--border-subtle)] rounded-lg appearance-none"
          value={density}
          oninput={(e) =>
            onUpdate(
              "rainDensity",
              parseInt((e.target as HTMLInputElement).value, 10),
            )}
        />
        <span class="text-[10px] text-[var(--text-faint)] font-mono">200</span>
      </div>
    </div>

    <div class="h-px bg-[var(--border-subtle)]"></div>

    <!-- Opacidade -->
    <div class="flex flex-col gap-1">
      <div class="flex items-center justify-between text-xs">
        <span
          class="text-[11px] text-[var(--text-base)] font-medium flex items-center gap-1.5"
        >
          <span>
            {#if isFire}
              Opacidade das Chamas
            {:else if isIce}
              Opacidade do Gelo
            {:else}
              Opacidade dos Binários
            {/if}
          </span>
          <span class="text-[9px] text-[var(--text-faint)] font-mono">
            ({opacityLabel})
          </span>
        </span>
        <span class="text-[10px] text-[var(--text-faint)] font-mono">
          {opacity}%
        </span>
      </div>
      <div class="flex items-center gap-2">
        <span class="text-[10px] text-[var(--text-faint)] font-mono">0</span>
        <input
          type="range"
          min="0"
          max="100"
          step="5"
          class="flex-1 accent-[var(--accent-primary)] cursor-pointer h-2 bg-slate-300 dark:bg-slate-700 border border-[var(--border-subtle)] rounded-lg appearance-none"
          value={opacity}
          oninput={(e) =>
            onUpdate(
              "rainOpacity",
              parseInt((e.target as HTMLInputElement).value, 10),
            )}
        />
        <span class="text-[10px] text-[var(--text-faint)] font-mono">100</span>
      </div>
    </div>

    <div class="h-px bg-[var(--border-subtle)]"></div>

    <!-- Velocidade de movimento -->
    <div class="flex flex-col gap-1">
      <div class="flex items-center justify-between text-xs">
        <span
          class="text-[11px] text-[var(--text-base)] font-medium flex items-center gap-1.5"
        >
          <span>Velocidade de Animação</span>
          <span class="text-[9px] text-[var(--text-faint)] font-mono">
            ({speedLabel})
          </span>
        </span>
        <span class="text-[10px] text-[var(--text-faint)] font-mono">
          {speed}%
        </span>
      </div>
      <div class="flex items-center gap-2">
        <span class="text-[10px] text-[var(--text-faint)] font-mono">25</span>
        <input
          type="range"
          min="25"
          max="300"
          step="5"
          class="flex-1 accent-[var(--accent-primary)] cursor-pointer h-2 bg-slate-300 dark:bg-slate-700 border border-[var(--border-subtle)] rounded-lg appearance-none"
          value={speed}
          oninput={(e) =>
            onUpdate(
              "rainSpeed",
              parseInt((e.target as HTMLInputElement).value, 10),
            )}
        />
        <span class="text-[10px] text-[var(--text-faint)] font-mono">300</span>
      </div>
    </div>
  </div>
{/if}
