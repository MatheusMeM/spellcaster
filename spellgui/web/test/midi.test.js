// Args do mapa MIDI: o parse do campo e a previa do "$", que tem que casar com o `expande` do
// engine (spellcore/engine/src/midi.rs).
// node --test spellgui/web/test/midi.test.js
"use strict";
const test = require("node:test");
const assert = require("node:assert");
const M = require("../midi.js");

test("parseArgs aceita objeto e vazio, recusa o resto", () => {
  assert.deepEqual(M.parseArgs(""), {});
  assert.deepEqual(M.parseArgs("   "), {});
  assert.deepEqual(M.parseArgs('{"address":1,"values":["$255"]}'), {
    address: 1,
    values: ["$255"],
  });
  assert.throws(() => M.parseArgs("{"), /JSON|Unexpected/);
  assert.throws(() => M.parseArgs("[1,2]"), /objeto JSON/);
  assert.throws(() => M.parseArgs("3"), /objeto JSON/);
});

test("preview substitui $ como o engine", () => {
  const v = 100 / 127;
  assert.deepEqual(M.preview({ address: 1, values: ["$255"], u: "$", n: "$127" }, v), {
    address: 1,
    values: [201],
    u: v,
    n: 100,
  });
  // pontas
  assert.equal(M.preview("$255", 0), 0);
  assert.equal(M.preview("$255", 1), 255);
  assert.equal(M.preview("$127", 1), 127);
  // texto que nao e' cifrao fica como esta'
  assert.equal(M.preview("$x", 0.5), "$x");
  assert.equal(M.preview("go", 0.5), "go");
  assert.equal(M.preview(true, 0.5), true);
  assert.deepEqual(M.preview(["$2", { a: "$4" }], 0.5), [1, { a: 2 }]);
});
