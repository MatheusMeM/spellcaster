"use strict";
// node --test spellgui/web/test/
// The rule of face.js: pick the view, order its widgets and turn a touch into a bus call.
// The drawing is DOM and has no rule.

const { test } = require("node:test");
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");
const Face = require("../face.js");

const QUATRO = JSON.parse(
  fs.readFileSync(path.join(__dirname, "..", "..", "..", "faces", "quatro.face.json"), "utf8")
);

test("pick: the requested view, or the first one declared", () => {
  assert.strictEqual(Face.pick(QUATRO, "compact").name, "compact");
  assert.strictEqual(Face.pick(QUATRO, "does_not_exist").name, "full");
  assert.strictEqual(Face.pick(QUATRO).name, "full");
  assert.deepStrictEqual(Face.pick(QUATRO, "compact").grid, [2, 1]);
});

test("widgets: the view rules the order and the set", () => {
  const ids = v => Face.widgets(QUATRO, Face.pick(QUATRO, v)).map(w => w.id);
  assert.deepStrictEqual(ids("full"), ["go", "next", "stop", "blackout"]);
  assert.deepStrictEqual(ids("compact"), ["go", "blackout"]);
  // an id that is not in the widget list does not become a hole
  assert.deepStrictEqual(
    Face.widgets(QUATRO, { widgets: ["go", "ghost"] }).map(w => w.id),
    ["go"]
  );
});

test("action: cmd goes to the registry, input goes to the graph", () => {
  const w = id => QUATRO.widgets.find(x => x.id === id);
  assert.deepStrictEqual(Face.action(w("go")), { cmd: "cue_go", args: {} });
  assert.deepStrictEqual(Face.action(w("stop")), { cmd: "stop", args: {} });
  assert.deepStrictEqual(Face.action(w("blackout")), { input: "widget:blackout", value: 1 });
  assert.deepStrictEqual(Face.action(w("blackout"), 0), { input: "widget:blackout", value: 0 });
  assert.deepStrictEqual(Face.action({ id: "f", input: "widget:f" }, "0.5"), {
    input: "widget:f",
    value: 0.5,
  });
  assert.strictEqual(Face.action({ id: "label_only" }), null);
  // input takes precedence: one widget does not do both things
  assert.deepStrictEqual(Face.action({ input: "widget:x", cmd: "stop" }), {
    input: "widget:x",
    value: 1,
  });
});

test("quatro.face.json: the four buttons of the target show fit the grid", () => {
  const [cols, rows] = QUATRO.views.full.grid;
  for (const w of QUATRO.widgets) {
    const [c, r, dc, dr] = w.at;
    assert.ok(c + dc <= cols && r + dr <= rows, w.id + " runs off the grid");
  }
});

test("applyProp: an invalid class name off the network does not throw", () => {
  const cls = [];
  const node = { classList: { toggle: (c, on) => { if (!/^[\w-]+$/.test(c)) throw new Error("bad"); cls.push([c, on]); } } };
  Face.applyProp(node, "glow", 1);
  Face.applyProp(node, "", 1);
  Face.applyProp(node, "a b", 1);
  assert.deepStrictEqual(cls, [["glow", true]]);
});
