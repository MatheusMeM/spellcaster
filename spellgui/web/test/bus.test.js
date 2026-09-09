"use strict";
// node --test spellgui/web/test/
// Bus.parse e' a unica regra do bus.js: o resto e' socket. Sem DOM, sem servidor.

const { test } = require("node:test");
const assert = require("node:assert");
const Bus = require("../bus.js");

test("parse: resposta com result", () => {
  const p = Bus.parse('{"id":7,"result":{"t":12.5},"rev":3}');
  assert.deepStrictEqual(p, { id: 7, result: { t: 12.5 }, rev: 3 });
  // resposta sem `rev` (barramento velho) nao inventa numero
  assert.deepStrictEqual(Bus.parse('{"id":7,"result":1}'), { id: 7, result: 1, rev: undefined });
});

test("parse: resposta com erro", () => {
  assert.deepStrictEqual(Bus.parse('{"id":7,"error":"sem player em execucao","rev":9}'), {
    id: 7,
    error: "sem player em execucao",
    rev: 9,
  });
  // result null com erro ausente continua sendo resultado
  assert.deepStrictEqual(Bus.parse('{"id":1,"result":null,"rev":0}'), {
    id: 1,
    result: null,
    rev: 0,
  });
});

test("rev: o bus guarda o maior visto e nunca anda para tras", () => {
  const b = new Bus({ offline: true });
  b.recv('{"id":1,"result":1,"rev":4}');
  assert.strictEqual(b.rev, 4);
  b.recv('{"id":2,"result":1,"rev":2}');
  assert.strictEqual(b.rev, 4);
  b.recv('{"id":3,"result":1,"rev":7}');
  assert.strictEqual(b.rev, 7);
});

test("parse binario: topic 1 (saida) e 2 (entrada) com 515 bytes", () => {
  const curto = new Uint8Array(300);
  curto[0] = 1;
  assert.strictEqual(Bus.parse(curto), null);
  const entrada = new Uint8Array(515);
  entrada[0] = 2; entrada[1] = 1;
  assert.strictEqual(Bus.parse(entrada).data.topic, 2);
  const outro = new Uint8Array(515);
  outro[0] = 3;
  assert.strictEqual(Bus.parse(outro), null);
});

test("parse: evento", () => {
  assert.deepStrictEqual(Bus.parse('{"event":"show","data":{"rev":5}}'), {
    event: "show",
    data: { rev: 5 },
  });
  assert.deepStrictEqual(Bus.parse('{"event":"widget","data":{"id":"go","prop":"glow","value":1}}'), {
    event: "widget",
    data: { id: "go", prop: "glow", value: 1 },
  });
});

test("parse: frame binario topic|universe LE|512", () => {
  const b = new Uint8Array(515);
  b[0] = 1; // topic 1 = dmx
  b[1] = 0x0d;
  b[2] = 0x01; // universo 269, little endian
  b[3] = 255;
  b[514] = 7;
  const p = Bus.parse(b);
  assert.strictEqual(p.event, "dmx");
  assert.strictEqual(p.data.topic, 1);
  assert.strictEqual(p.data.universe, 269);
  assert.strictEqual(p.data.data.length, 512);
  assert.strictEqual(p.data.data[0], 255);
  assert.strictEqual(p.data.data[511], 7);
  // ArrayBuffer cru (o que o WebSocket entrega com binaryType="arraybuffer")
  assert.strictEqual(Bus.parse(b.buffer).data.universe, 269);
});

test("parse: lixo nao derruba", () => {
  assert.strictEqual(Bus.parse("nao e' json"), null);
  assert.strictEqual(Bus.parse("[1,2,3]"), null);
  assert.strictEqual(Bus.parse('{"sem":"id nem event"}'), null);
  assert.strictEqual(Bus.parse(new Uint8Array(2)), null);
});

test("recv: entrega resultado a promessa pendente e evento ao inscrito", async () => {
  const b = new Bus({ offline: true });
  let visto = null;
  b.on("transport", d => (visto = d));
  b.recv('{"event":"transport","data":{"state":"play"}}');
  assert.deepStrictEqual(visto, { state: "play" });

  const p = new Promise((ok, err) => b.pend.set(3, { ok, err }));
  b.recv('{"id":3,"result":42}');
  assert.strictEqual(await p, 42);

  const q = new Promise((ok, err) => b.pend.set(4, { ok, err }));
  b.recv('{"id":4,"error":"nao deu"}');
  await assert.rejects(q, /nao deu/);
  assert.strictEqual(b.pend.size, 0, "pendencias limpas");
});

test("offline: a chamada vira eco e o log conta o que faria", async () => {
  const b = new Bus({ offline: true });
  let log = null;
  b.on("log", d => (log = d.text));
  assert.deepStrictEqual(await b.call("cue_go", {}), {
    offline: true,
    cmd: "cue_go",
    args: {},
  });
  assert.strictEqual(log, 'offline: cue_go {}');
  assert.deepStrictEqual(await b.input("widget:blackout", 1), {
    offline: true,
    cmd: "input",
    args: { key: "widget:blackout", value: 1 },
  });
});
