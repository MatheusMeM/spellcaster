"use strict";
// node --test spellgui/web/test/viewer.test.js
// Cobre o que o previz tem de logica: a interpolacao de `scale`/`rot` pelas keys do track, a
// escolha do quadro por `t` e `fps`, o cache por (clipe, quadro) e a cor da fixture pelo perfil.
// viewer.js e' script de navegador e depende de `window.TL`: carrega timeline.js antes, com
// `window` e `CK` falsos, sem DOM.

const { test } = require("node:test");
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");

function load() {
  const win = {};
  const CK = {
    clamp: (v, a, b) => (v < a ? a : v > b ? b : v),
    bisect: () => 0,
    near: () => -1,
    sel: () => ({}),
  };
  const TEATRO = require("../teatro.js");           // parte pura: a cor da fixture sai dela
  for (const f of ["timeline.js", "viewer.js"]) {
    const src = fs.readFileSync(path.join(__dirname, "..", f), "utf8");
    new Function("window", "CK", "TEATRO", src)(win, CK, TEATRO);
  }
  return win;
}

const win = load();
const VW = win.VW;

// ---- interpolacao de scale/rot -----------------------------------------

test("param sem keys devolve o padrao", () => {
  assert.strictEqual(VW.param(undefined, 3, 1), 1);
  assert.strictEqual(VW.param([], 3, 0), 0);
});

test("param segura o primeiro e o ultimo valor fora do intervalo", () => {
  const k = [[1, 10], [3, 30]];
  assert.strictEqual(VW.param(k, 0, 0), 10);
  assert.strictEqual(VW.param(k, 9, 0), 30);
});

test("param interpola linear entre dois keyframes", () => {
  assert.strictEqual(VW.param([[0, 0], [2, 90]], 1, 0), 45);
  assert.strictEqual(VW.param([[0, 1], [4, 2]], 3, 1), 1.75);
});

// A curva vale para o segmento que CHEGA no keyframe: `hold` segura o valor anterior ate o fim.
test("param respeita a curva do keyframe de chegada", () => {
  assert.strictEqual(VW.param([[0, 0], [2, 100, "hold"]], 1.9, 0), 0);
  assert.strictEqual(VW.param([[0, 0], [2, 100, "in"]], 1, 0), 25);
  assert.strictEqual(VW.param([[0, 0], [2, 100, "out"]], 1, 0), 75);
});

test("param com tres keys pega o segmento certo", () => {
  const k = [[0, 0], [1, 10], [2, 0]];
  assert.strictEqual(VW.param(k, 0.5, 0), 5);
  assert.strictEqual(VW.param(k, 1.5, 0), 5);
  assert.strictEqual(VW.param(k, 1, 0), 10);
});

// ---- quadro por t e fps -------------------------------------------------

test("index e floor(t * fps) enquanto o total de quadros nao chegou", () => {
  VW.n.clear();
  assert.strictEqual(VW.index("x.ild", 0, 30), 0);
  assert.strictEqual(VW.index("x.ild", 1, 30), 30);
  assert.strictEqual(VW.index("x.ild", 0.5, 10), 5);
  assert.strictEqual(VW.index("x.ild", -1, 30), 0);
  assert.strictEqual(VW.index("x.ild", 1, 0), 30, "fps zero cai no padrao 30");
});

test("com o total conhecido, o clipe repete", () => {
  VW.n.set("x.ild", 4);
  assert.strictEqual(VW.index("x.ild", 1, 30), 2);
  assert.strictEqual(VW.index("x.ild", 0.1, 30), 3);
  VW.n.clear();
});

// ---- cache por (clipe, quadro) ------------------------------------------

test("cache pede uma vez por quadro e devolve os pontos quando chegam", async () => {
  const pedidos = [];
  let solta;
  VW.clip.clear(); VW.n.clear();
  VW.call = (cmd, args) => {
    pedidos.push([cmd, args]);
    return new Promise(ok => { solta = ok; });
  };
  assert.strictEqual(VW.frame("a.ild", 7), null, "primeiro pedido ainda nao tem pontos");
  assert.strictEqual(VW.frame("a.ild", 7), null, "segundo desenho nao repede");
  assert.strictEqual(pedidos.length, 1);
  assert.deepStrictEqual(pedidos[0], ["clip_frame", { clip: "a.ild", index: 7 }]);

  solta({ frames: 12, points: [[0, 0, 255, 0, 0, 0]] });
  await Promise.resolve();
  assert.deepStrictEqual(VW.frame("a.ild", 7), [[0, 0, 255, 0, 0, 0]]);
  assert.strictEqual(pedidos.length, 1, "quadro em cache nao pede de novo");
  assert.strictEqual(VW.n.get("a.ild"), 12, "o total de quadros voltou com a resposta");
  assert.strictEqual(VW.frame("a.ild", 8), null, "outro quadro e outro pedido");
  assert.strictEqual(pedidos.length, 2);
});

test("erro do engine fica em cache: o desenho nao vira uma enxurrada de pedidos", async () => {
  let n = 0;
  VW.clip.clear();
  VW.call = () => { n++; return Promise.reject("sem servidor"); };
  VW.frame("b.ild", 0);
  await Promise.resolve();
  await Promise.resolve();
  assert.deepStrictEqual(VW.frame("b.ild", 0), []);
  VW.frame("b.ild", 0);
  assert.strictEqual(n, 1);
});

test("cache esvazia no teto em vez de crescer sem fim", () => {
  VW.clip.clear();
  VW.call = () => new Promise(() => {});
  for (let i = 0; i <= VW.cap; i++) VW.frame("c.ild", i);
  assert.ok(VW.clip.size <= VW.cap, "tamanho " + VW.clip.size);
});

// ---- cor da fixture pelo perfil ----------------------------------------

test("cor sai dos canais r/g/b no endereco da fixture", () => {
  const ch = [{ name: "r", offset: 0 }, { name: "g", offset: 1 }, { name: "b", offset: 2 }];
  const d = new Uint8Array(512);
  d[9] = 255; d[10] = 128; d[11] = 0;                     // endereco 10 = indice 9
  assert.deepStrictEqual(VW.cor(ch, d, 10), [255, 128, 0]);
});

test("perfil so com dimmer da branco vezes o dimmer", () => {
  const d = new Uint8Array(512);
  d[0] = 128;
  assert.deepStrictEqual(VW.cor([{ name: "dim", offset: 0 }], d, 1), [128, 128, 128]);
});

// A cor e' a do teatro de papel: `w` entra na mistura e `on` acende, senao as duas plantas
// mostrariam a mesma fixture de cores diferentes.
test("canal w e canal on contam, como no teatro", () => {
  const d = new Uint8Array(512);
  d[3] = 255;                                             // barra WLED so' no branco
  const wled = [{ name: "r", offset: 0 }, { name: "g", offset: 1 },
    { name: "b", offset: 2 }, { name: "w", offset: 3 }];
  assert.deepStrictEqual(VW.cor(wled, d, 1), [255, 255, 255]);
  d[0] = 255;
  assert.deepStrictEqual(VW.cor([{ name: "on", offset: 0 }], d, 1), [255, 255, 255]);
});

test("dimmer escala o rgb, e sem cor nenhuma a fixture fica vazia", () => {
  const d = new Uint8Array(512);
  d[0] = 200; d[3] = 0;
  const ch = [{ name: "r", offset: 0 }, { name: "dim", offset: 3 }];
  assert.strictEqual(VW.cor(ch, d, 1), null, "dimmer em zero: apagada, so' o contorno");
  assert.strictEqual(VW.cor([{ name: "pan", offset: 0 }], d, 1), null);
  assert.strictEqual(VW.cor(null, d, 1), null, "perfil que ainda nao chegou");
  assert.strictEqual(VW.cor([{ name: "r", offset: 0 }], null, 1), null, "sem frame de saida");
});

// ---- coluna dmx: saida, ou entrada quando a lane esta' armada -----------

test("fonte: lane armada le a entrada; a outra, a saida", () => {
  assert.deepStrictEqual(VW.fonte({ spec: { universe: 3 }, rec: true }),
    { u: 3, arm: true, lab: "dmx u3 in" });
  assert.deepStrictEqual(VW.fonte({ spec: { universe: 3 } }),
    { u: 3, arm: false, lab: "dmx u3 out" });
  assert.deepStrictEqual(VW.fonte({ spec: {} }), { u: 1, arm: false, lab: "dmx u1 out" });
  assert.deepStrictEqual(VW.fonte(undefined), { u: 1, arm: false, lab: "dmx u1 out" });
});
