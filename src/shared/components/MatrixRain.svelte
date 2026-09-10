<script lang="ts">
  import { onMount } from "svelte";
  import { configStore } from "../../core/stores/config.svelte";
  import { RAIN_DEFAULTS } from "../../core/types";

  let host = $state<HTMLDivElement | null>(null);

  const CHARS = "01";

  // Dense, crisp digital rain
  const FONT_FAMILY = '"JetBrains Mono", "Fira Code", ui-monospace, monospace';
  const FONT_SIZE = 16;
  const LINE_HEIGHT = 18; // Tight vertical spacing = more rain on screen
  const COL_GAP = 14; // Espaçamento entre colunas com densidade 100%
  const MIN_GAP = 10; // Nunca menor que o glifo, para não sobrepor
  const MAX_GAP = 56;
  const MIN_SPEED = 8; // Rows per second (slowest drop)
  const MAX_SPEED = 28; // Rows per second (fastest drop)
  const MIN_LENGTH = 6;
  const MAX_LENGTH = 26;

  /** Máximo de colunas que o shader acompanha (512 * 10px ≈ 5.120px de largura). */
  const MAX_COLS = 512;
  /** Trocas de bit por segundo — cada glifo tem o seu próprio instante. */
  const CHURN_HZ = 5;

  /** Ajustes de animação controlados pelo modal de tema. */
  interface RainOptions {
    density: number; // % — quantidade de chuva (100 = padrão)
    speed: number; // % — velocidade de queda (100 = padrão)
  }

  interface RainHandle {
    dispose: () => void;
    applyOptions: (opts: RainOptions) => void;
  }

  interface Drop {
    y: number; // Head position, measured in rows (fractional)
    baseSpeed: number; // Rows per second com velocidade 100%
    speed: number; // Rows per second já com o multiplicador
    baseLength: number; // Comprimento base (antes da escala de densidade)
    chars: string[];
    length: number;
    brightness: number;
  }

  // Opções reativas vindas do tema ativo
  const rainDensity = $derived(
    configStore.theme.rainDensity ?? RAIN_DEFAULTS.density,
  );
  const rainSpeed = $derived(
    configStore.theme.rainSpeed ?? RAIN_DEFAULTS.speed,
  );
  const rainOpacity = $derived(
    configStore.theme.rainOpacity ?? RAIN_DEFAULTS.opacity,
  );

  let handle: RainHandle | null = null;

  /** Espaçamento entre colunas para uma densidade (menor = mais chuva). */
  function gapFor(density: number): number {
    return Math.min(MAX_GAP, Math.max(MIN_GAP, COL_GAP * (100 / density)));
  }

  /** Acima de 100% as gotas também ficam mais longas (mais chuva por coluna). */
  function lengthScaleFor(density: number): number {
    return Math.max(1, density / 100);
  }

  onMount(() => {
    if (!host) return;
    const el = host;

    const makeCanvas = () => {
      const c = document.createElement("canvas");
      c.className = "matrix-rain-canvas";
      el.appendChild(c);
      return c;
    };

    let canvas = makeCanvas();
    let created: RainHandle | null = null;

    // --- Caminho principal: GPU (WebGL2) ---
    let gl: WebGL2RenderingContext | null = null;
    try {
      gl = canvas.getContext("webgl2", {
        alpha: false,
        antialias: false,
        depth: false,
        stencil: false,
        powerPreference: "high-performance",
      }) as WebGL2RenderingContext | null;
    } catch {
      gl = null;
    }

    if (gl) {
      created = startWebGLRain(canvas, gl);
      if (!created) {
        // Driver recusou o shader: libera o contexto e refaz o canvas para o fallback
        gl.getExtension("WEBGL_lose_context")?.loseContext();
        canvas.remove();
        canvas = makeCanvas();
      }
    }

    // --- Fallback: CPU (Canvas 2D) ---
    if (!created) created = startCanvasRain(canvas);

    handle = created;
    handle.applyOptions({ density: rainDensity, speed: rainSpeed });
    el.style.opacity = String(rainOpacity / 100);

    return () => {
      handle?.dispose();
      handle = null;
      el.querySelectorAll("canvas").forEach((c) => c.remove());
    };
  });

  // Aplica os controles do tema ao vivo (enquanto o usuário arrasta os sliders)
  $effect(() => {
    if (host) host.style.opacity = String(rainOpacity / 100);
    handle?.applyOptions({ density: rainDensity, speed: rainSpeed });
  });

  // ---------------------------------------------------------------------------
  // Canvas 2D — fallback quando não há WebGL2 (desenho na CPU)
  // ---------------------------------------------------------------------------

  function startCanvasRain(canvas: HTMLCanvasElement): RainHandle {
    const ctx = canvas.getContext("2d");
    if (!ctx) return { dispose: () => {}, applyOptions: () => {} };

    let columns: Drop[] = [];
    let lastTime = 0;
    let raf = 0;
    let density = RAIN_DEFAULTS.density;
    let speedMult = 1;
    let gap = COL_GAP;

    // Render at native device resolution so glyphs stay razor sharp on HiDPI screens
    function setupCanvas() {
      if (!ctx || !canvas) return;
      const dpr = window.devicePixelRatio || 1;
      const w = canvas.offsetWidth;
      const h = canvas.offsetHeight;
      if (!w || !h) return;

      canvas.width = Math.round(w * dpr);
      canvas.height = Math.round(h * dpr);
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      ctx.textBaseline = "top";
      ctx.shadowColor = "transparent";
      ctx.shadowBlur = 0;
    }

    function initColumns() {
      if (!ctx || !canvas) return;
      setupCanvas();

      const width = canvas.offsetWidth;
      const height = canvas.offsetHeight;

      // Clear canvas completely on init
      ctx.fillStyle = "#000000";
      ctx.fillRect(0, 0, width, height);

      const colCount = Math.min(MAX_COLS, Math.max(1, Math.floor(width / gap)));
      columns = [];

      for (let i = 0; i < colCount; i++) {
        columns.push(createDrop(height));
      }
    }

    function createDrop(viewHeight = canvas?.offsetHeight || 600): Drop {
      const scale = lengthScaleFor(density);
      const baseLength =
        MIN_LENGTH + Math.floor(Math.random() * (MAX_LENGTH - MIN_LENGTH));
      const length = Math.max(2, Math.round(baseLength * scale));
      const chars: string[] = [];
      for (let j = 0; j < length; j++) {
        chars.push(CHARS[Math.floor(Math.random() * CHARS.length)]);
      }
      const rowsVisible = viewHeight / LINE_HEIGHT;
      const baseSpeed = MIN_SPEED + Math.random() * (MAX_SPEED - MIN_SPEED);
      return {
        // Stagger the drops across the visible area so rain falls from frame one
        y: -length - Math.random() * rowsVisible,
        baseSpeed,
        speed: baseSpeed * speedMult,
        baseLength,
        chars,
        length,
        brightness: 0.65 + Math.random() * 0.35,
      };
    }

    function draw(timestamp: number) {
      if (!ctx || !canvas) return;

      const dt = Math.min((timestamp - lastTime) / 1000, 0.05);
      lastTime = timestamp;

      const width = canvas.offsetWidth;
      const height = canvas.offsetHeight;

      // Full clear every frame: no ghosting/smearing, so glyphs stay crisp
      ctx.fillStyle = "#000000";
      ctx.fillRect(0, 0, width, height);
      ctx.textBaseline = "top";

      for (let i = 0; i < columns.length; i++) {
        const drop = columns[i];
        const x = Math.round(
          i * gap + Math.max(1, (gap - FONT_SIZE * 0.6) / 2),
        );

        // Frequent glyph churn — the binary flickers like the real thing
        if (Math.random() < 0.08) {
          const mutIdx = Math.floor(Math.random() * drop.length);
          drop.chars[mutIdx] = CHARS[Math.floor(Math.random() * CHARS.length)];
        }

        const lastIdx = drop.length - 1;

        for (let j = 0; j < drop.length; j++) {
          const charY = Math.round((drop.y - (lastIdx - j)) * LINE_HEIGHT);

          if (charY < -LINE_HEIGHT || charY > height + LINE_HEIGHT) continue;

          const distFromHead = lastIdx - j; // 0 = head

          if (distFromHead === 0) {
            // Head: bright white — the leading character
            ctx.font = `bold ${FONT_SIZE}px ${FONT_FAMILY}`;
            ctx.fillStyle = `rgba(230, 255, 230, ${drop.brightness})`;
          } else if (distFromHead <= 2) {
            // Near head: bright green
            ctx.font = `bold ${FONT_SIZE}px ${FONT_FAMILY}`;
            const a = distFromHead === 1 ? 0.95 : 0.8;
            ctx.fillStyle = `rgba(0, 255, 65, ${drop.brightness * a})`;
          } else {
            // Trail: progressively dimmer green toward the tail
            ctx.font = `${FONT_SIZE}px ${FONT_FAMILY}`;
            const fade = 0.15 + 0.65 * (j / Math.max(1, lastIdx));
            ctx.fillStyle = `rgba(0, 255, 70, ${drop.brightness * fade})`;
          }

          ctx.fillText(drop.chars[j], x, charY);
        }

        // Always falling: distance is tied to real elapsed time
        drop.y += drop.speed * dt;

        // Recycle only once the entire trail has left the viewport
        if ((drop.y - drop.length) * LINE_HEIGHT > height) {
          columns[i] = createDrop(height);
        }
      }

      raf = requestAnimationFrame(draw);
    }

    initColumns();
    raf = requestAnimationFrame(draw);

    const ro = new ResizeObserver(() => initColumns());
    ro.observe(canvas);

    return {
      dispose: () => {
        cancelAnimationFrame(raf);
        ro.disconnect();
      },
      applyOptions: (opts) => {
        const densityChanged = opts.density !== density;
        density = opts.density;
        speedMult = opts.speed / 100;
        gap = gapFor(density);

        if (densityChanged) {
          // A densidade muda o espaçamento → reconstrói as colunas
          initColumns();
        } else {
          for (const drop of columns) drop.speed = drop.baseSpeed * speedMult;
        }
      },
    };
  }

  // ---------------------------------------------------------------------------
  // WebGL2 — os glifos são animados inteiramente na GPU (custo de CPU ~zero)
  // ---------------------------------------------------------------------------

  function startWebGLRain(
    canvas: HTMLCanvasElement,
    gl: WebGL2RenderingContext,
  ): RainHandle | null {
    const VERT = `#version 300 es
precision highp float;

#define MAX_COLS ${MAX_COLS}

const vec2 CORNERS[6] = vec2[6](
  vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(0.0, 1.0),
  vec2(0.0, 1.0), vec2(1.0, 0.0), vec2(1.0, 1.0)
);

layout(location = 0) in float a_x;     // coluna (px CSS)
layout(location = 1) in float a_dist;  // 0 = cabeça da gota
layout(location = 2) in float a_len;   // comprimento da gota
layout(location = 3) in float a_bright;
layout(location = 4) in float a_col;   // índice da coluna (a fase vem no uniform)

uniform vec2 u_res;   // tamanho em px CSS
uniform vec2 u_cell;  // tamanho da célula em px CSS
uniform float u_tick; // contador de trocas de bit
uniform float u_phase[MAX_COLS];

out float v_alpha;
out vec3 v_color;
out vec2 v_uv;

float hash(vec2 p) {
  return fract(sin(dot(p, vec2(12.9898, 78.233))) * 43758.5453);
}

void main() {
  vec2 corner = CORNERS[gl_VertexID];
  int col = int(clamp(a_col, 0.0, float(MAX_COLS - 1)));

  // Cabeça da gota em unidades de linha; a trilha fica acima dela
  float headRow = u_phase[col] - a_dist;
  vec2 pos = vec2(a_x + corner.x * u_cell.x, (headRow + corner.y) * u_cell.y);
  gl_Position = vec4(pos.x / u_res.x * 2.0 - 1.0, 1.0 - pos.y / u_res.y * 2.0, 0.0, 1.0);

  float nearHead = 1.0 - step(2.5, a_dist);

  // Cada glifo troca de bit no próprio instante, sem nenhum trabalho de CPU
  float seed = hash(vec2(a_col + 1.0, a_dist + 1.0));
  float churn = mod(floor(u_tick + seed), 512.0);
  float glyph = step(0.5, hash(vec2(a_col + 3.0, a_dist + churn)));

  // Atlas: 0/1 = peso normal, 2/3 = negrito (cabeça e vizinhança)
  float cellIdx = glyph + nearHead * 2.0;
  v_uv = vec2((cellIdx + corner.x) * 0.25, corner.y);

  if (a_dist < 0.5) {
    v_alpha = a_bright;
    v_color = vec3(0.90, 1.0, 0.90);
  } else if (a_dist < 1.5) {
    v_alpha = a_bright * 0.95;
    v_color = vec3(0.0, 1.0, 0.26);
  } else if (a_dist < 2.5) {
    v_alpha = a_bright * 0.80;
    v_color = vec3(0.0, 1.0, 0.26);
  } else {
    float fade = 0.15 + 0.65 * (1.0 - a_dist / max(1.0, a_len - 1.0));
    v_alpha = a_bright * fade;
    v_color = vec3(0.0, 1.0, 0.26);
  }
}`;

    const FRAG = `#version 300 es
precision highp float;

in float v_alpha;
in vec3 v_color;
in vec2 v_uv;
uniform sampler2D u_atlas;
out vec4 outColor;

void main() {
  float mask = texture(u_atlas, v_uv).a;
  outColor = vec4(v_color, mask * v_alpha);
}`;

    const compile = (type: number, src: string): WebGLShader | null => {
      const sh = gl.createShader(type);
      if (!sh) return null;
      gl.shaderSource(sh, src);
      gl.compileShader(sh);
      if (!gl.getShaderParameter(sh, gl.COMPILE_STATUS)) {
        console.error("[MatrixRain] shader:", gl.getShaderInfoLog(sh));
        gl.deleteShader(sh);
        return null;
      }
      return sh;
    };

    const vs = compile(gl.VERTEX_SHADER, VERT);
    const fs = compile(gl.FRAGMENT_SHADER, FRAG);
    const prog = gl.createProgram();
    if (!vs || !fs || !prog) return null;

    gl.attachShader(prog, vs);
    gl.attachShader(prog, fs);
    gl.linkProgram(prog);
    gl.deleteShader(vs);
    gl.deleteShader(fs);

    if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) {
      console.error("[MatrixRain] link:", gl.getProgramInfoLog(prog));
      gl.deleteProgram(prog);
      return null;
    }

    gl.useProgram(prog);

    const uRes = gl.getUniformLocation(prog, "u_res");
    const uCell = gl.getUniformLocation(prog, "u_cell");
    const uTick = gl.getUniformLocation(prog, "u_tick");
    const uPhase = gl.getUniformLocation(prog, "u_phase");
    const uAtlas = gl.getUniformLocation(prog, "u_atlas");
    if (!uRes || !uCell || !uTick || !uPhase || !uAtlas) {
      gl.deleteProgram(prog);
      return null;
    }

    // Atlas com 4 células: "0" e "1" em peso normal e em negrito
    const tex = gl.createTexture();
    const vao = gl.createVertexArray();
    const vbo = gl.createBuffer();
    if (!tex || !vao || !vbo) {
      gl.deleteTexture(tex);
      gl.deleteVertexArray(vao);
      gl.deleteBuffer(vbo);
      gl.deleteProgram(prog);
      return null;
    }

    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.uniform1i(uAtlas, 0);

    function uploadAtlas(dprVal: number) {
      const cellW = Math.max(1, Math.round(gap * dprVal));
      const cellH = Math.max(1, Math.round(LINE_HEIGHT * dprVal));
      const c = document.createElement("canvas");
      c.width = cellW * 4;
      c.height = cellH;
      const g = c.getContext("2d");
      if (g) {
        g.clearRect(0, 0, c.width, c.height);
        g.fillStyle = "#ffffff";
        g.textBaseline = "top";
        const px = Math.round(FONT_SIZE * dprVal);
        for (let i = 0; i < 2; i++) {
          // Centraliza o glifo na célula (colunas estreitas quando a densidade é alta)
          g.font = `${px}px ${FONT_FAMILY}`;
          const wn = g.measureText(CHARS[i]).width;
          g.fillText(CHARS[i], i * cellW + Math.max(0, (cellW - wn) / 2), 0);
          g.font = `bold ${px}px ${FONT_FAMILY}`;
          const wb = g.measureText(CHARS[i]).width;
          g.fillText(
            CHARS[i],
            (i + 2) * cellW + Math.max(0, (cellW - wb) / 2),
            0,
          );
        }
      }
      gl.bindTexture(gl.TEXTURE_2D, tex);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, c);
    }

    // Cada instância é um glifo; os atributos são estáticos (montados no resize)
    const STRIDE = 20;
    gl.bindVertexArray(vao);
    gl.bindBuffer(gl.ARRAY_BUFFER, vbo);
    const attribOffsets = [
      [0, 0],
      [1, 4],
      [2, 8],
      [3, 12],
      [4, 16],
    ] as const;
    for (const [loc, off] of attribOffsets) {
      gl.enableVertexAttribArray(loc);
      gl.vertexAttribPointer(loc, 1, gl.FLOAT, false, STRIDE, off);
      gl.vertexAttribDivisor(loc, 1);
    }
    gl.bindVertexArray(null);

    gl.disable(gl.DEPTH_TEST);
    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);

    const phases = new Float32Array(MAX_COLS);
    const spans = new Float32Array(MAX_COLS);
    const speeds = new Float32Array(MAX_COLS);
    const baseSpeeds = new Float32Array(MAX_COLS);
    const baseLengths = new Float32Array(MAX_COLS);
    const brights = new Float32Array(MAX_COLS);

    let colCount = 0;
    let instanceCount = 0;
    let density = RAIN_DEFAULTS.density;
    let speedMult = 1;
    let gap = COL_GAP;
    let dpr = 1;
    let disposed = false;
    let raf = 0;
    let last = 0;
    let elapsed = 0;

    /**
     * (Re)constrói as gotas. Com `preserve`, as colunas que continuam existindo
     * mantêm posição e estilo — evita "pulos" ao arrastar o slider de densidade.
     */
    function rebuild(preserve: boolean) {
      const w = canvas.offsetWidth;
      const h = canvas.offsetHeight;
      if (!w || !h) return;

      gap = gapFor(density);
      const scale = lengthScaleFor(density);
      const rows = h / LINE_HEIGHT;
      const nextCount = Math.min(MAX_COLS, Math.max(1, Math.floor(w / gap)));

      // Fase relativa atual de cada coluna, para reaproveitar depois do rebuild
      const keepRatios = new Float32Array(nextCount);
      for (let i = 0; i < nextCount; i++) {
        keepRatios[i] = spans[i] > 0 ? phases[i] / spans[i] : Math.random();
      }

      const data: number[] = [];
      for (let i = 0; i < nextCount; i++) {
        const keep = preserve && i < colCount && baseLengths[i] > 0;
        if (!keep) {
          baseLengths[i] =
            MIN_LENGTH + Math.floor(Math.random() * (MAX_LENGTH - MIN_LENGTH));
          baseSpeeds[i] = MIN_SPEED + Math.random() * (MAX_SPEED - MIN_SPEED);
          brights[i] = 0.65 + Math.random() * 0.35;
        }

        const length = Math.max(2, Math.round(baseLengths[i] * scale));
        const span = rows + length + 4;
        spans[i] = span;
        speeds[i] = baseSpeeds[i] * speedMult;
        // Fase espalhada: a chuva já cai desde o primeiro frame
        phases[i] = (keep ? keepRatios[i] : Math.random()) * span;

        const x = Math.round(
          i * gap + Math.max(1, (gap - FONT_SIZE * 0.6) / 2),
        );
        for (let j = 0; j < length; j++) {
          data.push(x, length - 1 - j, length, brights[i], i);
        }
      }

      colCount = nextCount;
      instanceCount = data.length / 5;
      gl.uniform2f(uCell, gap, LINE_HEIGHT);

      gl.bindVertexArray(vao);
      gl.bindBuffer(gl.ARRAY_BUFFER, vbo);
      gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(data), gl.STATIC_DRAW);
      gl.bindVertexArray(null);
    }

    function resize() {
      dpr = Math.min(window.devicePixelRatio || 1, 2);
      const w = canvas.offsetWidth;
      const h = canvas.offsetHeight;
      if (!w || !h) return;

      canvas.width = Math.round(w * dpr);
      canvas.height = Math.round(h * dpr);
      gl.viewport(0, 0, canvas.width, canvas.height);

      uploadAtlas(dpr);
      gl.uniform2f(uRes, w, h);

      rebuild(false);
    }

    function draw(timestamp: number) {
      const dt = Math.min((timestamp - last) / 1000, 0.05);
      last = timestamp;
      elapsed += dt;

      // Fase acumulada na CPU (1 valor por coluna): exata e sem perda de precisão
      for (let i = 0; i < colCount; i++) {
        const p = phases[i] + speeds[i] * dt;
        phases[i] = p >= spans[i] ? p % spans[i] : p;
      }
      gl.uniform1fv(uPhase, phases);
      gl.uniform1f(uTick, (elapsed * CHURN_HZ) % 1024);

      gl.clearColor(0, 0, 0, 1);
      gl.clear(gl.COLOR_BUFFER_BIT);

      if (instanceCount > 0) {
        gl.bindVertexArray(vao);
        gl.drawArraysInstanced(gl.TRIANGLES, 0, 6, instanceCount);
        gl.bindVertexArray(null);
      }

      raf = requestAnimationFrame(draw);
    }

    resize();
    raf = requestAnimationFrame(draw);

    const ro = new ResizeObserver(() => resize());
    ro.observe(canvas);

    // Refaz o atlas quando as fontes terminam de carregar (glifos mais nítidos)
    document.fonts.ready.then(() => {
      if (!disposed) uploadAtlas(dpr);
    });

    return {
      dispose: () => {
        disposed = true;
        cancelAnimationFrame(raf);
        ro.disconnect();
        gl.deleteBuffer(vbo);
        gl.deleteVertexArray(vao);
        gl.deleteTexture(tex);
        gl.deleteProgram(prog);
        gl.getExtension("WEBGL_lose_context")?.loseContext();
      },
      applyOptions: (opts) => {
        if (disposed) return;
        const densityChanged = opts.density !== density;
        density = opts.density;
        speedMult = opts.speed / 100;

        if (densityChanged) {
          // Muda o espaçamento (e o tamanho da célula no atlas) → reconstrói
          dpr = Math.min(window.devicePixelRatio || 1, 2);
          gl.viewport(0, 0, canvas.width, canvas.height);
          uploadAtlas(dpr);
          rebuild(true);
        } else {
          for (let i = 0; i < colCount; i++) {
            speeds[i] = baseSpeeds[i] * speedMult;
          }
        }
      },
    };
  }
</script>

<div class="matrix-rain-host" bind:this={host}></div>

<style>
  .matrix-rain-host {
    position: absolute;
    inset: 0;
    z-index: 0;
    pointer-events: none;
    opacity: 0.9;
    overflow: hidden;
  }

  .matrix-rain-host :global(.matrix-rain-canvas) {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    display: block;
  }
</style>
