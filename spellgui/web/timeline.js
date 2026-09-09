"use strict";
// timeline.js — timeline em canvas da GUI Tauri, sobre o canvaskit.
// Portada do prototipo Python (spellcaster/gui/web/timeline.js) e reescrita sobre CK: o kit cuida de
// pan, zoom, selecao, marquee, DPR e dirty-flag; aqui ficam tracks, keyframes, curvas, regua,
// snapping, scrub, in/out e loop. Cores so por token de design/tokens/spellcaster.css.
// Atalhos: design/SHORTCUTS.md (Premiere/Resolve), so os verbos que existem sem engine.
//
// Modelo: cada lane e um array de keyframes do .spell — spec.keys, ou spec.<param> (scale, rot, x...).
// Os keyframes moram em Float64Array/Uint8Array paralelos: o desenho nao aloca por frame e o hit-test
// e por bisect (CK.bisect). A edicao acontece no modelo local e volta pro JSON no commit().
//
// ponytail: sem WebSocket e sem engine ; o transporte e um relogio local e o commit so reescreve o
// JSON em memoria. Trocar por IPC (PRD §4) quando o app Tauri existir.

const CURVES = ["linear", "hold", "in", "out", "inout", "bezier"];
// Mesma matematica do engine (spellcaster/timeline/model.py, spellcore/engine): a curva vale para o
// segmento que CHEGA no keyframe.
const EASE = [
  u => u,
  () => 0,
  u => u * u,
  u => u * (2 - u),
  u => u * u * (3 - 2 * u),
  u => 3 * (1 - u) * (1 - u) * u * 0.42 + 3 * (1 - u) * u * u * 0.58 + u * u * u,
];
const STEPS = [0.04, 0.1, 0.2, 0.5, 1, 2, 5, 10, 15, 30, 60, 120, 300, 600, 1800];
const LANE_PARAMS = ["x", "y", "scale", "rot", "color"];
const SHUTTLE = [1, 2, 4, 8];

const TL = {
  show: null, lanes: [], k: null, clip: null, snaps: null,
  headW: 192, rulerH: 24, rowH: 32,          // multiplos de 8: grade de console (design/PRINCIPIOS.md §3)
  snap: true, cur: -1, drawn: 0,
  t: 0, rate: 0, loop: false, last: 0,
  col: {}, menuEl: null, msgEl: null,
  fps() { return (this.show && this.show.fps) || 30; },
  dur() { return (this.show && this.show.duration) || 60; },
  inT() { return +((this.show && this.show.in) || 0); },
  outT() { return +((this.show && this.show.out) || this.dur()); },
};
window.TL = TL;

const clamp = CK.clamp;
const pad2 = n => (n < 10 ? "0" : "") + n;
const t2x = t => TL.k.toScreen(t);
const x2t = x => TL.k.toWorld(x);
const laneY = i => TL.rulerH + i * TL.rowH - TL.k.view.y;

function tc(t, fps) {
  if (t < 0) t = 0;
  return pad2((t / 3600) | 0) + ":" + pad2(((t / 60) | 0) % 60) + ":" + pad2((t | 0) % 60) + ":" +
         pad2(Math.floor((t % 1) * fps));
}

const isKeys = v => Array.isArray(v) && v.length > 0 && Array.isArray(v[0]) &&
                    v[0].length >= 2 && typeof v[0][0] === "number";

// ---- modelo -------------------------------------------------------------
function mkLane(spec, si, param) {
  const src = (param ? spec[param] : spec.keys) || [];
  const n = src.length;
  const L = { spec, si, param, type: spec.type || "?", n,
              ts: new Float64Array(n || 8), vs: new Float64Array(n || 8),
              cu: new Uint8Array(n || 8), raw: new Array(n || 8),
              vmin: 0, vmax: 255, mute: !!spec.mute, solo: !!spec.solo, rec: false };
  L.name = (spec.name || spec.fixture || spec.file || spec.script || spec.clip ||
            (spec.type || "track") + " u" + (spec.universe || 1) + "/" + (spec.address || 1)) +
           (param ? "." + param : "");
  let mx = 0, mn = 0;
  for (let i = 0; i < n; i++) {
    const kf = src[i], v = kf[1];
    L.ts[i] = +kf[0];
    L.cu[i] = Math.max(0, CURVES.indexOf(kf[2] || "linear"));
    if (typeof v === "number") { L.vs[i] = v; L.raw[i] = null; }
    else { L.raw[i] = v; L.vs[i] = Array.isArray(v) && typeof v[0] === "number" ? v[0] : 0; }
    if (L.vs[i] > mx) mx = L.vs[i];
    if (L.vs[i] < mn) mn = L.vs[i];
  }
  // lane vazia nasce em 0..255 (DMX); param de laser (scale/rot/x/y) nasce em 0..1
  L.vmax = n === 0 ? (LANE_PARAMS.indexOf(param) >= 0 ? 1 : 255) : mx <= 1 ? 1 : mx <= 255 ? 255 : mx;
  L.vmin = Math.min(0, mn);
  return L;
}

TL.load = function (show) {
  TL.show = show;
  TL.lanes = [];
  TL.k.sel.clear();
  TL.cur = -1;
  TL.t = 0;
  TL.rate = 0;
  (show.tracks || []).forEach((spec, si) => {
    TL.lanes.push(mkLane(spec, si, null));                        // lane principal (spec.keys)
    for (const p of Object.keys(spec)) {
      if (p !== "keys" && isKeys(spec[p]) && (LANE_PARAMS.indexOf(p) >= 0 || spec.type === "fixture"))
        TL.lanes.push(mkLane(spec, si, p));
    }
  });
  if (!Array.isArray(show.markers)) show.markers = [];
  TL.fit();
  if (TL.msgEl) TL.msgEl.textContent = (show.name || "sem nome") + "  -  " +
    (show.tracks || []).length + " tracks, " + TL.lanes.length + " lanes";
  return TL.lanes.length;
};

TL.fetch = path => fetch(path).then(r => {
  if (!r.ok) throw new Error(path + ": HTTP " + r.status);
  return r.json();
}).then(TL.load);

// Valor da lane em t, igual ao engine: bisect + curva do keyframe seguinte.
TL.valueAt = function (L, t) {
  if (!L.n) return null;
  const i = CK.bisect(L.ts, L.n, t + 1e-12) - 1;
  if (i < 0) return L.vs[0];
  if (i + 1 >= L.n) return L.vs[i];
  const dt = L.ts[i + 1] - L.ts[i];
  if (dt <= 0) return L.vs[i + 1];
  const e = EASE[L.cu[i + 1]]((t - L.ts[i]) / dt);
  return L.vs[i] + (L.vs[i + 1] - L.vs[i]) * e;
};

function ensure(L, n) {
  if (n <= L.ts.length) return;
  const cap = Math.max(n, L.ts.length * 2);
  const ts = new Float64Array(cap), vs = new Float64Array(cap), cu = new Uint8Array(cap);
  ts.set(L.ts); vs.set(L.vs); cu.set(L.cu);
  L.ts = ts; L.vs = vs; L.cu = cu; L.raw.length = cap;
}

function resort(li) {
  const L = TL.lanes[li], n = L.n, idx = new Array(n);
  for (let i = 0; i < n; i++) idx[i] = i;
  idx.sort((a, b) => L.ts[a] - L.ts[b]);
  let ok = true;
  for (let i = 0; i < n; i++) if (idx[i] !== i) { ok = false; break; }
  if (ok) return;
  const ts = new Float64Array(L.ts.length), vs = new Float64Array(L.ts.length);
  const cu = new Uint8Array(L.ts.length), raw = new Array(L.ts.length);
  const old = TL.k.sel.get(li), ns = old ? new Set() : null;
  for (let i = 0; i < n; i++) {
    const j = idx[i];
    ts[i] = L.ts[j]; vs[i] = L.vs[j]; cu[i] = L.cu[j]; raw[i] = L.raw[j];
    if (old && old.has(j)) ns.add(i);
  }
  L.ts = ts; L.vs = vs; L.cu = cu; L.raw = raw;
  if (old) TL.k.sel.m.set(li, ns);
}

function addKey(li, t, v, raw, cu) {
  const L = TL.lanes[li];
  ensure(L, L.n + 1);
  L.ts[L.n] = t; L.vs[L.n] = v; L.raw[L.n] = raw === undefined ? null : raw; L.cu[L.n] = cu || 0;
  L.n++;
  resort(li);
  return CK.bisect(L.ts, L.n, t);
}

function delSelected() {
  let n = 0;
  TL.k.sel.m.forEach((ks, li) => {
    const L = TL.lanes[li];
    let w = 0;
    for (let i = 0; i < L.n; i++) {
      if (ks.has(i)) { n++; continue; }
      L.ts[w] = L.ts[i]; L.vs[w] = L.vs[i]; L.cu[w] = L.cu[i]; L.raw[w] = L.raw[i]; w++;
    }
    L.n = w;
  });
  TL.k.sel.clear();
  if (n) commit();
  return n;
}

// Devolve o modelo local para o JSON do show. Sem engine ainda: nada sai daqui pela rede.
// ponytail: reescreve o show inteiro em memoria ; virar key_set/key_del por keyframe quando houver IPC.
function commit() {
  for (const L of TL.lanes) {
    const out = new Array(L.n), inteiro = L.vmax === 255;
    for (let i = 0; i < L.n; i++) {
      const v = L.raw[i] !== null ? L.raw[i]
              : inteiro ? Math.round(L.vs[i]) : Math.round(L.vs[i] * 1e4) / 1e4;
      out[i] = [Math.round(L.ts[i] * 1e4) / 1e4, v, CURVES[L.cu[i]]];
    }
    if (L.param) L.spec[L.param] = out; else L.spec.keys = out;
    L.spec.mute = L.mute; L.spec.solo = L.solo;
  }
  TL.k.dirty = true;
}
TL.commit = commit;

// ---- transporte local ---------------------------------------------------
TL.locate = function (t) {
  TL.t = clamp(t, 0, TL.dur());
  TL.k.dirty = true;
};

TL.play = function (rate) {
  TL.rate = rate;
  TL.last = performance.now();
  TL.k.dirty = true;
};

function frame() {
  if (!TL.rate) return;
  const now = performance.now(), dt = (now - TL.last) / 1000;
  TL.last = now;
  let t = TL.t + dt * TL.rate;
  if (TL.loop && TL.rate > 0 && t >= TL.outT()) t = TL.inT();
  if (t <= 0 || t >= TL.dur()) TL.rate = 0;
  TL.locate(t);
}

// ---- snapping (markers, in/out, playhead, keyframes visiveis) ----------
function buildSnaps() {
  const s = [0, TL.dur(), TL.inT(), TL.outT(), TL.t];
  for (const m of TL.show.markers || []) s.push(+m);
  const lo = x2t(TL.headW), hi = x2t(TL.k.w);
  for (let li = 0; li < TL.lanes.length && s.length < 4000; li++) {
    const L = TL.lanes[li], sel = TL.k.sel.get(li);
    for (let i = CK.bisect(L.ts, L.n, lo); i < L.n && L.ts[i] <= hi; i++)
      if (!sel || !sel.has(i)) s.push(L.ts[i]);
  }
  s.sort((a, b) => a - b);
  TL.snaps = s;
  return s;
}
TL.buildSnaps = buildSnaps;

function snapT(t) {
  if (!TL.snap || !TL.snaps || !TL.snaps.length) return t;
  const i = CK.near(TL.snaps, TL.snaps.length, t, 8 / TL.k.view.zoom);
  return i < 0 ? t : TL.snaps[i];
}
TL.snapT = snapT;

// ---- cores (tokens) -----------------------------------------------------
function colors() {
  const cs = getComputedStyle(TL.k.cv);
  const g = (n, d) => cs.getPropertyValue(n).trim() || d;
  TL.col = {
    bg: g("--sc-bg", "#000"), panel: g("--sc-panel", "#0E0E0E"), panel2: g("--sc-panel-2", "#161616"),
    well: g("--sc-well", "#070707"), line: g("--sc-line", "#262626"),
    fg: g("--sc-fg", "#F7F5EB"), fg2: g("--sc-fg-2", "#B5B1A9"), fg3: g("--sc-fg-3", "#666361"),
    accent: g("--sc-accent", "#FFB000"), go: g("--sc-go", "#A8E05E"),
    live: g("--sc-live", "#FF2D1F"), rehearsal: g("--sc-rehearsal", "#B7AED9"),
    mono: g("--sc-mono", "Consolas, monospace"), label: g("--sc-label", "Arial Narrow, sans-serif"),
  };
  TL.k.dirty = true;
}
TL.colors = colors;

// ---- desenho ------------------------------------------------------------
function diamond(c, x, y, r) {
  c.moveTo(x, y - r); c.lineTo(x + r, y); c.lineTo(x, y + r); c.lineTo(x - r, y); c.closePath();
}

function draw(k) {
  const c = k.cx, W = k.w, H = k.h, col = TL.col, rh = TL.rowH, hw = TL.headW;
  const lo = x2t(hw), hi = x2t(W), fps = TL.fps();
  c.fillStyle = col.well;
  c.fillRect(0, 0, W, H);

  const first = Math.max(0, Math.floor(k.view.y / rh));
  const last = Math.min(TL.lanes.length - 1, Math.floor((k.view.y + H) / rh));
  TL.drawn = Math.max(0, last - first + 1);

  // ---- faixas: fundo, cabecalho, M / S / R ----
  c.textBaseline = "middle";
  for (let li = first; li <= last; li++) {
    const y = laneY(li), L = TL.lanes[li];
    c.fillStyle = li & 1 ? col.bg : col.well;
    c.fillRect(hw, y, W - hw, rh);
    c.fillStyle = col.panel;
    c.fillRect(0, y, hw, rh);
    if (li === TL.cur) {                                  // track focado: 1 px no accent (PRINCIPIOS §2)
      c.strokeStyle = col.accent; c.lineWidth = 1;
      c.strokeRect(0.5, y + 0.5, W - 1, rh - 1);
    }
    c.fillStyle = L.mute ? col.fg3 : col.fg;
    c.font = "12px " + col.label;
    c.fillText(L.name.length > 22 ? L.name.slice(0, 21) + "…" : L.name, 8, y + rh / 2 - 5);
    c.fillStyle = col.fg3;
    c.font = "10px " + col.mono;
    c.fillText(L.type + "  " + L.n + "k", 8, y + rh / 2 + 6);
    const flag = [["M", L.mute, col.accent], ["S", L.solo, col.go], ["R", L.rec, col.live]];
    for (let b = 0; b < 3; b++) {
      const [txt, on, cc] = flag[b], bx = hw - 58 + b * 19;
      c.fillStyle = on ? cc : col.bg;
      c.fillRect(bx, y + rh / 2 - 7, 15, 14);
      c.fillStyle = on ? col.bg : col.fg3;
      c.fillText(txt, bx + 4, y + rh / 2);
    }
    c.strokeStyle = col.line; c.lineWidth = 1;
    c.beginPath(); c.moveTo(0, y + rh - 0.5); c.lineTo(W, y + rh - 0.5); c.stroke();
  }

  c.save();
  c.beginPath(); c.rect(hw, TL.rulerH, W - hw, H - TL.rulerH); c.clip();

  // ---- limites do intervalo In/Out (a barra cinza fica na regua; aqui so as duas linhas) ----
  const xi = t2x(TL.inT()), xo = t2x(TL.outT());
  c.strokeStyle = col.fg3; c.lineWidth = 1;
  c.beginPath();
  for (const x of [xi, xo]) { c.moveTo(Math.round(x) + 0.5, TL.rulerH); c.lineTo(Math.round(x) + 0.5, H); }
  c.stroke();

  // ---- grade vertical ----
  let step = STEPS[STEPS.length - 1];
  for (const s of STEPS) if (s * k.view.zoom >= 64) { step = s; break; }
  c.strokeStyle = col.line; c.lineWidth = 1;
  c.beginPath();
  for (let t = Math.ceil(lo / step) * step; t < hi; t += step) {
    const x = Math.round(t2x(t)) + 0.5;
    c.moveTo(x, TL.rulerH); c.lineTo(x, H);
  }
  c.stroke();

  // ---- markers (cinza: SHORTCUTS.md) ----
  const mk = TL.show ? TL.show.markers || [] : [];
  if (mk.length) {
    c.strokeStyle = col.fg3; c.beginPath();
    for (const m of mk) {
      if (m < lo || m > hi) continue;
      const x = Math.round(t2x(m)) + 0.5;
      c.moveTo(x, TL.rulerH); c.lineTo(x, H);
    }
    c.stroke();
  }

  // ---- curva de valor: uma path por lane, amostrada como o engine interpola ----
  const dragLanes = k.drag && k.drag.mode === "keys" ? k.drag.lanes : null;
  c.strokeStyle = col.fg2; c.lineWidth = 1;
  c.beginPath();
  for (let li = first; li <= last; li++) {
    const L = TL.lanes[li], y = laneY(li), sc = (rh - 8) / ((L.vmax - L.vmin) || 1);
    const py = v => y + rh - 4 - (v - L.vmin) * sc;
    const i0 = dragLanes && dragLanes.has(li) ? 0 : Math.max(0, CK.bisect(L.ts, L.n, lo) - 1);
    for (let i = i0; i < L.n; i++) {
      const x = t2x(L.ts[i]);
      if (x > W + 20) break;
      if (i === i0) { c.moveTo(x, py(L.vs[i])); continue; }
      const cu = L.cu[i], x0 = t2x(L.ts[i - 1]);
      if (cu === 1) { c.lineTo(x, py(L.vs[i - 1])); c.lineTo(x, py(L.vs[i])); }
      else if (cu === 0) { c.lineTo(x, py(L.vs[i])); }
      else {
        const n = clamp(Math.round((x - x0) / 6), 2, 24), f = EASE[cu], dv = L.vs[i] - L.vs[i - 1];
        for (let s = 1; s <= n; s++) {
          const u = s / n;
          c.lineTo(x0 + (x - x0) * u, py(L.vs[i - 1] + dv * f(u)));
        }
      }
    }
  }
  c.stroke();

  // ---- keyframes: passe 0 normais, passe 1 selecionados ----
  for (let pass = 0; pass < 2; pass++) {
    c.beginPath();
    for (let li = first; li <= last; li++) {
      const L = TL.lanes[li], y = laneY(li), sc = (rh - 8) / ((L.vmax - L.vmin) || 1);
      const sel = k.sel.get(li);
      if (pass && !sel) continue;
      const i0 = dragLanes && dragLanes.has(li) ? 0 : CK.bisect(L.ts, L.n, lo);
      let lastX = -1e9;
      for (let i = i0; i < L.n; i++) {
        const x = t2x(L.ts[i]);
        if (x > W + 8) break;
        if (x < hw - 8) continue;
        if ((sel ? sel.has(i) : false) !== !!pass) continue;
        // ponytail: passo minimo de 3 px agrupa keyframes colados no zoom-out ; some so o desenho
        if (!pass && x - lastX < 3) continue;
        lastX = x;
        diamond(c, x, y + rh - 4 - (L.vs[i] - L.vmin) * sc, pass ? 5 : 4);
      }
    }
    c.fillStyle = pass ? col.accent : col.fg;
    c.fill();
  }
  c.restore();

  // ---- regua ----
  c.fillStyle = col.panel; c.fillRect(0, 0, W, TL.rulerH);
  c.strokeStyle = col.line; c.lineWidth = 1;
  c.beginPath(); c.moveTo(0, TL.rulerH - 0.5); c.lineTo(W, TL.rulerH - 0.5); c.stroke();
  c.fillStyle = col.fg3; c.font = "10px " + col.mono;
  c.beginPath();
  for (let t = Math.ceil(lo / step) * step; t < hi; t += step) {
    const x = t2x(t);
    if (x < hw) continue;
    c.moveTo(Math.round(x) + 0.5, TL.rulerH - 6); c.lineTo(Math.round(x) + 0.5, TL.rulerH);
    c.fillText(tc(t, fps), x + 3, 9);
  }
  c.strokeStyle = col.fg3; c.stroke();
  for (const m of mk) {
    const x = t2x(m);
    if (x < hw || x > W) continue;
    c.fillStyle = col.fg2;
    c.beginPath(); c.moveTo(x, TL.rulerH - 8); c.lineTo(x + 5, TL.rulerH - 1); c.lineTo(x - 5, TL.rulerH - 1); c.fill();
  }
  const bi = Math.max(hw, xi), bo = Math.min(W, xo);       // intervalo In-Out como barra cinza
  if (bo > bi) { c.fillStyle = col.fg3; c.fillRect(bi, TL.rulerH - 4, bo - bi, 3); }
  for (let b = 0; b < 2; b++) {                            // alcas de In / Out
    const x = b ? xo : xi;
    if (x < hw - 7 || x > W) continue;
    c.fillStyle = col.fg2;
    c.fillRect(b ? x - 7 : x, 1, 7, 8);
  }
  c.fillStyle = col.panel; c.fillRect(0, 0, hw, TL.rulerH);
  c.fillStyle = TL.rate ? col.live : col.fg;
  c.font = "13px " + col.mono;
  c.fillText(tc(TL.t, fps), 8, TL.rulerH / 2);
  c.fillStyle = col.fg3; c.font = "10px " + col.mono;
  c.fillText(TL.lanes.length + "L " + k.sel.count() + "sel" + (TL.snap ? " snap" : ""), 108, TL.rulerH / 2);

  // ---- playhead ----
  const xp = t2x(TL.t);
  if (xp >= hw - 1) {
    c.strokeStyle = TL.rate ? col.live : col.accent; c.lineWidth = 1;
    c.beginPath(); c.moveTo(Math.round(xp) + 0.5, 0); c.lineTo(Math.round(xp) + 0.5, H); c.stroke();
    c.fillStyle = TL.rate ? col.live : col.accent;
    c.beginPath(); c.moveTo(xp - 6, 0); c.lineTo(xp + 6, 0); c.lineTo(xp, 10); c.fill();
  }

  // ---- marquee ----
  const r = k.rect();
  if (r) {
    c.strokeStyle = col.accent; c.setLineDash([4, 3]);
    c.strokeRect(r.x0 + 0.5, r.y0 + 0.5, r.x1 - r.x0, r.y1 - r.y0);
    c.setLineDash([]);
  }
}

// ---- hit-test -----------------------------------------------------------
function laneAt(y) {
  if (y < TL.rulerH) return -1;
  const i = Math.floor((y - TL.rulerH + TL.k.view.y) / TL.rowH);
  return i >= 0 && i < TL.lanes.length ? i : -1;
}
TL.laneAt = laneAt;

// Keyframe sob (x, y) na lane li, ou -1. Bisect no tempo; o Y so desempata.
function keyAt(li, x, y) {
  const L = TL.lanes[li], t = x2t(x), tol = 7 / TL.k.view.zoom;
  const sc = (TL.rowH - 8) / ((L.vmax - L.vmin) || 1), ly = laneY(li);
  let best = -1, bd = 8;
  for (let i = Math.max(0, CK.bisect(L.ts, L.n, t - tol)); i < L.n && L.ts[i] <= t + tol; i++) {
    const d = Math.abs(t2x(L.ts[i]) - x) +
              Math.abs(ly + TL.rowH - 4 - (L.vs[i] - L.vmin) * sc - y) * 0.5;
    if (d < bd) { bd = d; best = i; }
  }
  return best;
}
TL.keyAt = keyAt;

// ---- interacao ----------------------------------------------------------
function onDown(p) {
  const k = TL.k, x = p.x, y = p.y;
  hideMenu();
  if (y < TL.rulerH) {                                    // regua: alcas de In/Out ou scrub
    if (x > TL.headW) {
      const d = t => Math.abs(t2x(t) - x);
      if (d(TL.inT()) < 7) { k.drag = { mode: "in" }; return true; }
      if (d(TL.outT()) < 7) { k.drag = { mode: "out" }; return true; }
      k.drag = { mode: "scrub" };
      TL.locate(x2t(x));
    }
    return true;
  }
  const li = laneAt(y);
  if (li < 0) { if (!p.shift) k.sel.clear(); k.dirty = true; return true; }
  TL.cur = li;
  const L = TL.lanes[li];
  if (x < TL.headW) {                                     // cabecalho: M / S / R
    const ly = laneY(li) + TL.rowH / 2;
    if (Math.abs(y - ly) < 8 && x > TL.headW - 58 && x < TL.headW - 5) {
      const b = Math.floor((x - (TL.headW - 58)) / 19);
      if (b === 0) L.mute = !L.mute;
      else if (b === 1) L.solo = !L.solo;
      else L.rec = !L.rec;   // ponytail: arma so o visual ; ligar no engine quando houver IPC/gravacao
      commit();
    }
    k.sel.clear();
    k.dirty = true;
    return true;
  }
  const ki = keyAt(li, x, y);
  if (ki >= 0) {
    if (p.shift) k.sel.toggle(li, ki);
    else if (!k.sel.has(li, ki)) { k.sel.clear(); k.sel.add(li, ki); }
    buildSnaps();
    const items = [];
    k.sel.each((l, i) => items.push({ li: l, ki: i, t: TL.lanes[l].ts[i], v: TL.lanes[l].vs[i] }));
    k.drag = { mode: "keys", x, y, items, lanes: new Set(k.sel.rows()), moved: false, anchor: L.ts[ki] };
    k.dirty = true;
    return true;
  }
  return false;                                           // vazio: o kit abre o marquee
}

function onMove(p, d) {
  const k = TL.k;
  if (d.mode === "scrub") TL.locate(x2t(p.x));
  else if (d.mode === "in") TL.show.in = clamp(snapT(x2t(p.x)), 0, TL.outT() - 0.01);
  else if (d.mode === "out") TL.show.out = clamp(snapT(x2t(p.x)), TL.inT() + 0.01, TL.dur());
  else if (d.mode === "keys") {
    d.moved = true;
    let dt = (p.x - d.x) / k.view.zoom;
    if (!p.shift) dt = snapT(d.anchor + dt) - d.anchor;   // Shift solta o snapping
    const dy = p.y - d.y;
    for (const it of d.items) {
      const L = TL.lanes[it.li];
      L.ts[it.ki] = Math.max(0, it.t + dt);
      if (L.raw[it.ki] === null && Math.abs(dy) > 1)
        L.vs[it.ki] = clamp(it.v - dy / (TL.rowH - 8) * (L.vmax - L.vmin), L.vmin, L.vmax);
    }
  }
  k.dirty = true;
}

function onUp(p, d) {
  if (d.mode === "keys" && d.moved) { for (const li of d.lanes) resort(li); commit(); }
  else if (d.mode === "in" || d.mode === "out") commit();
}

function onMarquee(r, add) {
  const k = TL.k;
  if (!add) k.sel.clear();
  const t0 = x2t(r.x0), t1 = x2t(r.x1);
  for (let li = 0; li < TL.lanes.length; li++) {
    const ly = laneY(li);
    if (ly + TL.rowH < r.y0 || ly > r.y1) continue;
    const L = TL.lanes[li], sc = (TL.rowH - 8) / ((L.vmax - L.vmin) || 1);
    for (let i = CK.bisect(L.ts, L.n, t0); i < L.n && L.ts[i] <= t1; i++) {
      const ky = ly + TL.rowH - 4 - (L.vs[i] - L.vmin) * sc;
      if (ky >= r.y0 - 4 && ky <= r.y1 + 4) k.sel.add(li, i);
    }
  }
  for (const li of k.sel.rows()) { TL.cur = li; break; }
}

// ---- menu de contexto (easing) -----------------------------------------
function hideMenu() { if (TL.menuEl) TL.menuEl.style.display = "none"; }

function onMenu(p) {
  const li = laneAt(p.y);
  if (li < 0 || !TL.menuEl) return;
  const ki = keyAt(li, p.x, p.y);
  if (ki >= 0 && !TL.k.sel.has(li, ki)) { TL.k.sel.clear(); TL.k.sel.add(li, ki); TL.k.dirty = true; }
  if (!TL.k.sel.count()) return;
  const r = TL.k.cv.getBoundingClientRect();
  TL.menuEl.style.display = "block";
  TL.menuEl.style.left = (r.left + p.x) + "px";
  TL.menuEl.style.top = (r.top + p.y) + "px";
}

function setCurve(name) {
  const ci = CURVES.indexOf(name);
  TL.k.sel.each((li, i) => { TL.lanes[li].cu[i] = ci; });
  hideMenu();
  commit();
}
TL.setCurve = setCurve;

// ---- atalhos (design/SHORTCUTS.md) --------------------------------------
function step(n) { TL.locate(TL.t + n / TL.fps()); }

function jumpKey(dir) {
  const L = TL.lanes[TL.cur];
  if (!L || !L.n) return;
  const i = CK.bisect(L.ts, L.n, TL.t + (dir > 0 ? 1e-6 : -1e-6));
  const j = dir > 0 ? i : i - 1;
  if (j >= 0 && j < L.n) TL.locate(L.ts[j]);
}

function jumpMarker(dir) {
  const m = (TL.show.markers || []).slice().sort((a, b) => a - b);
  const i = CK.bisect(m, m.length, TL.t + (dir > 0 ? 1e-6 : -1e-6));
  const j = dir > 0 ? i : i - 1;
  if (j >= 0 && j < m.length) TL.locate(m[j]);
}

function onKey(e) {
  if (/^(INPUT|SELECT|TEXTAREA)$/.test(e.target.tagName)) return;
  const k = TL.k, key = e.key, code = e.code, ctrl = e.ctrlKey, shift = e.shiftKey, alt = e.altKey;
  let used = true;
  if (code === "Space" && !ctrl) TL.play(TL.rate ? 0 : 1);
  else if ((key === "j" || key === "J") && !ctrl) TL.play(TL.rate < 0 ? -SHUTTLE[Math.min(3, SHUTTLE.indexOf(-TL.rate) + 1)] : -1);
  else if ((key === "k" || key === "K") && !ctrl) TL.play(0);
  else if ((key === "l" || key === "L") && !ctrl) TL.play(TL.rate > 0 ? SHUTTLE[Math.min(3, SHUTTLE.indexOf(TL.rate) + 1)] : 1);
  else if (key === "ArrowLeft" && ctrl && shift) jumpMarker(-1);
  else if (key === "ArrowRight" && ctrl && shift) jumpMarker(1);
  else if (key === "ArrowLeft") step(shift ? -5 : -1);
  else if (key === "ArrowRight") step(shift ? 5 : 1);
  else if (key === "ArrowUp") jumpKey(-1);
  else if (key === "ArrowDown") jumpKey(1);
  else if (key === "Home") TL.locate(0);
  else if (key === "End") TL.locate(TL.dur());
  else if ((key === "i" || key === "I") && alt) { TL.show.in = 0; commit(); }
  else if ((key === "o" || key === "O") && alt) { TL.show.out = TL.dur(); commit(); }
  else if ((key === "x" || key === "X") && alt) { TL.show.in = 0; TL.show.out = TL.dur(); commit(); }
  else if (key === "I" && shift) TL.locate(TL.inT());
  else if (key === "O" && shift) TL.locate(TL.outT());
  else if (key === "i") { TL.show.in = Math.min(TL.t, TL.outT() - 0.01); commit(); }
  else if (key === "o") { TL.show.out = Math.max(TL.t, TL.inT() + 0.01); commit(); }
  else if (ctrl && (key === "l" || key === "L")) TL.loop = !TL.loop;
  else if ((key === "m" || key === "M") && !ctrl) { TL.show.markers.push(+TL.t.toFixed(4)); TL.show.markers.sort((a, b) => a - b); }
  else if (key === "=" || key === "+") k.zoomAt(k.gutter + (k.w - k.gutter) / 2, 1.25);
  else if (key === "-" || key === "_") k.zoomAt(k.gutter + (k.w - k.gutter) / 2, 0.8);
  else if (key === "\\" || (shift && (key === "z" || key === "Z"))) TL.fit();
  else if (ctrl && (key === "k" || key === "K")) {
    if (TL.cur >= 0) {
      const L = TL.lanes[TL.cur], v = TL.valueAt(L, TL.t);
      k.sel.clear();
      k.sel.add(TL.cur, addKey(TL.cur, TL.t, v === null ? 0 : v, null, 0));
      commit();
    }
  } else if (ctrl && shift && (key === "a" || key === "A")) k.sel.clear();
  else if (ctrl && (key === "a" || key === "A")) {
    k.sel.clear();
    for (let li = 0; li < TL.lanes.length; li++) for (let i = 0; i < TL.lanes[li].n; i++) k.sel.add(li, i);
  } else if (ctrl && (key === "c" || key === "C" || key === "x" || key === "X")) {
    let t0 = Infinity;
    const cl = [];
    k.sel.each((li, i) => {
      const L = TL.lanes[li];
      cl.push({ li, t: L.ts[i], v: L.vs[i], raw: L.raw[i], cu: L.cu[i] });
      if (L.ts[i] < t0) t0 = L.ts[i];
    });
    for (const c of cl) c.t -= t0;
    TL.clip = cl;
    if (key === "x" || key === "X") delSelected();
  } else if (ctrl && (key === "v" || key === "V") && TL.clip) {
    k.sel.clear();
    for (const c of TL.clip) {
      if (c.li >= TL.lanes.length) continue;
      k.sel.add(c.li, addKey(c.li, TL.t + c.t, c.v, c.raw, c.cu));
    }
    commit();
  } else if (key === "Delete" || key === "Backspace") delSelected();
  else if (ctrl && shift && (key === "e" || key === "E")) {
    k.sel.each((li, i) => { const L = TL.lanes[li]; L.cu[i] = (L.cu[i] + 1) % CURVES.length; });
    commit();
  } else if (key === "s" && !ctrl) TL.snap = !TL.snap;
  else if (key === "D" && shift && TL.cur >= 0) { const L = TL.lanes[TL.cur]; L.mute = !L.mute; commit(); }
  else if (key === "S" && shift && TL.cur >= 0) { const L = TL.lanes[TL.cur]; L.solo = !L.solo; commit(); }
  else if ((key === "r" || key === "R") && !ctrl && TL.cur >= 0) TL.lanes[TL.cur].rec = !TL.lanes[TL.cur].rec;
  else used = false;
  if (used) { e.preventDefault(); k.dirty = true; }
}
TL.onKey = onKey;

TL.fit = function () {
  TL.k.gutter = TL.headW;
  TL.k.fit(0, TL.dur());
};

// ---- montagem -----------------------------------------------------------
TL.mount = function (cv, opts) {
  opts = opts || {};
  const k = TL.k = CK.attach(cv, draw);
  k.gutter = TL.headW;
  k.on = { down: onDown, move: onMove, up: onUp, marquee: onMarquee, menu: onMenu, frame };
  TL.menuEl = opts.menu || null;
  TL.msgEl = opts.msg || null;
  colors();
  if (TL.menuEl) {
    for (const n of CURVES) {
      const b = document.createElement("button");
      b.textContent = n;
      b.onclick = () => setCurve(n);
      TL.menuEl.appendChild(b);
    }
    const d = document.createElement("button");
    d.textContent = "apagar";
    d.onclick = () => { hideMenu(); delSelected(); };
    TL.menuEl.appendChild(d);
    addEventListener("pointerdown", e => { if (!TL.menuEl.contains(e.target)) hideMenu(); }, true);
  }
  addEventListener("keydown", onKey);
  return k;
};
