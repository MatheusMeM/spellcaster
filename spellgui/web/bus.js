"use strict";
// bus.js — cliente do barramento do `spellcore serve`. Contrato (spellcore/README.md):
//
//   HTTP  GET /commands -> Registry::schema()   GET /show -> show_get full   GET /<arquivo>
//   WS    /ws  request  {"id":7,"cmd":"locate","args":{"t":12.5}}
//              resposta {"id":7,"result":...} | {"id":7,"error":"texto"}
//              evento   {"event":"transport"|"show"|"log"|"widget","data":...}
//              binario  topic:u8 | universe:u16 LE | 512 bytes   (topic 1 = dmx de saida)
//
// Toda pagina (face, timeline, patchbay, teatro, laser) fala com o engine so' por aqui: quem
// tem logica e' o registry, isto e' encanamento.
//
// Modo offline: sem servidor a pagina continua abrindo. `call` valida o nome do comando contra
// `dev/commands.json` e responde localmente (show_get devolve o .spell apontado por `opts.show`).
// ponytail: offline nao simula o engine, so' deixa a pagina montar e mostrar o que faria
// ; apagar quando o `serve` for a unica forma de abrir a GUI.

function Bus(opts) {
  opts = opts || {};
  this.opts = opts;
  this.subs = {};                       // topico -> [fn]
  this.pend = new Map();                // id -> {ok, err}
  this.q = [];                          // requests antes do socket abrir
  this.seq = 0;
  this.offline = !!opts.offline;
  this.dev = opts.dev || "dev/";
  this.showUrl = opts.show || "../../shows/medgrupo.spell";
  this.ws = null;
  this.tries = 0;
  this._cmds = null;
}

// ---- parse: a unica funcao com regra, e a unica testada sem DOM ----------
// Devolve {id, result} | {id, error} | {event, data} | null (mensagem que nao e' do contrato).
Bus.parse = function (m) {
  if (typeof m !== "string") {
    const b = m instanceof Uint8Array ? m : new Uint8Array(m);
    if (b.length < 3) return null;
    return {
      event: "dmx",
      data: { topic: b[0], universe: b[1] | (b[2] << 8), data: b.subarray(3) },
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
    return v.error !== undefined && v.error !== null
      ? { id: v.id, error: String(v.error) }
      : { id: v.id, result: v.result };
  }
  if (typeof v.event === "string") return { event: v.event, data: v.data };
  return null;
};

Bus.prototype.on = function (topic, fn) {
  (this.subs[topic] || (this.subs[topic] = [])).push(fn);
  return () => {
    const a = this.subs[topic];
    const i = a.indexOf(fn);
    if (i >= 0) a.splice(i, 1);
  };
};

Bus.prototype.emit = function (topic, data) {
  for (const fn of this.subs[topic] || []) {
    try {
      fn(data);
    } catch (e) {
      if (typeof console !== "undefined") console.error(topic, e);
    }
  }
};

/// Entrega uma mensagem crua (texto ou binario) ao barramento. Publica para o teste poder
/// empurrar mensagem sem socket.
Bus.prototype.recv = function (m) {
  const p = Bus.parse(m);
  if (!p) return;
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
  const url =
    this.opts.url ||
    (location.protocol === "https:" ? "wss://" : "ws://") + location.host + "/ws";
  let ws;
  try {
    ws = new WebSocket(url);
  } catch (e) {
    this.fallback(e.message);
    return this;
  }
  ws.binaryType = "arraybuffer";
  this.ws = ws;
  ws.onopen = () => {
    this.tries = 0;
    this.emit("open", url);
    for (const m of this.q.splice(0)) ws.send(m);
  };
  ws.onmessage = e => this.recv(e.data);
  ws.onclose = () => {
    this.ws = null;
    for (const [, w] of this.pend) w.err(new Error("barramento caiu"));
    this.pend.clear();
    this.emit("close", null);
    // ponytail: backoff fixo de 1 s ate 5 tentativas, depois cai para offline
    // ; exponencial so' se o servidor passar a reiniciar sozinho.
    if (++this.tries > 5) return this.fallback("sem servidor");
    setTimeout(() => this.connect(), 1000);
  };
  ws.onerror = () => {};
  return this;
};

Bus.prototype.fallback = function (why) {
  if (this.offline) return;
  this.offline = true;
  this.emit("log", { text: "modo offline: " + why });
};

// ---- chamadas -----------------------------------------------------------
Bus.prototype.call = function (cmd, args) {
  if (this.offline || (!this.ws && this.tries > 5)) return this.local(cmd, args || {});
  const id = ++this.seq;
  const m = JSON.stringify({ id, cmd, args: args || {} });
  return new Promise((ok, err) => {
    this.pend.set(id, { ok, err });
    if (this.ws && this.ws.readyState === 1) this.ws.send(m);
    else this.q.push(m);
  });
};

/// `input {key, value}` — o caminho de widget, tecla e modulo ate' o graph vivo.
Bus.prototype.input = function (key, value) {
  return this.call("input", { key, value: +value });
};

Bus.prototype.commands = function () {
  if (this._cmds) return this._cmds;
  const url = this.offline ? this.dev + "commands.json" : "commands";
  this._cmds = fetch(url)
    .then(r => r.json())
    .catch(() => fetch(this.dev + "commands.json").then(r => r.json()));
  return this._cmds;
};

Bus.prototype.showGet = function () {
  return this.offline
    ? fetch(this.showUrl).then(r => r.json())
    : fetch("show").then(r => r.json());
};

/// Offline: valida o nome contra o catalogo e responde o que da' para responder sem engine.
Bus.prototype.local = function (cmd, args) {
  return this.commands().then(cs => {
    // ponytail: `input` esta' no contrato do barramento mas ainda nao no registry que gerou o
    // dev/commands.json ; tirar a excecao quando o `serve` estiver em main e o JSON regerado.
    if (cmd !== "input" && !cs.some(c => c.name === cmd)) {
      throw new Error("comando desconhecido: " + cmd);
    }
    this.emit("log", { text: "offline: " + cmd + " " + JSON.stringify(args) });
    if (cmd === "show_get") return this.showGet();
    return { offline: true, cmd, args };
  });
};

if (typeof window !== "undefined") window.Bus = Bus;
if (typeof module !== "undefined") module.exports = Bus;
