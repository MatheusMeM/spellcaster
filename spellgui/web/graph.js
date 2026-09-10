"use strict";
// graph.js — PATCHBAY: the editor of the graph command network (PRD §10, design/FUNCOES/orquestrador.md).
// Two halves in the same file:
//   GM  — PURE model (JSON Patch, edit ops, reconnect, group). No DOM: this is what node --test runs.
//   PB  — the page (canvas on top of canvaskit, Inspector, log, shortcuts). Only `PB.init()` touches the DOM.
//
// Every edit becomes `show_patch {ops}` on /graph/... and stacks the `undo` it returns (patch frente).
// Offline (no engine) the same path runs locally: GM.patch applies and returns the inverse.
//
// ponytail: no minimap, no auto-layout, no collaborative editing ; they come in when a real show
// goes past one screen of nodes.

// node --test: catalog.js is a CommonJS module; in the browser it has already declared CATALOG global.
if (typeof require === "function") global.CATALOG = require("./catalog.js");

// ---------------------------------------------------------------- pure model (no DOM)

const GM = {
  modules: {},          // module.json (module frente) already turned into a node definition, by name

  def(n) { return CATALOG.nodeDef(n, GM.modules); },

  // JSON Pointer (RFC 6901). ponytail: no unescape of ~0/~1 ; no graph segment has "/" or "~".
  seg(p) { return p.split("/").slice(1); },

  get(doc, path) {
    let c = doc;
    for (const s of GM.seg(path)) {
      if (c === null || c === undefined) return undefined;
      c = Array.isArray(c) ? c[+s] : c[s];
    }
    return c;
  },

  parent(doc, path) {
    const p = GM.seg(path), last = p.pop();
    let c = doc;
    for (const s of p) {
      if (c === null || c === undefined) return null;
      c = Array.isArray(c) ? c[+s] : c[s];
    }
    return c === null || c === undefined ? null : { c, last };
  },

  clone(v) { return structuredClone(v); },

  // Applies the whole list or nothing: returns {doc, undo, error}. `undo` undoes in reverse order.
  // ponytail: only add, remove and replace of RFC 6902 ; `test` and `move` come in when some op
  // builder needs them (reordering today is remove+add).
  patch(doc, ops) {
    const d = GM.clone(doc), undo = [];
    for (const op of ops) {
      const e = GM.one(d, op, undo);
      if (e) return { doc, undo: [], error: e };
    }
    undo.reverse();
    return { doc: d, undo, error: "" };
  },

  one(d, op, undo) {
    const p = op.path === undefined ? "" : op.path;
    const target = GM.parent(d, p);
    if (!target) return `path with no parent: ${p}`;
    const { c, last } = target;
    if (Array.isArray(c)) {
      const i = last === "-" ? c.length : +last;
      if (op.op === "add") {
        if (!(i >= 0 && i <= c.length)) return `index out of the list: ${p}`;
        c.splice(i, 0, GM.clone(op.value));
        undo.push({ op: "remove", path: p.replace(/\/[^/]*$/, "/" + i) });
        return "";
      }
      if (!(i >= 0 && i < c.length)) return `index out of the list: ${p}`;
      if (op.op === "remove") {
        undo.push({ op: "add", path: p.replace(/\/[^/]*$/, "/" + i), value: GM.clone(c[i]) });
        c.splice(i, 1);
        return "";
      }
      undo.push({ op: "replace", path: p, value: GM.clone(c[i]) });
      c[i] = GM.clone(op.value);
      return "";
    }
    if (typeof c !== "object") return `path is neither object nor list: ${p}`;
    const had = Object.prototype.hasOwnProperty.call(c, last);
    if (op.op === "remove") {
      if (!had) return `remove: ${p} does not exist`;
      undo.push({ op: "add", path: p, value: GM.clone(c[last]) });
      delete c[last];
      return "";
    }
    if (op.op === "replace" && !had) return `replace: ${p} does not exist`;
    undo.push(had ? { op: "replace", path: p, value: GM.clone(c[last]) } : { op: "remove", path: p });
    c[last] = GM.clone(op.value);
    return "";
  },

  // ------------------------------------------------------------ graph inside the show

  g(doc) { return (doc && doc.graph) || { nodes: [], edges: [] }; },

  // "node.pin" -> ["node", "pin"] (the same rsplit_once('.') of graph.rs: the id may have a dot,
  // the pin may not)
  side(s) { const i = String(s).lastIndexOf("."); return [s.slice(0, i), s.slice(i + 1)]; },

  idx(doc, id) { return GM.g(doc).nodes.findIndex(n => n.id === id); },
  node(doc, id) { return GM.g(doc).nodes.find(n => n.id === id) || null; },

  // Free id from the type: "logic.toggle" -> "toggle", "toggle2", ...
  newId(doc, type) {
    const base = String(type).split(".").pop();
    const ids = new Set(GM.g(doc).nodes.map(n => n.id));
    if (!ids.has(base)) return base;
    for (let i = 2; ; i++) if (!ids.has(base + i)) return base + i;
  },

  opsAdd(doc, node) {
    return [{ op: "add", path: "/graph/nodes/-", value: node }];
  },

  // A show with no `graph` (the case of medgrupo.spell): `add /graph/nodes/-` finds no parent
  // either here or in the engine. The FIRST edit creates the graph along with it, in the same op
  // list — and the undo of the add (`remove /graph`) gives the show back exactly as it was.
  withGraph(doc, ops) {
    if (doc && doc.graph) return ops;
    return [{ op: "add", path: "/graph", value: { nodes: [], edges: [] } }].concat(ops);
  },

  // Deletes nodes and every cable that touches them. Indices in descending order: the patch is
  // sequential.
  opsDel(doc, ids) {
    const g = GM.g(doc), set = new Set(ids), ops = [];
    const touches = e => set.has(GM.side(e[0])[0]) || set.has(GM.side(e[1])[0]);
    g.edges.forEach((e, i) => { if (touches(e)) ops.push({ op: "remove", path: `/graph/edges/${i}` }); });
    g.nodes.forEach((n, i) => { if (set.has(n.id)) ops.push({ op: "remove", path: `/graph/nodes/${i}` }); });
    return ops.reverse();
  },

  // Shift+Delete: deletes reconnecting the input of the node to its output (Blender
  // delete_reconnect). Each deleted node links the source of its first input cable to every
  // destination of the outputs.
  opsDelReconnect(doc, ids) {
    const g = GM.g(doc), set = new Set(ids), fresh = [];
    for (const id of ids) {
      const ins = g.edges.filter(e => GM.side(e[1])[0] === id && !set.has(GM.side(e[0])[0]));
      const outs = g.edges.filter(e => GM.side(e[0])[0] === id && !set.has(GM.side(e[1])[0]));
      if (!ins.length) continue;
      for (const s of outs) {
        const pair = [ins[0][0], s[1]];
        if (GM.why(doc, pair[0], pair[1])) continue;                 // incompatible type: no reconnect
        if (!fresh.some(x => x[0] === pair[0] && x[1] === pair[1])) fresh.push(pair);
      }
    }
    return GM.opsDel(doc, ids).concat(
      fresh.map(e => ({ op: "add", path: "/graph/edges/-", value: e })),
    );
  },

  // "" when the cable may exist, otherwise the reason.
  why(doc, from, to) {
    const [ai, ap] = GM.side(from), [bi, bp] = GM.side(to);
    const a = GM.node(doc, ai), b = GM.node(doc, bi);
    if (!a || !b) return "node does not exist";
    if (ai === bi) return "cable from the node into itself";
    return CATALOG.compat(CATALOG.port(GM.def(a), ap, true), CATALOG.port(GM.def(b), bp, false));
  },

  // An input takes ONE cable: linking replaces what was there (the "swap the input" of TouchDesigner).
  opsLink(doc, from, to) {
    const g = GM.g(doc), ops = [];
    g.edges.forEach((e, i) => { if (e[1] === to) ops.push({ op: "remove", path: `/graph/edges/${i}` }); });
    ops.reverse();
    ops.push({ op: "add", path: "/graph/edges/-", value: [from, to] });
    return ops;
  },

  opsKey(doc, id, key, value) {
    const i = GM.idx(doc, id);
    if (i < 0) return [];
    const had = Object.prototype.hasOwnProperty.call(GM.g(doc).nodes[i], key);
    return [{ op: had ? "replace" : "add", path: `/graph/nodes/${i}/${key}`, value: value }];
  },

  // Fields the Inspector shows for a node: the cfg of the TYPE (catalog.js) plus the universal
  // keys, each one with the value the node has today. The cfg lives FLAT on the node — `n.every`,
  // and not `n.cfg.every`: that is how the runtime reads it (`n.get("every")` in graph.rs), so the
  // patch of a field is `/graph/nodes/<i>/<field>`, which is what `opsKey` already writes.
  fields(doc, id) {
    const n = GM.node(doc, id);
    if (!n) return [];
    const d = GM.def(n);
    return Object.entries(Object.assign({}, d ? d.cfg : {}, CATALOG.UNIVERSAL))
      .map(([k, t]) => [k, t, n[k]]);
  },

  // Field text -> stored value, or {error}. A numeric field that is empty or has junk does NOT get
  // stored: NaN becomes `null` in the JSON and the engine would fall back to the default with
  // nobody seeing why.
  value(type, raw) {
    if (type === "bool") return { v: !!raw };
    if (type === "number") {
      const v = Number(raw);
      return raw === "" || Number.isNaN(v) ? { error: `invalid number: "${raw}"` } : { v };
    }
    if (type === "json") {
      try {
        return { v: JSON.parse(raw) };
      } catch (e) {
        return { error: `invalid json: ${e.message}` };
      }
    }
    return { v: String(raw) };
  },

  opsMove(doc, mov) {
    const ops = [];
    for (const [id, x, y] of mov) {
      ops.push(...GM.opsKey(doc, id, "x", Math.round(x)));
      ops.push(...GM.opsKey(doc, id, "y", Math.round(y)));
    }
    return ops;
  },

  // ------------------------------------------------------------ groups

  group(n) { return (n && n.group) || ""; },

  // The selection holds a node id OR the name of a closed group (the drawing treats the group as a
  // node); every edit works with node ids, so a group name becomes the list of its members.
  ids(doc, sel) {
    return [...sel].flatMap(s => (GM.node(doc, s) ? [s]
      : GM.g(doc).nodes.filter(n => GM.group(n) === s).map(n => n.id)));
  },

  // Nodes visible in the context `ctx` and the closed groups. Stable order: the file order.
  // ponytail: a group is a FLAT name (the `group` key of the node), with no nesting ; make it an
  // "a/b" path when a real show has a group inside a group.
  visible(doc, ctx) {
    const g = GM.g(doc), inside = [], groups = new Map();
    for (const n of g.nodes) {
      const gr = GM.group(n);
      if (gr === ctx) inside.push(n);
      else if (ctx === "") groups.set(gr, (groups.get(gr) || []).concat([n]));
    }
    return { nodes: inside, groups };
  },

  // Pins of a closed group: the cables that cross the border, ordered by id.pin (deterministic).
  groupPorts(doc, members) {
    const g = GM.g(doc), set = new Set(members.map(n => n.id)), ins = [], outs = [];
    for (const e of g.edges) {
      const from = set.has(GM.side(e[0])[0]), to = set.has(GM.side(e[1])[0]);
      if (from === to) continue;
      (to ? ins : outs).push({ inner: to ? e[1] : e[0], outer: to ? e[0] : e[1] });
    }
    const ord = (a, b) => a.inner.localeCompare(b.inner);
    ins.sort(ord); outs.sort(ord);
    return { ins, outs };
  },
};

// ---------------------------------------------------------------- the page

const NW = 176, TH = 22, PH = 14, CFGH = 14;   // node width, title height, pin step

const PB = {
  doc: { graph: { nodes: [], edges: [] } },
  ctx: "", sel: new Set(), selEdge: -1, undo: [], redo: [],
  errors: {}, k: null, bus: null, mx: 0, my: 0, el: {},
  editing: false,      // Inspector field in the middle of a commit (see PB.inspector)

  /// Socket open? The `Bus` drops a request on a closed socket; the patchbay prefers not to send.
  live() { return !!(PB.bus && PB.bus.ws && PB.bus.ws.readyState === 1); },

  // world -> screen
  sx(wx) { return (wx - PB.k.view.x) * PB.k.view.zoom; },
  sy(wy) { return wy * PB.k.view.zoom - PB.k.view.y; },

  // node height from the number of pins
  nodeH(n) {
    const d = GM.def(n);
    const p = d ? Math.max(Object.keys(d.ins).length, Object.keys(d.outs).length) : 1;
    return TH + Math.max(1, p) * PH + CFGH;
  },

  // Box of the node in world coordinates. During the drag the delta belongs to the drag: the model
  // only changes on `up`.
  box(n) {
    const d = PB.k && PB.k.drag && PB.k.drag.mode === "node" ? PB.k.drag : null;
    const m = d && d.mov.get(n.id);
    return {
      x: m ? m[0] + d.dx : +n.x || 0,
      y: m ? Math.max(0, m[1] + d.dy) : +n.y || 0,
      w: NW, h: PB.nodeH(n),
    };
  },

  // ------------------------------------------------------------ editing (a single path)

  // Sends the list to the engine. A refusal takes the local doc back to the previous state
  // (`GM.patch` does not touch the original: the reference we kept ALREADY is the untouched copy)
  // and `back` unwinds the stack — no local doc running ahead of the engine. Accepted or refused,
  // it revalidates at the end.
  send(ops, before, back) {
    if (!PB.live()) return Promise.resolve(PB.check());
    // The `rev` of the response is kept by `Bus` itself (`bus.rev`); it is against that one that
    // the event decides.
    return PB.bus.call("show_patch", { ops })
      .catch(e => {
        PB.doc = before;
        back();
        PB.k.invalidate();
        PB.inspector();
        PB.log("show_patch: " + e.message);
      })
      .then(PB.check);
  },

  // Every edit goes through here: sends show_patch, stacks the undo, revalidates with graph_check.
  apply(ops, label) {
    // An empty list = the edit generated no op at all (locked node, node that vanished). It goes
    // out through the log: a silent refusal becomes "clicked and nothing happened", which is what
    // the owner must not see.
    if (!ops.length) {
      PB.log("no effect: " + (label || "edit with no op"));
      return Promise.resolve();
    }
    ops = GM.withGraph(PB.doc, ops);
    const before = PB.doc;
    const r = GM.patch(PB.doc, ops);
    if (r.error) { PB.log("error: " + r.error); return Promise.resolve(); }
    PB.doc = r.doc;
    PB.redo.length = 0;
    PB.undo.push(r.undo);
    PB.k.invalidate();
    PB.inspector();
    if (label) PB.log(label);
    return PB.send(ops, before, () => PB.undo.pop());
  },

  step(stackA, stackB) {
    const ops = stackA.pop();
    if (!ops) return;
    const before = PB.doc;
    const r = GM.patch(PB.doc, ops);
    if (r.error) { PB.log("undo: " + r.error); return; }
    PB.doc = r.doc;
    stackB.push(r.undo);
    PB.k.invalidate();
    PB.inspector();
    PB.send(ops, before, () => { stackB.pop(); stackA.push(ops); });
  },

  // graph_check after each accepted edit and on reload: one error per node, marked on the box and
  // in the Inspector. NO argument — the command compiles the graph of the show OPEN IN THE ENGINE,
  // and that is the point: after an accepted show_patch the page doc is the engine one, so the
  // source of the validation is the engine. Offline (with no live bus) there is nothing to validate.
  // ponytail: the engine returns ONE error ; it becomes a count per node once graph_check returns
  // a list.
  check() {
    PB.errors = {};
    if (!PB.live()) {
      PB.el.warn.textContent = "";
      PB.el.warn.className = "over";
      PB.k.invalidate();
      return Promise.resolve();
    }
    return PB.bus.call("graph_check", {}).then(r => {
      const txt = r && r.error;
      if (txt) {
        const m = /"([^"]+)"/.exec(txt);              // graph.rs quotes the guilty node
        PB.errors[m ? m[1] : ""] = txt;
        PB.log("graph_check: " + txt);
      }
      PB.el.warn.textContent = txt ? "1 error" : `${(r && r.nodes) || 0} nodes ok`;
      PB.el.warn.className = "over" + (txt ? " ruim" : "");
      PB.k.invalidate();
    }).catch(() => {});
  },

  log(t) {
    const d = document.createElement("div");
    d.textContent = t;
    PB.el.log.appendChild(d);
    while (PB.el.log.childElementCount > 200) PB.el.log.firstChild.remove();
    PB.el.log.scrollTop = PB.el.log.scrollHeight;
  },

  // ------------------------------------------------------------ geometry

  // Point of the pin in world coordinates; a member of a closed group comes out through the group
  // border.
  anchor(id, pin, output) {
    const vis = PB.vis;
    const n = vis.map.get(id);
    if (n) {
      const c = PB.box(n), d = GM.def(n);
      const list = Object.keys(output ? d.outs : d.ins);
      const i = Math.max(0, list.indexOf(pin));
      return { x: c.x + (output ? c.w : 0), y: c.y + TH + i * PH + PH / 2 };
    }
    const gb = vis.ofMember.get(id);
    if (!gb) return null;
    const list = output ? gb.ports.outs : gb.ports.ins;
    const i = Math.max(0, list.findIndex(p => GM.side(p.inner)[0] === id));
    return { x: gb.x + (output ? gb.w : 0), y: gb.y + TH + i * PH + PH / 2 };
  },

  // Recomputes what is visible in the current context (loose nodes + closed groups).
  build() {
    const v = GM.visible(PB.doc, PB.ctx);
    const map = new Map(v.nodes.map(n => [n.id, n]));
    const boxes = [], ofMember = new Map();
    for (const [name, members] of v.groups) {
      const cs = members.map(n => PB.box(n));
      const ports = GM.groupPorts(PB.doc, members);
      const gb = {
        group: name, members, ports,
        x: Math.min(...cs.map(c => c.x)), y: Math.min(...cs.map(c => c.y)), w: NW,
        h: TH + Math.max(1, ports.ins.length, ports.outs.length) * PH + CFGH,
      };
      boxes.push(gb);
      for (const m of members) ofMember.set(m.id, gb);
    }
    PB.vis = { nodes: v.nodes, map, groups: boxes, ofMember };
  },

  // node or group under the world point (closed group on top, as in the drawing)
  findNode(x, y) {
    const inside = c => x >= c.x && x <= c.x + c.w && y >= c.y && y <= c.y + c.h;
    for (let i = PB.vis.groups.length - 1; i >= 0; i--) if (inside(PB.vis.groups[i])) return PB.vis.groups[i];
    for (let i = PB.vis.nodes.length - 1; i >= 0; i--) {
      if (inside(PB.box(PB.vis.nodes[i]))) return PB.vis.nodes[i];
    }
    return null;
  },

  // pin under the point (radius of 7 world px)
  findPin(x, y) {
    for (const n of PB.vis.nodes) {
      const d = GM.def(n);
      if (!d) continue;
      for (const output of [false, true]) {
        const list = Object.keys(output ? d.outs : d.ins);
        for (const p of list) {
          const a = PB.anchor(n.id, p, output);
          if (a && Math.abs(a.x - x) < 8 && Math.abs(a.y - y) < 7) {
            return { id: n.id, pin: p, output, x: a.x, y: a.y };
          }
        }
      }
    }
    return null;
  },

  // cable under the SCREEN point (canvas px): the drawing stroke itself answers the hit-test
  findCable(px, py) {
    const g = GM.g(PB.doc), cx = PB.k.cx;
    cx.save();                                                // the lineWidth below belongs to the
    cx.lineWidth = 10;                                        // hit-test, click tolerance in screen px
    let found = -1;
    for (let i = g.edges.length - 1; i >= 0 && found < 0; i--) {
      const p = PB.cablePoints(g.edges[i]);
      if (p && cx.isPointInStroke(wirePath(p), px * PB.k.dpr, py * PB.k.dpr)) found = i;
    }
    cx.restore();                                             // not to the drawing of the next frame.
    return found;
  },

  cablePoints(e) {
    const [ai, ap] = GM.side(e[0]), [bi, bp] = GM.side(e[1]);
    const a = PB.anchor(ai, ap, true), b = PB.anchor(bi, bp, false);
    if (!a || !b) return null;
    if (PB.vis.ofMember.get(ai) && PB.vis.ofMember.get(ai) === PB.vis.ofMember.get(bi)) return null;
    return { a, b };
  },
};

// Cable stroke in screen coordinates: the drawing and the hit-test use the same one.
function wirePath(p) {
  const d = Math.max(30, Math.abs(p.b.x - p.a.x) * 0.5), c = new Path2D();
  c.moveTo(PB.sx(p.a.x), PB.sy(p.a.y));
  c.bezierCurveTo(PB.sx(p.a.x + d), PB.sy(p.a.y), PB.sx(p.b.x - d), PB.sy(p.b.y),
                  PB.sx(p.b.x), PB.sy(p.b.y));
  return c;
}

// ---------------------------------------------------------------- drawing


function draw(k) {
  const cx = k.cx, z = k.view.zoom;
  PB.build();
  // A bare wheel scrolls the content (canvaskit.js), and the one who knows where the graph ends is
  // the graph: with no `ymax` the `Infinity` of canvaskit lets it scroll into the endless void.
  const bottom = PB.vis.nodes.reduce((m, n) => Math.max(m, PB.box(n).y + PB.box(n).h), 0);
  k.ymax = Math.max(0, (bottom + 40) * z - k.h);
  cx.clearRect(0, 0, k.w, k.h);
  cx.fillStyle = PB.col.well;
  cx.fillRect(0, 0, k.w, k.h);

  // console grid (8 px of the design system), only when it can be seen
  if (z > 0.35) {
    cx.strokeStyle = PB.col.hair;
    cx.lineWidth = 1;
    cx.beginPath();
    const p = 48 * z;
    for (let x = -((k.view.x * z) % p); x < k.w; x += p) { cx.moveTo(x | 0, 0); cx.lineTo(x | 0, k.h); }
    for (let y = -(k.view.y % p); y < k.h; y += p) { cx.moveTo(0, y | 0); cx.lineTo(k.w, y | 0); }
    cx.stroke();
  }

  // state boxes: they wrap the nodes whose "state" key points at the state node
  for (const s of PB.vis.nodes.filter(n => n.type === "state")) {
    const m = PB.vis.nodes.filter(n => n.state === s.id);
    if (!m.length) continue;
    const cs = m.map(n => PB.box(n));
    const x0 = Math.min(...cs.map(c => c.x)) - 12, y0 = Math.min(...cs.map(c => c.y)) - 12;
    const x1 = Math.max(...cs.map(c => c.x)) + NW + 12, y1 = Math.max(...cs.map(c => c.y + c.h)) + 12;
    cx.strokeStyle = PB.col.rehearsal;
    cx.setLineDash([6, 4]);
    cx.strokeRect(PB.sx(x0), PB.sy(y0), (x1 - x0) * z, (y1 - y0) * z);
    cx.setLineDash([]);
    cx.fillStyle = PB.col.rehearsal;
    cx.font = `${Math.max(9, 10 * z)}px ${PB.col.mono}`;
    cx.fillText(`state ${s.id}`, PB.sx(x0) + 4, PB.sy(y0) - 3);
  }

  // cables
  GM.g(PB.doc).edges.forEach((e, i) => {
    const p = PB.cablePoints(e);
    if (!p) return;
    cx.strokeStyle = i === PB.selEdge ? PB.col.accent : PB.col.fg3;
    cx.lineWidth = i === PB.selEdge ? 2 : 1.5;
    cx.stroke(wirePath(p));
  });

  // cable under construction
  if (k.drag && k.drag.mode === "wire") {
    cx.strokeStyle = PB.col.accent;
    cx.setLineDash([4, 3]);
    cx.beginPath();
    cx.moveTo(PB.sx(k.drag.x), PB.sy(k.drag.y));
    cx.lineTo(PB.mx, PB.my);
    cx.stroke();
    cx.setLineDash([]);
  }

  for (const n of PB.vis.nodes) nodeBox(cx, k, n);
  for (const gb of PB.vis.groups) groupBox(cx, k, gb);

  const m = k.rect();
  if (m) {
    cx.strokeStyle = PB.col.accent;
    cx.setLineDash([3, 3]);
    cx.strokeRect(m.x0, m.y0, m.x1 - m.x0, m.y1 - m.y0);
    cx.setLineDash([]);
  }

  cx.fillStyle = PB.col.fg3;
  cx.font = `11px ${PB.col.mono}`;
  cx.fillText(PB.ctx ? `group: ${PB.ctx}  (Ctrl+[ leaves)` : "root", 8, k.h - 8);
}

function pin(cx, x, y, type, strong) {
  const s = CATALOG.PORT_SHAPE[type] || "circ";
  cx.fillStyle = strong ? PB.col.accent : PB.col.fg2;
  cx.beginPath();
  if (s === "tri") { cx.moveTo(x - 3, y - 4); cx.lineTo(x + 4, y); cx.lineTo(x - 3, y + 4); }
  else if (s === "sq") cx.rect(x - 3, y - 3, 6, 6);
  else if (s === "dia") { cx.moveTo(x, y - 4); cx.lineTo(x + 4, y); cx.lineTo(x, y + 4); cx.lineTo(x - 4, y); }
  else cx.arc(x, y, 3.5, 0, 6.284);
  cx.fill();
}

function summary(n, d) {
  const k = Object.keys(d.cfg)[0];
  if (n.type === "module") return n.module || "";
  return k && n[k] !== undefined ? `${k}=${n[k]}` : "";
}

function nodeBox(cx, k, n) {
  const z = k.view.zoom, c = PB.box(n), d = GM.def(n);
  const x = PB.sx(c.x), y = PB.sy(c.y), w = c.w * z, h = c.h * z;
  if (x > k.w || y > k.h || x + w < 0 || y + h < 0) return;
  const sel = PB.sel.has(n.id), err = PB.errors[n.id];
  cx.globalAlpha = n.mute ? 0.4 : 1;
  cx.fillStyle = PB.col.panel2;
  cx.fillRect(x, y, w, h);
  cx.strokeStyle = err ? PB.col.live : sel ? PB.col.accent : PB.col.line;
  cx.lineWidth = sel || err ? 2 : 1;
  cx.strokeRect(x, y, w, h);
  cx.fillStyle = PB.col.panel;
  cx.fillRect(x, y, w, TH * z);
  if (z > 0.3) {
    const f = Math.max(7, 11 * z);
    cx.fillStyle = PB.col[d ? "no_" + d.fam : "live"] || PB.col.fg;
    cx.font = `${f}px ${PB.col.mono}`;
    cx.fillText(n.label || n.id, x + 6, y + TH * z * 0.68);
    cx.fillStyle = PB.col.fg3;
    cx.font = `${Math.max(6, 9 * z)}px ${PB.col.mono}`;
    cx.fillText(n.type, x + w - 6 - cx.measureText(n.type).width, y + TH * z * 0.68);
    if (d) cx.fillText(summary(n, d), x + 6, y + h - 4);
    const mark = (n.mute ? "M" : "") + (n.lock ? "L" : "") + (err ? "!" : "");
    if (mark) { cx.fillStyle = PB.col.live; cx.fillText(mark, x + w - 14, y + h - 4); }
  }
  if (d) {
    Object.values(d.ins).forEach((t, i) => pin(cx, x, PB.sy(c.y + TH + i * PH + PH / 2), t, sel));
    Object.values(d.outs).forEach((t, i) => pin(cx, x + w, PB.sy(c.y + TH + i * PH + PH / 2), t, sel));
  }
  cx.globalAlpha = 1;
}

function groupBox(cx, k, gb) {
  const z = k.view.zoom, x = PB.sx(gb.x), y = PB.sy(gb.y), w = gb.w * z, h = gb.h * z;
  cx.fillStyle = PB.col.panel;
  cx.fillRect(x, y, w, h);
  cx.strokeStyle = PB.sel.has(gb.group) ? PB.col.accent : PB.col.fg3;
  cx.lineWidth = 2;
  cx.strokeRect(x, y, w, h);
  if (z > 0.3) {
    cx.fillStyle = PB.col.fg;
    cx.font = `${Math.max(7, 11 * z)}px ${PB.col.mono}`;
    cx.fillText(`[${gb.group}]`, x + 6, y + TH * z * 0.68);
    cx.fillStyle = PB.col.fg3;
    cx.font = `${Math.max(6, 9 * z)}px ${PB.col.mono}`;
    cx.fillText(`${gb.members.length} nodes  Ctrl+]`, x + 6, y + h - 4);
  }
  gb.ports.ins.forEach((p, i) => pin(cx, x, PB.sy(gb.y + TH + i * PH + PH / 2), "number", false));
  gb.ports.outs.forEach((p, i) => pin(cx, x + w, PB.sy(gb.y + TH + i * PH + PH / 2), "number", false));
}

// ---------------------------------------------------------------- Inspector, catalogue, search

// One Inspector field. It writes ONLY on `change` — Enter or leaving the field — never on a key:
// a patch per character fills the undo stack and sends one show_patch per letter.
// ponytail: widgets.js does not serve here (it speaks the JSON Schema of the registry and writes on
// `input`, one key = one patch) ; unify them once the catalogue cfg becomes a schema like the one
// of the commands.
function field(lab, type, value, onSet) {
  const l = document.createElement("label");
  l.textContent = lab;
  let i;
  if (type === "bool") {
    i = document.createElement("input");
    i.type = "checkbox";
    i.checked = !!value;
    i.onchange = () => onSet(i.checked);
  } else if (type.startsWith("enum:")) {
    i = document.createElement("select");
    // An empty option while the node does not have the field: without it the select already shows
    // the first option and choosing exactly that one fires no `change` — the value never got
    // stored.
    const opts = type.slice(5).split("|");
    if (value === undefined) opts.unshift("");
    i.innerHTML = opts.map(o => `<option>${o}</option>`).join("");
    i.value = value === undefined ? "" : String(value);
    i.onchange = () => { if (i.value !== "") onSet(i.value); };
  } else {
    i = document.createElement("input");
    i.type = type === "number" ? "number" : "text";
    i.value = type === "json" ? JSON.stringify(value === undefined ? null : value)
      : (value === undefined ? "" : value);
    i.onchange = () => {
      const r = GM.value(type, i.value);
      if (r.error) return PB.log(`${lab}: ${r.error}`);
      onSet(r.v);
    };
  }
  l.appendChild(i);
  return l;
}

PB.inspector = function () {
  // An edit coming from the Inspector itself does NOT redraw the fields: `PB.apply` calls from here
  // and rebuilding the DOM in the middle of the `change` kills the click that was going to the next
  // field (the mouseup target vanishes between the mousedown and the mouseup). The engine refusal
  // falls outside this guard (it is asynchronous) and redraws, which is when the field NEEDS to go
  // back to the previous value.
  if (PB.editing) return;
  const box = PB.el.insp;
  box.textContent = "";
  const ids = [...PB.sel];
  if (ids.length !== 1) {
    box.appendChild(Object.assign(document.createElement("div"), {
      className: "over", textContent: ids.length ? `${ids.length} selected` : "nothing selected",
    }));
    return;
  }
  const n = GM.node(PB.doc, ids[0]);
  if (!n) return;
  const t = Object.assign(document.createElement("div"), { className: "over", textContent: `${n.id} — ${n.type}` });
  box.appendChild(t);
  if (PB.errors[n.id]) {
    box.appendChild(Object.assign(document.createElement("div"), { className: "ruim", textContent: PB.errors[n.id] }));
  }
  for (const [key, type, value] of GM.fields(PB.doc, n.id)) {
    box.appendChild(field(key, type, value, v => {
      PB.editing = true;
      try {
        return PB.apply(GM.opsKey(PB.doc, n.id, key, v), `${n.id}.${key} = ${v}`);
      } finally {
        PB.editing = false;
      }
    }));
  }
};

// The type list: the left column (q = "") and the Shift+A search are the same one.
function list(box, q, make) {
  box.textContent = "";
  const types = CATALOG.search(q, GM.modules);
  if (!types.length) {                      // an empty box with no explanation was the dead end
    box.appendChild(Object.assign(document.createElement("div"), {
      className: "over", textContent: `no type matches "${q}"`,
    }));
    return;
  }
  for (const t of types) {
    const b = document.createElement("button");
    b.textContent = t;
    b.onclick = () => make(t);
    box.appendChild(b);
  }
}

function catalog() {
  list(PB.el.cat, "", t => newNode(t, PB.k.toWorld(PB.k.w / 2), (PB.k.view.y + PB.k.h / 2) / PB.k.view.zoom));
}

function newNode(type, x, y) {
  const mod = type.startsWith("module:") ? type.slice(7) : "";
  const t = mod ? "module" : type;
  const n = { id: GM.newId(PB.doc, mod || t), type: t, x: Math.round(x), y: Math.round(y) };
  if (mod) n.module = mod;
  if (PB.ctx) n.group = PB.ctx;
  PB.sel = new Set([n.id]);
  return PB.apply(GM.opsAdd(PB.doc, n), `+ ${t} ${n.id}`);
}

// Shift+A: a search that filters from the first character (rule 10) and creates at the cursor.
function openSearch() {
  const el = PB.el.add, r = PB.el.cv.getBoundingClientRect();   // mx/my are the canvas ones; the menu is fixed
  el.style.display = "block";
  el.style.left = (r.left + PB.mx) + "px";
  el.style.top = (r.top + PB.my) + "px";
  const inp = el.querySelector("input"), box = el.querySelector("div");
  const wx = PB.k.toWorld(PB.mx), wy = (PB.my + PB.k.view.y) / PB.k.view.zoom;
  inp.value = "";
  const paint = () => list(box, inp.value, t => { el.style.display = "none"; newNode(t, wx, wy); });
  inp.oninput = paint;
  inp.onkeydown = e => {
    e.stopPropagation();
    if (e.key === "Escape") el.style.display = "none";
    if (e.key !== "Enter") return;
    // Enter with no item in the list: say why. It used to be a mute no-op — the Enter created
    // nothing and there was no click to give, because the list was empty.
    const b = box.querySelector("button");
    if (b) b.click();
    else PB.log(`no type matches "${inp.value}"`);
  };
  paint();
  inp.focus();
}

// ---------------------------------------------------------------- shortcuts

function onKey(e) {
  if (e.target.tagName === "INPUT" || e.target.tagName === "SELECT") return;
  const ids = GM.ids(PB.doc, PB.sel);          // a closed group name becomes the member ids
  // keyboard with no case: "A" and "a" are the same key; the modifier is what rules (SHORTCUTS.md)
  const k = e.key.length === 1 ? e.key.toLowerCase() : e.key;
  if (k === "a" && e.shiftKey && !e.ctrlKey) { e.preventDefault(); return openSearch(); }
  if (k === "Delete") {
    e.preventDefault();
    if (!ids.length) {                          // nothing selected: Delete acts on the selected cable
      const i = PB.selEdge;
      if (i < 0) return;
      PB.selEdge = -1;
      return void PB.apply([{ op: "remove", path: `/graph/edges/${i}` }], "- cable");
    }
    const ops = e.shiftKey ? GM.opsDelReconnect(PB.doc, ids) : GM.opsDel(PB.doc, ids);
    PB.sel.clear();
    return void PB.apply(ops, (e.shiftKey ? "- reconnecting " : "- ") + ids.join(" "));
  }
  if (e.ctrlKey && k === "z") {
    e.preventDefault();
    return e.shiftKey ? PB.step(PB.redo, PB.undo) : PB.step(PB.undo, PB.redo);
  }
  if (e.ctrlKey && k === "]") {
    const n = GM.node(PB.doc, ids[0]);
    const target = PB.vis.groups.find(g => PB.sel.has(g.group)) || (n && GM.group(n) ? { group: GM.group(n) } : null);
    if (target) { PB.ctx = target.group; PB.sel.clear(); PB.k.invalidate(); }
    return;
  }
  if (e.ctrlKey && k === "[") {
    PB.ctx = "";
    PB.sel.clear();
    return PB.k.invalidate();
  }
  if (e.ctrlKey && k === "a") {
    e.preventDefault();
    PB.sel = new Set(PB.vis.nodes.map(n => n.id));
    PB.inspector();
    return PB.k.invalidate();
  }
  if (k === "m" && !e.ctrlKey && ids.length) {
    const ops = ids.flatMap(id => GM.opsKey(PB.doc, id, "mute", !GM.node(PB.doc, id).mute));
    return void PB.apply(ops, "mute " + ids.join(" "));
  }
  if (k === "l" && !e.ctrlKey && ids.length) {
    const ops = ids.flatMap(id => GM.opsKey(PB.doc, id, "lock", !GM.node(PB.doc, id).lock));
    return void PB.apply(ops, "lock " + ids.join(" "));
  }
  if (k === "g" && e.shiftKey && ids.length) {
    const name = prompt("group:", PB.ctx || "group");
    if (name === null) return;
    return void PB.apply(ids.flatMap(id => GM.opsKey(PB.doc, id, "group", name)), "group " + name);
  }
  if (k === "z" && e.shiftKey && !e.ctrlKey) return frameAll();
}

function frameAll() {
  PB.k.resize();
  if (PB.k.w < 2) return requestAnimationFrame(frameAll);   // the show arrives before the canvas layout
  PB.build();
  const ns = PB.vis.nodes;
  if (!ns.length) return;
  const x0 = Math.min(...ns.map(n => +n.x || 0)), x1 = Math.max(...ns.map(n => (+n.x || 0) + NW));
  PB.k.fit(x0 - 20, x1 + 20);
  PB.k.view.y = Math.min(...ns.map(n => +n.y || 0)) * PB.k.view.zoom - 20;
  PB.k.invalidate();
}

// ---------------------------------------------------------------- init

PB.init = function (opts) {
  PB.el = opts;
  const cv = opts.cv;
  PB.col = CK.colors(cv);

  const k = CK.attach(cv, draw);
  PB.k = k;
  k.view.zoom = 1;
  PB.build();

  const zoom0 = k.zoomAt;
  k.zoomAt = (px, f) => {                        // the kit anchors only X; the graph Y anchors too
    const wy = (PB.my + k.view.y) / k.view.zoom;
    zoom0(px, f);
    k.view.y = Math.max(0, wy * k.view.zoom - PB.my);
  };

  const world = p => ({ x: k.toWorld(p.x), y: (p.y + k.view.y) / k.view.zoom });

  k.on.hover = p => { PB.mx = p.x; PB.my = p.y; };

  k.on.down = p => {
    PB.mx = p.x; PB.my = p.y;
    const w = world(p);
    const pn = PB.findPin(w.x, w.y);
    if (pn) {
      k.drag = { mode: "wire", from: pn, x: pn.x, y: pn.y };
      return true;
    }
    const n = PB.findNode(w.x, w.y);
    if (n) {
      const id = n.id || n.group;
      if (!p.shift && !PB.sel.has(id)) PB.sel.clear();
      PB.sel.add(id);
      PB.selEdge = -1;
      PB.inspector();
      const mov = new Map();
      for (const i of GM.ids(PB.doc, PB.sel)) {
        const nd = GM.node(PB.doc, i);
        if (nd && !nd.lock) mov.set(i, [+nd.x || 0, +nd.y || 0]);
      }
      k.drag = { mode: "node", mov, w, dx: 0, dy: 0 };
      k.dirty = true;
      return true;
    }
    PB.selEdge = PB.findCable(p.x, p.y);
    if (PB.selEdge >= 0) { PB.sel.clear(); PB.inspector(); k.dirty = true; return true; }
    return false;
  };

  // Drag = delta in the `drag` (the drawing adds it in PB.box); the model only changes on `up`, by
  // patch.
  k.on.move = (p, d) => {
    PB.mx = p.x; PB.my = p.y;
    if (d.mode === "node") {
      const w = world(p);
      d.dx = w.x - d.w.x;
      d.dy = w.y - d.w.y;
    }
    k.dirty = true;
  };

  k.on.up = (p, d) => {
    PB.mx = p.x; PB.my = p.y;
    const w = world(p);
    if (d.mode === "wire") {
      const target = PB.findPin(w.x, w.y);
      if (!target || target.output === d.from.output) return;
      const [a, b] = d.from.output ? [d.from, target] : [target, d.from];
      const from = `${a.id}.${a.pin}`, to = `${b.id}.${b.pin}`;
      const why = GM.why(PB.doc, from, to);
      if (why) { PB.log(`cable refused: ${why}`); return; }
      PB.apply(GM.opsLink(PB.doc, from, to), `cable ${from} -> ${to}`);
      return;
    }
    if (d.mode === "node" && (d.dx || d.dy)) {
      const mov = [...d.mov].map(([id, [x0, y0]]) => [id, x0 + d.dx, Math.max(0, y0 + d.dy)]);
      PB.apply(GM.opsMove(PB.doc, mov), "");
    }
  };

  k.on.marquee = (r, add) => {
    if (!add) PB.sel.clear();
    for (const n of PB.vis.nodes) {
      const c = PB.box(n);
      const x = PB.sx(c.x), y = PB.sy(c.y);
      if (x < r.x1 && x + c.w * k.view.zoom > r.x0 && y < r.y1 && y + c.h * k.view.zoom > r.y0) PB.sel.add(n.id);
    }
    PB.inspector();
  };

  addEventListener("keydown", onKey);
  addEventListener("pointerdown", e => {
    if (!PB.el.add.contains(e.target)) PB.el.add.style.display = "none";
  }, true);

  catalog();
  PB.inspector();
  k.loop();
};

// "Open". `path` is the path as the ENGINE sees it: "shows/patchbay_demo.spell", relative to the
// repo root — it is the same string that `serve` and `python -m http.server` at the root serve
// under "/".
// With an engine: `load` in the engine and then ITS show through `show_get`. With no engine: the
// file. ONE path at a time — the fetch running alongside the show_get was the race that left the
// page editing one document and the engine another.
PB.load = function (path) {
  if (PB.live()) {
    return PB.bus.call("load", { file: path })
      .then(() => PB.reload(true))
      .catch(e => PB.log(`load ${path}: ${e.message}`));
  }
  return fetch("/" + path).then(r => r.json()).then(d => {
    PB.doc = d;
    requestAnimationFrame(frameAll);
    PB.log(`show ${path}: ${GM.g(PB.doc).nodes.length} nodes`);
    return PB.check();
  }).catch(e => PB.log(`${path}: ${e.message}`));
};

// Boot: connects the bus and lets the socket decide the source. Opened -> the engine show; did not
// open (the `Bus` emits `close` even on a socket that never came up) -> the file, only once.
PB.connect = function (path) {
  PB.bus = new Bus({}).connect();
  PB.bus.on("log", d => PB.log(d && d.text ? d.text : JSON.stringify(d)));
  // `bus.rev` is the highest rev ever seen in a response: an event above that is an outside edit.
  PB.bus.on("show", d => { if (d && d.rev > PB.bus.rev) PB.reload(); });
  let file = true;                                 // is the fetch still on the table?
  PB.bus.on("open", () => {
    file = false;                                  // engine found: the file never comes in again —
    PB.el.state.textContent = "engine";            // a socket that drops later does not erase what
    PB.reload(true);                               // the engine sent (it comes back in 1 s).
  });
  PB.bus.on("close", () => {
    PB.el.state.textContent = "offline";
    if (file) { file = false; PB.load(path); }
  });
};

PB.reload = function (frame) {
  PB.bus.call("show_get", { full: true }).then(s => {
    PB.doc = s;
    if (frame) requestAnimationFrame(frameAll); else PB.k.invalidate();
    PB.check();
  }).catch(e => PB.log("show_get: " + e.message));
  PB.bus.call("module_list", {}).then(l => {
    for (const m of l || []) {
      PB.bus.call("module_get", { name: m.name || m }).then(mo => {
        GM.modules[mo.name] = CATALOG.moduleDef(mo);
        catalog();
      }).catch(() => {});
    }
  }).catch(() => {});     // module frente not in main yet: no live modules, only the closed catalogue
};

if (typeof module !== "undefined" && module.exports) module.exports = { GM };
if (typeof window !== "undefined") { window.GM = GM; window.PB = PB; }
