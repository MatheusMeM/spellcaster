"use strict";
// canvaskit.js — Spellcaster canvas kit, shared by the timeline and (R9) by the graph.
// It does what PRD §6 asks of the R5 base: pan, zoom at the cursor, selection (click, shift,
// marquee), hit-test by bisect on a time-ordered list, dirty flag (only redraws when marked) and
// DPR.
//
// Model: one object per canvas. `view.x` is the time (or world X) at the left edge of the usable
// area, `view.zoom` is px per time unit, `view.y` is the vertical scroll in px. The Y axis
// belongs to the client: the kit does not know what a track is. `gutter` is the fixed column on
// the left (track header).
//
// Mouse wheel (valid for the three canvas pages): wheel = scroll the content (`view.y`),
// Shift+wheel = move in time (`view.x`), Ctrl+wheel = zoom at the cursor. Every event calls
// `preventDefault` with `passive:false`: that is what stops the PAGE from scrolling (it was the
// scrollbar that took the toolbar away) and Ctrl+wheel from zooming the browser. `view.y` never
// goes past 0 at the bottom nor past `k.ymax` at the top; whoever knows the content height writes
// `k.ymax` (the timeline does it while drawing), and whoever does not write it has no ceiling.
//
// ponytail: one 2D context per canvas, no layers and no tile cache ; virtualize by viewport and
// then WebGL (PRD §8) only when drawing goes past 16 ms with a real show.

const CK = {
  ZMIN: 0.02,
  ZMAX: 8000,

  clamp(v, a, b) { return v < a ? a : v > b ? b : v; },

  // First index with ts[i] >= t. Base of every hit-test and of every draw by viewport.
  bisect(ts, n, t) {
    let lo = 0, hi = n;
    while (lo < hi) { const m = (lo + hi) >> 1; if (ts[m] < t) lo = m + 1; else hi = m; }
    return lo;
  },

  // Index of the item closest to t within tol, or -1. Ordered list; looks only at the two neighbours.
  near(ts, n, t, tol) {
    const i = CK.bisect(ts, n, t);
    let best = -1, bd = tol;
    for (let j = i - 1; j <= i; j++) {
      if (j < 0 || j >= n) continue;
      const d = Math.abs(ts[j] - t);
      if (d <= bd) { bd = d; best = j; }
    }
    return best;
  },

  // Colors from `design/tokens` read off the element (the canvas inherits the vars). ONE table for
  // the three canvas pages: what a page does not use costs one var read, not another file.
  // ponytail: hex fallback baked in ; drop it when tokens.css is guaranteed on every page.
  colors(el) {
    const cs = getComputedStyle(el || document.documentElement);
    const g = (n, d) => cs.getPropertyValue(n).trim() || d;
    return {
      bg: g("--sc-bg", "#000"), panel: g("--sc-panel", "#0E0E0E"), panel2: g("--sc-panel-2", "#161616"),
      well: g("--sc-well", "#070707"), line: g("--sc-line", "#262626"), hair: g("--sc-hair", "#222"),
      fg: g("--sc-fg", "#F7F5EB"), fg2: g("--sc-fg-2", "#B5B1A9"), fg3: g("--sc-fg-3", "#666361"),
      accent: g("--sc-accent", "#FFB000"), go: g("--sc-go", "#A8E05E"),
      live: g("--sc-live", "#FF2D1F"), rehearsal: g("--sc-rehearsal", "#B7AED9"),
      no_in: g("--sc-node-in", "#7FD1FF"), no_logic: g("--sc-node-logic", "#B7AED9"),
      no_cmd: g("--sc-node-cmd", "#FFB000"), no_out: g("--sc-node-out", "#A8E05E"),
      no_module: g("--sc-node-logic", "#B7AED9"),
      mono: g("--sc-mono", "Consolas, monospace"), label: g("--sc-label", "Arial Narrow, sans-serif"),
    };
  },

  // Sparse selection: row (track/lane) -> Set of indices.
  sel() {
    const m = new Map();
    return {
      m,                                     // row -> Set; whoever wants to read walks `m` directly
      has(r, i) { const s = m.get(r); return !!s && s.has(i); },
      add(r, i) { let s = m.get(r); if (!s) m.set(r, s = new Set()); s.add(i); },
      toggle(r, i) {
        const s = m.get(r);
        if (s && s.has(i)) { s.delete(i); if (!s.size) m.delete(r); } else this.add(r, i);
      },
      clear() { m.clear(); },
      count() { let n = 0; for (const s of m.values()) n += s.size; return n; },
      each(fn) { for (const [r, s] of m) for (const i of s) fn(r, i); },
    };
  },

  // Attaches a canvas. `draw(k)` draws everything; the kit only calls it when someone marked dirty.
  attach(cv, draw) {
    const k = {
      cv, cx: cv.getContext("2d"), w: 0, h: 0, dpr: 1,
      view: { x: 0, zoom: 40, y: 0 },
      gutter: 0, dirty: true, drag: null, marquee: null, ymax: Infinity,
      sel: CK.sel(),
      on: {},                       // down, move, up, hover, menu, marquee, frame — all optional
    };

    k.toScreen = t => k.gutter + (t - k.view.x) * k.view.zoom;
    k.toWorld = x => k.view.x + (x - k.gutter) / k.view.zoom;
    k.invalidate = () => { k.dirty = true; };

    k.resize = () => {
      const r = cv.getBoundingClientRect();
      if (r.width < 1) return;                          // hidden panel: do not touch the scale
      k.dpr = window.devicePixelRatio || 1;
      k.w = Math.max(1, r.width | 0);
      k.h = Math.max(1, r.height | 0);
      cv.width = (k.w * k.dpr) | 0;
      cv.height = (k.h * k.dpr) | 0;
      k.cx.setTransform(k.dpr, 0, 0, k.dpr, 0, 0);
      k.dirty = true;
    };

    k.panY = y => { k.view.y = CK.clamp(y, 0, Math.max(0, k.ymax)); k.dirty = true; };

    k.zoomAt = (px, f) => {                           // zoom anchored at the cursor
      const t = k.toWorld(px);
      k.view.zoom = CK.clamp(k.view.zoom * f, CK.ZMIN, CK.ZMAX);
      k.view.x = t - (px - k.gutter) / k.view.zoom;
      k.dirty = true;
    };

    k.fit = (t0, t1) => {
      k.view.zoom = CK.clamp(Math.max(1, k.w - k.gutter) / Math.max(0.001, t1 - t0), CK.ZMIN, CK.ZMAX);
      k.view.x = t0;
      k.dirty = true;
    };

    k.rect = () => {
      const m = k.marquee;
      return m && { x0: Math.min(m.x0, m.x1), x1: Math.max(m.x0, m.x1),
                    y0: Math.min(m.y0, m.y1), y1: Math.max(m.y0, m.y1) };
    };

    const pt = e => ({ x: e.offsetX, y: e.offsetY, t: k.toWorld(e.offsetX), button: e.button,
                       shift: e.shiftKey, ctrl: e.ctrlKey, alt: e.altKey, e });
    // ponytail: best-effort capture ; a synthetic event (test) has no valid pointerId and does not capture.
    const cap = e => { try { cv.setPointerCapture(e.pointerId); } catch (err) { /* no capture */ } };

    // Pan (middle button), zoom (wheel) and marquee (drag on empty space) belong to the kit.
    // The client gets the click first: if `on.down` returns true, the kit does not start a marquee.
    cv.addEventListener("pointerdown", e => {
      const p = pt(e);
      cv.focus();
      if (e.button === 1) {
        k.drag = { mode: "pan", x: p.x, y: p.y, vx: k.view.x, vy: k.view.y };
        cap(e);
        return e.preventDefault();
      }
      if (e.button === 2) { if (k.on.menu) k.on.menu(p); return; }
      cap(e);
      if (k.on.down && k.on.down(p)) return;
      if (!p.shift) k.sel.clear();
      k.marquee = { x0: p.x, y0: p.y, x1: p.x, y1: p.y, add: p.shift };
      k.drag = { mode: "marquee" };
      k.dirty = true;
    });

    cv.addEventListener("pointermove", e => {
      const p = pt(e), d = k.drag;
      if (!d) { if (k.on.hover) k.on.hover(p); return; }
      if (d.mode === "pan") {
        k.view.x = d.vx - (p.x - d.x) / k.view.zoom;
        k.panY(d.vy - (p.y - d.y));
      } else if (d.mode === "marquee") {
        k.marquee.x1 = p.x; k.marquee.y1 = p.y;
        if (k.on.marquee) k.on.marquee(k.rect(), k.marquee.add);
        k.dirty = true;
      } else if (k.on.move) {
        k.on.move(p, d);
      }
    });

    const up = e => {
      const d = k.drag;
      k.drag = null;
      k.marquee = null;
      if (d && d.mode !== "pan" && d.mode !== "marquee" && k.on.up) k.on.up(pt(e), d);
      k.dirty = true;
    };
    cv.addEventListener("pointerup", up);
    cv.addEventListener("pointercancel", up);

    // Wheel: with no modifier it scrolls the content, Shift moves in time, Ctrl zooms at the cursor.
    // With Shift, Windows sends the delta in `deltaX`; with Ctrl the browser would zoom the page —
    // that is why preventDefault comes before everything and the listener is `passive:false`.
    cv.addEventListener("wheel", e => {
      e.preventDefault();
      if (e.ctrlKey) k.zoomAt(e.offsetX, Math.exp(-e.deltaY * 0.0015));
      else if (e.shiftKey) { k.view.x += (e.deltaX || e.deltaY) / k.view.zoom; k.dirty = true; }
      else k.panY(k.view.y + e.deltaY);
    }, { passive: false });

    cv.addEventListener("contextmenu", e => e.preventDefault());

    // One rAF per canvas: `on.frame` advances the client clock, the draw only happens when dirty.
    k.loop = () => {
      const step = () => {
        if (k.on.frame) k.on.frame();
        if (k.dirty && cv.offsetWidth > 0) { k.dirty = false; draw(k); }
        requestAnimationFrame(step);
      };
      requestAnimationFrame(step);
    };
    k.redraw = () => { k.dirty = false; draw(k); };     // synchronous draw (test and capture)

    k.resize();
    new ResizeObserver(k.resize).observe(cv);
    return k;
  },
};

window.CK = CK;
