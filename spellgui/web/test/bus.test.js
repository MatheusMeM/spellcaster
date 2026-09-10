"use strict";
// node --test spellgui/web/test/
// Bus.parse is the only rule of bus.js: the rest is socket. No DOM, no server.

const { test } = require("node:test");
const assert = require("node:assert");
const Bus = require("../bus.js");

test("parse: response with result", () => {
  const p = Bus.parse('{"id":7,"result":{"t":12.5},"rev":3}');
  assert.deepStrictEqual(p, { id: 7, result: { t: 12.5 }, rev: 3 });
  // a response with no `rev` (old bus) does not invent a number
  assert.deepStrictEqual(Bus.parse('{"id":7,"result":1}'), { id: 7, result: 1, rev: undefined });
});

test("parse: response with error", () => {
  assert.deepStrictEqual(Bus.parse('{"id":7,"error":"no player running","rev":9}'), {
    id: 7,
    error: "no player running",
    rev: 9,
  });
  // a null result with no error is still a result
  assert.deepStrictEqual(Bus.parse('{"id":1,"result":null,"rev":0}'), {
    id: 1,
    result: null,
    rev: 0,
  });
});

test("rev: the bus keeps the highest it saw and never walks back", () => {
  const b = new Bus({ offline: true });
  b.recv('{"id":1,"result":1,"rev":4}');
  assert.strictEqual(b.rev, 4);
  b.recv('{"id":2,"result":1,"rev":2}');
  assert.strictEqual(b.rev, 4);
  b.recv('{"id":3,"result":1,"rev":7}');
  assert.strictEqual(b.rev, 7);
});

test("binary parse: topic 1 (output) and 2 (input) with 515 bytes", () => {
  const short = new Uint8Array(300);
  short[0] = 1;
  assert.strictEqual(Bus.parse(short), null);
  const input = new Uint8Array(515);
  input[0] = 2; input[1] = 1;
  assert.strictEqual(Bus.parse(input).data.topic, 2);
  const other = new Uint8Array(515);
  other[0] = 3;
  assert.strictEqual(Bus.parse(other), null);
});

test("parse: event", () => {
  assert.deepStrictEqual(Bus.parse('{"event":"show","data":{"rev":5}}'), {
    event: "show",
    data: { rev: 5 },
  });
  assert.deepStrictEqual(Bus.parse('{"event":"widget","data":{"id":"go","prop":"glow","value":1}}'), {
    event: "widget",
    data: { id: "go", prop: "glow", value: 1 },
  });
});

test("parse: binary frame topic|universe LE|512", () => {
  const b = new Uint8Array(515);
  b[0] = 1; // topic 1 = dmx
  b[1] = 0x0d;
  b[2] = 0x01; // universe 269, little endian
  b[3] = 255;
  b[514] = 7;
  const p = Bus.parse(b);
  assert.strictEqual(p.event, "dmx");
  assert.strictEqual(p.data.topic, 1);
  assert.strictEqual(p.data.universe, 269);
  assert.strictEqual(p.data.data.length, 512);
  assert.strictEqual(p.data.data[0], 255);
  assert.strictEqual(p.data.data[511], 7);
  // raw ArrayBuffer (what the WebSocket hands over with binaryType="arraybuffer")
  assert.strictEqual(Bus.parse(b.buffer).data.universe, 269);
});

test("parse: junk does not bring it down", () => {
  assert.strictEqual(Bus.parse("this is not json"), null);
  assert.strictEqual(Bus.parse("[1,2,3]"), null);
  assert.strictEqual(Bus.parse('{"neither":"id nor event"}'), null);
  assert.strictEqual(Bus.parse(new Uint8Array(2)), null);
});

test("recv: it hands the result to the pending promise and the event to the subscriber", async () => {
  const b = new Bus({ offline: true });
  let seen = null;
  b.on("transport", d => (seen = d));
  b.recv('{"event":"transport","data":{"state":"play"}}');
  assert.deepStrictEqual(seen, { state: "play" });

  const p = new Promise((ok, err) => b.pend.set(3, { ok, err }));
  b.recv('{"id":3,"result":42}');
  assert.strictEqual(await p, 42);

  const q = new Promise((ok, err) => b.pend.set(4, { ok, err }));
  b.recv('{"id":4,"error":"it did not work"}');
  await assert.rejects(q, /it did not work/);
  assert.strictEqual(b.pend.size, 0, "pending calls cleared");
});

test("offline: the call becomes an echo and the log tells what it would do", async () => {
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
