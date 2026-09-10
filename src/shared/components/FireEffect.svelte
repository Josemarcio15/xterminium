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
      canvas.getContext("webgl2", { alpha: true, depth: false, antialias: false, preserveDrawingBuffer: false }) ||
      canvas.getContext("webgl", { alpha: true, depth: false, antialias: false, preserveDrawingBuffer: false });

    if (!gl) {
      return createFallback2DRenderer(canvas, initialOpts);
    }

    // Shaders GLSL para renderização procedural de chamas fluidas com Simplex/FBM Noise
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

      // Funções de ruído procedural para chamas orgânicas
      float hash(vec2 p) {
        p = fract(p * vec2(123.34, 456.21));
        p += dot(p, p + 45.32);
        return fract(p.x * p.y);
      }

      float noise(vec2 p) {
        vec2 i = floor(p);
        vec2 f = fract(p);
        f = f * f * (3.0 - 2.0 * f);
        float a = hash(i);
        float b = hash(i + vec2(1.0, 0.0));
        float c = hash(i + vec2(0.0, 1.0));
        float d = hash(i + vec2(1.0, 1.0));
        return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
      }

      float fbm(vec2 p) {
        float v = 0.0;
        float a = 0.5;
        vec2 shift = vec2(100.0);
        mat2 rot = mat2(cos(0.5), sin(0.5), -sin(0.5), cos(0.5));
        for (int i = 0; i < 4; ++i) {
          v += a * noise(p);
          p = rot * p * 2.0 + shift;
          a *= 0.5;
        }
        return v;
      }

      void main() {
        vec2 uv = v_uv;
        // Chamas sobem a partir de baixo (y = 0 até 1)
        float t = u_time * (u_speed / 100.0) * 1.8;
        
        // Distorção das ondas de calor
        vec2 q = uv * vec2(4.0, 2.5);
        q.y -= t * 1.4;
        
        float n1 = fbm(q + vec2(0.0, -t * 0.5));
        vec2 r = vec2(
          fbm(q + 4.0 * n1 + vec2(1.7, 9.2) + 0.15 * t),
          fbm(q + 4.0 * n1 + vec2(8.3, 2.8) + 0.126 * t)
        );
        
        float f = fbm(q + 4.0 * r);
        
        // Gradiente vertical da chama: mais intensa embaixo, dissipa em cima
        // Invertemos UV para a base ser o rodapé
        float flameHeight = 1.0 - uv.y;
        float intensityFactor = (u_density / 100.0);
        
        // Forma da base das labaredas
        float c = f * 1.8 * pow(flameHeight, 1.4) * intensityFactor;
        c = clamp(c, 0.0, 1.5);
        
        // Paleta térmica de fogo
        // Preto -> Vermelho Escuro -> Laranja Queimado -> Amarelo Ouro -> Branco incandescente
        vec3 colDarkRed = vec3(0.35, 0.02, 0.01);
        vec3 colOrange  = vec3(1.0, 0.28, 0.02);
        vec3 colYellow  = vec3(1.0, 0.82, 0.15);
        vec3 colWhite   = vec3(1.0, 0.98, 0.85);

        vec3 color = mix(colDarkRed, colOrange, smoothstep(0.1, 0.45, c));
        color = mix(color, colYellow, smoothstep(0.45, 0.8, c));
        color = mix(color, colWhite, smoothstep(0.8, 1.25, c));

        // Partículas adicionais de brasas cintilantes flutuando
        vec2 sparkUV = uv * vec2(18.0, 12.0);
        sparkUV.y -= t * 3.2;
        float sparkNoise = hash(floor(sparkUV));
        float sparkDist = length(fract(sparkUV) - vec2(0.5, 0.5));
        float sparks = step(0.965, sparkNoise) * (1.0 - smoothstep(0.0, 0.35, sparkDist)) * (1.0 - uv.y * 0.7);
        color += sparks * vec3(1.0, 0.75, 0.3) * 1.5;

        float alpha = smoothstep(0.05, 0.35, c) * (u_opacity / 100.0);
        alpha = clamp(alpha + sparks * 0.6, 0.0, 1.0) * (1.0 - uv.y * 0.4);

        gl_FragColor = vec4(color * alpha, alpha * 0.85);
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
      gl.enable(gl.BLEND);
      gl.blendFunc(gl.ONE, gl.ONE_MINUS_SRC_ALPHA);

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

  // Fallback em caso de indisponibilidade de WebGL
  function createFallback2DRenderer(
    canvas: HTMLCanvasElement,
    initialOpts: FireOptions,
  ): FireHandle {
    const ctx = canvas.getContext("2d");
    let opts = { ...initialOpts };
    let animationId = 0;
    let t = 0;

    function draw() {
      if (!ctx) return;
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      ctx.fillStyle = `rgba(255, 87, 34, ${opts.opacity * 0.003})`;
      ctx.fillRect(0, canvas.height * 0.6, canvas.width, canvas.height * 0.4);
      animationId = requestAnimationFrame(draw);
    }
    animationId = requestAnimationFrame(draw);

    return {
      dispose: () => {
        cancelAnimationFrame(animationId);
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
