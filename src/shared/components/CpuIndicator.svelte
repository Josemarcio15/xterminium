<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  interface CpuTempData {
    tdie?: number;
    max_temp: number;
    avg_temp: number;
    core_temps: number[];
  }

  let cpuUsage = $state(0);
  let cpuTemp = $state<CpuTempData | null>(null);

  // Armazena a temperatura máxima histórica registrada na sessão (se temp atual > max anterior)
  let maxRecordedTemp = $state(0);

  // Cores da barra de Uso de CPU
  const cpuBarColor = $derived(
    cpuUsage < 50
      ? "bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.4)]"
      : cpuUsage < 80
        ? "bg-amber-400 shadow-[0_0_8px_rgba(251,191,36,0.4)]"
        : "bg-rose-500 shadow-[0_0_8px_rgba(244,63,94,0.5)]"
  );

  const cpuDotColor = $derived(
    cpuUsage < 50
      ? "bg-emerald-400"
      : cpuUsage < 80
        ? "bg-amber-400"
        : "bg-rose-500"
  );

  // Temperatura atual de referência (o pico atual entre os núcleos)
  const currentTemp = $derived(cpuTemp?.max_temp ?? 0);

  // Escala de 0-100 °C = 0-100% da altura da barra (mínimo de 4% para visibilidade)
  const tempPercent = $derived(
    Math.min(100, Math.max(4, Math.round(currentTemp)))
  );

  const tempBarColor = $derived(
    currentTemp < 55
      ? "bg-sky-400 shadow-[0_0_8px_rgba(56,189,248,0.4)]"
      : currentTemp < 75
        ? "bg-amber-400 shadow-[0_0_8px_rgba(251,191,36,0.4)]"
        : "bg-rose-500 shadow-[0_0_8px_rgba(244,63,94,0.5)]"
  );

  const tempDotColor = $derived(
    currentTemp < 55
      ? "bg-sky-400"
      : currentTemp < 75
        ? "bg-amber-400"
        : "bg-rose-500"
  );

  async function fetchData() {
    try {
      const usage = await invoke<number>("get_cpu_usage");
      if (typeof usage === "number" && !isNaN(usage)) {
        cpuUsage = Math.min(100, Math.max(0, usage));
      }
    } catch {}

    try {
      const temp = await invoke<CpuTempData | null>("get_cpu_temp");
      cpuTemp = temp;
      if (temp && temp.max_temp > 0) {
        // Se a temperatura atual for maior que a anterior registrada, salva a nova máxima
        if (temp.max_temp > maxRecordedTemp) {
          maxRecordedTemp = temp.max_temp;
        }
      }
    } catch {}
  }

  onMount(() => {
    fetchData();
    const interval = setInterval(fetchData, 2000);
    return () => clearInterval(interval);
  });
</script>

<div class="flex items-center gap-1.5 px-1 py-1 select-none">
  <!-- 1. Barra vertical de USO de CPU -->
  <div class="relative group cursor-default">
    <div
      class="w-[12px] h-[18px] bg-white/10 dark:bg-white/15 rounded-[3px] overflow-hidden p-[1px] border border-white/10 flex flex-col justify-end"
    >
      <div
        class="w-full rounded-[1.5px] transition-all duration-500 ease-out {cpuBarColor}"
        style:height="{Math.min(100, Math.max(6, cpuUsage))}%"
      ></div>
    </div>

    <!-- Tooltip de Uso -->
    <div
      class="absolute top-full right-0 mt-1.5 pointer-events-none opacity-0 scale-95 translate-y-[-2px] group-hover:opacity-100 group-hover:scale-100 group-hover:translate-y-0 transition-all duration-150 ease-out z-[999]"
    >
      <div
        class="flex items-center gap-1.5 px-2 py-1 rounded-md bg-(--bg-panel) border border-(--border-panel) shadow-xl backdrop-blur-md whitespace-nowrap"
      >
        <span class="w-1.5 h-1.5 rounded-full {cpuDotColor}"></span>
        <span class="text-[10px] text-(--text-muted) font-mono tracking-wider font-semibold">USO</span>
        <span class="text-[11px] font-mono font-medium text-(--text-base)">
          {cpuUsage.toFixed(1)}%
        </span>
      </div>
    </div>
  </div>

  <!-- 2. Barra vertical de TEMPERATURA de CPU (se Core Temp estiver ativo) -->
  {#if cpuTemp && cpuTemp.core_temps && cpuTemp.core_temps.length > 0}
    <div class="relative group cursor-default">
      <div
        class="w-[12px] h-[18px] bg-white/10 dark:bg-white/15 rounded-[3px] overflow-hidden p-[1px] border border-white/10 flex flex-col justify-end"
      >
        <div
          class="w-full rounded-[1.5px] transition-all duration-500 ease-out {tempBarColor}"
          style:height="{tempPercent}%"
        ></div>
      </div>

      <!-- Tooltip de Temperatura detalhando a máxima registrada e todos os núcleos -->
      <div
        class="absolute top-full right-0 mt-1.5 pointer-events-none opacity-0 scale-95 translate-y-[-2px] group-hover:opacity-100 group-hover:scale-100 group-hover:translate-y-0 transition-all duration-150 ease-out z-[999]"
      >
        <div
          class="flex flex-col gap-1.5 p-2 rounded-md bg-(--bg-panel) border border-(--border-panel) shadow-2xl backdrop-blur-md whitespace-nowrap min-w-[155px]"
        >
          <!-- Cabeçalho do Tooltip: Temperatura Atual -->
          <div class="flex items-center justify-between gap-3 pb-1 border-b border-(--border-subtle)">
            <div class="flex items-center gap-1.5">
              <span class="w-1.5 h-1.5 rounded-full {tempDotColor}"></span>
              <span class="text-[10px] text-(--text-muted) font-mono tracking-wider font-semibold">TEMP ATUAL</span>
            </div>
            <span class="text-[11px] font-mono font-bold text-(--text-base)">
              {currentTemp.toFixed(1)}°C
            </span>
          </div>

          <!-- Linha com a Máxima Registrada -->
          <div class="flex items-center justify-between gap-3 pb-1 border-b border-(--border-subtle) text-[10px] font-mono">
            <span class="text-(--text-muted) flex items-center gap-1">
              <span>🔥</span>
              <span>MÁX REGISTRADA</span>
            </span>
            <span class="font-bold text-amber-400">
              {maxRecordedTemp.toFixed(1)}°C
            </span>
          </div>

          <!-- Grade de temperaturas por núcleo -->
          <div class="grid grid-cols-2 gap-x-3 gap-y-0.5 font-mono text-[10px] pt-0.5">
            {#each cpuTemp.core_temps as temp, idx}
              <div class="flex items-center justify-between gap-1.5">
                <span class="text-(--text-faint)">C{idx}:</span>
                <span class="font-medium {temp >= 75 ? 'text-rose-400' : temp >= 55 ? 'text-amber-400' : 'text-sky-300'}">
                  {temp.toFixed(0)}°C
                </span>
              </div>
            {/each}
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>
