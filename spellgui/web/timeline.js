"use strict";
// timeline.js — canvas timeline of the Tauri GUI, on top of canvaskit.
// Ported from the Python prototype (spellcaster/gui/web/timeline.js) and rewritten on top of CK:
// the kit takes care of pan, zoom, selection, marquee, DPR and the dirty flag; here live tracks,
// keyframes, curves, ruler, snapping, scrub, in/out and loop. Colors only from tokens of
// design/tokens/spellcaster.css.
// Shortcuts: design/SHORTCUTS.md (Premiere/Resolve). Ctrl+S saves through the engine; Alt+M opens
// the monitor.
//
// Model: each lane is an array of keyframes from the .spell — spec.keys, or spec.<param> (scale,
// rot, x...). The keyframes live in parallel Float64Array/Uint8Array: drawing allocates nothing
// per frame and the hit-test is a bisect (CK.bisect). Editing happens in the local model, goes
// back to JSON on commit() and, when there is a `spellcore serve`, becomes a registry call
// (key_set/key_del/show_patch/...) over the WS.
//
// With no server nothing changes: the show comes from a fetch, the transport is the local clock
// and the commit only rewrites the JSON in memory. With a server the playhead comes from the
// `transport` events and the show reloads when someone outside edits it (`show` event with a rev
// above what the last response brought).
//
// Loop is ENGINE state, over the In-Out range of the show: the button and Ctrl+L call `loop_set`
// and `TL.loop` only reflects what arrives in the `transport` event. The page simulates no loop —
// when it did, the client jumped back to In and the engine kept playing to the end of the show.

const CURVES = ["linear", "hold", "in", "out", "inout", "bezier"];
// Same maths as the engine (spellcaster/timeline/model.py, spellcore/engine): the curve applies to
// the segment that ARRIVES at the keyframe.
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
const MONH = 120;                                  // fixed height of the previz strip (viewer.js)

const TL = {
  show: null, lanes: [], k: null, clip: null, snaps: null,
  headW: 192, rulerH: 24, rowH: 32,          // multiples of 8: console grid (design/PRINCIPIOS.md §3)
  snap: true, cur: -1, drawn: 0,
  t: 0, rate: 0, loop: false, last: 0,
  col: {}, menuEl: null, msgEl: null,
  onState: () => {},                               // index.html syncs the transport buttons
  onOpen: () => {},                                // Ctrl+O: the page decides how a show is opened
  file: "", rev: 0, tstate: null,                  // engine: show path, revision, transport
  mon: false, dmx: new Map(), dmxIn: new Map(),    // monitor: last binary frame per universe (out / in)
  fps() { return (this.show && this.show.fps) || 30; },
  dur() { return (this.show && this.show.duration) || 60; },
  inT() { return +((this.show && this.show.in) || 0); },
  outT() { return +((this.show && this.show.out) || this.dur()); },
};
window.TL = TL;
// Hooks of the previz (viewer.js): the curve of a keyframe and the bus, without duplicating either.
TL.ease = name => EASE[Math.max(0, CURVES.indexOf(name || "linear"))];
TL.call = (cmd, args) => BUS.call(cmd, args);

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

// ---- bus (spellcore/README.md, `serve` section) -------------------------
// ponytail: a 30-line WS client in here ; swap it for bus.js (face frente) once that exists.
const BUS = {
  ws: null, id: 0, pend: new Map(), on: {}, ever: false,
  live() { return !!this.ws && this.ws.readyState === 1; },
  call(cmd, args) {
    if (!this.live()) return Promise.reject("no server");
    const id = ++this.id;
    this.ws.send(JSON.stringify({ id: id, cmd: cmd, args: args || {} }));
    return new Promise((ok, no) => this.pend.set(id, [ok, no]));
  },
  msg(d) {
    if (typeof d !== "string") {                       // binary frame of the monitor
      const f = TL.frameBin(d);
      if (f) {
        (f.topic === 2 ? TL.dmxIn : TL.dmx).set(f.universe, f.data);
        if (TL.mon) TL.k.dirty = true;
      }
      return;
    }
    const m = JSON.parse(d);
    if (m.event) { if (BUS.on[m.event]) BUS.on[m.event](m.data); return; }
    // Every bus response carries the engine `rev`: that is what separates the echo of our own
    // edit from the edit of another client.
    if (typeof m.rev === "number") TL.rev = Math.max(TL.rev, m.rev);
    const p = this.pend.get(m.id);
    if (!p) return;
    this.pend.delete(m.id);
    if (m.error === undefined) p[0](m.result); else p[1](m.error);
  },
  open(url) {
    let w;
    try { w = this.ws = new WebSocket(url); } catch (e) { TL.log("no server: " + e.message); return; }
    w.binaryType = "arraybuffer";
    w.onmessage = e => BUS.msg(e.data);
    // a restarted engine comes back with rev = 0, and TL.revEvent only corrects the count upwards:
    // without zeroing here, every `show` event of the new process would come below the old number
    // and be swallowed.
    w.onopen = () => { BUS.ever = true; TL.rev = 0; TL.reload(); };
    w.onclose = () => {
      BUS.ws = null;
      TL.tstate = null;
      if (TL.k) TL.k.dirty = true;
      // only reconnects if it ever connected: with no serve (python -m http.server) the page stays
      // offline in peace
      if (BUS.ever) setTimeout(() => BUS.open(url), 2000);
      else TL.log("offline: no spellcore serve");
    };
  },
};

// Binary frame of the monitor: topic:u8 | universe:u16 LE | 512 bytes.
// topic 1 = output dmx; topic 2 = INPUT dmx (show.inputs), what the engine records.
TL.frameBin = function (buf) {
  const b = new Uint8Array(buf);
  if (b.length < 515 || (b[0] !== 1 && b[0] !== 2)) return null;
  return { topic: b[0], universe: b[1] | (b[2] << 8), data: b.subarray(3, 515) };
};

// A list of local edits becomes the list of registry calls. Pure function: this is the one the
// test covers. Edits: {k:"set"|"move", track, t, from?, value, curve} | {k:"del", track, t} |
// {k:"field", path, value}. All the `del` go out before all the `set` — moving two neighbouring
// keyframes onto each other would erase the freshly written one if the ops were interleaved.
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

// Sends the calls. With no server it does nothing: offline mode stays as it is today.
// There is ONE revision counter, the engine one (`engine::edit::rev()`): every response brings it
// and `BUS.msg` keeps it in `TL.rev`. No running the count ahead before sending.
// ponytail: the echo of our own edit arriving BEFORE the response costs one reload
// (`d.rev > TL.rev` because the response has not arrived yet) ; it goes away when the `show` event
// carries the id of the client that edited.
function send(calls) {
  if (!BUS.live()) return;
  for (const c of calls) {
    BUS.call(c.cmd, c.args).catch(err => TL.log(c.cmd + ": " + err));
  }
}

// `show` event: a rev above the last one seen in a response came from another client and forces a
// reload; below or equal is our own echo, or a late event. Pure function: this is the one the test
// covers. The max resyncs the local count on any drift.
TL.revEvent = (rev, expected) => ({ rev: Math.max(rev, expected), reload: rev > expected });

const hasPlayer = () => TL.tstate === "play" || TL.tstate === "pause";

const isKeys = v => Array.isArray(v) && v.length > 0 && Array.isArray(v[0]) &&
                    v[0].length >= 2 && typeof v[0][0] === "number";

// ---- model --------------------------------------------------------------
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
  // an empty lane is born at 0..255 (DMX); a laser param (scale/rot/x/y) is born at 0..1
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
    TL.lanes.push(mkLane(spec, si, null));                        // main lane (spec.keys)
    for (const p of Object.keys(spec)) {
      if (p !== "keys" && isKeys(spec[p]) && (LANE_PARAMS.indexOf(p) >= 0 || spec.type === "fixture"))
        TL.lanes.push(mkLane(spec, si, p));
    }
  });
  // Sorted once here: `jumpMarker` and the drawing read the list as it is.
  if (!Array.isArray(show.markers)) show.markers = [];
  else show.markers.sort((a, b) => a - b);
  TL.fit();
  UNDO.last = JSON.stringify(show);          // undo mark: the show as it has just arrived
  if (TL.msgEl) TL.msgEl.textContent = (show.name || "unnamed") + "  -  " +
    (show.tracks || []).length + " tracks, " + TL.lanes.length + " lanes";
  return TL.lanes.length;
};

// Opening another file clears the undo stack: the copies belong to ANOTHER show.
TL.fetch = path => fetch(path).then(r => {
  if (!r.ok) throw new Error(path + ": HTTP " + r.status);
  return r.json();
}).then(sh => { UNDO.back.length = 0; UNDO.fwd.length = 0; return TL.load(sh); });

// Value of the lane at t, same as the engine: bisect + curve of the next keyframe.
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
  const ts = new Float64Array(L.ts.length), vs = new Float64Array(L.ts.length);
  const cu = new Uint8Array(L.ts.length), raw = new Array(L.ts.length);
  const old = TL.k.sel.m.get(li), ns = old ? new Set() : null;
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

// Edit of a keyframe of lane li (index i after the change; fromT = old time, if it changed).
// A parameter lane (spec.scale, spec.x, ...) has no command of its own in the registry.
// ponytail: a parameter lane goes whole through show_patch ; make it key_set once key_set can
// write into spec.<param> and not only into spec.keys.
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

// A freshly created keyframe: the `resort` of addKey has already moved the indices, so find it by
// time.
function eAt(li, t) {
  const L = TL.lanes[li];
  return eKey(li, CK.bisect(L.ts, L.n, t));
}

// Undo/redo: a stack of copies of the whole show, taken on commit (the PREVIOUS state is what
// `UNDO.last` has held since the previous commit). Undo sends the copy back with `show_set`.
// ponytail: 20 copies of the whole JSON, with no granularity per edit ; make it the `undo` stack
// that `show_patch` already returns once the registry can stack it on the engine side.
const UNDO = { back: [], fwd: [], last: null, max: 20 };

function undoMark() {
  const cur = JSON.stringify(TL.show);
  if (UNDO.last !== null && UNDO.last !== cur) {
    UNDO.back.push(UNDO.last);
    if (UNDO.back.length > UNDO.max) UNDO.back.shift();
    UNDO.fwd.length = 0;
  }
  UNDO.last = cur;
}

// Swaps the show keeping the view and the focused lane: the engine reload and the undo take the
// same path.
function swap(sh) {
  const cur = TL.cur, view = TL.k && { x: TL.k.view.x, y: TL.k.view.y, zoom: TL.k.view.zoom };
  TL.load(sh);
  TL.cur = Math.min(cur, TL.lanes.length - 1);
  if (view) { TL.k.view.x = view.x; TL.k.view.y = view.y; TL.k.view.zoom = view.zoom; }
  TL.k.dirty = true;
}

// dir -1 undoes, +1 redoes. With a server the whole show goes back through `show_set`; without it,
// local only.
TL.undo = function (dir) {
  const from = dir > 0 ? UNDO.fwd : UNDO.back, to = dir > 0 ? UNDO.back : UNDO.fwd;
  if (!from.length) return TL.log(dir > 0 ? "nothing to redo" : "nothing to undo");
  to.push(UNDO.last);
  const s = from.pop();
  swap(JSON.parse(s));
  UNDO.last = s;                                  // `swap` reloads the show; the mark is this copy
  if (BUS.live()) send([{ cmd: "show_set", args: { data: JSON.parse(s) } }]);
  TL.log(dir > 0 ? "redone" : "undone");
};

// Writes the local model back into the show JSON and, with a server, sends the edits to the
// registry. `eds` describes what changed; without `eds` it only rewrites the local JSON (today's
// offline mode).
function commit(eds) {
  for (const L of TL.lanes) {
    if (L.param) L.spec[L.param] = laneOut(L); else L.spec.keys = laneOut(L);
    L.spec.mute = L.mute; L.spec.solo = L.solo;
  }
  undoMark();
  TL.k.dirty = true;
  if (!eds || !eds.length || !BUS.live()) return;
  const end = [];
  for (const e of eds) {                          // whole lane: one op with the already written array
    if (e.k !== "lane") { end.push(e); continue; }
    const L = TL.lanes[e.li];
    end.push({ k: "field", path: "/tracks/" + L.si + "/" + L.param, value: L.spec[L.param] });
  }
  send(TL.ops(end));
}
TL.commit = commit;

// A show field through show_patch (in, out, markers, track mute/solo).
function field(path, value) { commit([{ k: "field", path: path, value: value }]); }

function trackFlag(L, key, on) { field("/tracks/" + L.si + "/" + key, on); }

// In and Out are a pair, and this is the only funnel: the I/O keys, the buttons and the ruler
// handles all pass through here. A new limit that crosses the other does NOT collapse the range
// (that was how In/Out ended up 10 ms apart after a drag on the ruler): the one that was crossed
// goes to the end — In to 0, Out to the duration.
TL.setInOut = function (i, o) {
  const dur = TL.dur(), oldA = TL.inT(), oldB = TL.outT();
  let a = i === null || i === undefined ? oldA : clamp(+i, 0, dur);
  let b = o === null || o === undefined ? oldB : clamp(+o, 0, dur);
  if (a >= b) { if (o === null || o === undefined) b = dur; else a = 0; }
  TL.show.in = a;
  TL.show.out = b;
  // The requested limit ALWAYS goes out, even when equal to the old one: at the end of the handle
  // drag `onMove` has already written `TL.show.in`, so `a === oldA` is the normal case — comparing
  // here would lose the whole drag. The `else if` covers the other limit, which only changes when
  // it was crossed.
  const eds = [];
  if (i !== null && i !== undefined) eds.push({ k: "field", path: "/in", value: a });
  else if (a !== oldA) eds.push({ k: "field", path: "/in", value: a });
  if (o !== null && o !== undefined) eds.push({ k: "field", path: "/out", value: b });
  else if (b !== oldB) eds.push({ k: "field", path: "/out", value: b });
  commit(eds);
  if (TL.loop) TL.setLoop(true);        // the loop lives in the engine with the range: resend the new one
  return [a, b];
};

// ---- transport ----------------------------------------------------------
// A single funnel: every play/pause/stop and every jump goes through here. With a live player the
// engine is in charge (the playhead comes from the `transport` events); without it, the usual
// local clock.
function setT(t) {
  TL.t = clamp(t, 0, TL.dur());
  TL.k.dirty = true;
}

// Every point that changes the transport tells the page (TL.onState): without it the bar would
// poll the state on a timer.
function setRate(r) {
  TL.rate = r;
  TL.onState();
}

TL.locate = function (t) {
  setT(t);
  if (hasPlayer()) send([{ cmd: "locate", args: { t: TL.t } }]);
};

// ponytail: the engine has no shuttle and no reverse rate ; with a live player J/L become
// pause/play and the x2/x4/x8 stays offline only. It goes when the registry gains a rate command.
// Loop: an engine command (`loop_set`), over the In-Out range of the open show. `TL.loop` is only
// the reflection — the real value arrives in the `transport` event. With no engine only the button
// lights up.
TL.setLoop = function (on) {
  TL.loop = !!on;
  TL.onState();
  if (TL.k) TL.k.dirty = true;
  if (BUS.live()) send([{ cmd: "loop_set", args: { on: TL.loop } }]);
};

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

// Ctrl+S writes to the path of the last opened file (empty file); Ctrl+Shift+S asks for the path.
// ponytail: the browser `prompt` for "save as" ; make it a GUI dialog once there is one.
TL.save = function (file) {
  if (!BUS.live()) return TL.log("Ctrl+S: no server (the show stays in memory only)");
  send([{ cmd: "show_save", args: { file: file || "" } }]);
  if (file) TL.file = file;
};

TL.saveAs = function () {
  const f = prompt("save as (.spell):", TL.file || "shows/new.spell");
  if (f) TL.save(f);
};

// Ctrl+N: the new show comes from the engine (show_new returns the whole .spell); with no engine,
// an empty one.
TL.showNew = function () {
  if (!BUS.live()) return swap({ name: "new show", fps: 30, duration: 60, tracks: [], markers: [] });
  BUS.call("show_new", {}).then(sh => { TL.file = ""; swap(sh); })
    .catch(e => TL.log("show_new: " + e));
};

// Local clock of offline mode (no engine). No loop here: what repeats is the engine player.
function frame() {
  if (!TL.rate) return;
  const now = performance.now(), dt = (now - TL.last) / 1000;
  TL.last = now;
  const t = TL.t + dt * TL.rate;
  if (hasPlayer()) return setT(t);       // the engine is in charge; here we only interpolate between events
  if (t <= 0 || t >= TL.dur()) setRate(0);
  setT(t);
}

// ---- link to the engine -------------------------------------------------
// The engine transport is the source: state, time and loop. A named function because the test
// calls it.
TL.onTransport = d => {
  TL.tstate = d.state;
  if (typeof d.loop === "boolean") TL.loop = d.loop;
  setRate(d.state === "play" ? 1 : 0);
  TL.last = performance.now();
  setT(d.t);
};
BUS.on.transport = d => TL.onTransport(d);

// The `show` event only goes out when the engine `rev` changes; only what came from outside forces
// a reload.
BUS.on.show = d => {
  const r = TL.revEvent(d.rev, TL.rev);
  TL.rev = r.rev;
  if (r.reload) TL.reload();
};

BUS.on.log = d => TL.log(d.text);

TL.reload = function () {
  return fetch("/show").then(r => r.json()).then(sh => {
    swap(sh);
    return TL.recPull();                  // the lanes are born disarmed: the engine is the one who knows
  }).catch(e => TL.log("GET /show: " + e.message));
};

TL.connect = function () {
  BUS.open("ws://" + location.host + "/ws");
};

// ---- tracks -------------------------------------------------------------
// Args of `track_add` per track type. `laser` carries the .ild clip, `fx` carries the .rhai script
// (the two fields the registry adds to the track). Pure function: this is the one the test covers.
TL.trackArgs = function (kind, file) {
  const a = { type: kind || "dmx", universe: 1, address: 1, name: "" };
  if (a.type === "laser") a.clip = file || "";
  else if (a.type === "fx") a.script = file || "";
  return a;
};

TL.trackAdd = function (kind, file) {
  const args = TL.trackArgs(kind, file);
  const spec = { type: args.type, universe: 1, address: 1, keys: [] };   // same as track_add
  if (args.clip) spec.clip = args.clip;
  if (args.script) spec.script = args.script;
  TL.show.tracks.push(spec);
  send([{ cmd: "track_add", args: args }]);
  TL.load(TL.show);
  TL.cur = TL.lanes.length - 1;
};

// The .ild files the engine can see (`laser_files` command, default directory `shows/`). With no
// server there is no list: the page shows an empty menu instead of a native dialog.
TL.laserFiles = function () {
  if (!BUS.live()) return Promise.resolve([]);
  return BUS.call("laser_files", { dir: "" }).then(r => (r && r.files) || []).catch(e => {
    TL.log("laser_files: " + e);
    return [];
  });
};

// ---- record arm ---------------------------------------------------------
// The arm does NOT live in the .spell: the engine is the one that records, and the state comes
// from it (`rec_arm`/`rec_state`). Single funnel for the R button, the R key and the bar button.
TL.recArm = function (li, on) {
  const L = TL.lanes[li];
  if (!L) return;
  if (on === undefined) on = !L.rec;
  L.rec = on;                                   // optimistic; the `rec_state` of the response corrects it
  if (TL.k) TL.k.dirty = true;
  if (!BUS.live()) return TL.log("R: no server, nothing records");
  BUS.call("rec_arm", { track: L.si, on: on })
     .then(() => TL.recPull())
     .catch(e => { L.rec = false; if (TL.k) TL.k.dirty = true; TL.log("rec_arm: " + e); });
};

TL.recPull = function () {
  if (!BUS.live()) return Promise.resolve(null);
  return BUS.call("rec_state", {}).then(st => { TL.recApply(st); return st; }).catch(() => null);
};

// Marks in the lanes the tracks the engine says are armed. Pure function: this is the one the test
// covers (one lane per track, and the laser parameters share the track of their parent).
TL.recApply = function (st) {
  const arm = (st && st.tracks) || [];
  for (const L of TL.lanes) L.rec = !L.param && arm.indexOf(L.si) >= 0;
  if (TL.k) TL.k.dirty = true;
};

TL.trackDel = function () {
  const L = TL.lanes[TL.cur];
  if (!L) return;
  const i = L.si;
  TL.show.tracks.splice(i, 1);
  send([{ cmd: "track_del", args: { index: i } }]);
  TL.load(TL.show);
};

// ---- snapping (markers, in/out, playhead, visible keyframes) -----------
// While dragging a ruler handle, In and Out stay OUT of the list: the handle snapped to the other
// one and the range collapsed (In and Out 10 ms apart).
function buildSnaps() {
  const d = TL.k.drag, handle = !!d && (d.mode === "in" || d.mode === "out");
  const s = handle ? [0, TL.dur(), TL.t] : [0, TL.dur(), TL.inT(), TL.outT(), TL.t];
  for (const m of TL.show.markers) s.push(+m);
  const lo = x2t(TL.headW), hi = x2t(TL.k.w);
  for (let li = 0; li < TL.lanes.length && s.length < 4000; li++) {
    const L = TL.lanes[li], sel = TL.k.sel.m.get(li);
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

// ---- colors (tokens) ----------------------------------------------------
function colors() {
  TL.col = CK.colors(TL.k.cv);
  TL.k.dirty = true;
}
TL.colors = colors;

// ---- drawing ------------------------------------------------------------
function diamond(c, x, y, r) {
  c.moveTo(x, y - r); c.lineTo(x + r, y); c.lineTo(x, y + r); c.lineTo(x - r, y); c.closePath();
}

function draw(k) {
  const c = k.cx, W = k.w, H = k.h, col = TL.col, rh = TL.rowH, hw = TL.headW;
  const lo = x2t(hw), hi = x2t(W), fps = TL.fps();
  c.fillStyle = col.well;
  c.fillRect(0, 0, W, H);

  // Ceiling of the kit vertical scroll (wheel and middle button): the last lane stops at the
  // bottom instead of disappearing upwards. It lives in the drawing because this is where the
  // usable height is known.
  k.ymax = Math.max(0, TL.lanes.length * rh + TL.rulerH + (TL.mon ? MONH : 0) - H);

  const first = Math.max(0, Math.floor(k.view.y / rh));
  const last = Math.min(TL.lanes.length - 1, Math.floor((k.view.y + H) / rh));
  TL.drawn = Math.max(0, last - first + 1);

  // ---- rows: background, header, M / S / R ----
  c.textBaseline = "middle";
  for (let li = first; li <= last; li++) {
    const y = laneY(li), L = TL.lanes[li];
    c.fillStyle = li & 1 ? col.bg : col.well;
    c.fillRect(hw, y, W - hw, rh);
    c.fillStyle = col.panel;
    c.fillRect(0, y, hw, rh);
    if (li === TL.cur) {                                  // focused track: 1 px in the accent (PRINCIPIOS §2)
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

  // ---- In/Out range limits (the grey bar lives on the ruler; here only the two lines) ----
  const xi = t2x(TL.inT()), xo = t2x(TL.outT());
  c.strokeStyle = col.fg3; c.lineWidth = 1;
  c.beginPath();
  for (const x of [xi, xo]) { c.moveTo(Math.round(x) + 0.5, TL.rulerH); c.lineTo(Math.round(x) + 0.5, H); }
  c.stroke();

  // ---- vertical grid ----
  let step = STEPS[STEPS.length - 1];
  for (const s of STEPS) if (s * k.view.zoom >= 64) { step = s; break; }
  c.strokeStyle = col.line; c.lineWidth = 1;
  c.beginPath();
  for (let t = Math.ceil(lo / step) * step; t < hi; t += step) {
    const x = Math.round(t2x(t)) + 0.5;
    c.moveTo(x, TL.rulerH); c.lineTo(x, H);
  }
  c.stroke();

  // ---- markers (grey: SHORTCUTS.md) ----
  const mk = TL.show ? TL.show.markers : [];
  if (mk.length) {
    c.strokeStyle = col.fg3; c.beginPath();
    for (const m of mk) {
      if (m < lo || m > hi) continue;
      const x = Math.round(t2x(m)) + 0.5;
      c.moveTo(x, TL.rulerH); c.lineTo(x, H);
    }
    c.stroke();
  }

  // ---- value curve: one path per lane, sampled the way the engine interpolates ----
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

  // ---- keyframes: pass 0 normal, pass 1 selected ----
  for (let pass = 0; pass < 2; pass++) {
    c.beginPath();
    for (let li = first; li <= last; li++) {
      const L = TL.lanes[li], y = laneY(li), sc = (rh - 8) / ((L.vmax - L.vmin) || 1);
      const sel = k.sel.m.get(li);
      if (pass && !sel) continue;
      const i0 = dragLanes && dragLanes.has(li) ? 0 : CK.bisect(L.ts, L.n, lo);
      let lastX = -1e9;
      for (let i = i0; i < L.n; i++) {
        const x = t2x(L.ts[i]);
        if (x > W + 8) break;
        if (x < hw - 8) continue;
        if ((sel ? sel.has(i) : false) !== !!pass) continue;
        // ponytail: a minimum step of 3 px groups keyframes stuck together when zoomed out ; only
        // the drawing goes
        if (!pass && x - lastX < 3) continue;
        lastX = x;
        diamond(c, x, y + rh - 4 - (L.vs[i] - L.vmin) * sc, pass ? 5 : 4);
      }
    }
    c.fillStyle = pass ? col.accent : col.fg;
    c.fill();
  }
  c.restore();

  // ---- ruler ----
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
  const bi = Math.max(hw, xi), bo = Math.min(W, xo);       // In-Out range as a grey bar
  if (bo > bi) { c.fillStyle = col.fg3; c.fillRect(bi, TL.rulerH - 4, bo - bi, 3); }
  for (let b = 0; b < 2; b++) {                            // In / Out handles
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

  // ---- previz: dmx + ILDA frame + patch plan at the playhead (Alt+M) ----
  // ponytail: a strip laid over the bottom, with no panel of its own ; make it a panel once the
  // GUI has a layout.
  if (TL.mon && window.VW) window.VW.draw(c, { x: 0, y: H - MONH, w: W, h: MONH }, TL.t);

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

// Keyframe under (x, y) in lane li, or -1. Bisect on time; the Y only breaks ties.
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

// ---- interaction --------------------------------------------------------
function onDown(p) {
  const k = TL.k, x = p.x, y = p.y;
  hideMenu();
  if (y < TL.rulerH) {                                    // ruler: In/Out handles or scrub
    if (x > TL.headW) {
      const d = t => Math.abs(t2x(t) - x);
      if (d(TL.inT()) < 7) { k.drag = { mode: "in" }; buildSnaps(); return true; }
      if (d(TL.outT()) < 7) { k.drag = { mode: "out" }; buildSnaps(); return true; }
      k.drag = { mode: "scrub" };
      TL.locate(x2t(x));
    }
    return true;
  }
  const li = laneAt(y);
  if (li < 0) { if (!p.shift) k.sel.clear(); k.dirty = true; return true; }
  TL.cur = li;
  const L = TL.lanes[li];
  if (x < TL.headW) {                                     // header: M / S / R
    const ly = laneY(li) + TL.rowH / 2;
    if (Math.abs(y - ly) < 8 && x > TL.headW - 58 && x < TL.headW - 5) {
      const b = Math.floor((x - (TL.headW - 58)) / 19);
      if (b === 0) { L.mute = !L.mute; trackFlag(L, "mute", L.mute); }
      else if (b === 1) { L.solo = !L.solo; trackFlag(L, "solo", L.solo); }
      else TL.recArm(li);
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
    k.drag = { mode: "keys", x, y, items, lanes: new Set(k.sel.m.keys()), moved: false, anchor: L.ts[ki] };
    k.dirty = true;
    return true;
  }
  return false;                                           // empty: the kit opens the marquee
}

function onMove(p, d) {
  const k = TL.k;
  if (d.mode === "scrub") { setT(x2t(p.x)); d.moved = true; }
  // During the drag only the show border limits it; whoever crossed the other limit is normalized
  // by the `TL.setInOut` of onUp — before, the clamp here left In and Out 10 ms apart.
  else if (d.mode === "in") TL.show.in = clamp(snapT(x2t(p.x)), 0, TL.dur());
  else if (d.mode === "out") TL.show.out = clamp(snapT(x2t(p.x)), 0, TL.dur());
  else if (d.mode === "keys") {
    d.moved = true;
    let dt = (p.x - d.x) / k.view.zoom;
    if (!p.shift) dt = snapT(d.anchor + dt) - d.anchor;   // Shift releases the snapping
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
    // the edits go out BEFORE the resort: it is the resort that shuffles the indices of d.items
    const eds = d.items.map(it => eKey(it.li, it.ki, it.t));
    for (const li of d.lanes) resort(li);
    commit(eds);
  } else if (d.mode === "in") TL.setInOut(TL.show.in, null);
  else if (d.mode === "out") TL.setInOut(null, TL.show.out);
  else if (d.mode === "scrub" && d.moved) TL.locate(TL.t);   // one locate at the end, not one per frame
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
  for (const li of k.sel.m.keys()) { TL.cur = li; break; }
}

// ---- context menu (easing) ---------------------------------------------
function hideMenu() { if (TL.menuEl) TL.menuEl.style.display = "none"; }

// Opens the easing menu at (x, y) of the canvas. Right button and Ctrl+E come in through here.
function menuAt(x, y) {
  if (!TL.menuEl || !TL.k.sel.count()) return;
  const r = TL.k.cv.getBoundingClientRect();
  TL.menuEl.style.display = "block";
  TL.menuEl.style.left = (r.left + x) + "px";
  TL.menuEl.style.top = (r.top + y) + "px";
}

function onMenu(p) {
  const li = laneAt(p.y);
  if (li < 0 || !TL.menuEl) return;
  const ki = keyAt(li, p.x, p.y);
  if (ki >= 0 && !TL.k.sel.has(li, ki)) { TL.k.sel.clear(); TL.k.sel.add(li, ki); TL.k.dirty = true; }
  menuAt(p.x, p.y);
}

function setCurve(name) {
  const ci = CURVES.indexOf(name), eds = [];
  TL.k.sel.each((li, i) => { TL.lanes[li].cu[i] = ci; eds.push(eKey(li, i)); });
  hideMenu();
  commit(eds);
}
TL.setCurve = setCurve;

// ---- shortcuts (design/SHORTCUTS.md) ------------------------------------
function step(n) { TL.locate(TL.t + n / TL.fps()); }

/// Moves to the neighbouring instant in an ALREADY ordered list (lane keys, show markers).
function jump(ts, n, dir) {
  const i = CK.bisect(ts, n, TL.t + (dir > 0 ? 1e-6 : -1e-6));
  const j = dir > 0 ? i : i - 1;
  if (j >= 0 && j < n) TL.locate(ts[j]);
}

function jumpKey(dir) {
  const L = TL.lanes[TL.cur];
  if (L && L.n) jump(L.ts, L.n, dir);
}

function jumpMarker(dir) {
  const m = TL.show.markers;
  jump(m, m.length, dir);
}

// Shift+M: edits the marker closest to the playhead. A marker here is only an instant (a number)
// in the show — it has no name to rename: the prompt takes the new time, and empty deletes it.
// ponytail: the browser `prompt` ; it becomes a field on the ruler itself once the GUI has a dialog.
function editMarker() {
  const m = TL.show.markers;
  const i = CK.near(m, m.length, TL.t, 12 / TL.k.view.zoom);
  if (i < 0) return TL.log("Shift+M: no marker near the playhead");
  const r = prompt("marker " + tc(m[i], TL.fps()) + ": time in seconds (empty deletes)", String(m[i]));
  if (r === null) return;
  if (r.trim() === "") m.splice(i, 1);
  else {
    const v = parseFloat(r);
    if (!isFinite(v)) return TL.log("invalid time: " + r);
    m[i] = clamp(v, 0, TL.dur());
    m.sort((a, b) => a - b);
  }
  field("/markers", m);
}

// K held + J/L steps frame by frame (Premiere). The keyup of `mount` only exists because of this.
let kHeld = false;

function onKey(e) {
  if (/^(INPUT|SELECT|TEXTAREA)$/.test(e.target.tagName)) return;
  const k = TL.k, key = e.key, code = e.code, ctrl = e.ctrlKey, shift = e.shiftKey, alt = e.altKey;
  const kb = key.length === 1 ? key.toLowerCase() : key;   // the key; the Shift is read in `shift`
  let used = true;
  if (code === "Space" && !ctrl) TL.play(TL.rate ? 0 : 1);
  // K held + J/L = frame by frame; on their own, J/L are the shuttle (each press doubles, max 8x).
  else if (kb === "j" && kHeld && !ctrl) step(-1);
  else if (kb === "l" && kHeld && !ctrl) step(1);
  else if (kb === "j" && !ctrl) TL.play(TL.rate < 0 ? -Math.min(8, -TL.rate * 2) : -1);
  else if (kb === "k" && !ctrl) { kHeld = true; TL.play(0); }
  else if (kb === "l" && !ctrl) TL.play(TL.rate > 0 ? Math.min(8, TL.rate * 2) : 1);
  else if (key === "ArrowLeft" && ctrl && shift) jumpMarker(-1);
  else if (key === "ArrowRight" && ctrl && shift) jumpMarker(1);
  else if (key === "ArrowLeft") step(shift ? -5 : -1);
  else if (key === "ArrowRight") step(shift ? 5 : 1);
  else if (key === "ArrowUp") jumpKey(-1);
  else if (key === "ArrowDown") jumpKey(1);
  else if (key === "Home") TL.locate(0);
  else if (key === "End") TL.locate(TL.dur());
  else if (kb === "m" && alt) { TL.mon = !TL.mon; TL.onState(); }
  else if (kb === "i" && alt) TL.setInOut(0, null);
  else if (kb === "o" && alt) TL.setInOut(null, TL.dur());
  else if (kb === "x" && alt) TL.setInOut(0, TL.dur());
  else if (kb === "i" && shift) TL.locate(TL.inT());
  else if (kb === "o" && shift) TL.locate(TL.outT());
  else if (kb === "i" && !ctrl) TL.setInOut(TL.t, null);
  else if (kb === "o" && !ctrl) TL.setInOut(null, TL.t);
  else if (ctrl && kb === "l") TL.setLoop(!TL.loop);
  else if (ctrl && shift && kb === "s") TL.saveAs();
  else if (ctrl && kb === "s") TL.save();
  else if (ctrl && kb === "n") TL.showNew();
  else if (ctrl && kb === "o") TL.onOpen();
  else if (ctrl && shift && kb === "z") TL.undo(1);
  else if (ctrl && kb === "z") TL.undo(-1);
  else if (kb === "m" && shift) editMarker();
  else if (kb === "m" && !ctrl) {
    TL.show.markers.push(+TL.t.toFixed(4));
    TL.show.markers.sort((a, b) => a - b);   // the list stays ordered: `jumpMarker` counts on it
    field("/markers", TL.show.markers);
  }
  else if (key === "=" || key === "+") k.zoomAt(k.gutter + (k.w - k.gutter) / 2, 1.25);
  else if (key === "-" || key === "_") k.zoomAt(k.gutter + (k.w - k.gutter) / 2, 0.8);
  else if (key === "\\" || (shift && kb === "z")) TL.fit();
  else if (ctrl && kb === "k") {
    if (TL.cur >= 0) {
      const L = TL.lanes[TL.cur], v = TL.valueAt(L, TL.t);
      k.sel.clear();
      k.sel.add(TL.cur, addKey(TL.cur, TL.t, v === null ? 0 : v, null, 0));
      commit([eAt(TL.cur, TL.t)]);
    }
  } else if (ctrl && shift && kb === "a") k.sel.clear();
  else if (ctrl && kb === "a") {
    k.sel.clear();
    for (let li = 0; li < TL.lanes.length; li++) for (let i = 0; i < TL.lanes[li].n; i++) k.sel.add(li, i);
  } else if (ctrl && (kb === "c" || kb === "x")) {
    let t0 = Infinity;
    const cl = [];
    k.sel.each((li, i) => {
      const L = TL.lanes[li];
      cl.push({ li, t: L.ts[i], v: L.vs[i], raw: L.raw[i], cu: L.cu[i] });
      if (L.ts[i] < t0) t0 = L.ts[i];
    });
    for (const c of cl) c.t -= t0;
    TL.clip = cl;
    if (kb === "x") delSelected();
  } else if (ctrl && kb === "v" && TL.clip) {
    k.sel.clear();
    const ts = [];
    for (const c of TL.clip) {
      if (c.li >= TL.lanes.length) continue;
      k.sel.add(c.li, addKey(c.li, TL.t + c.t, c.v, c.raw, c.cu));
      ts.push([c.li, TL.t + c.t]);
    }
    commit(ts.map(a => eAt(a[0], a[1])));
  } else if (key === "Delete" || key === "Backspace") delSelected();
  else if (ctrl && shift && kb === "e") {
    const eds = [];
    k.sel.each((li, i) => { const L = TL.lanes[li]; L.cu[i] = (L.cu[i] + 1) % CURVES.length; eds.push(eKey(li, i)); });
    commit(eds);
  } else if (ctrl && kb === "e") {
    if (k.sel.count()) menuAt(clamp(t2x(TL.t), TL.headW, Math.max(TL.headW, k.w - 90)),
                              laneY(Math.max(0, TL.cur)) + TL.rowH);
    else TL.log("Ctrl+E: select a keyframe");
  } else if (kb === "s" && !ctrl && !shift) TL.snap = !TL.snap;
  else if (kb === "d" && shift && TL.cur >= 0) { const L = TL.lanes[TL.cur]; L.mute = !L.mute; trackFlag(L, "mute", L.mute); }
  else if (kb === "s" && shift && TL.cur >= 0) { const L = TL.lanes[TL.cur]; L.solo = !L.solo; trackFlag(L, "solo", L.solo); }
  else if (kb === "r" && !ctrl && TL.cur >= 0) TL.recArm(TL.cur);
  else used = false;
  if (used) { e.preventDefault(); k.dirty = true; }
}
TL.onKey = onKey;

TL.fit = function () {
  if (TL.k.w < 2) TL.k.resize();   // show loaded before the first layout: measure the canvas now
  TL.k.fit(0, TL.dur());
};

// ---- mounting -----------------------------------------------------------
TL.mount = function (cv, menuEl, msgEl) {
  const k = TL.k = CK.attach(cv, draw);
  k.gutter = TL.headW;
  k.on = { down: onDown, move: onMove, up: onUp, marquee: onMarquee, menu: onMenu, frame };
  TL.menuEl = menuEl || null;
  TL.msgEl = msgEl || null;
  colors();
  if (TL.menuEl) {
    for (const n of CURVES) {
      const b = document.createElement("button");
      b.textContent = n;
      b.onclick = () => setCurve(n);
      TL.menuEl.appendChild(b);
    }
    const d = document.createElement("button");
    d.textContent = "delete";
    d.onclick = () => { hideMenu(); delSelected(); };
    TL.menuEl.appendChild(d);
    addEventListener("pointerdown", e => { if (!TL.menuEl.contains(e.target)) hideMenu(); }, true);
  }
  addEventListener("keydown", onKey);
  addEventListener("keyup", e => { if (e.key && e.key.toLowerCase() === "k") kHeld = false; });
  // Alt+Tab with K held sends no keyup: without this, J/L stay stuck in frame-by-frame.
  addEventListener("blur", () => { kHeld = false; });
  return k;
};
