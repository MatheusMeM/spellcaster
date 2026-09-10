// Canvas timeline (Premiere/After Effects/Chataigne): ruler with timecode, zoom, pan, playhead with
// scrub, tracks with mute/solo, diamond keyframes, multiple selection, dragging in time and value,
// copy/paste, per-keyframe easing, snapping, in/out and loop, markers coming from the `markers` command.
//
// Model: each lane is an array of .spell keyframes -- spec.keys, or spec.<param> (scale, rot, dim...).
// The keyframes live in parallel Float64Array/Uint8Array: the drawing allocates nothing per frame and the
// hit test is by bisect. Editing happens in the local model and goes to the engine through show_set (one
// single command, debounced by 250 ms).
// ponytail: commit sends the whole show ; use key_set/key_del per keyframe if a show ever passes ~1 MB.
"use strict";

const CURVES = ["linear", "hold", "in", "out", "inout", "bezier"];
const STEPS = [0.04, 0.1, 0.2, 0.5, 1, 2, 5, 10, 15, 30, 60, 120, 300, 600, 1800];
const LANE_PARAMS = ["x", "y", "scale", "rot", "color"];   // laser params; fixture comes in through the same test

const TL = {
  show: null, lanes: [], sel: new Map(), clip: null,
  t0: 0, pxs: 40, scrollY: 0, headW: 190, rulerH: 26, rowH: 26,
  snap: true, dirty: true, drag: null, marquee: null, cur: -1,
  cv: null, cx: null, w: 0, h: 0, dpr: 1, col: {}, bench: null,
  fps() { return (this.show && this.show.fps) || 30; },
  dur() { return (this.show && this.show.duration) || 60; },
  inT() { return +((this.show && this.show.in) || 0); },
  outT() { return +((this.show && this.show.out) || this.dur()); },
};
window.TL = TL;

// ---- utilities ----------------------------------------------------------
const t2x = t => TL.headW + (t - TL.t0) * TL.pxs;
const x2t = x => TL.t0 + (x - TL.headW) / TL.pxs;
const laneY = i => TL.rulerH + i * TL.rowH - TL.scrollY;
const clamp = (v, a, b) => (v < a ? a : v > b ? b : v);
const pad2 = n => (n < 10 ? "0" : "") + n;

function tc(t, fps) {
  if (t < 0) t = 0;
  return pad2((t / 3600) | 0) + ":" + pad2(((t / 60) | 0) % 60) + ":" + pad2((t | 0) % 60) + ":" +
         pad2(Math.floor((t % 1) * fps));
}

function bisect(ts, n, t) {                       // first index with ts[i] >= t
  let lo = 0, hi = n;
  while (lo < hi) { const m = (lo + hi) >> 1; if (ts[m] < t) lo = m + 1; else hi = m; }
  return lo;
}

const isKeys = v => Array.isArray(v) && v.length > 0 && Array.isArray(v[0]) &&
                    v[0].length >= 2 && typeof v[0][0] === "number";

// ---- model --------------------------------------------------------------
function mkLane(spec, si, param) {
  const src = (param ? spec[param] : spec.keys) || [];
  const n = src.length;
  const L = { spec, si, param, type: spec.type || "?", n,
              ts: new Float64Array(n || 8), vs: new Float64Array(n || 8),
              cu: new Uint8Array(n || 8), raw: new Array(n || 8),
              vmin: 0, vmax: 255, mute: !!spec.mute, solo: !!spec.solo };
  L.name = (spec.name || spec.fixture || spec.file || spec.clip ||
            (spec.type || "track") + " u" + (spec.universe || 1) + "/" + (spec.address || 1)) +
           (param ? "." + param : "");
  let mx = 0, mn = 0;
  for (let i = 0; i < n; i++) {
    const k = src[i], v = k[1];
    L.ts[i] = +k[0];
    L.cu[i] = Math.max(0, CURVES.indexOf(k[2] || "linear"));
    if (typeof v === "number") { L.vs[i] = v; L.raw[i] = null; }
    else { L.raw[i] = v; L.vs[i] = Array.isArray(v) && typeof v[0] === "number" ? v[0] : 0; }
    if (L.vs[i] > mx) mx = L.vs[i];
    if (L.vs[i] < mn) mn = L.vs[i];
  }
  // an empty lane starts at 0..255 (DMX); a laser param (scale/rot/x/y) starts at 0..1
  L.vmax = n === 0 ? (LANE_PARAMS.indexOf(param) >= 0 ? 1 : 255) : mx <= 1 ? 1 : mx <= 255 ? 255 : mx;
  L.vmin = Math.min(0, mn);
  return L;
}

function build(show) {
  TL.show = show;
  TL.sel.clear();
  TL.cur = -1;
  const lanes = TL.lanes = [];
  (show.tracks || []).forEach((spec, si) => {
    lanes.push(mkLane(spec, si, null));                       // main lane (spec.keys)
    for (const k of Object.keys(spec)) {
      if (k !== "keys" && isKeys(spec[k]) && (LANE_PARAMS.indexOf(k) >= 0 || spec.type === "fixture"))
        lanes.push(mkLane(spec, si, k));
    }
  });
  if (show.duration) { App.transport.dur = show.duration; document.getElementById("scrub").max = show.duration; }
  TL.dirty = true;
  if (window.Inspector) Inspector.update();
}

function ensure(L, n) {
  if (n <= L.ts.length) return;
  const cap = Math.max(n, L.ts.length * 2), ts = new Float64Array(cap), vs = new Float64Array(cap), cu = new Uint8Array(cap);
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
  const ts = new Float64Array(L.ts.length), vs = new Float64Array(L.ts.length), cu = new Uint8Array(L.ts.length);
  const raw = new Array(L.ts.length), old = TL.sel.get(li), ns = old ? new Set() : null;
  for (let i = 0; i < n; i++) {
    const j = idx[i];
    ts[i] = L.ts[j]; vs[i] = L.vs[j]; cu[i] = L.cu[j]; raw[i] = L.raw[j];
    if (old && old.has(j)) ns.add(i);
  }
  L.ts = ts; L.vs = vs; L.cu = cu; L.raw = raw;
  if (old) TL.sel.set(li, ns);
}

function addKey(li, t, v, raw, cu) {
  const L = TL.lanes[li];
  ensure(L, L.n + 1);
  L.ts[L.n] = t; L.vs[L.n] = v; L.raw[L.n] = raw === undefined ? null : raw; L.cu[L.n] = cu || 0;
  L.n++;
  resort(li);
  return bisect(L.ts, L.n, t);
}

function delSelected() {
  let n = 0;
  for (const [li, ks] of TL.sel) {
    const L = TL.lanes[li];
    let w = 0;
    for (let i = 0; i < L.n; i++) {
      if (ks.has(i)) { n++; continue; }
      L.ts[w] = L.ts[i]; L.vs[w] = L.vs[i]; L.cu[w] = L.cu[i]; L.raw[w] = L.raw[i]; w++;
    }
    L.n = w;
  }
  TL.sel.clear();
  if (n) commit();
  return n;
}

let commitTimer = 0;
function commit() {
  TL.dirty = true;
  for (const L of TL.lanes) {
    const out = new Array(L.n), whole = L.vmax === 255;
    for (let i = 0; i < L.n; i++) {
      const v = L.raw[i] !== null ? L.raw[i] : whole ? Math.round(L.vs[i]) : Math.round(L.vs[i] * 1e4) / 1e4;
      out[i] = [Math.round(L.ts[i] * 1e4) / 1e4, v, CURVES[L.cu[i]]];
    }
    if (L.param) L.spec[L.param] = out; else L.spec.keys = out;
    L.spec.mute = L.mute; L.spec.solo = L.solo;
  }
  clearTimeout(commitTimer);
  commitTimer = setTimeout(() => {
    App.rpc("show_set", { data: JSON.stringify(TL.show) }).catch(e => App.log("show_set: " + e.message));
  }, 250);
  if (window.Inspector) Inspector.update();
}

// ---- selection ----------------------------------------------------------
TL.commit = commit;                                  // inspector.js edits the model and asks for the commit here
TL.resort = resort;

const selHas = (li, ki) => { const s = TL.sel.get(li); return !!s && s.has(ki); };
function selAdd(li, ki) { let s = TL.sel.get(li); if (!s) TL.sel.set(li, s = new Set()); s.add(ki); }
function selDel(li, ki) { const s = TL.sel.get(li); if (s) { s.delete(ki); if (!s.size) TL.sel.delete(li); } }
function selCount() { let n = 0; for (const s of TL.sel.values()) n += s.size; return n; }
TL.selCount = selCount;

// ---- snapping -----------------------------------------------------------
function buildSnaps() {
  const s = [0, TL.dur(), TL.inT(), TL.outT(), App.now()];
  for (const m of TL.show.markers || []) s.push(+m);
  const lo = TL.t0, hi = x2t(TL.w);
  for (let li = 0; li < TL.lanes.length && s.length < 4000; li++) {
    const L = TL.lanes[li], sel = TL.sel.get(li);
    for (let i = bisect(L.ts, L.n, lo); i < L.n && L.ts[i] <= hi; i++) if (!sel || !sel.has(i)) s.push(L.ts[i]);
  }
  s.sort((a, b) => a - b);
  TL.snaps = s;
}

function snapT(t) {
  if (!TL.snap || !TL.snaps) return t;
  const s = TL.snaps, i = bisect(s, s.length, t), tol = 8 / TL.pxs;
  let best = t, d = tol;
  for (let j = Math.max(0, i - 1); j <= Math.min(s.length - 1, i); j++) {
    const dd = Math.abs(s[j] - t);
    if (dd < d) { d = dd; best = s[j]; }
  }
  return best;
}

// ---- drawing ------------------------------------------------------------
function colors() {
  const cs = getComputedStyle(TL.cv);
  const g = n => cs.getPropertyValue(n).trim();
  TL.col = { bg: g("--bg") || "#0a0a0a", panel: g("--panel") || "#141414", chrome: g("--chrome") || "#1c1c1c",
             accent: g("--accent") || "#f2c300", accent2: g("--accent2") || "#b4d435",
             text: g("--text") || "#eee", muted: g("--muted") || "#888", font: g("--font") || "sans-serif" };
  TL.dirty = true;
}

function diamond(c, x, y, r) {
  c.moveTo(x, y - r); c.lineTo(x + r, y); c.lineTo(x, y + r); c.lineTo(x - r, y); c.closePath();
}

function draw() {
  const c = TL.cx, W = TL.w, H = TL.h, col = TL.col, rh = TL.rowH, hw = TL.headW;
  const lo = TL.t0, hi = x2t(W), fps = TL.fps();
  c.fillStyle = col.panel; c.fillRect(0, 0, W, H);

  // ---- lanes (alternating background + name + mute/solo) ----
  const first = Math.max(0, Math.floor((TL.scrollY) / rh));
  const last = Math.min(TL.lanes.length - 1, Math.floor((TL.scrollY + H) / rh));
  c.font = "11px " + col.font;
  c.textBaseline = "middle";
  for (let li = first; li <= last; li++) {
    const y = laneY(li), L = TL.lanes[li];
    c.fillStyle = li & 1 ? col.bg : col.panel;
    c.fillRect(0, y, W, rh);
    c.fillStyle = col.chrome; c.fillRect(0, y, hw, rh);
    if (li === TL.cur) { c.fillStyle = col.accent; c.globalAlpha = .13; c.fillRect(0, y, W, rh); c.globalAlpha = 1; }
    c.fillStyle = L.mute ? col.muted : col.text;
    c.fillText(L.name.length > 20 ? L.name.slice(0, 19) + "…" : L.name, 8, y + rh / 2 - 5);
    c.fillStyle = col.muted;
    c.fillText(L.type + "  " + L.n + "k", 8, y + rh / 2 + 6);
    for (let b = 0; b < 2; b++) {                                   // M and S
      const on = b ? L.solo : L.mute, bx = hw - 40 + b * 20;
      c.fillStyle = on ? (b ? col.accent2 : col.accent) : col.bg;
      c.fillRect(bx, y + rh / 2 - 7, 16, 14);
      c.fillStyle = on ? col.bg : col.muted;
      c.fillText(b ? "S" : "M", bx + 5, y + rh / 2);
    }
    c.strokeStyle = col.bg; c.lineWidth = 1;
    c.beginPath(); c.moveTo(0, y + rh - .5); c.lineTo(W, y + rh - .5); c.stroke();
  }

  // ---- in/out region ----
  c.save();
  c.beginPath(); c.rect(hw, TL.rulerH, W - hw, H - TL.rulerH); c.clip();
  const xi = t2x(TL.inT()), xo = t2x(TL.outT());
  c.fillStyle = col.bg; c.globalAlpha = .45;
  if (xi > hw) c.fillRect(hw, TL.rulerH, xi - hw, H);
  if (xo < W) c.fillRect(xo, TL.rulerH, W - xo, H);
  c.globalAlpha = 1;

  // ---- vertical grid ----
  let step = STEPS[STEPS.length - 1];
  for (const s of STEPS) if (s * TL.pxs >= 64) { step = s; break; }
  c.strokeStyle = col.bg; c.lineWidth = 1;
  c.beginPath();
  for (let t = Math.ceil(lo / step) * step; t < hi; t += step) {
    const x = Math.round(t2x(t)) + .5;
    c.moveTo(x, TL.rulerH); c.lineTo(x, H);
  }
  c.stroke();

  // ---- markers ----
  const mk = TL.show ? TL.show.markers || [] : [];
  if (mk.length) {
    c.strokeStyle = col.accent2; c.globalAlpha = .55; c.beginPath();
    for (const m of mk) { if (m < lo || m > hi) continue; const x = Math.round(t2x(m)) + .5; c.moveTo(x, TL.rulerH); c.lineTo(x, H); }
    c.stroke(); c.globalAlpha = 1;
  }

  // ---- keyframes ----
  const dragLanes = TL.drag && TL.drag.mode === "keys" ? TL.drag.lanes : null;
  c.strokeStyle = col.muted; c.lineWidth = 1;
  c.beginPath();                                                     // value line (a single path)
  for (let li = first; li <= last; li++) {
    const L = TL.lanes[li], y = laneY(li), h = rh - 8, sc = h / ((L.vmax - L.vmin) || 1);
    const i0 = dragLanes && dragLanes.has(li) ? 0 : Math.max(0, bisect(L.ts, L.n, lo) - 1);
    let started = false, px = 0, py = 0;
    for (let i = i0; i < L.n; i++) {
      const x = t2x(L.ts[i]);
      if (x > W + 20) break;
      const yy = y + rh - 4 - (L.vs[i] - L.vmin) * sc;
      if (!started) { c.moveTo(x, yy); started = true; }
      else { if (L.cu[i] === 1) c.lineTo(x, py); c.lineTo(x, yy); }
      px = x; py = yy;
    }
  }
  c.stroke();

  for (let pass = 0; pass < 2; pass++) {                             // 0 = normal, 1 = selected
    c.beginPath();
    for (let li = first; li <= last; li++) {
      const L = TL.lanes[li], y = laneY(li), h = rh - 8, sc = h / ((L.vmax - L.vmin) || 1);
      const sel = TL.sel.get(li);
      if (pass && !sel) continue;
      const i0 = dragLanes && dragLanes.has(li) ? 0 : bisect(L.ts, L.n, lo);
      let lastX = -1e9;
      for (let i = i0; i < L.n; i++) {
        const x = t2x(L.ts[i]);
        if (x > W + 8) break;
        if (x < hw - 8) continue;
        const on = sel ? sel.has(i) : false;
        if (on !== !!pass) continue;
        // ponytail: a 3 px minimum step groups keyframes packed together when zoomed out ; only the drawing goes, the data stays
        if (!pass && x - lastX < 3) continue;
        lastX = x;
        diamond(c, x, y + rh - 4 - (L.vs[i] - L.vmin) * sc, pass ? 5 : 4);
      }
    }
    c.fillStyle = pass ? col.accent : col.text;
    c.fill();
  }
  c.restore();

  // ---- ruler ----
  c.fillStyle = col.chrome; c.fillRect(0, 0, W, TL.rulerH);
  c.strokeStyle = col.muted; c.lineWidth = 1; c.globalAlpha = .5;
  c.beginPath(); c.moveTo(0, TL.rulerH - .5); c.lineTo(W, TL.rulerH - .5); c.stroke(); c.globalAlpha = 1;
  c.fillStyle = col.muted; c.font = "10px Consolas, monospace";
  c.beginPath();
  for (let t = Math.ceil(lo / step) * step; t < hi; t += step) {
    const x = t2x(t);
    if (x < TL.headW) continue;
    c.moveTo(Math.round(x) + .5, TL.rulerH - 6); c.lineTo(Math.round(x) + .5, TL.rulerH);
    c.fillText(tc(t, fps), x + 3, 9);
  }
  c.strokeStyle = col.muted; c.stroke();
  for (const m of mk) {                                              // markers on the ruler
    const x = t2x(m);
    if (x < TL.headW || x > W) continue;
    c.fillStyle = col.accent2;
    c.beginPath(); c.moveTo(x, TL.rulerH - 8); c.lineTo(x + 5, TL.rulerH - 1); c.lineTo(x - 5, TL.rulerH - 1); c.fill();
  }
  for (let b = 0; b < 2; b++) {                                      // in/out handles
    const x = b ? xo : xi;
    if (x < TL.headW - 6 || x > W) continue;
    c.fillStyle = col.accent2;
    c.fillRect(b ? x - 7 : x, 1, 7, 8);
  }
  c.fillStyle = col.chrome; c.fillRect(0, 0, TL.headW, TL.rulerH);
  c.fillStyle = col.muted; c.font = "10px " + col.font;
  c.fillText(TL.lanes.length + " tracks  " + selCount() + " sel", 8, TL.rulerH / 2);

  // ---- playhead ----
  const xp = t2x(App.now());
  if (xp >= TL.headW - 1) {
    c.strokeStyle = col.accent; c.lineWidth = 1;
    c.beginPath(); c.moveTo(Math.round(xp) + .5, 0); c.lineTo(Math.round(xp) + .5, H); c.stroke();
    c.fillStyle = col.accent;
    c.beginPath(); c.moveTo(xp - 6, 0); c.lineTo(xp + 6, 0); c.lineTo(xp, 10); c.fill();
  }

  // ---- marquee ----
  if (TL.marquee) {
    const m = TL.marquee;
    c.strokeStyle = col.accent; c.setLineDash([4, 3]);
    c.strokeRect(Math.min(m.x0, m.x1) + .5, Math.min(m.y0, m.y1) + .5, Math.abs(m.x1 - m.x0), Math.abs(m.y1 - m.y0));
    c.setLineDash([]);
  }
}

function resize() {
  const r = TL.cv.getBoundingClientRect();
  if (r.width < 1) return;                                                     // panel hidden: do not touch the scale
  const w = Math.max(200, r.width | 0);
  TL.dpr = window.devicePixelRatio || 1;
  if (TL.w > 200 && w !== TL.w) TL.pxs *= (w - TL.headW) / (TL.w - TL.headW);   // keep the time window
  TL.w = w; TL.h = Math.max(120, r.height | 0);
  TL.cv.width = (TL.w * TL.dpr) | 0; TL.cv.height = (TL.h * TL.dpr) | 0;
  TL.cx.setTransform(TL.dpr, 0, 0, TL.dpr, 0, 0);
  TL.dirty = true;
}

// ---- hit test -----------------------------------------------------------
function laneAt(y) {
  if (y < TL.rulerH) return -1;
  const i = Math.floor((y - TL.rulerH + TL.scrollY) / TL.rowH);
  return i >= 0 && i < TL.lanes.length ? i : -1;
}

function keyAt(li, x, y) {
  const L = TL.lanes[li], t = x2t(x), tol = 7 / TL.pxs;
  const sc = (TL.rowH - 8) / ((L.vmax - L.vmin) || 1), ly = laneY(li);
  let best = -1, bd = 8;
  for (let i = Math.max(0, bisect(L.ts, L.n, t - tol)); i < L.n && L.ts[i] <= t + tol; i++) {
    const d = Math.abs(t2x(L.ts[i]) - x) + Math.abs(ly + TL.rowH - 4 - (L.vs[i] - L.vmin) * sc - y) * .5;
    if (d < bd) { bd = d; best = i; }
  }
  return best;
}

// ---- interaction --------------------------------------------------------
let spaceDown = false;

function onDown(e) {
  const x = e.offsetX, y = e.offsetY;
  hideMenu();
  TL.cv.focus();
  if (e.button === 1 || spaceDown) {
    TL.drag = { mode: "pan", x, y, t0: TL.t0, sy: TL.scrollY };
    return e.preventDefault();
  }
  if (e.button === 2) return;
  if (y < TL.rulerH) {                                          // ruler: in/out or scrub
    if (x > TL.headW) {
      const d = t => Math.abs(t2x(t) - x);
      if (d(TL.inT()) < 7) { TL.drag = { mode: "in" }; return; }
      if (d(TL.outT()) < 7) { TL.drag = { mode: "out" }; return; }
      TL.drag = { mode: "scrub" };
      App.locate(clamp(x2t(x), 0, TL.dur()));
      TL.dirty = true;
    }
    return;
  }
  const li = laneAt(y);
  if (li < 0) { if (!e.shiftKey) { TL.sel.clear(); TL.dirty = true; } return; }
  TL.cur = li;
  const L = TL.lanes[li];
  if (x < TL.headW) {                                           // name column: M / S
    const ly = laneY(li) + TL.rowH / 2;
    if (Math.abs(y - ly) < 8 && x > TL.headW - 42 && x < TL.headW - 4) {
      if (x < TL.headW - 22) L.mute = !L.mute; else L.solo = !L.solo;
      commit();
    }
    TL.sel.clear();
    TL.dirty = true;
    if (window.Inspector) Inspector.update();
    return;
  }
  const ki = keyAt(li, x, y);
  if (ki >= 0) {
    if (e.shiftKey) { if (selHas(li, ki)) selDel(li, ki); else selAdd(li, ki); }
    else if (!selHas(li, ki)) { TL.sel.clear(); selAdd(li, ki); }
    buildSnaps();
    const items = [];
    for (const [l, ks] of TL.sel) for (const i of ks) items.push({ li: l, ki: i, t: TL.lanes[l].ts[i], v: TL.lanes[l].vs[i] });
    TL.drag = { mode: "keys", x, y, items, lanes: new Set(TL.sel.keys()), moved: false, anchor: TL.lanes[li].ts[ki] };
    TL.dirty = true;
    if (window.Inspector) Inspector.update();
    return;
  }
  if (!e.shiftKey) TL.sel.clear();
  TL.marquee = { x0: x, y0: y, x1: x, y1: y };
  TL.drag = { mode: "marquee" };
  TL.dirty = true;
}

function onMove(e) {
  const x = e.offsetX, y = e.offsetY, d = TL.drag;
  if (!d) return;
  if (d.mode === "pan") { TL.t0 = d.t0 - (x - d.x) / TL.pxs; TL.scrollY = Math.max(0, d.sy - (y - d.y)); }
  else if (d.mode === "scrub") App.locate(clamp(x2t(x), 0, TL.dur()));
  else if (d.mode === "in") TL.show.in = clamp(snapT(x2t(x)), 0, TL.outT() - .01);
  else if (d.mode === "out") TL.show.out = clamp(snapT(x2t(x)), TL.inT() + .01, TL.dur());
  else if (d.mode === "marquee") { TL.marquee.x1 = x; TL.marquee.y1 = y; applyMarquee(e.shiftKey); }
  else if (d.mode === "keys") {
    d.moved = true;
    let dt = (x - d.x) / TL.pxs;
    if (!e.shiftKey) dt = snapT(d.anchor + dt) - d.anchor;         // Shift releases the snapping
    const dy = y - d.y;
    for (const it of d.items) {
      const L = TL.lanes[it.li];
      L.ts[it.ki] = Math.max(0, it.t + dt);
      if (L.raw[it.ki] === null && Math.abs(dy) > 1)
        L.vs[it.ki] = clamp(it.v - dy / (TL.rowH - 8) * (L.vmax - L.vmin), L.vmin, L.vmax);
    }
  }
  TL.dirty = true;
}

function applyMarquee(add) {
  const m = TL.marquee, x0 = Math.min(m.x0, m.x1), x1 = Math.max(m.x0, m.x1);
  const y0 = Math.min(m.y0, m.y1), y1 = Math.max(m.y0, m.y1);
  if (!add) TL.sel.clear();
  const t0 = x2t(x0), t1 = x2t(x1);
  for (let li = 0; li < TL.lanes.length; li++) {
    const ly = laneY(li);
    if (ly + TL.rowH < y0 || ly > y1) continue;
    const L = TL.lanes[li], sc = (TL.rowH - 8) / ((L.vmax - L.vmin) || 1);
    for (let i = bisect(L.ts, L.n, t0); i < L.n && L.ts[i] <= t1; i++) {
      const ky = ly + TL.rowH - 4 - (L.vs[i] - L.vmin) * sc;
      if (ky >= y0 - 4 && ky <= y1 + 4) selAdd(li, i);
    }
  }
  for (const li of TL.sel.keys()) { TL.cur = li; break; }      // the inspector follows the marquee track
  if (window.Inspector) Inspector.update();
}

function onUp() {
  const d = TL.drag;
  TL.drag = null;
  TL.marquee = null;
  if (!d) return;
  if (d.mode === "keys" && d.moved) { for (const li of d.lanes) resort(li); commit(); }
  else if (d.mode === "in" || d.mode === "out") commit();
  TL.dirty = true;
}

function onWheel(e) {
  e.preventDefault();
  if (e.shiftKey) { TL.scrollY = Math.max(0, TL.scrollY + e.deltaY); }
  else {
    const t = x2t(e.offsetX), f = Math.exp(-e.deltaY * 0.0015);
    TL.pxs = clamp(TL.pxs * f, 0.2, 4000);
    TL.t0 = t - (e.offsetX - TL.headW) / TL.pxs;
  }
  TL.dirty = true;
}

// ---- context menu (easing) ---------------------------------------------
function hideMenu() { if (TL.menuEl) TL.menuEl.style.display = "none"; }

function onMenu(e) {
  e.preventDefault();
  const li = laneAt(e.offsetY);
  if (li < 0) return;
  const ki = keyAt(li, e.offsetX, e.offsetY);
  if (ki >= 0 && !selHas(li, ki)) { TL.sel.clear(); selAdd(li, ki); TL.dirty = true; }
  if (!selCount()) return;
  const m = TL.menuEl;
  m.style.display = "block";                                   // .tl-menu is position:fixed
  m.style.left = e.clientX + "px";
  m.style.top = e.clientY + "px";
}

function setCurve(name) {
  const ci = CURVES.indexOf(name);
  for (const [li, ks] of TL.sel) for (const i of ks) TL.lanes[li].cu[i] = ci;
  hideMenu();
  commit();
}

// ---- keyboard -----------------------------------------------------------
function onKey(e) {
  if (/^(INPUT|SELECT|TEXTAREA)$/.test(e.target.tagName)) return;
  if (location.hash.slice(1) && location.hash !== "#timeline") return;
  if (e.code === "Space" && !e.ctrlKey) spaceDown = true;
  if (e.key === "Delete" || e.key === "Backspace") { if (delSelected()) e.preventDefault(); }
  else if (e.ctrlKey && e.key === "c") {
    let t0 = Infinity;
    const cl = [];
    for (const [li, ks] of TL.sel) for (const i of ks) { const L = TL.lanes[li]; cl.push({ li, t: L.ts[i], v: L.vs[i], raw: L.raw[i], cu: L.cu[i] }); if (L.ts[i] < t0) t0 = L.ts[i]; }
    for (const k of cl) k.t -= t0;
    TL.clip = cl;
    App.log("timeline: " + cl.length + " keyframes copied");
  } else if (e.ctrlKey && e.key === "v" && TL.clip) {
    const t = App.now();
    TL.sel.clear();
    for (const k of TL.clip) { if (k.li >= TL.lanes.length) continue; const i = addKey(k.li, t + k.t, k.v, k.raw, k.cu); selAdd(k.li, i); }
    commit();
  } else if (e.key === "i" && !e.ctrlKey) { TL.show.in = App.now(); commit(); }
  else if (e.key === "o" && !e.ctrlKey) { TL.show.out = App.now(); commit(); }
}

// ---- drawing loop -------------------------------------------------------
function tick() {
  if (App.transport.state === "play") {
    const t = App.now();
    if (App.transport.loop && t >= TL.outT()) App.locate(TL.inT());
    TL.dirty = true;
  }
  if (TL.dirty && TL.cv.offsetParent !== null) { TL.dirty = false; draw(); }
  if (TL.bench) TL.bench.frames++;
  requestAnimationFrame(tick);
}

// ---- performance measurement (used in the visual check) -----------------
// ponytail: synthetic generator inside the panel itself ; it moves out of here if it becomes a benchmark suite.
TL.measure = function (nTracks, nKeys, frames) {
  const save = { lanes: TL.lanes, show: TL.show, t0: TL.t0, pxs: TL.pxs, sel: TL.sel };
  const lanes = [];
  for (let li = 0; li < nTracks; li++) {
    const L = { spec: {}, si: li, param: null, type: "dmx", n: nKeys, name: "synth " + li,
                ts: new Float64Array(nKeys), vs: new Float64Array(nKeys), cu: new Uint8Array(nKeys),
                raw: new Array(nKeys), vmin: 0, vmax: 255, mute: false, solo: false };
    for (let i = 0; i < nKeys; i++) { L.ts[i] = i * 0.05; L.vs[i] = 128 + 127 * Math.sin(i * .1 + li); L.raw[i] = null; }
    lanes.push(L);
  }
  TL.lanes = lanes;
  TL.sel = new Map();
  TL.show = { fps: 30, duration: nKeys * 0.05, markers: [] };
  draw();                                            // warm-up (compiles the path, allocates the buffer)
  const t0 = performance.now();
  for (let f = 0; f < frames; f++) {
    TL.t0 = f * 0.13;                                // scrolls
    TL.pxs = 40 + 30 * Math.sin(f / 7);              // and zooms at the same time
    draw();
  }
  const ms = (performance.now() - t0) / frames;
  Object.assign(TL, save);
  TL.dirty = true;
  return { tracks: nTracks, keys: nKeys, total: nTracks * nKeys, ms: +ms.toFixed(2),
           fps: Math.round(1000 / ms) };
};

// ---- mounting -----------------------------------------------------------
// Registration waits for DOMContentLoaded: App.route() uses elements that app.js only grabs there.
addEventListener("DOMContentLoaded", () => App.panel("timeline", {
  mount(el) {
    el.innerHTML =
      '<div class="tl-bar">' +
      '  <input id="tl-file" value="shows/medgrupo.spell" title="path of the .spell on the server">' +
      '  <button id="tl-open">Open</button><button id="tl-save">Save</button>' +
      '  <span class="sep"></span>' +
      '  <select id="tl-type"><option>dmx</option><option>fixture</option><option>osc</option>' +
      '<option>artnet</option><option>media</option><option>cue</option><option>laser</option></select>' +
      '  <button id="tl-add">+ Track</button><button id="tl-deltrack">- Track</button>' +
      '  <span class="sep"></span>' +
      '  <input id="tl-video" placeholder="video or .wav" title="file for the markers command">' +
      '  <button id="tl-markers">Import video cuts</button>' +
      '  <span class="sep"></span>' +
      '  <button id="tl-in">In</button><button id="tl-out">Out</button><button id="tl-loop">Loop</button>' +
      '  <button id="tl-snap" class="on">Snap</button><button id="tl-fit">Fit</button>' +
      '  <span class="tl-msg" id="tl-msg"></span>' +
      '</div>' +
      '<div class="tl-body"><canvas id="tl-cv" tabindex="0"></canvas><div class="tl-insp" id="tl-insp"></div></div>' +
      '<div class="tl-menu" id="tl-menu"></div>';

    TL.cv = el.querySelector("#tl-cv");
    TL.cx = TL.cv.getContext("2d");
    TL.menuEl = el.querySelector("#tl-menu");
    const msg = el.querySelector("#tl-msg");
    for (const n of CURVES) {
      const b = document.createElement("button");
      b.textContent = n;
      b.onclick = () => setCurve(n);
      TL.menuEl.appendChild(b);
    }
    const del = document.createElement("button");
    del.textContent = "delete";
    del.onclick = () => { hideMenu(); delSelected(); };
    TL.menuEl.appendChild(del);

    colors();
    resize();
    new ResizeObserver(resize).observe(TL.cv);
    document.addEventListener("skin", colors);

    TL.cv.addEventListener("mousedown", onDown);
    TL.cv.addEventListener("mousemove", onMove);
    addEventListener("mouseup", onUp);
    TL.cv.addEventListener("wheel", onWheel, { passive: false });
    TL.cv.addEventListener("contextmenu", onMenu);
    TL.cv.addEventListener("dblclick", e => {                    // a double click creates a keyframe
      const li = laneAt(e.offsetY);
      if (li < 0 || e.offsetX < TL.headW) return;
      const L = TL.lanes[li], v = clamp(L.vmin + (laneY(li) + TL.rowH - 4 - e.offsetY) / (TL.rowH - 8) * (L.vmax - L.vmin), L.vmin, L.vmax);
      TL.sel.clear();
      selAdd(li, addKey(li, Math.max(0, snapT(x2t(e.offsetX))), Math.round(v), null, 0));
      commit();
    });
    addEventListener("keydown", onKey);
    addEventListener("keyup", e => { if (e.code === "Space") spaceDown = false; });

    const q = id => el.querySelector("#tl-" + id);
    const load = sh => { build(sh); fit(); msg.textContent = (sh.name || "") + "  " + (sh.tracks || []).length + " tracks"; };
    const fit = () => { TL.pxs = (TL.w - TL.headW) / (TL.dur() || 60); TL.t0 = 0; TL.dirty = true; };
    q("open").onclick = () => App.rpc("show_open", { file: q("file").value }).then(load).catch(e => msg.textContent = e.message);
    q("save").onclick = () => App.rpc("show_save", { file: q("file").value }).then(p => msg.textContent = "saved " + p).catch(e => msg.textContent = e.message);
    q("add").onclick = () => App.rpc("track_add", { type: q("type").value }).then(() => App.rpc("show_get").then(load));
    q("deltrack").onclick = () => {
      if (TL.cur < 0) return;
      App.rpc("track_del", { index: TL.lanes[TL.cur].si }).then(() => App.rpc("show_get").then(load));
    };
    q("markers").onclick = () => {
      msg.textContent = "reading cuts...";
      App.rpc("markers", { video: q("video").value }).then(ts => {
        TL.show.markers = ts;
        msg.textContent = ts.length + " cuts imported";
        commit();
      }).catch(e => msg.textContent = e.message);
    };
    q("in").onclick = () => { TL.show.in = App.now(); commit(); };
    q("out").onclick = () => { TL.show.out = App.now(); commit(); };
    q("loop").onclick = () => { App.toggleLoop(); q("loop").classList.toggle("on", App.transport.loop); TL.dirty = true; };
    q("snap").onclick = () => { TL.snap = !TL.snap; q("snap").classList.toggle("on", TL.snap); };
    q("fit").onclick = fit;

    if (window.Inspector) Inspector.mount(el.querySelector("#tl-insp"));
    const first = () => App.rpc("show_get").then(load).catch(() => setTimeout(first, 600));
    first();
    requestAnimationFrame(tick);
  },
}));
