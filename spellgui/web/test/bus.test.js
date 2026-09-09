"use strict";
// node --test spellgui/web/test/
// Bus.parse e' a unica regra do bus.js: o resto e' socket. Sem DOM, sem servidor.

const { test } = require("node:test");
const assert = require("node:assert");
const Bus = require("../bus.js");

test("parse: resposta com result", () => {
  const p = Bus.parse('{"id":7,"result":{"t":12.5}}');
  assert.deepStrictEqual(p, { id: 7, result: { t: 12.5 } });
});

test("parse: resposta com erro", () => {
  assert.deepStrictEqual(Bus.parse('{"id":7,"error":"sem player em execucao"}'), {
    id: 7,
    error: "sem player em execucao",
  });
  // result null com erro ausente continua sendo resultado
  assert.deepStrictEqual(Bus.parse('{"id":1,"result":null}'), { id: 1, result: null });
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

test("offline: comando fora do catalogo e' recusado, show_get devolve o show local", async () => {
  const b = new Bus({ offline: true });
  b._cmds = Promise.resolve([{ name: "cue_go" }, { name: "show_get" }]);
  b.showGet = () => Promise.resolve({ name: "medgrupo" });
  assert.deepStrictEqual(await b.call("cue_go", {}), {
    offline: true,
    cmd: "cue_go",
    args: {},
  });
  assert.deepStrictEqual(await b.call("show_get", {}), { name: "medgrupo" });
  await assert.rejects(b.call("nao_existe", {}), /comando desconhecido: nao_existe/);
  // `input` esta' no contrato do barramento mesmo sem estar no dev/commands.json de hoje
  assert.strictEqual((await b.input("widget:blackout", 1)).cmd, "input");
});
