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

// ---- display e controles (frente ui-3d-usab) ----
const { oledLines, CONTROLS, kindOf, PAGES } = require("../laser3d/engine.js");

test("oledLines: desligado so' diz que esta' desligado; ligado, a linha grande e' o estado", () => {
  assert.strictEqual(oledLines({ power: false }, {})[1], "DESLIGADO");
  const desarm = oledLines({ power: true, key: false, lock: true, page: 0, kpps: 30000, show: [[1, 2, 3]], frame: 0 }, {});
  assert.strictEqual(desarm[1], "DESARMADO");
  assert.match(desarm[0], /^STATUS +1\/7$/);
  assert.strictEqual(oledLines({ power: true, key: true, lock: true, page: 0, kpps: 30000, show: [[]], frame: 0 }, {})[1], "LIVE");
  assert.strictEqual(oledLines({ power: true, key: true, lock: false, page: 0, kpps: 30000, show: [[]], frame: 0 }, {})[1], "SCAN FAIL");
});

test("oledLines: campo em edicao leva '>' e a pagina ERRO mostra o ultimo erro", () => {
  const dmx = oledLines({ power: true, page: PAGES.indexOf("DMX"), edit: true, field: 1, dmx: 7, univ: 3 }, {});
  assert.strictEqual(dmx[1], "ADDR 007");
  assert.strictEqual(dmx[2][0], " ", "campo 0 nao esta' selecionado");
  assert.strictEqual(dmx[3][0], ">", "campo 1 selecionado");
  const erro = oledLines({ power: true, page: PAGES.indexOf("ERRO"), err: { msg: "laser_open: sem DAC", when: "20:34:00" } }, {});
  assert.strictEqual(erro[1], "ERRO");
  assert.strictEqual(erro[2], " LASER_OPEN: SEM DAC");
  assert.ok(oledLines({ power: true, page: 0 }, {}).every(l => /^[\x20-\x7e]*$/.test(l)), "display so' ASCII");
});

test("CONTROLS: um controle, uma familia; energia e interlock num ponto so'", () => {
  const fam = new Set(["toggle", "momentary", "valor", "conector", "navegacao"]);
  for (const k in CONTROLS) assert.ok(fam.has(CONTROLS[k][0]), k + " tem familia conhecida");
  assert.strictEqual(kindOf("acin"), "conector", "powerCON e' plugue");
  assert.strictEqual(kindOf("power"), "toggle", "rocker liga e desliga");
  const toggles = Object.keys(CONTROLS).filter(k => CONTROLS[k][0] === "toggle");
  assert.deepStrictEqual(toggles.sort(), ["interlock", "keyswitch", "power"], "cada toggle e' uma funcao unica");
  assert.strictEqual(kindOf("fusivel"), "", "nao existe fusivel");
});
