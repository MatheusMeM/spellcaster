// A funcao pura do cenario: frame DMX + patch + perfil -> cor e intensidade de cada fixture.
// node --test spellgui/web/test/teatro.test.js
"use strict";
const test = require("node:test");
const assert = require("node:assert");
const T = require("../teatro.js");

const par = {
  name: "PAR LED RGB 3ch",
  channels: [{ name: "r", offset: 0 }, { name: "g", offset: 1 }, { name: "b", offset: 2 }],
};
const dimmer = { name: "Dimmer 1ch", channels: [{ name: "dim", offset: 0 }] };
const wash = {
  name: "Wash 4ch",
  channels: [{ name: "r", offset: 0 }, { name: "g", offset: 1 },
             { name: "b", offset: 2 }, { name: "w", offset: 3 }],
};
const bsw = {
  name: "BSW 17ch",
  channels: [{ name: "pan", offset: 0, fine: 1 }, { name: "dim", offset: 9 },
             { name: "color", offset: 4, wheel: { branco: 0 } }],
};
const perfis = { par_rgb_3: par, dimmer_1: dimmer, wash_bx402_4: wash, bsw_scorpio_17: bsw };

// universo com valores em enderecos 1-based
function uni(pares) {
  const a = new Array(512).fill(0);
  for (const [addr, v] of pares) a[addr - 1] = v;
  return a;
}

test("cor e intensidade por tipo de perfil", () => {
  const patch = [
    { name: "par 1", profile: "par_rgb_3", universe: 1, address: 1 },
    { name: "ribalta", profile: "wash_bx402_4", universe: 1, address: 10 },
    { name: "fresnel", profile: "dimmer_1", universe: 1, address: 20 },
    { name: "movel", profile: "bsw_scorpio_17", universe: 2, address: 1 },
  ];
  const dmx = {
    1: uni([[1, 255], [2, 0], [3, 0], [10, 10], [11, 20], [12, 30], [13, 200], [20, 128]]),
    2: uni([[1, 77], [10, 64]]),
  };
  const [p, w, f, m] = T.look(patch, perfis, dmx);

  assert.deepStrictEqual(p.rgb, [255, 0, 0], "RGB puro");
  assert.strictEqual(p.intensity, 1);
  assert.deepStrictEqual(p.channels, { r: 255, g: 0, b: 0 });

  // o w soma nos tres canais e satura em 255
  assert.deepStrictEqual(w.rgb, [210, 220, 230]);
  assert.strictEqual(w.intensity, 200 / 255, "sem dimmer, manda o canal de cor mais aceso");

  // dimmer nao tem cor: lampada branca com a intensidade do canal
  assert.deepStrictEqual(f.rgb, [255, 255, 255]);
  assert.strictEqual(f.intensity, 128 / 255);

  // universo 2, offset do dimmer no meio do perfil; canal sem nome fica de fora
  assert.strictEqual(m.universe, 2);
  assert.strictEqual(m.channels.pan, 77);
  assert.strictEqual(m.intensity, 64 / 255, "dim manda mesmo com canais de cor ausentes");
  assert.deepStrictEqual(m.rgb, [255, 255, 255]);
});

test("saturacao do branco e ausencia de frame", () => {
  const patch = [{ name: "w", profile: "wash_bx402_4", universe: 1, address: 1 }];
  const cheio = T.look(patch, perfis, { 1: uni([[1, 200], [4, 200]]) })[0];
  assert.deepStrictEqual(cheio.rgb, [255, 200, 200], "r + w satura em 255");

  // universo que ainda nao chegou pelo barramento: tudo apagado, nada quebra
  const escuro = T.look(patch, perfis, {})[0];
  assert.deepStrictEqual(escuro.rgb, [0, 0, 0]);
  assert.strictEqual(escuro.intensity, 0);
});

test("perfil ausente e endereco fora do universo nao quebram", () => {
  const patch = [
    { name: "sem perfil", profile: "nao_existe", universe: 1, address: 1 },
    { name: "no fim", profile: "par_rgb_3", universe: 1, address: 511 },
  ];
  const [a, b] = T.look(patch, perfis, { 1: uni([[511, 90], [512, 91]]) });
  assert.deepStrictEqual(a.channels, {});
  assert.strictEqual(a.intensity, 0);
  assert.deepStrictEqual(a.rgb, [255, 255, 255], "sem perfil: branco apagado");
  assert.deepStrictEqual(b.rgb, [90, 91, 0], "canal 513 nao existe e vale 0");
});

test("posicao padrao espalha em fileira e a salva manda", () => {
  const patch = [
    { name: "a", profile: "dimmer_1", universe: 1, address: 1 },
    { name: "b", profile: "dimmer_1", universe: 1, address: 2, pos: [0.5, 0.9] },
  ];
  const fs = T.look(patch, perfis, {});
  assert.strictEqual(fs[0].pos, null);
  assert.deepStrictEqual(fs[1].pos, [0.5, 0.9]);
  const [x0] = T.posicao(fs[0], 2);
  assert.ok(x0 > 0 && x0 < 1, "posicao padrao dentro da tela: " + x0);
  assert.deepStrictEqual(T.posicao(fs[1], 2), [0.5, 0.9]);
});
