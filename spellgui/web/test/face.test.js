"use strict";
// node --test spellgui/web/test/
// A regra do face.js: escolher a view, ordenar os widgets dela e traduzir um toque em chamada
// do barramento. O desenho e' DOM e nao tem regra.

const { test } = require("node:test");
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");
const Face = require("../face.js");

const QUATRO = JSON.parse(
  fs.readFileSync(path.join(__dirname, "..", "..", "..", "faces", "quatro.face.json"), "utf8")
);

test("pick: view pedida, primeira declarada, ou sintetica", () => {
  assert.strictEqual(Face.pick(QUATRO, "compact").name, "compact");
  assert.strictEqual(Face.pick(QUATRO, "nao_existe").name, "full");
  assert.strictEqual(Face.pick(QUATRO).name, "full");
  const v = Face.pick({ widgets: [{ id: "a" }, { id: "b" }] });
  assert.deepStrictEqual(v.widgets, ["a", "b"]);
});

test("widgets: a view manda na ordem e no conjunto", () => {
  const ids = v => Face.widgets(QUATRO, Face.pick(QUATRO, v)).map(w => w.id);
  assert.deepStrictEqual(ids("full"), ["go", "next", "stop", "blackout"]);
  assert.deepStrictEqual(ids("compact"), ["go", "blackout"]);
  // id que nao existe na lista de widgets nao vira um buraco
  assert.deepStrictEqual(
    Face.widgets(QUATRO, { widgets: ["go", "fantasma"] }).map(w => w.id),
    ["go"]
  );
});

test("action: cmd vai ao registry, input vai ao graph", () => {
  const w = id => QUATRO.widgets.find(x => x.id === id);
  assert.deepStrictEqual(Face.action(w("go")), { cmd: "cue_go", args: {} });
  assert.deepStrictEqual(Face.action(w("stop")), { cmd: "stop", args: {} });
  assert.deepStrictEqual(Face.action(w("blackout")), { input: "widget:blackout", value: 1 });
  assert.deepStrictEqual(Face.action(w("blackout"), 0), { input: "widget:blackout", value: 0 });
  assert.deepStrictEqual(Face.action({ id: "f", input: "widget:f" }, "0.5"), {
    input: "widget:f",
    value: 0.5,
  });
  assert.strictEqual(Face.action({ id: "so_label" }), null);
  // input tem precedencia: um widget nao faz as duas coisas
  assert.deepStrictEqual(Face.action({ input: "widget:x", cmd: "stop" }), {
    input: "widget:x",
    value: 1,
  });
});

test("grid: 'CxL' e o default", () => {
  assert.deepStrictEqual(Face.grid("4x2"), [4, 2]);
  assert.deepStrictEqual(Face.grid("12 X 8"), [12, 8]);
  assert.deepStrictEqual(Face.grid("torto"), [4, 2]);
  assert.deepStrictEqual(Face.grid(undefined), [4, 2]);
});

test("quatro.face.json: os quatro botoes do show-alvo cabem na grade", () => {
  const [cols, rows] = Face.grid(QUATRO.views.full.grid);
  for (const w of QUATRO.widgets) {
    const [c, r, dc, dr] = w.at;
    assert.ok(c + dc <= cols && r + dr <= rows, w.id + " sai da grade");
  }
  assert.strictEqual(QUATRO.mode, "performance");
});
