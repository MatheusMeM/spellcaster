"use strict";
// bus.js — cliente do barramento do `spellcore serve`. Contrato (spellcore/README.md):
//
//   HTTP  GET /commands -> Registry::schema()   GET /show -> show_get full   GET /<arquivo>
//   WS    /ws  request  {"id":7,"cmd":"locate","args":{"t":12.5}}
//              resposta {"id":7,"result":...} | {"id":7,"error":"texto"}
//              evento   {"event":"transport"|"show"|"log"|"widget","data":...}
//              binario  topic:u8 | universe:u16 LE | 512 bytes   (topic 1 = dmx de saida,
//                       topic 2 = dmx de entrada, o universo de `show.inputs`)
//
// Toda pagina (face, timeline, patchbay, teatro, laser) fala com o engine so' por aqui: quem
// tem logica e' o registry, isto e' encanamento.
//
// Modo offline: sem servidor a pagina continua abrindo; `call` responde localmente e os fetch
// vao para os arquivos abaixo.
// ponytail: offline nao simula o engine, so' deixa a pagina montar e mostrar o que faria
// ; apagar quando o `serve` for a unica forma de abrir a GUI.

function Bus(opts) {
  this.subs = {};                       // topico -> [fn]
  this.pend = new Map();                // id -> {ok, err}
  this.seq = 0;
  this.offline = !!(opts && opts.offline);
  this.ws = null;
  this.rev = 0;                         // maior revisao do show ja' vista numa resposta
}

// As paginas vivem em /spellgui/web/: caminho relativo nao acha /commands nem /show.
Bus.DEV = "/spellgui/web/dev/commands.json";
Bus.SHOW = "/shows/medgrupo.spell";

// ---- parse: a unica funcao com regra, e a unica testada sem DOM ----------
// Devolve {id, result} | {id, error} | {event, data} | null (mensagem que nao e' do contrato).
Bus.parse = function (m) {
  if (typeof m !== "string") {
    const b = m instanceof Uint8Array ? m : new Uint8Array(m);
    // topic 1 = dmx de saida e topic 2 = dmx de entrada, 515 bytes; outro topico nao tem consumidor
    if (b.length < 515 || (b[0] !== 1 && b[0] !== 2)) return null;
    return {
      event: "dmx",
      data: { topic: b[0], universe: b[1] | (b[2] << 8), data: b.subarray(3, 515) },
    };
  }
  let v;
  try {
    v = JSON.parse(m);
  } catch (e) {
    return null;
  }
  if (!v || typeof v !== "object") return null;
  if (typeof v.id === "number") {
    const r = typeof v.rev === "number" ? v.rev : undefined;
    return v.error != null
      ? { id: v.id, error: String(v.error), rev: r }
      : { id: v.id, result: v.result, rev: r };
  }
  if (typeof v.event === "string") return { event: v.event, data: v.data };
  return null;
};

Bus.prototype.on = function (topic, fn) {
  (this.subs[topic] || (this.subs[topic] = [])).push(fn);
};

Bus.prototype.emit = function (topic, data) {
  for (const fn of this.subs[topic] || []) fn(data);
};

/// Entrega uma mensagem crua (texto ou binario) ao barramento. Publica para o teste poder
/// empurrar mensagem sem socket.
Bus.prototype.recv = function (m) {
  const p = Bus.parse(m);
  if (!p) return;
  if (typeof p.rev === "number") this.rev = Math.max(this.rev, p.rev);
  if (p.event) return this.emit(p.event, p.data);
  const w = this.pend.get(p.id);
  if (!w) return;
  this.pend.delete(p.id);
  if (p.error !== undefined) w.err(new Error(p.error));
  else w.ok(p.result);
};

// ---- socket -------------------------------------------------------------
Bus.prototype.connect = function () {
  if (this.offline || typeof WebSocket === "undefined") return this;
  const url = (location.protocol === "https:" ? "wss://" : "ws://") + location.host + "/ws";
  const ws = new WebSocket(url);
  ws.binaryType = "arraybuffer";
  this.ws = ws;
  ws.onopen = () => this.emit("open", url);
  ws.onmessage = e => this.recv(e.data);
  ws.onclose = () => {
    this.ws = null;
    for (const [, w] of this.pend) w.err(new Error("barramento caiu"));
    this.pend.clear();
    this.emit("close", null);
    setTimeout(() => this.connect(), 1000);
  };
  return this;
};

// ---- chamadas -----------------------------------------------------------
// ponytail: request com o socket fechado e' descartado (a promessa fica pendente ate' o
// `onclose` seguinte) ; fila so' se alguma pagina precisar mandar antes de abrir.
Bus.prototype.call = function (cmd, args) {
  if (this.offline) return this.local(cmd, args || {});
  const id = ++this.seq;
  return new Promise((ok, err) => {
    this.pend.set(id, { ok, err });
    if (this.ws && this.ws.readyState === 1) {
      this.ws.send(JSON.stringify({ id, cmd, args: args || {} }));
    }
  });
};

/// `input {key, value}` — o caminho de widget, tecla e modulo ate' o graph vivo.
Bus.prototype.input = function (key, value) {
  return this.call("input", { key, value: +value });
};

Bus.prototype.commands = function () {
  return fetch(this.offline ? Bus.DEV : "/commands").then(r => r.json());
};

Bus.prototype.showGet = function () {
  return fetch(this.offline ? Bus.SHOW : "/show").then(r => r.json());
};

/// Offline nao tem engine: a chamada vira eco, para a pagina montar e o log mostrar o que faria.
Bus.prototype.local = function (cmd, args) {
  this.emit("log", { text: "offline: " + cmd + " " + JSON.stringify(args) });
  return Promise.resolve({ offline: true, cmd, args });
};

if (typeof window !== "undefined") window.Bus = Bus;
if (typeof module !== "undefined") module.exports = Bus;
