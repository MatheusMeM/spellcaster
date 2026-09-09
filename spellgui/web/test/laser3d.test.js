"use strict";
// node --test spellgui/web/test/
// A parte da pagina do laser 3D que roda sem WebGL: o manifesto das acoes (o que a frente MIDI
// vai consumir), o mapa estado -> comando do registry, e o leitor de ILDA.

const { test } = require("node:test");
const assert = require("node:assert");

// bind.js e ilda.js sao arquivos de navegador: dois stubs bastam (loopback do DOM).
global.window = {};
global.document = { addEventListener() {} };

const Bind = require("../laser3d/bind.js");
const ILDA = require("../laser3d/ilda.js");
const { cmdFor } = require("../laser3d/engine.js");

test("Bind.manifest: uma linha por acao, com tecla, MIDI e tipo", () => {
  Bind.def("key.toggle", "chave: arma", () => {}, { key: "S", get: () => true });
  Bind.def("kpps", "kpps (fader)", () => {}, { type: "cc" });
  Bind.def("mudo", "sem tecla", () => {});
  const m = Bind.manifest();
  assert.deepStrictEqual(m.map(a => a.id), ["key.toggle", "kpps", "mudo"], "ordem de declaracao");
  assert.deepStrictEqual(m[0], { id: "key.toggle", label: "chave: arma", key: "S", midi: "", type: "btn" });
  assert.strictEqual(m[1].type, "cc", "fader e' cc, nao botao");
  assert.strictEqual(m[2].key, "", "acao sem tecla nao inventa tecla");
});

test("cmdFor: a chave abre e fecha o DAC", () => {
  assert.deepStrictEqual(cmdFor("key", true, { dac: "etherdream", host: "10.0.0.9", kpps: 30000 }),
    { cmd: "laser_open", args: { dac: "etherdream", host: "10.0.0.9", kpps: 30 } });
  assert.deepStrictEqual(cmdFor("key", false, { feed: 7 }), { cmd: "laser_close", args: { feed: 7 } });
  assert.strictEqual(cmdFor("key", false, {}), null, "desarmar sem feed aberto nao manda nada");
});

test("cmdFor: play toca o .ild no feed, e o transporte anda com o show", () => {
  assert.deepStrictEqual(cmdFor("play", true, { feed: 2, file: "shows/a.ild", fps: 25 }),
    { cmd: "laser_play", args: { feed: 2, file: "shows/a.ild", fps: 25, loop: true } });
  assert.deepStrictEqual(cmdFor("play", false, { feed: 2 }), { cmd: "laser_stop", args: { feed: 2 } });
  assert.strictEqual(cmdFor("play", true, { feed: 2 }), null, "sem arquivo nao ha' o que tocar");
  assert.strictEqual(cmdFor("play", true, {}), null, "sem feed nao ha' onde tocar");
  assert.deepStrictEqual(cmdFor("transport", true, { show: "medgrupo" }), { cmd: "resume", args: {} });
  assert.deepStrictEqual(cmdFor("transport", false, { show: "medgrupo" }), { cmd: "pause", args: {} });
  assert.strictEqual(cmdFor("transport", true, {}), null, "sem show carregado o transporte fica quieto");
});

test("cmdFor: os parametros que o laser_param aceita hoje, e so' eles", () => {
  const st = { feed: 1 };
  assert.deepStrictEqual(cmdFor("size", 1.25, st), { cmd: "laser_param", args: { feed: 1, path: "geo/scale", value: 1.25 } });
  assert.deepStrictEqual(cmdFor("lim.g", 0.5, st), { cmd: "laser_param", args: { feed: 1, path: "limit/g", value: 0.5 } });
  // interlock fechado = obturador aberto
  assert.deepStrictEqual(cmdFor("lock", true, st), { cmd: "laser_param", args: { feed: 1, path: "shutter", value: 0 } });
  assert.deepStrictEqual(cmdFor("lock", false, st), { cmd: "laser_param", args: { feed: 1, path: "shutter", value: 1 } });
  for (const id of ["kpps", "gam.r", "dmx", "buffer", "speed", "net.sacn", "power", "cam.show"]) {
    assert.strictEqual(cmdFor(id, 1, st), null, id + " ainda nao tem comando: fica local");
  }
  assert.strictEqual(cmdFor("size", 1, {}), null, "sem feed aberto nao ha' o que parametrizar");
});

test("ILDA: le de volta o frame que escreveu (formato 5, RGB)", () => {
  const frame = [
    { x: 0, y: 0, r: 255, g: 0, b: 0, bl: true },
    { x: 10000, y: -10000, r: 0, g: 255, b: 0, bl: false },
    { x: -32768, y: 32767, r: 1, g: 2, b: 3, bl: false },
  ];
  const d = ILDA.parse(ILDA.write([frame]).buffer);
  assert.strictEqual(d.frames.length, 1);
  assert.strictEqual(d.name, "SPELL", "o cabecalho leva o nome de quem escreveu");
  assert.deepStrictEqual(d.frames[0], frame);
});

test("ILDA: dado que nao e' ILDA devolve zero frame, sem estourar", () => {
  assert.deepStrictEqual(ILDA.parse(new Uint8Array(64).buffer).frames, []);
});
