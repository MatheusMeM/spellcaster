"use strict";
// catalog.js — the CLOSED graph catalogue (spellcore/script/src/graph.rs, PRD §10) as DATA, plus
// the state and module nodes (graph-runtime frente) and the universal keys (mute, lock, group, x, y).
// Whoever draws the PATCHBAY reads from here: there is no hand-written node list in graph.js.
//
// Port type is an EDITOR RULE: the runtime carries everything as f64. A cable only links compatible
// types; conversion is a visible node (math.map, logic.toggle), never a hidden coercion
// (design/FUNCOES/orquestrador.md §1, "Cable").
//
// ponytail: `trigger` and `bool` are the SAME wire (the runtime reads "on = >= 0.5" and the rising
// edge with subiu()), so the two link ; separate them for good once the runtime has a signal type.

// Pin shape in the drawing. A node family has NO color (orquestrador.md §2); a port type has SHAPE.
const PORT_SHAPE = {
  trigger: "tri", bool: "sq", number: "circ", color: "dia", xy: "dia", frame: "dia", dmx: "dia",
};

const PULSE = { trigger: 1, bool: 1 };

// cfg: field -> Inspector widget type ("string", "number", "bool", "enum:a|b", "json").
const CAT = {
  "in.widget": { fam: "in", cfg: { widget: "string" }, ins: {}, outs: { press: "trigger" } },
  "in.key": { fam: "in", cfg: { key: "string" }, ins: {}, outs: { down: "trigger" } },
  "in.osc": { fam: "in", cfg: { address: "string" }, ins: {}, outs: { out: "number" } },
  "in.midi": { fam: "in", cfg: { midi: "string" }, ins: {}, outs: { out: "number" } },
  "in.marker": { fam: "in", cfg: { marker: "string" }, ins: {}, outs: { out: "trigger" } },
  "in.timer": { fam: "in", cfg: { every: "number" }, ins: {}, outs: { out: "trigger" } },
  "in.state": {
    fam: "in",
    cfg: { what: "enum:t|dmx", universe: "number", address: "number" },
    ins: {}, outs: { out: "number" },
  },
  "logic.and": { fam: "logic", cfg: {}, ins: { a: "bool", b: "bool" }, outs: { out: "bool" } },
  "logic.or": { fam: "logic", cfg: {}, ins: { a: "bool", b: "bool" }, outs: { out: "bool" } },
  "logic.not": { fam: "logic", cfg: {}, ins: { in: "bool" }, outs: { out: "bool" } },
  "logic.latch": {
    fam: "logic", cfg: {}, ins: { set: "trigger", reset: "trigger" }, outs: { out: "bool" },
  },
  "logic.toggle": { fam: "logic", cfg: {}, ins: { in: "trigger" }, outs: { out: "bool" } },
  "logic.debounce": {
    fam: "logic", cfg: { ms: "number" }, ins: { in: "trigger" }, outs: { out: "trigger" },
  },
  "logic.counter": {
    fam: "logic", cfg: { step: "number" },
    ins: { in: "trigger", reset: "trigger" }, outs: { out: "number" },
  },
  "logic.select": {
    fam: "logic", cfg: {},
    ins: { a: "number", b: "number", sel: "bool" }, outs: { out: "number" },
  },
  "math.map": {
    fam: "logic",
    cfg: { in_min: "number", in_max: "number", out_min: "number", out_max: "number", clamp: "bool" },
    ins: { in: "number" }, outs: { out: "number" },
  },
  "math.curve": {
    fam: "logic", cfg: { curve: "enum:linear|hold|in|out|inout|bezier", c: "json" },
    ins: { in: "number" }, outs: { out: "number" },
  },
  "math.expr": {
    fam: "logic", cfg: { expr: "string" },
    ins: { a: "number", b: "number" }, outs: { out: "number" },
  },
  "time.delay": { fam: "logic", cfg: { ms: "number" }, ins: { in: "number" }, outs: { out: "number" } },
  "time.hold": { fam: "logic", cfg: { ms: "number" }, ins: { in: "trigger" }, outs: { out: "bool" } },
  "cmd": {
    fam: "cmd", cfg: { cmd: "string", args: "json" },
    ins: { trigger: "trigger" }, outs: { done: "trigger" },
  },
  "out.widget": {
    fam: "out", cfg: { widget: "string", prop: "string", hold_ms: "number" },
    ins: { in: "number" }, outs: {},
  },
  "out.osc": { fam: "out", cfg: { address: "string" }, ins: { in: "number" }, outs: {} },
  "out.param": { fam: "out", cfg: { target: "string" }, ins: { in: "number" }, outs: {} },
  "out.notify": { fam: "out", cfg: { text: "string" }, ins: { in: "trigger" }, outs: {} },
  "state": {
    fam: "logic", cfg: { group: "string", initial: "bool" },
    ins: { enter: "trigger", exit: "trigger" }, outs: { active: "bool" },
  },
  "module": { fam: "module", cfg: { module: "string" }, ins: {}, outs: {} },
};

// Keys accepted on ANY node. The runtime ignores the unknown ones (confirmed in graph.rs).
const UNIVERSAL = { mute: "bool", lock: "bool", state: "string", group: "string", label: "string" };

const PARAM_PORT = {
  float: "number", int: "number", bool: "bool", trigger: "trigger",
  color: "color", string: "string", enum: "number", xy: "xy",
};

// module.json (module frente) -> node definition: one input pin per parameter and per command, one
// output pin per value.
function moduleDef(m) {
  const ins = {}, outs = {};
  for (const [p, d] of Object.entries(m.parameters || {})) ins[p] = PARAM_PORT[d && d.type] || "number";
  for (const c of Object.keys(m.commands || {})) ins[c] = "trigger";
  for (const [p, d] of Object.entries(m.values || {})) outs[p] = PARAM_PORT[d && d.type] || "number";
  return { fam: "module", cfg: { module: "string" }, ins, outs, module: m.name };
}

// Definition of a show node: catalogue type, or a live module through the `module` field.
function nodeDef(node, modules) {
  const t = node && node.type;
  if (t === "module" && modules && modules[node.module]) return modules[node.module];
  return CAT[t] || null;
}

function port(def, pin, output) {
  if (!def) return null;
  const m = output ? def.outs : def.ins;
  return Object.prototype.hasOwnProperty.call(m, pin) ? m[pin] : null;
}

// A cable only links compatible types. Returns "" when it can, otherwise the reason in one sentence.
function compat(a, b) {
  if (!a) return "output pin does not exist";
  if (!b) return "input pin does not exist";
  if (a === b) return "";
  if (PULSE[a] && PULSE[b]) return "";
  return `type ${a} does not link into ${b}: use a conversion node (math.map, logic.toggle)`;
}

// Comparable name: no case and no separator. The type name has a dot ("in.timer") and the operator
// types fast: a dot that did not land, a space in its place or Caps on left the list EMPTY — and an
// empty list creates no node either on Enter or on click, because there is no item to click.
function key(s) { return String(s).toLowerCase().replace(/[^a-z0-9]/g, ""); }

// List for the Shift+A menu: search from the first character (rule 10 of FUNCOES/README).
// ponytail: substring of the key, no fuzzy and no ranking ; it comes in when the catalogue grows
// past one screen.
function search(q, modules) {
  const names = Object.keys(CAT).concat(Object.keys(modules || {}).map(n => `module:${n}`));
  const s = key(q);
  return names.filter(n => key(n).includes(s)).sort();
}

const CATALOG = { CAT, UNIVERSAL, PORT_SHAPE, moduleDef, nodeDef, port, compat, search };
if (typeof module !== "undefined" && module.exports) module.exports = CATALOG;
if (typeof window !== "undefined") window.CATALOG = CATALOG;
