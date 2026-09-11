<script lang="ts">
  import { onMount } from "svelte";
  import { configStore } from "../../core/stores/config.svelte";
  import { RAIN_DEFAULTS } from "../../core/types";

  let host = $state<HTMLDivElement | null>(null);

  // Opções reativas vindas do tema ativo
  const iceDensity = $derived(
    configStore.theme.rainDensity ?? RAIN_DEFAULTS.density,
  );
  const iceSpeed = $derived(
    configStore.theme.rainSpeed ?? RAIN_DEFAULTS.speed,
  );
  const iceOpacity = $derived(
    configStore.theme.rainOpacity ?? RAIN_DEFAULTS.opacity,
  );

  interface IceOptions {
    density: number;
    speed: number;
    opacity: number;
  }

  interface IceHandle {
    dispose: () => void;
    applyOptions: (opts: IceOptions) => void;
  }

  let handle: IceHandle | null = null;

  onMount(() => {
    if (!host) return;

    handle = createIceRenderer(host, {
      density: iceDensity,
      speed: iceSpeed,
      opacity: iceOpacity,
    });

    return () => {
      handle?.dispose();
      handle = null;
    };
  });

  $effect(() => {
    const opts = {
      density: iceDensity,
      speed: iceSpeed,
      opacity: iceOpacity,
    };
    handle?.applyOptions(opts);
  });

  function createIceRenderer(
    container: HTMLElement,
    initialOpts: IceOptions,
  ): IceHandle {
    const canvas = document.createElement("canvas");
    canvas.className = "ice-effect-canvas";
    container.appendChild(canvas);

    let opts = { ...initialOpts };
    let gl: WebGL2RenderingContext | WebGLRenderingContext | null =
      canvas.getContext("webgl2", { alpha: true, depth: false, antialias: false, preserveDrawingBuffer: false }) ||
      canvas.getContext("webgl", { alpha: true, depth: false, antialias: false, preserveDrawingBuffer: false });

    if (!gl) {
      return createFallback2DRenderer(canvas, initialOpts);
    }

    // Shaders GLSL para renderização procedural de cristais de gelo, nevasca suave e geada
    const vsSource = `
      attribute vec2 a_pos;
      varying vec2 v_uv;
      void main() {
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
        p = fract(p * vec2(234.34, 435.345));
        p += dot(p, p + 34.23);
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

      // Ruído "ridged": veios finos, que dão cara de cristal de gelo
      float ridged(vec2 p) {
        float v = 0.0;
        float a = 0.5;
        mat2 rot = mat2(0.8, 0.6, -0.6, 0.8);
        for (int i = 0; i < 4; i++) {
          v += a * (1.0 - abs(noise(p) * 2.0 - 1.0));
          p = rot * p * 2.15;
          a *= 0.5;
        }
        return v;
      }

      // Geada: cristais crescendo da moldura para dentro
      float frost(vec2 uv, float aspect, float density) {
        vec2 pa = vec2(uv.x * aspect, uv.y) * 3.1;
        float vein = pow(clamp(ridged(pa), 0.0, 1.0), 2.6);

        // Distância até a borda mais próxima (unidades de altura)
        vec2 d = min(uv, 1.0 - uv);
        float edge = min(d.x * aspect, d.y);
        float mask = 1.0 - smoothstep(0.02, 0.22, edge);

        // Irregularidade: a geada não virá uma faixa uniforme
        float var = 0.45 + 0.75 * noise(vec2(uv.x * aspect, uv.y) * 5.0);
        return vein * mask * var * density;
      }

      // Uma camada de neve caindo. depth: 0 = longe, 1 = perto
      float snowLayer(vec2 uv, float aspect, float scale, float depth, float spawn, float t) {
        vec2 st = vec2(uv.x * aspect, uv.y) * scale;
        // Queda + deriva lateral (as camadas próximas caem e oscilam mais)
        st.y += t * (0.22 + depth * 1.30);
        st.x += sin(t * (0.22 + depth * 0.45) + st.y * 0.18) * (0.45 + depth * 0.85);

        vec2 id = floor(st);
        vec2 f = fract(st) - 0.5;

        float h1 = hash21(id);
        float h2 = hash21(id + 17.31);
        float h3 = hash21(id + 41.77);

        // Nem toda célula tem floco: "spawn" controla a frequência
        if (h3 > spawn) return 0.0;

        // Jitter: o floco não fica no centro exato da célula
        vec2 jit = vec2(h1, h2) - 0.5;
        vec2 q = f - jit * 0.55;

        // Raio em unidades de tela -> convertido para unidades de célula
        float rad = (0.0035 + 0.0035 * h1) * (0.40 + depth * 1.05) * 1.25 * scale;

        float r = length(q);
        float core = smoothstep(rad, rad * 0.15, r);
        float halo = smoothstep(rad * 3.2, 0.0, r) * 0.14;

        // Braços do cristal (apenas nos flocos mais próximos)
        float ang = atan(q.y, q.x);
        float spokes = pow(abs(cos(ang * 3.0)), 10.0) * smoothstep(rad * 3.4, rad * 0.2, r);
        float arms = spokes * step(0.5, depth) * 0.5;

        return (core + arms + halo) * (0.55 + 0.45 * depth);
      }

      void main() {
        vec2 uv = v_uv;
        float aspect = u_resolution.x / max(u_resolution.y, 1.0);
        float density = clamp(u_density / 100.0, 0.05, 2.0);
        float speed = clamp(u_speed / 100.0, 0.0, 3.0);
        float opacity = clamp(u_opacity / 100.0, 0.0, 1.0);
        float t = u_time * speed;

        // Frequência de flocos por camada (densidade do tema = mais neve)
        float freq = 0.35 + 0.75 * density;
        float sp1 = clamp(0.150 * freq, 0.0, 0.9);
        float sp2 = clamp(0.068 * freq, 0.0, 0.9);
        float sp3 = clamp(0.034 * freq, 0.0, 0.9);

        // Três camadas de neve com paralaxe
        float s1 = snowLayer(uv, aspect, 10.0, 0.22, sp1, t) * 0.55;
        float s2 = snowLayer(uv, aspect, 16.0, 0.60, sp2, t) * 0.80;
        float s3 = snowLayer(uv, aspect, 24.0, 1.00, sp3, t) * 1.00;
        float snow = s1 + s2 + s3;

        // Geada reforçada nas bordas
        float fr = frost(uv, aspect, density) * 1.40;

        // Paleta gélida: azul profundo -> ciano cristalino -> branco
        vec3 deepIce = vec3(0.010, 0.170, 0.340);
        vec3 cyanIce = vec3(0.090, 0.540, 0.850);
        vec3 pureIce = vec3(0.870, 0.960, 1.000);
        vec3 snowCol = vec3(0.820, 0.930, 1.000);

        // --- composição pré-multiplicada (over) ---
        // 1) névoa fria de fundo
        float hazeA = 0.088 * density;
        vec3 acc = vec3(0.030, 0.120, 0.240) * hazeA;
        float accA = hazeA;

        // 2) geada nas bordas
        vec3 frostCol = mix(deepIce, cyanIce, clamp(fr * 1.9, 0.0, 1.0));
        frostCol = mix(frostCol, pureIce, clamp(fr * fr * 1.7, 0.0, 1.0));
        float frostA = clamp(fr * 0.85, 0.0, 0.85);
        acc = frostCol * frostA + acc * (1.0 - frostA);
        accA = frostA + accA * (1.0 - frostA);

        // 3) neve caindo
        float snowA = clamp(snow * 1.15, 0.0, 1.0);
        acc = snowCol * snowA + acc * (1.0 - snowA);
        accA = snowA + accA * (1.0 - snowA);

        // Saída pré-multiplicada: o canvas é composto por cima do fundo do app
        acc *= opacity;
        accA *= opacity;
        gl_FragColor = vec4(acc, accA);
      }
    `;

    function compileShader(type: number, src: string) {
      if (!gl) return null;
      const sh = gl.createShader(type);
      if (!sh) return null;
      gl.shaderSource(sh, src);
      gl.compileShader(sh);
      if (!gl.getShaderParameter(sh, gl.COMPILE_STATUS)) {
        console.error("[IceEffect] shader error:", gl.getShaderInfoLog(sh));
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
      console.error("[IceEffect] link error:", gl.getProgramInfoLog(prog));
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
      new Float32Array([
        -1, -1,
         1, -1,
        -1,  1,
        -1,  1,
         1, -1,
         1,  1,
      ]),
      gl.STATIC_DRAW,
    );

    let animationId = 0;
    let startTime = performance.now();
    let dpr = Math.min(window.devicePixelRatio || 1, 2);

    function resize() {
      const rect = container.getBoundingClientRect();
      const w = Math.max(300, Math.floor(rect.width));
      const h = Math.max(200, Math.floor(rect.height));
      if (canvas.width !== Math.floor(w * dpr) || canvas.height !== Math.floor(h * dpr)) {
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

      // O shader devolve cor pré-multiplicada e o buffer é limpo a cada frame
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
      applyOptions: (newOpts: IceOptions) => {
        opts = { ...newOpts };
      },
    };
  }

  // Fallback em caso de indisponibilidade de WebGL: neve e geada vetoriais
  function createFallback2DRenderer(
    canvas: HTMLCanvasElement,
    initialOpts: IceOptions,
  ): IceHandle {
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
      const t = (now - start) * 0.001 * speed;

      ctx.clearRect(0, 0, w, h);

      // Geada acumulada nas bordas
      const frost = ctx.createRadialGradient(
        w / 2,
        h / 2,
        Math.min(w, h) * 0.3,
        w / 2,
        h / 2,
        Math.max(w, h) * 0.62,
      );
      frost.addColorStop(0, "rgba(3, 26, 51, 0)");
      frost.addColorStop(1, `rgba(8, 86, 145, ${0.55 * opacity})`);
      ctx.fillStyle = frost;
      ctx.fillRect(0, 0, w, h);

      // Neve caindo
      const count = Math.round((w / 12) * density);
      ctx.fillStyle = `rgba(210, 238, 255, ${0.9 * opacity})`;
      for (let i = 0; i < count; i++) {
        const r1 = rand(i + 1);
        const r2 = rand(i + 3.7);
        const depth = 0.35 + 0.65 * r1;
        const span = h + 40;
        const x = (r2 * w + Math.sin(t * 0.6 + i) * 18 * depth + w) % w;
        const y = ((t * (30 + 90 * depth) + r1 * span) % span) - 20;
        const rad = 0.7 + 2.4 * depth;
        ctx.beginPath();
        ctx.arc(x, y, rad, 0, Math.PI * 2);
        ctx.fill();
      }

      animationId = requestAnimationFrame(draw);
    }
    animationId = requestAnimationFrame(draw);

    return {
      dispose: () => {
        cancelAnimationFrame(animationId);
        ro.disconnect();
        if (canvas.parentNode) canvas.parentNode.removeChild(canvas);
      },
      applyOptions: (newOpts: IceOptions) => {
        opts = { ...newOpts };
      },
    };
  }
</script>

<div class="ice-effect-host" bind:this={host}></div>

<style>
  .ice-effect-host {
    position: absolute;
    inset: 0;
    z-index: 0;
    pointer-events: none;
    overflow: hidden;
  }

  .ice-effect-host :global(.ice-effect-canvas) {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    display: block;
  }
</style>
