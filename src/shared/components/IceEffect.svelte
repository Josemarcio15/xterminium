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

      float hash(vec2 p) {
        p = fract(p * vec2(234.34, 435.345));
        p += dot(p, p + 34.23);
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

      // Flocos de neve / cristais com movimento e drift de vento
      float snowflakeLayer(vec2 uv, float speedMult, float scale, float t) {
        vec2 st = uv * scale;
        // Vento lateral e queda vertical suave
        st.x += sin(t * 0.5 * speedMult + st.y * 0.3) * 0.6;
        st.y += t * 1.2 * speedMult;

        vec2 id = floor(st);
        vec2 f = fract(st);

        float h = hash(id);
        if (h < 0.35) return 0.0; // Espaçamento entre partículas

        vec2 center = vec2(0.5) + vec2(sin(h * 6.28 + t * 0.3), cos(h * 6.28 + t * 0.3)) * 0.22;
        float d = length(f - center);

        // Núcleo brilhante e halo translúcido do floco de gelo
        float radius = 0.05 + h * 0.08;
        float flake = smoothstep(radius, radius * 0.2, d);
        float halo = smoothstep(radius * 2.8, 0.0, d) * 0.35;
        return (flake + halo) * (0.6 + 0.4 * sin(h * 10.0 + t * 2.0));
      }

      // Efeito de geada procedural (Frost / Cristais de gelo)
      float frostPattern(vec2 uv, float t) {
        vec2 p = uv * 3.5;
        float n1 = noise(p + vec2(t * 0.02, -t * 0.03));
        float n2 = noise(p * 2.2 - vec2(n1 * 1.5, t * 0.04));
        float frost = pow(n2, 2.2);
        
        // Bordas da tela recebem mais acúmulo de gelo (vinheta gélida)
        vec2 edgeDist = abs(uv - 0.5) * 2.0;
        float borderVignette = pow(length(edgeDist) * 0.65, 2.5);
        return frost * borderVignette;
      }

      void main() {
        vec2 uv = v_uv;
        float t = u_time * (u_speed / 100.0) * 0.9;
        float densityFactor = u_density / 100.0;

        // 3 camadas de profundidade de neve/cristais caindo com parallax
        float layer1 = snowflakeLayer(uv, 0.4, 10.0, t) * 0.6;
        float layer2 = snowflakeLayer(uv, 0.8, 18.0, t) * 0.85;
        float layer3 = snowflakeLayer(uv, 1.3, 26.0, t) * 1.0;

        float totalSnow = (layer1 + layer2 + layer3) * densityFactor;

        // Geada cristalina nas bordas / fundo
        float frost = frostPattern(uv, t) * densityFactor * 0.75;

        // Cores gélidas
        // Azul profundo gélido -> Ciano cristalino -> Branco puro de gelo
        vec3 colDeepIce = vec3(0.01, 0.22, 0.42);
        vec3 colCyanIce = vec3(0.12, 0.68, 0.95);
        vec3 colPureIce = vec3(0.88, 0.97, 1.0);

        vec3 color = mix(colDeepIce, colCyanIce, clamp(frost * 1.4, 0.0, 1.0));
        color = mix(color, colPureIce, clamp(totalSnow * 1.2 + frost * 0.5, 0.0, 1.0));

        // Partículas adicionais de cintilação de gelo
        float alphaSnow = clamp(totalSnow, 0.0, 1.0);
        float alphaFrost = clamp(frost * 0.65, 0.0, 0.8);
        float alpha = (alphaSnow + alphaFrost) * (u_opacity / 100.0);

        gl_FragColor = vec4(color * alpha, alpha * 0.9);
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
      applyOptions: (newOpts: IceOptions) => {
        opts = { ...newOpts };
      },
    };
  }

  function createFallback2DRenderer(
    canvas: HTMLCanvasElement,
    initialOpts: IceOptions,
  ): IceHandle {
    const ctx = canvas.getContext("2d");
    let opts = { ...initialOpts };
    let animationId = 0;

    function draw() {
      if (!ctx) return;
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      ctx.fillStyle = `rgba(0, 180, 216, ${opts.opacity * 0.003})`;
      ctx.fillRect(0, 0, canvas.width, canvas.height);
      animationId = requestAnimationFrame(draw);
    }
    animationId = requestAnimationFrame(draw);

    return {
      dispose: () => {
        cancelAnimationFrame(animationId);
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
