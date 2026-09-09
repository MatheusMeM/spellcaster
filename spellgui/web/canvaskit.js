"use strict";
// canvaskit.js — kit de canvas do Spellcaster, compartilhado pela timeline e (R9) pelo graph.
// Faz o que o PRD §6 pede da base da R5: pan, zoom no cursor, selecao (clique, shift, marquee),
// hit-test por bisect em lista ordenada por tempo, dirty-flag (so redesenha quando marcado) e DPR.
//
// Modelo: um objeto por canvas. `view.x` e o tempo (ou X do mundo) na borda esquerda da area util,
// `view.zoom` e px por unidade de tempo, `view.y` e a rolagem vertical em px. O eixo Y e do cliente:
// o kit nao sabe o que e um track. `gutter` e a coluna fixa a esquerda (cabecalho de track).
//
// ponytail: um contexto 2D por canvas, sem camadas nem cache de tile ; virtualizar por viewport e
// depois WebGL (PRD §8) so quando o desenho passar de 16 ms com show real.

const CK = {
  ZMIN: 0.02,
  ZMAX: 8000,

  clamp(v, a, b) { return v < a ? a : v > b ? b : v; },

  // Primeiro indice com ts[i] >= t. Base de todo hit-test e de todo desenho por viewport.
  bisect(ts, n, t) {
    let lo = 0, hi = n;
    while (lo < hi) { const m = (lo + hi) >> 1; if (ts[m] < t) lo = m + 1; else hi = m; }
    return lo;
  },

  // Indice do item mais proximo de t dentro de tol, ou -1. Lista ordenada; olha so os dois vizinhos.
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

  // Cores do `design/tokens` lidas do elemento (o canvas herda as vars). UMA tabela para as tres
  // paginas de canvas: o que a pagina nao usa custa uma leitura de var, nao um arquivo a mais.
  // ponytail: fallback hex embutido ; sai quando o tokens.css for garantido em toda pagina.
  cores(el) {
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

  // Selecao esparsa: linha (track/lane) -> Set de indices.
  sel() {
    const m = new Map();
    return {
      m,                                     // linha -> Set; quem quer ler percorre `m` direto
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

  // Liga um canvas. `draw(k)` desenha tudo; o kit so o chama quando alguem marcou dirty.
  attach(cv, draw) {
    const k = {
      cv, cx: cv.getContext("2d"), w: 0, h: 0, dpr: 1,
      view: { x: 0, zoom: 40, y: 0 },
      gutter: 0, dirty: true, drag: null, marquee: null,
      sel: CK.sel(),
      on: {},                       // down, move, up, hover, menu, marquee, frame — todos opcionais
    };

    k.toScreen = t => k.gutter + (t - k.view.x) * k.view.zoom;
    k.toWorld = x => k.view.x + (x - k.gutter) / k.view.zoom;
    k.invalidate = () => { k.dirty = true; };

    k.resize = () => {
      const r = cv.getBoundingClientRect();
      if (r.width < 1) return;                          // painel escondido: nao mexe na escala
      k.dpr = window.devicePixelRatio || 1;
      k.w = Math.max(1, r.width | 0);
      k.h = Math.max(1, r.height | 0);
      cv.width = (k.w * k.dpr) | 0;
      cv.height = (k.h * k.dpr) | 0;
      k.cx.setTransform(k.dpr, 0, 0, k.dpr, 0, 0);
      k.dirty = true;
    };

    k.zoomAt = (px, f) => {                             // zoom ancorado no cursor
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
    // ponytail: captura best-effort ; evento sintetico (teste) nao tem pointerId valido e nao captura.
    const cap = e => { try { cv.setPointerCapture(e.pointerId); } catch (err) { /* sem captura */ } };

    // Pan (botao do meio), zoom (roda), marquee (arrasto no vazio) sao do kit.
    // O cliente pega o clique antes: se `on.down` devolver true, o kit nao inicia marquee.
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
        k.view.y = Math.max(0, d.vy - (p.y - d.y));
        k.dirty = true;
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

    cv.addEventListener("wheel", e => {
      e.preventDefault();
      if (e.shiftKey) { k.view.y = Math.max(0, k.view.y + e.deltaY); k.dirty = true; }
      else k.zoomAt(e.offsetX, Math.exp(-e.deltaY * 0.0015));
    }, { passive: false });

    cv.addEventListener("contextmenu", e => e.preventDefault());

    // Um rAF por canvas: `on.frame` adianta o tempo do cliente, o desenho so sai se estiver dirty.
    k.loop = () => {
      const step = () => {
        if (k.on.frame) k.on.frame();
        if (k.dirty && cv.offsetWidth > 0) { k.dirty = false; draw(k); }
        requestAnimationFrame(step);
      };
      requestAnimationFrame(step);
    };
    k.redraw = () => { k.dirty = false; draw(k); };     // desenho sincrono (teste e captura)

    k.resize();
    new ResizeObserver(k.resize).observe(cv);
    return k;
  },
};

window.CK = CK;
