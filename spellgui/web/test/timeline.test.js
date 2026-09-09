"use strict";
// node --test spellgui/web/test/timeline.test.js
// Cobre as duas funcoes puras que a timeline ganhou ao ligar no engine: a traducao de uma edicao
// local em chamadas do registry (TL.ops) e a leitura do frame binario do monitor (TL.frameBin).
// timeline.js e' script de navegador: carrega com `window` e `CK` falsos, sem DOM.

const { test } = require("node:test");
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");

function loadTL() {
  const src = fs.readFileSync(path.join(__dirname, "..", "timeline.js"), "utf8");
  const win = {};
  const CK = {
    clamp: (v, a, b) => (v < a ? a : v > b ? b : v),
    bisect(ts, n, t) {
      let lo = 0, hi = n;
      while (lo < hi) { const m = (lo + hi) >> 1; if (ts[m] < t) lo = m + 1; else hi = m; }
      return lo;
    },
    near: () => -1,
    sel: () => ({}),
  };
  new Function("window", "CK", src)(win, CK);
  return win.TL;
}

const TL = loadTL();

test("keyframe novo vira key_set com t, value e curva", () => {
  const c = TL.ops([{ k: "set", track: 2, t: 1.5, value: 200, curve: "inout" }]);
  assert.deepStrictEqual(c, [
    { cmd: "key_set", args: { track: 2, t: 1.5, value: 200, curve: "inout" } },
  ]);
});

test("keyframe apagado vira key_del", () => {
  assert.deepStrictEqual(TL.ops([{ k: "del", track: 0, t: 3.25 }]), [
    { cmd: "key_del", args: { track: 0, t: 3.25 } },
  ]);
});

test("keyframe arrastado vira key_del do tempo antigo mais key_set do novo", () => {
  const c = TL.ops([{ k: "move", track: 1, from: 2, t: 4, value: 10, curve: "linear" }]);
  assert.deepStrictEqual(c, [
    { cmd: "key_del", args: { track: 1, t: 2 } },
    { cmd: "key_set", args: { track: 1, t: 4, value: 10, curve: "linear" } },
  ]);
});

// Dois keyframes vizinhos que trocam de lugar: se as ops se intercalassem, o key_del do segundo
// apagaria o key_set do primeiro. Por isso todo del sai antes de todo set.
test("todo key_del sai antes de todo key_set", () => {
  const c = TL.ops([
    { k: "move", track: 0, from: 1, t: 2, value: 5, curve: "linear" },
    { k: "move", track: 0, from: 2, t: 3, value: 6, curve: "hold" },
  ]);
  assert.deepStrictEqual(c.map(x => x.cmd), ["key_del", "key_del", "key_set", "key_set"]);
  assert.deepStrictEqual(c.map(x => x.args.t), [1, 2, 2, 3]);
});

test("campos do show viram um show_patch so, na ordem", () => {
  const c = TL.ops([
    { k: "field", path: "/in", value: 2 },
    { k: "field", path: "/out", value: 8 },
    { k: "field", path: "/markers", value: [1, 2] },
  ]);
  assert.strictEqual(c.length, 1);
  assert.strictEqual(c[0].cmd, "show_patch");
  assert.deepStrictEqual(c[0].args.ops, [
    { op: "add", path: "/in", value: 2 },
    { op: "add", path: "/out", value: 8 },
    { op: "add", path: "/markers", value: [1, 2] },
  ]);
});

test("mistura: keyframes primeiro, show_patch por ultimo", () => {
  const c = TL.ops([
    { k: "field", path: "/tracks/0/mute", value: true },
    { k: "set", track: 0, t: 0, value: 1, curve: "linear" },
    { k: "del", track: 0, t: 9 },
  ]);
  assert.deepStrictEqual(c.map(x => x.cmd), ["key_del", "key_set", "show_patch"]);
});

test("sem edicao, nenhuma chamada", () => {
  assert.deepStrictEqual(TL.ops([]), []);
});

// Frame binario do barramento: topic:u8 | universe:u16 LE | 512 bytes.
test("frame binario do monitor: universo em little endian e 512 canais", () => {
  const b = new Uint8Array(515);
  b[0] = 1;
  b[1] = 0x02; b[2] = 0x01;            // universo 258
  b[3] = 255; b[514] = 7;
  const f = TL.frameBin(b.buffer);
  assert.strictEqual(f.universe, 258);
  assert.strictEqual(f.data.length, 512);
  assert.strictEqual(f.data[0], 255);
  assert.strictEqual(f.data[511], 7);
});

test("frame de outro topico ou curto demais e ignorado", () => {
  const outro = new Uint8Array(515);
  outro[0] = 2;
  assert.strictEqual(TL.frameBin(outro.buffer), null);
  assert.strictEqual(TL.frameBin(new Uint8Array(10).buffer), null);
});
