"use strict";
// node --test spellgui/web/test/
// A regra do widgets.js: schema -> tipo de widget, e valores dos campos -> args do request.
// Os schemas usados aqui sao os que o `schemars` gera de verdade (spellgui/web/dev/commands.json).

const { test } = require("node:test");
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");
const WG = require("../widgets.js");

const CMDS = JSON.parse(
  fs.readFileSync(path.join(__dirname, "..", "dev", "commands.json"), "utf8")
);
const cmd = n => CMDS.find(c => c.name === n);

test("kindOf: um tipo de widget por tipo de parametro", () => {
  assert.strictEqual(WG.kindOf({ type: "boolean" }), "toggle");
  assert.strictEqual(WG.kindOf({ type: "integer", format: "uint16", minimum: 0 }), "spin");
  assert.strictEqual(WG.kindOf({ type: "number", format: "double" }), "num");
  assert.strictEqual(WG.kindOf({ type: "number", minimum: 0, maximum: 1 }), "slider");
  assert.strictEqual(WG.kindOf({ type: "string" }), "text");
  assert.strictEqual(WG.kindOf({ type: "string", enum: ["linear", "hold"] }), "select");
  assert.strictEqual(WG.kindOf({ type: "object" }), "json");
  assert.strictEqual(WG.kindOf({ type: "array" }), "json");
  assert.strictEqual(WG.kindOf({}), "json", "sem tipo = JSON cru");
  // Option<T> do Rust vira ["T","null"]: o null nao muda o widget
  assert.strictEqual(WG.kindOf({ type: ["integer", "null"] }), "spin");
  assert.strictEqual(WG.kindOf({ type: ["string", "null"] }), "text");
  // enum sem type declarado ainda e' select
  assert.strictEqual(WG.kindOf({ enum: ["a", "b"] }), "select");
});

test("kindOfCommand: comando sem parametro e' trigger", () => {
  assert.strictEqual(WG.kindOfCommand(cmd("stop")), "button");
  assert.strictEqual(WG.kindOfCommand(cmd("transport_state")), "button");
  assert.strictEqual(WG.kindOfCommand(cmd("locate")), "form");
  assert.strictEqual(WG.kindOfCommand(cmd("key_set")), "form");
});

test("kindOf sobre os comandos reais do registry", () => {
  const p = n => cmd(n).params.properties;
  assert.strictEqual(WG.kindOf(p("locate").t), "num");
  assert.strictEqual(WG.kindOf(p("load").file), "text");
  assert.strictEqual(WG.kindOf(p("cue_go").index), "spin", "Option<usize>");
  assert.strictEqual(WG.kindOf(p("show_get").full), "toggle");
  assert.strictEqual(WG.kindOf(p("play_show").loop), "toggle");
  assert.strictEqual(WG.kindOf(p("key_set").curve), "text");
});

test("coerce: texto do campo vira o tipo do schema", () => {
  assert.strictEqual(WG.coerce("num", "12.5"), 12.5);
  assert.strictEqual(WG.coerce("spin", "3.7"), 4);
  assert.strictEqual(WG.coerce("spin", ""), 0);
  assert.strictEqual(WG.coerce("toggle", "true"), true);
  assert.strictEqual(WG.coerce("toggle", false), false);
  assert.strictEqual(WG.coerce("text", 5), "5");
  assert.deepStrictEqual(WG.coerce("json", '{"a":1}'), { a: 1 });
  assert.strictEqual(WG.coerce("json", "play"), "play", "texto que nao e JSON vai como texto");
  assert.deepStrictEqual(WG.coerce("json", "[255,0,0]"), [255, 0, 0]);
});

test("args: monta o request do comando a partir dos campos", () => {
  assert.deepStrictEqual(WG.args(cmd("locate"), { t: "12.5" }), { t: 12.5 });
  assert.deepStrictEqual(WG.args(cmd("stop"), {}), {}, "trigger nao tem argumento");
  // opcional em branco nao vai: quem decide o default e' o `#[serde(default)]` do Rust
  assert.deepStrictEqual(WG.args(cmd("cue_go"), { index: "" }), {});
  assert.deepStrictEqual(WG.args(cmd("cue_go"), { index: "2" }), { index: 2 });
  assert.deepStrictEqual(WG.args(cmd("show_get"), { file: "", full: true }), { full: true });
  // obrigatorio em branco vai como default do tipo (o engine reclama, nao a pagina)
  assert.deepStrictEqual(WG.args(cmd("load"), {}), { file: "" });
  const ks = WG.args(cmd("key_set"), { track: "1", t: "2.5", value: "255", curve: "hold" });
  assert.deepStrictEqual(ks, { track: 1, t: 2.5, value: 255, curve: "hold" });
  // campo que o schema nao declara nao entra no request
  assert.deepStrictEqual(WG.args(cmd("locate"), { t: "1", inventado: "x" }), { t: 1 });
});
