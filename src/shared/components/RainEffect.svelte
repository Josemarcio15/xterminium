<script lang="ts">
  import { onMount } from "svelte";
  import { configStore } from "../../core/stores/config.svelte";
  import { RAIN_DEFAULTS } from "../../core/types";

  let host = $state<HTMLDivElement | null>(null);

  // Opções reativas do tema ativo
  const rainDensity = $derived(
    configStore.theme.rainDensity ?? RAIN_DEFAULTS.density,
  );
  const rainSpeed = $derived(
    configStore.theme.rainSpeed ?? RAIN_DEFAULTS.speed,
  );
  const rainOpacity = $derived(
    configStore.theme.rainOpacity ?? RAIN_DEFAULTS.opacity,
  );

  interface RainOptions {
    density: number;
    speed: number;
    opacity: number;
  }

  interface RainHandle {
    dispose: () => void;
    applyOptions: (opts: RainOptions) => void;
  }

  let handle: RainHandle | null = null;

  onMount(() => {
    if (!host) return;

    handle = createWaterRainRenderer(host, {
      density: rainDensity,
      speed: rainSpeed,
      opacity: rainOpacity,
    });

    return () => {
      handle?.dispose();
      handle = null;
    };
  });

  $effect(() => {
    handle?.applyOptions({
      density: rainDensity,
      speed: rainSpeed,
      opacity: rainOpacity,
    });
  });

  /**
   * Renderizador realista de chuva com:
   * 1. Pingos de chuva em perspectiva e velocidade com parallax (gotas finas ao fundo, gotas longas e velozes à frente)
   * 2. Gotas escorrendo em vidro / janela (condensação fluida)
   * 3. Ondulações/pingos circulares concêntricos (ripples/ondas de água batendo no chão)
   * Feito com Canvas 2D de alta performance e cálculo vetorial suave.
   */
  function createWaterRainRenderer(
    container: HTMLElement,
    initialOpts: RainOptions,
  ): RainHandle {
    const canvas = document.createElement("canvas");
    canvas.className = "water-rain-canvas";
    container.appendChild(canvas);

    const ctx = canvas.getContext("2d", { alpha: true });
    if (!ctx) {
      return { dispose: () => {}, applyOptions: () => {} };
    }

    let opts = { ...initialOpts };
    let animationId = 0;
    let width = 0;
    let height = 0;
    let dpr = 1;

    // Interface para gotas em queda rápida (chuva)
    interface FallingDrop {
      x: number;
      y: number;
      length: number;
      baseSpeed: number;
      thickness: number;
      opacity: number;
      z: number; // 0 (longe) a 1 (perto)
    }

    // Interface para anéis de impacto (ondulações ao pingar)
    interface Ripple {
      x: number;
      y: number;
      radius: number;
      maxRadius: number;
      alpha: number;
      speed: number;
    }

    let fallingDrops: FallingDrop[] = [];
    let ripples: Ripple[] = [];

    function initDrops() {
      fallingDrops = [];
      ripples = [];

      const baseCount = Math.floor((width * height) / 12000);
      const dropCount = Math.max(
        30,
        Math.floor(baseCount * (opts.density / 100)),
      );

      for (let i = 0; i < dropCount; i++) {
        const z = Math.random();
        fallingDrops.push({
          x: Math.random() * (width + 100) - 50,
          y: Math.random() * height,
          length: 12 + z * 28,
          baseSpeed: 7 + z * 11,
          thickness: 0.8 + z * 1.5,
          opacity: 0.2 + z * 0.55,
          z,
        });
      }
    }

    function resize() {
      const rect = container.getBoundingClientRect();
      width = Math.max(300, Math.floor(rect.width));
      height = Math.max(200, Math.floor(rect.height));
      dpr = Math.min(window.devicePixelRatio || 1, 2);

      canvas.width = Math.floor(width * dpr);
      canvas.height = Math.floor(height * dpr);
      ctx?.scale(dpr, dpr);

      initDrops();
    }

    resize();
    const ro = new ResizeObserver(() => resize());
    ro.observe(container);

    let lastTime = performance.now();

    function renderLoop(now: number) {
      if (!ctx) return;
      const dt = Math.min((now - lastTime) / 1000, 0.05);
      lastTime = now;

      ctx.clearRect(0, 0, width, height);

      const globalAlpha = opts.opacity / 100;
      const speedMult = opts.speed / 100;

      // 1. Renderiza os anéis de ondulação no chão (Ripples de impacto)
      ctx.lineWidth = 1.2;
      for (let i = ripples.length - 1; i >= 0; i--) {
        const r = ripples[i];
        r.radius += r.speed * dt * 25 * Math.max(0.3, speedMult);
        r.alpha -= dt * 1.6;

        if (r.alpha <= 0 || r.radius >= r.maxRadius) {
          ripples.splice(i, 1);
          continue;
        }

        ctx.save();
        ctx.beginPath();
        // Círculo achatado simulando perspectiva do chão (elipse)
        ctx.ellipse(r.x, r.y, r.radius, r.radius * 0.35, 0, 0, Math.PI * 2);
        ctx.strokeStyle = `rgba(186, 230, 253, ${r.alpha * globalAlpha * 0.8})`;
        ctx.stroke();
        ctx.restore();
      }

      // 2. Renderiza as gotas de chuva caindo em velocidade
      // Inclinação suave simulando vento calmo (~12 graus)
      const windDrift = 0.22;

      for (let i = 0; i < fallingDrops.length; i++) {
        const d = fallingDrops[i];
        const currentSpeed = d.baseSpeed * speedMult;

        d.y += currentSpeed * dt * 60;
        d.x += currentSpeed * dt * 60 * windDrift;

        // Se chegou ao rodapé ou passou da borda, cria ondulação e reinicia no topo
        if (d.y > height - 10) {
          if (d.z > 0.4 && Math.random() < 0.6) {
            ripples.push({
              x: d.x,
              y: height - Math.random() * 25,
              radius: 1.5,
              maxRadius: 8 + d.z * 16,
              alpha: 0.7 * d.opacity,
              speed: 1.0 + d.z * 1.2,
            });
          }

          d.y = -d.length - Math.random() * 50;
          d.x = Math.random() * (width + 200) - 100;
        }

        // Desenha o risco da gota (gradiente translúcido da cauda para a ponta)
        const dropLen =
          d.length * Math.max(0.5, Math.min(2.0, speedMult * 0.8 + 0.2));
        const headX = d.x;
        const headY = d.y;
        const tailX = d.x - dropLen * windDrift;
        const tailY = d.y - dropLen;

        ctx.beginPath();
        const grad = ctx.createLinearGradient(tailX, tailY, headX, headY);
        grad.addColorStop(0, "rgba(186, 230, 253, 0)");
        grad.addColorStop(
          0.85,
          `rgba(224, 242, 254, ${d.opacity * globalAlpha * 0.65})`,
        );
        grad.addColorStop(
          1,
          `rgba(255, 255, 255, ${d.opacity * globalAlpha * 0.95})`,
        );

        ctx.strokeStyle = grad;
        ctx.lineWidth = d.thickness;
        ctx.lineCap = "round";

        ctx.moveTo(tailX, tailY);
        ctx.lineTo(headX, headY);
        ctx.stroke();
      }

      animationId = requestAnimationFrame(renderLoop);
    }

    animationId = requestAnimationFrame(renderLoop);

    return {
      dispose: () => {
        cancelAnimationFrame(animationId);
        ro.disconnect();
        if (canvas.parentNode) canvas.parentNode.removeChild(canvas);
      },
      applyOptions: (newOpts: RainOptions) => {
        const densityChanged = newOpts.density !== opts.density;
        opts = { ...newOpts };
        if (densityChanged) {
          initDrops();
        }
      },
    };
  }
</script>

<div class="water-rain-host" bind:this={host}></div>

<style>
  .water-rain-host {
    position: absolute;
    inset: 0;
    z-index: 0;
    pointer-events: none;
    overflow: hidden;
  }

  .water-rain-host :global(.water-rain-canvas) {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    display: block;
  }
</style>
