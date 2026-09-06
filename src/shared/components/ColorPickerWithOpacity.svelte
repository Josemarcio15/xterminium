<script lang="ts">
  import { parseColorWithOpacity, formatColorWithOpacity } from '../../core/utils/color';

  interface Props {
    value: string;
    allowOpacity?: boolean;
    label?: string;
    onChange: (color: string) => void;
  }

  let { value = '#000000', allowOpacity = true, label, onChange }: Props = $props();

  const parsed = $derived(parseColorWithOpacity(value));

  function handleColorInput(e: Event) {
    const input = e.target as HTMLInputElement;
    const newHex = input.value;
    const newColor = allowOpacity 
      ? formatColorWithOpacity(newHex, parsed.opacity)
      : newHex;
    onChange(newColor);
  }

  function handleOpacityInput(e: Event) {
    const input = e.target as HTMLInputElement;
    const newOpacity = parseInt(input.value, 10) || 0;
    const newColor = formatColorWithOpacity(parsed.hex, newOpacity);
    onChange(newColor);
  }
</script>

<div class="flex items-center gap-2">
  <!-- Swatch Preview com padrão xadrez atrás caso haja transparência -->
  <div 
    class="w-6 h-6 rounded-md border border-[var(--border-panel)] shrink-0 shadow-xs relative overflow-hidden bg-checkerboard"
    title={value}
  >
    <div 
      class="w-full h-full" 
      style="background-color: {value};"
    ></div>
  </div>

  <!-- Color Input nativo (hex 6 dígitos) -->
  <input
    type="color"
    class="w-8 h-6 rounded cursor-pointer border border-[var(--border-panel)] bg-transparent shrink-0"
    value={parsed.hex}
    oninput={handleColorInput}
    aria-label={label || "Cor"}
  />

  {#if allowOpacity}
    <!-- Slider de Opacidade Embutido -->
    <div class="flex items-center gap-1.5 shrink-0 bg-[var(--bg-item-input)] px-2 py-0.5 rounded-md border border-[var(--border-subtle)]">
      <span class="text-[9px] text-[var(--text-faint)] font-mono select-none">α</span>
      <input
        type="range"
        min="0"
        max="100"
        step="1"
        value={parsed.opacity}
        class="w-14 h-1.5 accent-violet-500 cursor-pointer bg-slate-300 dark:bg-slate-700 rounded-lg"
        oninput={handleOpacityInput}
        title="Opacidade: {parsed.opacity}%"
      />
      <span class="text-[9px] font-mono text-[var(--text-muted)] w-6 text-right select-none">
        {parsed.opacity}%
      </span>
    </div>
  {/if}
</div>

<style>
  .bg-checkerboard {
    background-image: linear-gradient(45deg, #2a2d3d 25%, transparent 25%), 
                      linear-gradient(-45deg, #2a2d3d 25%, transparent 25%), 
                      linear-gradient(45deg, transparent 75%, #2a2d3d 75%), 
                      linear-gradient(-45deg, transparent 75%, #2a2d3d 75%);
    background-size: 8px 8px;
    background-position: 0 0, 0 4px, 4px -4px, -4px 0px;
    background-color: #171926;
  }
</style>
