// Args of the MIDI map: the parse of the field and the preview of the "$", which has to match the
// `expande` of the engine (spellcore/engine/src/midi.rs).
// node --test spellgui/web/test/midi.test.js
"use strict";
const test = require("node:test");
const assert = require("node:assert");
const M = require("../midi.js");

test("parseArgs takes an object and empty, refuses the rest", () => {
  assert.deepEqual(M.parseArgs(""), {});
  assert.deepEqual(M.parseArgs("   "), {});
  assert.deepEqual(M.parseArgs('{"address":1,"values":["$255"]}'), {
    address: 1,
    values: ["$255"],
  });
  assert.throws(() => M.parseArgs("{"), /JSON|Unexpected/);
  assert.throws(() => M.parseArgs("[1,2]"), /JSON object/);
  assert.throws(() => M.parseArgs("3"), /JSON object/);
});

test("preview replaces $ the way the engine does", () => {
  const v = 100 / 127;
  assert.deepEqual(M.preview({ address: 1, values: ["$255"], u: "$", n: "$127" }, v), {
    address: 1,
    values: [201],
    u: v,
    n: 100,
  });
  // ends
  assert.equal(M.preview("$255", 0), 0);
  assert.equal(M.preview("$255", 1), 255);
  assert.equal(M.preview("$127", 1), 127);
  // text that is not a dollar sign stays as it is
  assert.equal(M.preview("$x", 0.5), "$x");
  assert.equal(M.preview("go", 0.5), "go");
  assert.equal(M.preview(true, 0.5), true);
  assert.deepEqual(M.preview(["$2", { a: "$4" }], 0.5), [1, { a: 2 }]);
});
