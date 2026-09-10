"use strict";
// bus.js — client of the `spellcore serve` bus. Contract (spellcore/README.md):
//
//   HTTP  GET /commands -> Registry::schema()   GET /show -> show_get full   GET /<file>
//   WS    /ws  request  {"id":7,"cmd":"locate","args":{"t":12.5}}
//              response {"id":7,"result":...} | {"id":7,"error":"text"}
//              event    {"event":"transport"|"show"|"log"|"widget","data":...}
//              binary   topic:u8 | universe:u16 LE | 512 bytes   (topic 1 = output dmx,
//                       topic 2 = input dmx, the universe of `show.inputs`)
//
// Every page (face, timeline, patchbay, teatro, laser) talks to the engine only through here:
// the logic lives in the registry, this is plumbing.
//
// Offline mode: with no server the page still opens; `call` answers locally and the fetches go
// to the files below.
// ponytail: offline does not simulate the engine, it only lets the page mount and show what it
// would do ; drop it when `serve` is the only way to open the GUI.

function Bus(opts) {
  this.subs = {};                       // topic -> [fn]
  this.pend = new Map();                // id -> {ok, err}
  this.seq = 0;
  this.offline = !!(opts && opts.offline);
  this.ws = null;
  this.rev = 0;                         // highest show revision ever seen in a response
}

// The pages live in /spellgui/web/: a relative path finds neither /commands nor /show.
Bus.DEV = "/spellgui/web/dev/commands.json";
Bus.SHOW = "/shows/medgrupo.spell";

// ---- parse: the only function with a rule, and the only one tested without DOM ----
// Returns {id, result} | {id, error} | {event, data} | null (message outside the contract).
Bus.parse = function (m) {
  if (typeof m !== "string") {
    const b = m instanceof Uint8Array ? m : new Uint8Array(m);
    // topic 1 = output dmx and topic 2 = input dmx, 515 bytes; any other topic has no consumer
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

/// Delivers a raw message (text or binary) to the bus. Public so the test can push a message
/// without a socket.
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
    for (const [, w] of this.pend) w.err(new Error("bus went down"));
    this.pend.clear();
    this.emit("close", null);
    setTimeout(() => this.connect(), 1000);
  };
  return this;
};

// ---- calls --------------------------------------------------------------
// ponytail: a request with the socket closed is dropped (the promise stays pending until the
// next `onclose`) ; a queue only if some page needs to send before the socket opens.
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

/// `input {key, value}` — the path from widget, key and module to the live graph.
Bus.prototype.input = function (key, value) {
  return this.call("input", { key, value: +value });
};

Bus.prototype.commands = function () {
  return fetch(this.offline ? Bus.DEV : "/commands").then(r => r.json());
};

Bus.prototype.showGet = function () {
  return fetch(this.offline ? Bus.SHOW : "/show").then(r => r.json());
};

/// Offline has no engine: the call becomes an echo, so the page mounts and the log shows what it
/// would do.
Bus.prototype.local = function (cmd, args) {
  this.emit("log", { text: "offline: " + cmd + " " + JSON.stringify(args) });
  return Promise.resolve({ offline: true, cmd, args });
};

if (typeof window !== "undefined") window.Bus = Bus;
if (typeof module !== "undefined") module.exports = Bus;
