"use strict";
// node --test spellgui/web/test/timeline.test.js
// Cobre as tres funcoes puras que a timeline ganhou ao ligar no engine: a traducao de uma edicao
// local em chamadas do registry (TL.ops), a leitura do frame binario do monitor (TL.frameBin) e a
// decisao de recarregar o show pelo `rev` do evento `show` (TL.revEvento).
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
  assert.strictEqual(f.topic, 1);
  assert.strictEqual(f.universe, 258);
  assert.strictEqual(f.data.length, 512);
  assert.strictEqual(f.data[0], 255);
  assert.strictEqual(f.data[511], 7);
});

// topic 2 = dmx de ENTRADA (show.inputs): mesmo formato, outro destino no desenho do monitor.
test("frame do topico 2 e a entrada, nao a saida", () => {
  const b = new Uint8Array(515);
  b[0] = 2; b[1] = 1; b[3] = 99;
  const f = TL.frameBin(b.buffer);
  assert.strictEqual(f.topic, 2);
  assert.strictEqual(f.universe, 1);
  assert.strictEqual(f.data[0], 99);
});

test("frame de topico sem consumidor ou curto demais e ignorado", () => {
  const outro = new Uint8Array(515);
  outro[0] = 3;
  assert.strictEqual(TL.frameBin(outro.buffer), null);
  assert.strictEqual(TL.frameBin(new Uint8Array(10).buffer), null);
});

// ---- +Track: um menu de tipo em vez de so' dmx ---------------------------
test("+Track dmx manda os campos de sempre", () => {
  assert.deepStrictEqual(TL.trackArgs("dmx"), {
    type: "dmx", universe: 1, address: 1, label: "",
  });
  assert.deepStrictEqual(TL.trackArgs(), { type: "dmx", universe: 1, address: 1, label: "" });
});

test("+Track laser leva o clipe .ild", () => {
  assert.deepStrictEqual(TL.trackArgs("laser", "medgrupo_laser.ild"), {
    type: "laser", universe: 1, address: 1, label: "", clip: "medgrupo_laser.ild",
  });
});

test("+Track fx leva o script .rhai", () => {
  assert.deepStrictEqual(TL.trackArgs("fx", "medgrupo.rhai"), {
    type: "fx", universe: 1, address: 1, label: "", script: "medgrupo.rhai",
  });
});

// ---- record arm: o estado vem do engine, nao do .spell -------------------
test("rec_state marca so' as lanes dos tracks armados", () => {
  TL.lanes = [{ si: 0, rec: true }, { si: 1, rec: false }, { si: 1, param: "scale", rec: true }];
  TL.recApply({ recording: true, tracks: [1] });
  assert.deepStrictEqual(TL.lanes.map(L => L.rec), [false, true, false]);
});

test("sem nada armado, toda lane desarma", () => {
  TL.lanes = [{ si: 0, rec: true }, { si: 1, rec: true }];
  TL.recApply({ recording: false, tracks: [] });
  assert.deepStrictEqual(TL.lanes.map(L => L.rec), [false, false]);
  TL.recApply(null);
  assert.deepStrictEqual(TL.lanes.map(L => L.rec), [false, false]);
});

// Evento `show`: ha' UM contador, o do engine, e toda resposta do barramento o traz. `TL.rev` e'
// o maior `rev` ja' visto numa resposta; recarregar so' quando o evento passa desse numero.
test("eco da propria edicao nao recarrega", () => {
  assert.deepStrictEqual(TL.revEvento(5, 5), { rev: 5, reload: false });
});

test("rev acima do esperado e edicao de outro cliente: recarrega", () => {
  assert.deepStrictEqual(TL.revEvento(6, 5), { rev: 6, reload: true });
});

test("evento atrasado com edicoes ainda em voo nao recarrega nem atrasa a conta", () => {
  assert.deepStrictEqual(TL.revEvento(4, 6), { rev: 6, reload: false });
});

// O contador antigo (`expect++` por chamada) nunca voltava de um desvio (comando contado a mais,
// evento perdido, pagina aberta contra um engine que ja' tinha revisoes): engolia os reloads de
// fora para sempre. Com `rev` o desvio custa um reload e a conta volta ao numero do engine.
test("conta desalinhada se conserta no primeiro evento", () => {
  const r = TL.revEvento(9, 1);
  assert.deepStrictEqual(r, { rev: 9, reload: true });
  assert.deepStrictEqual(TL.revEvento(10, r.rev), { rev: 10, reload: true });
});

// Reconexao: o processo novo comeca em rev = 0. `revEvento` so' corrige a conta para cima, entao
// o onopen zera TL.rev antes do reload; sem isso o primeiro `show` do engine novo (rev 1) cairia
// abaixo da conta velha e nao recarregaria nada.
test("depois do reset da reconexao, o primeiro evento do engine novo recarrega", () => {
  assert.deepStrictEqual(TL.revEvento(1, 0), { rev: 1, reload: true });
});

test("sem o reset, o engine reiniciado seria engolido", () => {
  assert.deepStrictEqual(TL.revEvento(1, 37), { rev: 37, reload: false });
});
