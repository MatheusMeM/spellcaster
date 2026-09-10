"use strict";
// node --test spellgui/web/test/
// The rule of widgets.js: schema -> widget type, and field values -> args of the request.
// The schemas used here are the ones `schemars` really generates (spellgui/web/dev/commands.json).

const { test } = require("node:test");
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");
const WG = require("../widgets.js");

const CMDS = JSON.parse(
  fs.readFileSync(path.join(__dirname, "..", "dev", "commands.json"), "utf8")
);
const cmd = n => CMDS.find(c => c.name === n);

test("kindOf: one widget type per parameter type", () => {
  assert.strictEqual(WG.kindOf({ type: "boolean" }), "toggle");
  assert.strictEqual(WG.kindOf({ type: "integer", format: "uint16", minimum: 0 }), "spin");
  assert.strictEqual(WG.kindOf({ type: "number", format: "double" }), "num");
  assert.strictEqual(WG.kindOf({ type: "number", minimum: 0, maximum: 1 }), "slider");
  assert.strictEqual(WG.kindOf({ type: "string" }), "text");
  assert.strictEqual(WG.kindOf({ type: "string", enum: ["linear", "hold"] }), "select");
  assert.strictEqual(WG.kindOf({ type: "object" }), "json");
  assert.strictEqual(WG.kindOf({ type: "array" }), "json");
  assert.strictEqual(WG.kindOf({}), "json", "no type = raw JSON");
  // Option<T> in Rust becomes ["T","null"]: the null does not change the widget
  assert.strictEqual(WG.kindOf({ type: ["integer", "null"] }), "spin");
  assert.strictEqual(WG.kindOf({ type: ["string", "null"] }), "text");
  // an enum with no declared type is still a select
  assert.strictEqual(WG.kindOf({ enum: ["a", "b"] }), "select");
});

test("kindOfCommand: a command with no parameter is a trigger", () => {
  assert.strictEqual(WG.kindOfCommand(cmd("stop")), "button");
  assert.strictEqual(WG.kindOfCommand(cmd("transport_state")), "button");
  assert.strictEqual(WG.kindOfCommand(cmd("locate")), "form");
  assert.strictEqual(WG.kindOfCommand(cmd("key_set")), "form");
});

test("kindOf over the real commands of the registry", () => {
  const p = n => cmd(n).params.properties;
  assert.strictEqual(WG.kindOf(p("locate").t), "num");
  assert.strictEqual(WG.kindOf(p("load").file), "text");
  assert.strictEqual(WG.kindOf(p("cue_go").index), "spin", "Option<usize>");
  assert.strictEqual(WG.kindOf(p("show_get").full), "toggle");
  assert.strictEqual(WG.kindOf(p("play_show").loop), "toggle");
  assert.strictEqual(WG.kindOf(p("key_set").curve), "text");
});

test("coerce: the text of the field becomes the type of the schema", () => {
  assert.strictEqual(WG.coerce("num", "12.5"), 12.5);
  assert.strictEqual(WG.coerce("spin", "3.7"), 4);
  assert.strictEqual(WG.coerce("spin", ""), 0);
  assert.strictEqual(WG.coerce("toggle", "true"), true);
  assert.strictEqual(WG.coerce("toggle", false), false);
  assert.strictEqual(WG.coerce("text", 5), "5");
  assert.deepStrictEqual(WG.coerce("json", '{"a":1}'), { a: 1 });
  assert.strictEqual(WG.coerce("json", "play"), "play", "text that is not JSON goes as text");
  assert.deepStrictEqual(WG.coerce("json", "[255,0,0]"), [255, 0, 0]);
});

test("args: it builds the request of the command out of the fields", () => {
  assert.deepStrictEqual(WG.args(cmd("locate"), { t: "12.5" }), { t: 12.5 });
  assert.deepStrictEqual(WG.args(cmd("stop"), {}), {}, "a trigger has no argument");
  // an empty optional does not go: what decides the default is the `#[serde(default)]` in Rust
  assert.deepStrictEqual(WG.args(cmd("cue_go"), { index: "" }), {});
  assert.deepStrictEqual(WG.args(cmd("cue_go"), { index: "2" }), { index: 2 });
  assert.deepStrictEqual(WG.args(cmd("show_get"), { file: "", full: true }), { full: true });
  // an empty required field goes as the default of the type (the engine complains, not the page)
  assert.deepStrictEqual(WG.args(cmd("load"), {}), { file: "" });
  const ks = WG.args(cmd("key_set"), { track: "1", t: "2.5", value: "255", curve: "hold" });
  assert.deepStrictEqual(ks, { track: 1, t: 2.5, value: 255, curve: "hold" });
  // a field the schema does not declare does not enter the request
  assert.deepStrictEqual(WG.args(cmd("locate"), { t: "1", made_up: "x" }), { t: 1 });
});
