// Monitor de saida: cada universo vira uma grade 16 x 32 (512 canais) com VU por canal, alimentada
// pelo topico binario "dmx". O buffer de cada universo e alocado uma vez; o desenho so pinta
// retangulos com cores de uma LUT de 256 entradas — nada e alocado por frame.
"use strict";

(function () {
  const COLS = 32, ROWS = 16, PADX = 14, HEAD = 16;
  const unis = new Map();                       // numero -> {data, peak}
  let cv, cx, box, dirty = true, lut = null, lut2 = null, W = 0, H = 0, dpr = 1, seen = 0;

  function rgb(css) {
    const m = css.trim();
    if (m[0] === "#") {
      const h = m.length === 4 ? m[1] + m[1] + m[2] + m[2] + m[3] + m[3] : m.slice(1, 7);
      return [parseInt(h.slice(0, 2), 16), parseInt(h.slice(2, 4), 16), parseInt(h.slice(4, 6), 16)];
    }
    const n = m.match(/\d+/g);
    return n ? [+n[0], +n[1], +n[2]] : [242, 195, 0];
  }

  function palette() {
    const cs = getComputedStyle(cv);
    const a = rgb(cs.getPropertyValue("--accent") || "#f2c300");
    const b = rgb(cs.getPropertyValue("--accent2") || "#b4d435");
    lut = new Array(256);
    lut2 = "rgb(" + b[0] + "," + b[1] + "," + b[2] + ")";
    for (let v = 0; v < 256; v++) {
      const f = 0.10 + 0.90 * (v / 255);
      lut[v] = "rgb(" + ((a[0] * f) | 0) + "," + ((a[1] * f) | 0) + "," + ((a[2] * f) | 0) + ")";
    }
    dirty = true;
  }

  function feed(u, data) {
    let s = unis.get(u);
    if (!s) unis.set(u, s = { data: new Uint8Array(512), peak: new Float32Array(512) });
    s.data.set(data.length === 512 ? data : data.subarray(0, 512));
    seen = performance.now();
    dirty = true;
  }

  function resize() {
    const r = cv.getBoundingClientRect();
    if (r.width < 1) return;                                                   // painel escondido
    dpr = window.devicePixelRatio || 1;
    W = Math.max(200, r.width | 0);
    H = Math.max(120, r.height | 0);
    cv.width = (W * dpr) | 0;
    cv.height = (H * dpr) | 0;
    cx.setTransform(dpr, 0, 0, dpr, 0, 0);
    dirty = true;
  }

  function draw() {
    const cs = getComputedStyle(cv);
    const bg = cs.getPropertyValue("--bg").trim(), muted = cs.getPropertyValue("--muted").trim();
    const text = cs.getPropertyValue("--text").trim();
    cx.clearRect(0, 0, W, H);
    cx.font = "11px Consolas, monospace";
    cx.textBaseline = "top";
    if (!unis.size) {
      cx.fillStyle = muted;
      cx.fillText("sem frames: de play (o topico dmx so anda com o player rodando)", 4, 8);
      return;
    }
    // celula cresce para ocupar o painel: com poucos universos a grade fica legivel de longe
    const perRow = Math.ceil(Math.sqrt(unis.size)), nrow = Math.ceil(unis.size / perRow);
    const CW = Math.max(4, Math.min(24, ((W - 8) / perRow - PADX) / COLS)) | 0;
    const CH = Math.max(4, Math.min(CW, ((H - 8) / nrow - HEAD - 10) / ROWS)) | 0;
    const bw = COLS * CW + PADX, bh = ROWS * CH + HEAD + 10;
    let i = 0;
    for (const [u, s] of unis) {
      const ox = (i % perRow) * bw, oy = ((i / perRow) | 0) * bh;
      i++;
      if (oy > H) break;
      cx.fillStyle = text;
      cx.fillText("U" + u, ox, oy);
      const d = s.data, pk = s.peak;
      for (let ch = 0; ch < 512; ch++) {
        const v = d[ch];
        const x = ox + (ch % COLS) * CW, y = oy + HEAD + ((ch / COLS) | 0) * CH;
        cx.fillStyle = v ? lut[v] : bg;
        cx.fillRect(x, y, CW - 1, CH - 1);
        const p = pk[ch] = v > pk[ch] ? v : pk[ch] * 0.94;
        if (p > 8 && p > v + 4) {                       // marca de pico (VU) acima do valor atual
          cx.fillStyle = lut2;
          cx.fillRect(x, y, CW - 1, 1);
        }
      }
    }
  }

  function tick() {
    if (performance.now() - seen < 1500) dirty = true;   // deixa o pico decair depois do ultimo frame
    if (dirty && cv.offsetParent !== null) { dirty = false; draw(); }
    requestAnimationFrame(tick);
  }

  // O registro espera o DOMContentLoaded: App.route() usa elementos que app.js so pega la.
addEventListener("DOMContentLoaded", () => App.panel("outputs", {
    mount(el) {
      el.innerHTML = '<div class="out-bar">Universos: grade 16 x 32 = 512 canais. Brilho = valor, ' +
                     'risco no topo = pico.<span class="tl-msg" id="out-msg"></span></div>' +
                     '<canvas id="out-cv"></canvas>';
      cv = el.querySelector("#out-cv");
      cx = cv.getContext("2d");
      box = el.querySelector("#out-msg");
      palette();
      resize();
      new ResizeObserver(resize).observe(cv);
      document.addEventListener("skin", palette);
      App.on("dmx", (u, data) => {
        feed(u, data);
        if (box) box.textContent = unis.size + " universos";
      });
      requestAnimationFrame(tick);
    },
  }));
})();
