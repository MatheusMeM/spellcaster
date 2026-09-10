"use strict";
// node --test spellgui/web/test/
// A tabela do menu de VIDEO (`video.js`), que e' a parte dele que roda sem WebGL.
// O que se testa e' a invariante da tabela, nao o three.js:
//   toda opcao tem valor valido nas quatro predefinicoes
//   `set` fora de faixa (ou fora da lista) e' RECUSADO, e o valor antigo fica
//   `preset` seguido de `set` num valor diferente vira "custom"
//   `load` de JSON quebrado cai no padrao, sem lancar

const { test } = require("node:test");
const assert = require("node:assert");

// localStorage de mentira: o modulo grava e le' por aqui, e o teste mexe nele direto
const LS = { v: {}, getItem(k) { return k in this.v ? this.v[k] : null; }, setItem(k, s) { this.v[k] = String(s); }, removeItem(k) { delete this.v[k]; } };
global.window = global.window || {};
global.localStorage = LS;
const VIDEO = require("../laser3d/video.js");

test("a tabela: id unico, grupo conhecido, e as quatro predefinicoes validas", () => {
  const seen = new Set();
  for (const o of VIDEO.OPTS) {
    assert.ok(!seen.has(o.id), "id repetido: " + o.id); seen.add(o.id);
    assert.ok(VIDEO.GRUPOS.indexOf(o.g) >= 0, o.id + ": grupo desconhecido " + o.g);
    assert.ok(o.label && o.label.length, o.id + ": sem rotulo");
    for (const p of VIDEO.PRESETS) {
      const v = o.p[p];
      assert.notStrictEqual(v, undefined, o.id + ": sem valor na predefinicao " + p);
      VIDEO.preset(p);
      assert.strictEqual(String(VIDEO.get(o.id)), String(v), o.id + " em " + p);
    }
    if (o.type === "range") { assert.ok(o.min < o.max, o.id + ": faixa invertida"); assert.ok(o.step > 0, o.id + ": passo zero"); }
    else assert.ok(o.vals.length >= 2, o.id + ": select com menos de duas opcoes");
  }
});

test("cada predefinicao existe e se identifica pelo proprio nome", () => {
  for (const p of VIDEO.PRESETS) {
    assert.strictEqual(VIDEO.preset(p), true);
    assert.strictEqual(VIDEO.presetName(), p, "aplicou " + p + " e o nome nao bateu");
  }
  assert.strictEqual(VIDEO.preset("epico"), false, "predefinicao que nao existe nao pode aplicar");
});

test("set fora de faixa e' recusado e o valor antigo fica", () => {
  VIDEO.preset("alto");
  const fov = VIDEO.get("fov");
  for (const bad of [29, 91, 1e9, -1, NaN, Infinity, "muito", null, undefined, true, {}]) {
    assert.strictEqual(VIDEO.set("fov", bad), false, "aceitou fov=" + String(bad));
    assert.strictEqual(VIDEO.get("fov"), fov, "fov mudou com valor recusado " + String(bad));
  }
  assert.strictEqual(VIDEO.set("fov", 30), true, "30 esta' na faixa");
  assert.strictEqual(VIDEO.get("fov"), 30);

  VIDEO.preset("alto");
  const som = VIDEO.get("sombras");
  for (const bad of ["3072", 3072, "sim", "", "0.0"]) {
    assert.strictEqual(VIDEO.set("sombras", bad), false, "aceitou sombras=" + String(bad));
    assert.strictEqual(VIDEO.get("sombras"), som);
  }
  assert.strictEqual(VIDEO.set("sombras", 4096), true, "4096 esta' na lista");
  assert.strictEqual(VIDEO.set("naoexiste", 1), false, "id que nao existe nao entra");
});

test("preset seguido de set vira custom, e desfazer na mao volta ao nome", () => {
  VIDEO.preset("alto");
  assert.strictEqual(VIDEO.presetName(), "alto");
  const antes = VIDEO.get("sombras");
  assert.strictEqual(VIDEO.set("sombras", "0"), true);
  assert.strictEqual(VIDEO.presetName(), "custom", "mexeu numa linha e continuou 'alto'");
  VIDEO.set("sombras", antes);
  assert.strictEqual(VIDEO.presetName(), "alto", "voltou o valor na mao e o nome nao voltou");
});

test("load de JSON quebrado cai no padrao, e linha ruim nao derruba as boas", () => {
  for (const lixo of ["{", "", "null", "[1,2,3]", '"texto"', "{oops}"]) {
    LS.v["sc-laser-video"] = lixo;
    assert.doesNotThrow(() => VIDEO.load(), "load lancou com " + JSON.stringify(lixo));
    if (lixo !== "") assert.strictEqual(VIDEO.presetName(), VIDEO.defaultPreset(), "JSON " + JSON.stringify(lixo) + " nao caiu no padrao");
  }
  // metade boa, metade lixo: a boa entra, a ruim fica no padrao
  VIDEO.preset("alto");
  const padraoSombra = VIDEO.get("sombras");
  LS.v["sc-laser-video"] = JSON.stringify({ sombras: "9999", fov: 61, escala: "0.5", bloomForca: "nao" });
  VIDEO.load();
  assert.strictEqual(VIDEO.get("fov"), 61, "linha boa nao entrou");
  assert.strictEqual(VIDEO.get("sombras"), padraoSombra, "linha ruim entrou");
  assert.strictEqual(VIDEO.get("bloomForca"), VIDEO.opt("bloomForca").p[VIDEO.defaultPreset()], "range com texto entrou");
  delete LS.v["sc-laser-video"];
});

test("onChange recebe o id que mudou, e null quando foi a predefinicao inteira", () => {
  const vistos = [];
  VIDEO.onChange(id => vistos.push(id));
  VIDEO.preset("ultra");
  VIDEO.set("nevoa", .1);
  VIDEO.set("nevoa", .1);            // mesmo valor: nao avisa de novo
  VIDEO.set("nevoa", 99);            // recusado: nao avisa
  VIDEO.onChange(null);
  assert.deepStrictEqual(vistos, [null, "nevoa"]);
});
