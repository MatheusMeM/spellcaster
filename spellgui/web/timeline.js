"use strict";
// timeline.js — timeline em canvas da GUI Tauri, sobre o canvaskit.
// Portada do prototipo Python (spellcaster/gui/web/timeline.js) e reescrita sobre CK: o kit cuida de
// pan, zoom, selecao, marquee, DPR e dirty-flag; aqui ficam tracks, keyframes, curvas, regua,
// snapping, scrub, in/out e loop. Cores so por token de design/tokens/spellcaster.css.
// Atalhos: design/SHORTCUTS.md (Premiere/Resolve). Ctrl+S salva pelo engine; Alt+M abre o monitor.
//
// Modelo: cada lane e um array de keyframes do .spell — spec.keys, ou spec.<param> (scale, rot, x...).
// Os keyframes moram em Float64Array/Uint8Array paralelos: o desenho nao aloca por frame e o hit-test
// e por bisect (CK.bisect). A edicao acontece no modelo local, volta pro JSON no commit() e, quando
// ha `spellcore serve`, vira chamada do registry (key_set/key_del/show_patch/...) pelo WS.
//
// Sem servidor nada muda: o show vem de fetch, o transporte e o relogio local e o commit so reescreve
// o JSON em memoria. Com servidor o playhead vem dos eventos `transport` e o show recarrega quando
// alguem de fora mexe (evento `show` com rev acima do que a ultima resposta trouxe).

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
const MONH = 56;                                   // altura do monitor de 512 barras

const TL = {
  show: null, lanes: [], k: null, clip: null, snaps: null,
  headW: 192, rulerH: 24, rowH: 32,          // multiplos de 8: grade de console (design/PRINCIPIOS.md §3)
  snap: true, cur: -1, drawn: 0,
  t: 0, rate: 0, loop: false, last: 0,
  col: {}, menuEl: null, msgEl: null,
  onState: () => {},                               // index.html sincroniza os botoes de transporte
  file: "", rev: 0, tstate: null,                  // engine: caminho do show, revisao, transporte
  mon: false, dmx: new Map(),                      // monitor: ultimo frame binario por universo
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

TL.log = function (s) { if (TL.msgEl) TL.msgEl.textContent = s; };

// ---- barramento (spellcore/README.md, secao `serve`) --------------------
// ponytail: cliente WS de 30 linhas aqui dentro ; trocar por bus.js (frente face) quando ele existir.
const BUS = {
  ws: null, id: 0, pend: new Map(), on: {}, ever: false,
  live() { return !!this.ws && this.ws.readyState === 1; },
  call(cmd, args) {
    if (!this.live()) return Promise.reject("sem servidor");
    const id = ++this.id;
    this.ws.send(JSON.stringify({ id: id, cmd: cmd, args: args || {} }));
    return new Promise((ok, no) => this.pend.set(id, [ok, no]));
  },
  msg(d) {
    if (typeof d !== "string") {                       // frame binario do monitor
      const f = TL.frameBin(d);
      if (f) { TL.dmx.set(f.universe, f.data); if (TL.mon) TL.k.dirty = true; }
      return;
    }
    const m = JSON.parse(d);
    if (m.event) { if (BUS.on[m.event]) BUS.on[m.event](m.data); return; }
    // Toda resposta do barramento carrega o `rev` do engine: e' o que separa o eco da nossa
    // propria edicao da edicao de outro cliente.
    if (typeof m.rev === "number") TL.rev = Math.max(TL.rev, m.rev);
    const p = this.pend.get(m.id);
    if (!p) return;
    this.pend.delete(m.id);
    if (m.error === undefined) p[0](m.result); else p[1](m.error);
  },
  open(url) {
    let w;
    try { w = this.ws = new WebSocket(url); } catch (e) { TL.log("sem servidor: " + e.message); return; }
    w.binaryType = "arraybuffer";
    w.onmessage = e => BUS.msg(e.data);
    // engine reiniciado volta com rev = 0, e TL.revEvento so' corrige a conta para cima: sem
    // zerar aqui, todo evento `show` do processo novo viria abaixo do numero velho e engolido.
    w.onopen = () => { BUS.ever = true; TL.rev = 0; TL.reload(); };
    w.onclose = () => {
      BUS.ws = null;
      TL.tstate = null;
      if (TL.k) TL.k.dirty = true;
      // so reconecta se um dia conectou: sem serve (python -m http.server) a pagina fica offline em paz
      if (BUS.ever) setTimeout(() => BUS.open(url), 2000);
      else TL.log("offline: sem spellcore serve");
    };
  },
};

// Frame binario do monitor: topic:u8 | universe:u16 LE | 512 bytes. topic 1 = dmx de saida.
TL.frameBin = function (buf) {
  const b = new Uint8Array(buf);
  if (b.length < 515 || b[0] !== 1) return null;
  return { universe: b[1] | (b[2] << 8), data: b.subarray(3, 515) };
};

// Uma lista de edicoes locais vira a lista de chamadas do registry. Funcao pura: e' esta que o
// teste cobre. Edicoes: {k:"set"|"move", track, t, from?, value, curve} | {k:"del", track, t} |
// {k:"field", path, value}. Os `del` saem todos antes dos `set` — mover dois keyframes vizinhos
// um sobre o outro apagaria o recem-escrito se as ops se intercalassem.
TL.ops = function (eds) {
  const del = [], set = [], patch = [];
  for (const e of eds) {
    if (e.k === "field") { patch.push({ op: "add", path: e.path, value: e.value }); continue; }
    if (e.k === "del" || e.k === "move")
      del.push({ cmd: "key_del", args: { track: e.track, t: e.k === "move" ? e.from : e.t } });
    if (e.k === "set" || e.k === "move")
      set.push({ cmd: "key_set", args: { track: e.track, t: e.t, value: e.value, curve: e.curve } });
  }
  const out = del.concat(set);
  if (patch.length) out.push({ cmd: "show_patch", args: { ops: patch } });
  return out;
};

// Manda as chamadas. Sem servidor nao faz nada: o modo offline continua o de hoje.
// Ha' UM contador de revisao, o do engine (`engine::edit::rev()`): toda resposta o traz e
// `BUS.msg` o guarda em `TL.rev`. Nada de adiantar a conta antes do envio.
// ponytail: o eco da nossa propria edicao que chegue ANTES da resposta custa um reload
// (`d.rev > TL.rev` porque a resposta ainda nao veio) ; some quando o evento `show` carregar o
// id do cliente que editou.
function send(calls) {
  if (!BUS.live()) return;
  for (const c of calls) {
    BUS.call(c.cmd, c.args).catch(err => TL.log(c.cmd + ": " + err));
  }
}

// Evento `show`: rev maior que o ultimo visto numa resposta veio de outro cliente e manda
// recarregar; menor ou igual e' eco nosso, ou evento atrasado. Funcao pura: e' esta que o teste
// cobre. O maximo ressincroniza a conta local em qualquer desvio.
TL.revEvento = (rev, esperado) => ({ rev: Math.max(rev, esperado), reload: rev > esperado });

const hasPlayer = () => TL.tstate === "play" || TL.tstate === "pause";

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
  setRate(0);
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
  const eds = [];
  TL.k.sel.m.forEach((ks, li) => {
    const L = TL.lanes[li];
    let w = 0;
    for (let i = 0; i < L.n; i++) {
      if (ks.has(i)) { n++; eds.push(eDel(li, L.ts[i])); continue; }
      L.ts[w] = L.ts[i]; L.vs[w] = L.vs[i]; L.cu[w] = L.cu[i]; L.raw[w] = L.raw[i]; w++;
    }
    L.n = w;
  });
  TL.k.sel.clear();
  if (n) commit(eds);
  return n;
}

const ktime = t => Math.round(t * 1e4) / 1e4;
const kval = (L, i) => L.raw[i] !== null ? L.raw[i]
                     : L.vmax === 255 ? Math.round(L.vs[i]) : Math.round(L.vs[i] * 1e4) / 1e4;

function laneOut(L) {
  const out = new Array(L.n);
  for (let i = 0; i < L.n; i++) out[i] = [ktime(L.ts[i]), kval(L, i), CURVES[L.cu[i]]];
  return out;
}

// Edicao de um keyframe da lane li (indice i depois da mudanca; fromT = tempo antigo, se mudou).
// Lane de parametro (spec.scale, spec.x, ...) nao tem comando proprio no registry.
// ponytail: lane de parametro vai inteira por show_patch ; virar key_set quando key_set souber
// escrever em spec.<param> e nao so' em spec.keys.
function eKey(li, i, fromT) {
  const L = TL.lanes[li];
  if (L.param) return { k: "lane", li: li };
  const e = { k: "set", track: L.si, t: ktime(L.ts[i]), value: kval(L, i), curve: CURVES[L.cu[i]] };
  if (fromT !== undefined && Math.abs(ktime(fromT) - e.t) > 1e-9) { e.k = "move"; e.from = ktime(fromT); }
  return e;
}

function eDel(li, t) {
  const L = TL.lanes[li];
  return L.param ? { k: "lane", li: li } : { k: "del", track: L.si, t: ktime(t) };
}

// Keyframe recem-criado: o `resort` do addKey ja mexeu nos indices, entao acha pelo tempo.
function eAt(li, t) {
  const L = TL.lanes[li];
  return eKey(li, CK.bisect(L.ts, L.n, t));
}

// Devolve o modelo local para o JSON do show e, com servidor, manda as edicoes ao registry.
// `eds` descreve o que mudou; sem `eds` so' reescreve o JSON local (modo offline de hoje).
function commit(eds) {
  for (const L of TL.lanes) {
    if (L.param) L.spec[L.param] = laneOut(L); else L.spec.keys = laneOut(L);
    L.spec.mute = L.mute; L.spec.solo = L.solo;
  }
  TL.k.dirty = true;
  if (!eds || !eds.length || !BUS.live()) return;
  const fim = [];
  for (const e of eds) {                          // lane inteira: uma op com o array ja' gravado
    if (e.k !== "lane") { fim.push(e); continue; }
    const L = TL.lanes[e.li];
    fim.push({ k: "field", path: "/tracks/" + L.si + "/" + L.param, value: L.spec[L.param] });
  }
  send(TL.ops(fim));
}
TL.commit = commit;

// Campo do show por show_patch (in, out, markers, mute/solo de track).
function field(path, value) { commit([{ k: "field", path: path, value: value }]); }

function trackFlag(L, key, on) { field("/tracks/" + L.si + "/" + key, on); }

TL.setInOut = function (i, o) {
  const eds = [];
  if (i !== null && i !== undefined) { TL.show.in = i; eds.push({ k: "field", path: "/in", value: i }); }
  if (o !== null && o !== undefined) { TL.show.out = o; eds.push({ k: "field", path: "/out", value: o }); }
  commit(eds);
};

// ---- transporte ---------------------------------------------------------
// Um funil so': todo play/pause/stop e todo salto passam por aqui. Com player vivo quem manda e' o
// engine (o playhead vem dos eventos `transport`); sem ele, o relogio local de sempre.
function setT(t) {
  TL.t = clamp(t, 0, TL.dur());
  TL.k.dirty = true;
}

// Todo ponto que muda o transporte avisa a pagina (TL.onState): sem isso a barra pesquisaria o
// estado por temporizador.
function setRate(r) {
  TL.rate = r;
  TL.onState();
}

TL.locate = function (t) {
  setT(t);
  if (hasPlayer()) send([{ cmd: "locate", args: { t: TL.t } }]);
};

// ponytail: o engine nao tem shuttle nem rate reverso ; com player vivo J/L viram pause/play e o
// x2/x4/x8 continua so' no modo offline. Sai quando o registry ganhar um comando de rate.
TL.play = function (rate) {
  if (hasPlayer()) {
    send([rate > 0 ? { cmd: "resume", args: {} } : { cmd: "pause", args: {} }]);
    return;
  }
  if (rate > 0 && BUS.live() && TL.file) {
    send([{ cmd: "play_show", args: { file: TL.file, loop: TL.loop } }]);
    return;
  }
  setRate(rate);
  TL.last = performance.now();
  TL.k.dirty = true;
};

TL.stop = function () {
  if (BUS.live()) send([{ cmd: "stop", args: {} }]);
  TL.tstate = null;
  setRate(0);
  setT(0);
};

TL.save = function () {
  if (BUS.live()) send([{ cmd: "show_save", args: { file: "" } }]);
  else TL.log("Ctrl+S: sem servidor (o show fica so' em memoria)");
};

function frame() {
  if (!TL.rate) return;
  const now = performance.now(), dt = (now - TL.last) / 1000;
  TL.last = now;
  const t = TL.t + dt * TL.rate;
  if (hasPlayer()) return setT(t);       // o engine manda; aqui so' interpola entre os eventos
  if (TL.loop && TL.rate > 0 && t >= TL.outT()) return setT(TL.inT());
  if (t <= 0 || t >= TL.dur()) setRate(0);
  setT(t);
}

// ---- ligacao com o engine ----------------------------------------------
BUS.on.transport = d => {
  TL.tstate = d.state;
  setRate(d.state === "play" ? 1 : 0);
  TL.last = performance.now();
  setT(d.t);
};

// O `show` so' sai quando o `rev` do engine muda; so' recarrega o que veio de fora.
BUS.on.show = d => {
  const r = TL.revEvento(d.rev, TL.rev);
  TL.rev = r.rev;
  if (r.reload) TL.reload();
};

BUS.on.log = d => TL.log(d.text);

TL.reload = function () {
  return fetch("/show").then(r => r.json()).then(sh => {
    const cur = TL.cur, view = TL.k && { x: TL.k.view.x, y: TL.k.view.y, zoom: TL.k.view.zoom };
    TL.load(sh);
    TL.cur = cur;
    if (view) { TL.k.view.x = view.x; TL.k.view.y = view.y; TL.k.view.zoom = view.zoom; }
    TL.k.dirty = true;
  }).catch(e => TL.log("GET /show: " + e.message));
};

TL.connect = function () {
  BUS.open("ws://" + location.host + "/ws");
};

// ---- tracks -------------------------------------------------------------
TL.trackAdd = function () {
  TL.show.tracks.push({ type: "dmx", universe: 1, address: 1, keys: [] });  // igual ao track_add
  send([{ cmd: "track_add", args: { type: "dmx", universe: 1, address: 1, label: "" } }]);
  TL.load(TL.show);
  TL.cur = TL.lanes.length - 1;
};

TL.trackDel = function () {
  const L = TL.lanes[TL.cur];
  if (!L) return;
  const i = L.si;
  TL.show.tracks.splice(i, 1);
  send([{ cmd: "track_del", args: { index: i } }]);
  TL.load(TL.show);
};

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
  c.fillText(TL.lanes.length + "L " + k.sel.count() + "sel" + (TL.snap ? " snap" : "") +
             (BUS.live() ? " eng" + (TL.rev === null ? "" : ":" + TL.rev) : ""), 108, TL.rulerH / 2);

  // ---- playhead ----
  const xp = t2x(TL.t);
  if (xp >= hw - 1) {
    c.strokeStyle = TL.rate ? col.live : col.accent; c.lineWidth = 1;
    c.beginPath(); c.moveTo(Math.round(xp) + 0.5, 0); c.lineTo(Math.round(xp) + 0.5, H); c.stroke();
    c.fillStyle = TL.rate ? col.live : col.accent;
    c.beginPath(); c.moveTo(xp - 6, 0); c.lineTo(xp + 6, 0); c.lineTo(xp, 10); c.fill();
  }

  // ---- monitor: 512 barras do ultimo frame binario do universo da lane focada (Alt+M) ----
  // ponytail: faixa sobreposta no rodape, sem painel proprio ; virar painel quando a GUI tiver layout.
  if (TL.mon) {
    const mh = MONH, y0 = H - mh, sel = TL.lanes[TL.cur];
    const u = sel ? +(sel.spec.universe || 1) : 1, d = TL.dmx.get(u);
    c.fillStyle = col.panel;
    c.fillRect(0, y0, W, mh);
    c.strokeStyle = col.line; c.lineWidth = 1;
    c.beginPath(); c.moveTo(0, y0 + 0.5); c.lineTo(W, y0 + 0.5); c.stroke();
    if (d) {
      const bw = (W - 12) / 512, ph = mh - 18;
      c.fillStyle = col.accent;
      for (let i = 0; i < 512; i++) {
        const v = d[i];
        if (v) c.fillRect(6 + i * bw, y0 + mh - 4 - v / 255 * ph, Math.max(1, bw - 0.4), v / 255 * ph);
      }
    }
    c.fillStyle = col.fg3;
    c.font = "10px " + col.mono;
    c.fillText("u" + u + (d ? "" : "  sem frame"), 6, y0 + 8);
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
      if (b === 0) { L.mute = !L.mute; trackFlag(L, "mute", L.mute); }
      else if (b === 1) { L.solo = !L.solo; trackFlag(L, "solo", L.solo); }
      // ponytail: record arm e' so o estado visual da lane ; ligar quando o engine gravar
      else { L.rec = !L.rec; commit(); }
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
  if (d.mode === "scrub") { setT(x2t(p.x)); d.moved = true; }
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
  if (d.mode === "keys" && d.moved) {
    // as edicoes saem ANTES do resort: e' o resort que embaralha os indices de d.items
    const eds = d.items.map(it => eKey(it.li, it.ki, it.t));
    for (const li of d.lanes) resort(li);
    commit(eds);
  } else if (d.mode === "in") TL.setInOut(TL.show.in, null);
  else if (d.mode === "out") TL.setInOut(null, TL.show.out);
  else if (d.mode === "scrub" && d.moved) TL.locate(TL.t);   // um locate no fim, nao um por frame
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
  const ci = CURVES.indexOf(name), eds = [];
  TL.k.sel.each((li, i) => { TL.lanes[li].cu[i] = ci; eds.push(eKey(li, i)); });
  hideMenu();
  commit(eds);
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
  else if ((key === "m" || key === "M") && alt) { TL.mon = !TL.mon; TL.onState(); }
  else if ((key === "i" || key === "I") && alt) TL.setInOut(0, null);
  else if ((key === "o" || key === "O") && alt) TL.setInOut(null, TL.dur());
  else if ((key === "x" || key === "X") && alt) TL.setInOut(0, TL.dur());
  else if (key === "I" && shift) TL.locate(TL.inT());
  else if (key === "O" && shift) TL.locate(TL.outT());
  else if (key === "i") TL.setInOut(Math.min(TL.t, TL.outT() - 0.01), null);
  else if (key === "o") TL.setInOut(null, Math.max(TL.t, TL.inT() + 0.01));
  else if (ctrl && (key === "l" || key === "L")) TL.loop = !TL.loop;
  else if (ctrl && (key === "s" || key === "S")) TL.save();
  else if ((key === "m" || key === "M") && !ctrl) {
    TL.show.markers.push(+TL.t.toFixed(4));
    TL.show.markers.sort((a, b) => a - b);
    field("/markers", TL.show.markers);
  }
  else if (key === "=" || key === "+") k.zoomAt(k.gutter + (k.w - k.gutter) / 2, 1.25);
  else if (key === "-" || key === "_") k.zoomAt(k.gutter + (k.w - k.gutter) / 2, 0.8);
  else if (key === "\\" || (shift && (key === "z" || key === "Z"))) TL.fit();
  else if (ctrl && (key === "k" || key === "K")) {
    if (TL.cur >= 0) {
      const L = TL.lanes[TL.cur], v = TL.valueAt(L, TL.t);
      k.sel.clear();
      k.sel.add(TL.cur, addKey(TL.cur, TL.t, v === null ? 0 : v, null, 0));
      commit([eAt(TL.cur, TL.t)]);
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
    const ts = [];
    for (const c of TL.clip) {
      if (c.li >= TL.lanes.length) continue;
      k.sel.add(c.li, addKey(c.li, TL.t + c.t, c.v, c.raw, c.cu));
      ts.push([c.li, TL.t + c.t]);
    }
    commit(ts.map(a => eAt(a[0], a[1])));
  } else if (key === "Delete" || key === "Backspace") delSelected();
  else if (ctrl && shift && (key === "e" || key === "E")) {
    const eds = [];
    k.sel.each((li, i) => { const L = TL.lanes[li]; L.cu[i] = (L.cu[i] + 1) % CURVES.length; eds.push(eKey(li, i)); });
    commit(eds);
  } else if (key === "s" && !ctrl) TL.snap = !TL.snap;
  else if (key === "D" && shift && TL.cur >= 0) { const L = TL.lanes[TL.cur]; L.mute = !L.mute; trackFlag(L, "mute", L.mute); }
  else if (key === "S" && shift && TL.cur >= 0) { const L = TL.lanes[TL.cur]; L.solo = !L.solo; trackFlag(L, "solo", L.solo); }
  else if ((key === "r" || key === "R") && !ctrl && TL.cur >= 0) TL.lanes[TL.cur].rec = !TL.lanes[TL.cur].rec;
  else used = false;
  if (used) { e.preventDefault(); k.dirty = true; }
}
TL.onKey = onKey;

TL.fit = function () {
  TL.k.gutter = TL.headW;
  if (TL.k.w < 2) TL.k.resize();   // show carregado antes do primeiro layout: mede o canvas agora
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
