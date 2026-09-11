<script lang="ts">
  import { onMount } from "svelte";
  import { configStore } from "../../core/stores/config.svelte";
  import { RAIN_DEFAULTS } from "../../core/types";

  let host = $state<HTMLDivElement | null>(null);

  // Opções reativas vindas do tema ativo
  const fireDensity = $derived(
    configStore.theme.rainDensity ?? RAIN_DEFAULTS.density,
  );
  const fireSpeed = $derived(
    configStore.theme.rainSpeed ?? RAIN_DEFAULTS.speed,
  );
  const fireOpacity = $derived(
    configStore.theme.rainOpacity ?? RAIN_DEFAULTS.opacity,
  );

  interface FireOptions {
    density: number;
    speed: number;
    opacity: number;
  }

  interface FireHandle {
    dispose: () => void;
    applyOptions: (opts: FireOptions) => void;
  }

  let handle: FireHandle | null = null;

  onMount(() => {
    if (!host) return;

    handle = createFireRenderer(host, {
      density: fireDensity,
      speed: fireSpeed,
      opacity: fireOpacity,
    });

    return () => {
      handle?.dispose();
      handle = null;
    };
  });

  $effect(() => {
    const opts = {
      density: fireDensity,
      speed: fireSpeed,
      opacity: fireOpacity,
    };
    handle?.applyOptions(opts);
  });

  function createFireRenderer(
    container: HTMLElement,
    initialOpts: FireOptions,
  ): FireHandle {
    const canvas = document.createElement("canvas");
    canvas.className = "fire-effect-canvas";
    container.appendChild(canvas);

    let opts = { ...initialOpts };
    let gl: WebGL2RenderingContext | WebGLRenderingContext | null =
      canvas.getContext("webgl2", {
        alpha: true,
        depth: false,
        antialias: false,
        preserveDrawingBuffer: false,
      }) ||
      canvas.getContext("webgl", {
        alpha: true,
        depth: false,
        antialias: false,
        preserveDrawingBuffer: false,
      });

    if (!gl) {
      return createFallback2DRenderer(canvas, initialOpts);
    }

    // Shaders GLSL para renderização procedural de chamas fluidas com Simplex/FBM Noise
    const vsSource = `
      attribute vec2 a_pos;
      varying vec2 v_uv;
      void main() {
        // v_uv.x: 0 (esquerda) a 1 (direita)
        // v_uv.y: 0 (fundo/chão) a 1 (topo da tela)
        v_uv = (a_pos + 1.0) * 0.5;
        gl_Position = vec4(a_pos, 0.0, 1.0);
      }
    `;

    const fsSource = `
      precision highp float;
      varying vec2 v_uv;
      uniform vec2 u_resolution;
      uniform float u_time;
      uniform float u_density;
      uniform float u_speed;
      uniform float u_opacity;

      float hash21(vec2 p) {
        p = fract(p * vec2(123.34, 456.21));
        p += dot(p, p + 45.32);
        return fract(p.x * p.y);
      }

      float noise(vec2 p) {
        vec2 i = floor(p);
        vec2 f = fract(p);
        f = f * f * (3.0 - 2.0 * f);
        float a = hash21(i);
        float b = hash21(i + vec2(1.0, 0.0));
        float c = hash21(i + vec2(0.0, 1.0));
        float d = hash21(i + vec2(1.0, 1.0));
        return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
      }

      // FBM orgânico multicamadas
      float fbm(vec2 p) {
        float v = 0.0;
        float a = 0.5;
        mat2 rot = mat2(0.8, 0.6, -0.6, 0.8);
        for (int i = 0; i < 5; ++i) {
          v += a * noise(p);
          p = rot * p * 2.03;
          a *= 0.5;
        }
        return v;
      }

      // Rampa térmica: vermelho escuro -> laranja -> amarelo -> branco
      vec3 fireColor(float h) {
        vec3 col = vec3(0.26, 0.02, 0.0);
        col = mix(col, vec3(0.88, 0.12, 0.02), smoothstep(0.00, 0.15, h));
        col = mix(col, vec3(1.00, 0.45, 0.05), smoothstep(0.15, 0.34, h));
        col = mix(col, vec3(1.00, 0.80, 0.24), smoothstep(0.34, 0.58, h));
        col = mix(col, vec3(1.00, 0.97, 0.85), smoothstep(0.58, 0.88, h));
        return col;
      }

      void main() {
        vec2 uv = v_uv;
        float aspect = u_resolution.x / max(u_resolution.y, 1.0);

        float speed = clamp(u_speed / 100.0, 0.0, 3.0);
        float density = clamp(u_density / 100.0, 0.05, 2.0);
        float opacity = clamp(u_opacity / 100.0, 0.0, 1.0);

        // O fogo tem cadência própria: bem mais lento que a chuva
        float t = u_time * speed * 0.9;

        // Altura da zona de chamas (densidade = labaredas mais altas)
        float maxH = 0.32 + 0.36 * density;
        // y normalizado: 0 na base, 1 no topo da zona de chamas
        float y = uv.y / maxH;

        vec2 p = vec2(uv.x * aspect, uv.y);

        // As pontas oscilam ("lamber") e a base fica firme
        float sway = mix(0.30, 1.0, clamp(y, 0.0, 1.0));
        float w1 = fbm(vec2(p.x * 1.20, uv.y * 1.70 - t * 0.55)) - 0.5;
        float w2 = fbm(vec2(p.x * 2.60, uv.y * 3.40 - t * 1.05) + 31.7) - 0.5;
        float xw = p.x + (w1 + w2 * 0.40) * 0.32 * sway;

        // Campo de chamas: três escalas de ruído alongado na vertical.
        // A escala x é bem maior que a y -> filamentos verticais (línguas).
        float n1 = fbm(vec2(xw * 4.50, y * 2.20 - t * 0.90));
        float n2 = fbm(vec2(xw * 9.90, y * 4.84 - t * 1.50) + 17.3);
        float n3 = fbm(vec2(xw * 19.80, y * 9.68 - t * 2.20) + 71.9);
        float f = n1 * 0.55 + n2 * 0.29 + n3 * 0.16;
        // Contraste: separa as línguas e escava os vazios entre elas
        f = clamp((f - 0.36) * 2.50, 0.0, 1.0);

        // Cintilância global: o fogo "respira"
        float flicker = 0.94 + 0.06 * noise(vec2(t * 0.7, 3.1));

        // Calor: forte na base e sumindo com a altura -> línguas de alturas variadas
        float heat = clamp((f * 1.95 - 1.10 * y) * flicker, 0.0, 1.0);

        // Núcleo incandescente junto à base
        float core = smoothstep(0.40, 0.0, y);
        heat = clamp(heat + core * 0.22 * f, 0.0, 1.0);

        vec3 col = fireColor(pow(heat, 0.88));
        float alpha = smoothstep(0.05, 0.38, heat);

        // Leito incandescente: brasa viva unindo as chamas na base
        float bedMask = smoothstep(0.13, 0.0, uv.y);
        float bed = bedMask * bedMask * (0.35 + 0.65 * fbm(vec2(xw * 3.0, t * 0.30)));
        col += vec3(1.0, 0.50, 0.15) * bed * 0.50;
        alpha = max(alpha, bed * 0.5);

        // Brasas/pardais subindo
        vec2 sp = vec2(uv.x * aspect * 26.0, uv.y * 20.0 - t * 4.5);
        sp.x += sin(sp.y * 0.8 + t * 1.8) * 1.3;
        vec2 spCell = floor(sp);
        float spHash = hash21(spCell);
        float spOn = step(0.975, spHash);
        float spDist = length(fract(sp) - 0.5);
        float ember = smoothstep(0.32, 0.0, spDist) * spOn
                    * smoothstep(1.0, 0.35, uv.y) * smoothstep(0.0, 0.06, uv.y);
        col += vec3(1.0, 0.7, 0.25) * ember * 1.2;
        alpha = clamp(alpha + ember * 0.85, 0.0, 1.0);

        // Saída pré-multiplicada: o canvas é composto por cima do fundo do app
        alpha *= opacity;
        gl_FragColor = vec4(col * alpha, alpha);
      }
    `;

    function compileShader(type: number, src: string) {
      if (!gl) return null;
      const sh = gl.createShader(type);
      if (!sh) return null;
      gl.shaderSource(sh, src);
      gl.compileShader(sh);
      if (!gl.getShaderParameter(sh, gl.COMPILE_STATUS)) {
        console.error("[FireEffect] shader error:", gl.getShaderInfoLog(sh));
        gl.deleteShader(sh);
        return null;
      }
      return sh;
    }

    const vs = compileShader(gl.VERTEX_SHADER, vsSource);
    const fs = compileShader(gl.FRAGMENT_SHADER, fsSource);
    if (!vs || !fs) return createFallback2DRenderer(canvas, initialOpts);

    const prog = gl.createProgram();
    if (!prog) return createFallback2DRenderer(canvas, initialOpts);
    gl.attachShader(prog, vs);
    gl.attachShader(prog, fs);
    gl.linkProgram(prog);

    if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) {
      console.error("[FireEffect] link error:", gl.getProgramInfoLog(prog));
      return createFallback2DRenderer(canvas, initialOpts);
    }

    const aPosLoc = gl.getAttribLocation(prog, "a_pos");
    const uResLoc = gl.getUniformLocation(prog, "u_resolution");
    const uTimeLoc = gl.getUniformLocation(prog, "u_time");
    const uDensityLoc = gl.getUniformLocation(prog, "u_density");
    const uSpeedLoc = gl.getUniformLocation(prog, "u_speed");
    const uOpacityLoc = gl.getUniformLocation(prog, "u_opacity");

    const posBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, posBuffer);
    gl.bufferData(
      gl.ARRAY_BUFFER,
      new Float32Array([-1, -1, 1, -1, -1, 1, -1, 1, 1, -1, 1, 1]),
      gl.STATIC_DRAW,
    );

    let animationId = 0;
    let startTime = performance.now();
    let dpr = Math.min(window.devicePixelRatio || 1, 2);

    function resize() {
      const rect = container.getBoundingClientRect();
      const w = Math.max(300, Math.floor(rect.width));
      const h = Math.max(200, Math.floor(rect.height));
      if (
        canvas.width !== Math.floor(w * dpr) ||
        canvas.height !== Math.floor(h * dpr)
      ) {
        canvas.width = Math.floor(w * dpr);
        canvas.height = Math.floor(h * dpr);
        if (gl) gl.viewport(0, 0, canvas.width, canvas.height);
      }
    }

    resize();
    const ro = new ResizeObserver(() => resize());
    ro.observe(container);

    function renderLoop(now: number) {
      if (!gl) return;
      const elapsed = (now - startTime) * 0.001;

      gl.useProgram(prog);

      // O fragment shader já devolve cor pré-multiplicada e o buffer é limpo a
      // cada frame; sem blending evita-se acúmulo (fogo estourado em branco).
      gl.disable(gl.BLEND);
      gl.clearColor(0, 0, 0, 0);
      gl.clear(gl.COLOR_BUFFER_BIT);

      gl.bindBuffer(gl.ARRAY_BUFFER, posBuffer);
      gl.enableVertexAttribArray(aPosLoc);
      gl.vertexAttribPointer(aPosLoc, 2, gl.FLOAT, false, 0, 0);

      gl.uniform2f(uResLoc, canvas.width, canvas.height);
      gl.uniform1f(uTimeLoc, elapsed);
      gl.uniform1f(uDensityLoc, opts.density);
      gl.uniform1f(uSpeedLoc, opts.speed);
      gl.uniform1f(uOpacityLoc, opts.opacity);

      gl.drawArrays(gl.TRIANGLES, 0, 6);
      animationId = requestAnimationFrame(renderLoop);
    }

    animationId = requestAnimationFrame(renderLoop);

    return {
      dispose: () => {
        cancelAnimationFrame(animationId);
        ro.disconnect();
        if (canvas.parentNode) canvas.parentNode.removeChild(canvas);
      },
      applyOptions: (newOpts: FireOptions) => {
        opts = { ...newOpts };
      },
    };
  }

  // Fallback em caso de indisponibilidade de WebGL: labaredas vetoriais simples
  function createFallback2DRenderer(
    canvas: HTMLCanvasElement,
    initialOpts: FireOptions,
  ): FireHandle {
    const ctx = canvas.getContext("2d");
    let opts = { ...initialOpts };
    let animationId = 0;
    const start = performance.now();

    function resize() {
      const rect = canvas.getBoundingClientRect();
      canvas.width = Math.max(300, Math.floor(rect.width));
      canvas.height = Math.max(200, Math.floor(rect.height));
    }
    resize();
    const ro = new ResizeObserver(() => resize());
    ro.observe(canvas);

    // Ruído estável por índice (sem Math.random, evita cintilação caótica)
    function rand(i: number): number {
      const x = Math.sin(i * 12.9898) * 43758.5453;
      return x - Math.floor(x);
    }

    function draw(now: number) {
      if (!ctx) return;
      const w = canvas.width;
      const h = canvas.height;
      const density = Math.max(0.05, opts.density / 100);
      const speed = Math.max(0.05, opts.speed / 100);
      const opacity = Math.max(0, Math.min(1, opts.opacity / 100));
      const t = (now - start) * 0.001 * speed * 0.9;
      const hMax = h * Math.min(0.8, 0.32 + 0.36 * density);

      ctx.clearRect(0, 0, w, h);
      ctx.globalCompositeOperation = "lighter";

      const count = Math.max(4, Math.round((w / 26) * density));
      for (let i = 0; i < count; i++) {
        const r1 = rand(i + 1);
        const r2 = rand(i + 7.3);
        const cx = ((i + 0.5) / count + Math.sin(t * 0.8 + i) * 0.012) * w;
        const flick =
          0.72 + 0.28 * Math.abs(Math.sin(t * (1.1 + r1) + i * 1.7));
        const fh = hMax * (0.4 + 0.6 * r2) * flick;
        const fw = Math.max(10, (w / count) * 0.5);
        const grad = ctx.createRadialGradient(
          cx,
          h,
          0,
          cx,
          h,
          Math.max(fw, fh),
        );
        grad.addColorStop(0, `rgba(255, 245, 210, ${0.85 * opacity})`);
        grad.addColorStop(0.22, `rgba(255, 176, 48, ${0.6 * opacity})`);
        grad.addColorStop(0.55, `rgba(226, 66, 12, ${0.3 * opacity})`);
        grad.addColorStop(1, "rgba(120, 12, 0, 0)");
        ctx.fillStyle = grad;
        ctx.beginPath();
        ctx.ellipse(cx, h, fw, fh, 0, 0, Math.PI * 2);
        ctx.fill();
      }

      ctx.globalCompositeOperation = "source-over";
      animationId = requestAnimationFrame(draw);
    }
    animationId = requestAnimationFrame(draw);

    return {
      dispose: () => {
        cancelAnimationFrame(animationId);
        ro.disconnect();
        if (canvas.parentNode) canvas.parentNode.removeChild(canvas);
      },
      applyOptions: (newOpts: FireOptions) => {
        opts = { ...newOpts };
      },
    };
  }
</script>

<div class="fire-effect-host" bind:this={host}></div>

<style>
  .fire-effect-host {
    position: absolute;
    inset: 0;
    z-index: 0;
    pointer-events: none;
    overflow: hidden;
  }

  .fire-effect-host :global(.fire-effect-canvas) {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    display: block;
  }
</style>
