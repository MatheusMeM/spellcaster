"use strict";
// catalog.js — o catalogo FECHADO do graph (spellcore/script/src/graph.rs, PRD §10) como DADO,
// mais os nos state e module (frente graph-runtime) e as chaves universais (mute, lock, group, x, y).
// Quem desenha o PATCHBAY le daqui: nao ha lista de nos escrita a mao em graph.js.
//
// Tipo de porta e REGRA DO EDITOR: o runtime carrega tudo como f64. Cabo so liga tipos
// compativeis; conversao e no visivel (math.map, logic.toggle), nunca coercao escondida
// (design/FUNCOES/orquestrador.md §1, "Cabo").
//
// ponytail: `trigger` e `bool` sao o MESMO fio (o runtime le "ligado = >= 0.5" e a borda de subida
// com subiu()), entao os dois se ligam ; separar de vez quando o runtime tiver tipo de sinal.

// Forma do pino no desenho. Familia de no NAO tem cor (orquestrador.md §2); tipo de porta tem FORMA.
const PORT_SHAPE = {
  trigger: "tri", bool: "sq", number: "circ", color: "dia", xy: "dia", frame: "dia", dmx: "dia",
};

const PULSO = { trigger: 1, bool: 1 };

// cfg: campo -> tipo do widget do Inspector ("string", "number", "bool", "enum:a|b", "json").
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

// Chaves aceitas em QUALQUER no. O runtime ignora as desconhecidas (confirmado em graph.rs).
const UNIVERSAL = { mute: "bool", lock: "bool", state: "string", group: "string", label: "string" };

const PARAM_PORT = {
  float: "number", int: "number", bool: "bool", trigger: "trigger",
  color: "color", string: "string", enum: "number", xy: "xy",
};

// module.json (frente module) -> definicao de no: um pino de entrada por parameter e por command,
// um pino de saida por value.
function moduleDef(m) {
  const ins = {}, outs = {};
  for (const [p, d] of Object.entries(m.parameters || {})) ins[p] = PARAM_PORT[d && d.type] || "number";
  for (const c of Object.keys(m.commands || {})) ins[c] = "trigger";
  for (const [p, d] of Object.entries(m.values || {})) outs[p] = PARAM_PORT[d && d.type] || "number";
  return { fam: "module", cfg: { module: "string" }, ins, outs, module: m.name };
}

// Definicao de um no do show: tipo do catalogo, ou modulo vivo pelo campo `module`.
function nodeDef(node, modules) {
  const t = node && node.type;
  if (t === "module" && modules && modules[node.module]) return modules[node.module];
  return CAT[t] || null;
}

function port(def, pin, saida) {
  if (!def) return null;
  const m = saida ? def.outs : def.ins;
  return Object.prototype.hasOwnProperty.call(m, pin) ? m[pin] : null;
}

// Cabo so liga tipos compativeis. Devolve "" quando pode, senao o motivo em uma frase.
function compat(a, b) {
  if (!a) return "pino de saida nao existe";
  if (!b) return "pino de entrada nao existe";
  if (a === b) return "";
  if (PULSO[a] && PULSO[b]) return "";
  return `tipo ${a} nao liga em ${b}: use um no de conversao (math.map, logic.toggle)`;
}

// Lista para o menu Shift+A: busca a partir do primeiro caractere (regra 10 de FUNCOES/README).
function busca(q, modules) {
  const nomes = Object.keys(CAT).concat(Object.keys(modules || {}).map(n => `module:${n}`));
  const s = (q || "").toLowerCase();
  return nomes.filter(n => n.toLowerCase().includes(s)).sort();
}

const CATALOG = { CAT, UNIVERSAL, PORT_SHAPE, moduleDef, nodeDef, port, compat, busca };
if (typeof module !== "undefined" && module.exports) module.exports = CATALOG;
if (typeof window !== "undefined") window.CATALOG = CATALOG;
