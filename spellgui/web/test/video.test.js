"use strict";
// node --test spellgui/web/test/
// The table of the VIDEO menu (`video.js`), which is the part of it that runs without WebGL.
// What is tested is the invariant of the table, not three.js:
//   every option has a valid value in the four presets
//   `set` out of range (or off the list) is REFUSED, and the old value stays
//   `preset` followed by `set` with a different value becomes "custom"
//   `load` of broken JSON falls back to the default, without throwing

const { test } = require("node:test");
const assert = require("node:assert");

// fake localStorage: the module writes and reads through here, and the test touches it directly
const LS = { v: {}, getItem(k) { return k in this.v ? this.v[k] : null; }, setItem(k, s) { this.v[k] = String(s); }, removeItem(k) { delete this.v[k]; } };
global.window = global.window || {};
global.localStorage = LS;
const VIDEO = require("../laser3d/video.js");

test("the table: unique id, known group, and the four presets valid", () => {
  const seen = new Set();
  for (const o of VIDEO.OPTS) {
    assert.ok(!seen.has(o.id), "repeated id: " + o.id); seen.add(o.id);
    assert.ok(VIDEO.GROUPS.indexOf(o.g) >= 0, o.id + ": unknown group " + o.g);
    assert.ok(o.label && o.label.length, o.id + ": no label");
    for (const p of VIDEO.PRESETS) {
      const v = o.p[p];
      assert.notStrictEqual(v, undefined, o.id + ": no value in preset " + p);
      VIDEO.preset(p);
      assert.strictEqual(String(VIDEO.get(o.id)), String(v), o.id + " in " + p);
    }
    if (o.type === "range") { assert.ok(o.min < o.max, o.id + ": inverted range"); assert.ok(o.step > 0, o.id + ": zero step"); }
    else assert.ok(o.vals.length >= 2, o.id + ": select with fewer than two options");
  }
});

test("every preset exists and identifies itself by its own name", () => {
  for (const p of VIDEO.PRESETS) {
    assert.strictEqual(VIDEO.preset(p), true);
    assert.strictEqual(VIDEO.presetName(), p, "applied " + p + " and the name did not match");
  }
  assert.strictEqual(VIDEO.preset("epic"), false, "a preset that does not exist cannot be applied");
});

test("set out of range is refused and the old value stays", () => {
  VIDEO.preset("high");
  const fov = VIDEO.get("fov");
  for (const bad of [29, 91, 1e9, -1, NaN, Infinity, "plenty", null, undefined, true, {}]) {
    assert.strictEqual(VIDEO.set("fov", bad), false, "accepted fov=" + String(bad));
    assert.strictEqual(VIDEO.get("fov"), fov, "fov changed with the refused value " + String(bad));
  }
  assert.strictEqual(VIDEO.set("fov", 30), true, "30 is in range");
  assert.strictEqual(VIDEO.get("fov"), 30);

  VIDEO.preset("high");
  const sh = VIDEO.get("shadows");
  for (const bad of ["3072", 3072, "yes", "", "0.0"]) {
    assert.strictEqual(VIDEO.set("shadows", bad), false, "accepted shadows=" + String(bad));
    assert.strictEqual(VIDEO.get("shadows"), sh);
  }
  assert.strictEqual(VIDEO.set("shadows", 4096), true, "4096 is on the list");
  assert.strictEqual(VIDEO.set("doesnotexist", 1), false, "an id that does not exist does not get in");
});

test("preset followed by set becomes custom, and undoing it by hand brings the name back", () => {
  VIDEO.preset("high");
  assert.strictEqual(VIDEO.presetName(), "high");
  const before = VIDEO.get("shadows");
  assert.strictEqual(VIDEO.set("shadows", "0"), true);
  assert.strictEqual(VIDEO.presetName(), "custom", "touched a row and it stayed 'high'");
  VIDEO.set("shadows", before);
  assert.strictEqual(VIDEO.presetName(), "high", "put the value back by hand and the name did not come back");
});

test("load of broken JSON falls back to the default, and a bad row does not take the good ones down", () => {
  for (const junk of ["{", "", "null", "[1,2,3]", '"text"', "{oops}"]) {
    LS.v["sc-laser-video"] = junk;
    assert.doesNotThrow(() => VIDEO.load(), "load threw with " + JSON.stringify(junk));
    if (junk !== "") assert.strictEqual(VIDEO.presetName(), VIDEO.defaultPreset(), "JSON " + JSON.stringify(junk) + " did not fall back to the default");
  }
  // half good, half junk: the good one gets in, the bad one keeps the default
  VIDEO.preset("high");
  const defaultShadow = VIDEO.get("shadows");
  LS.v["sc-laser-video"] = JSON.stringify({ shadows: "9999", fov: 61, scale: "0.5", bloomStrength: "no" });
  VIDEO.load();
  assert.strictEqual(VIDEO.get("fov"), 61, "the good row did not get in");
  assert.strictEqual(VIDEO.get("shadows"), defaultShadow, "the bad row got in");
  assert.strictEqual(VIDEO.get("bloomStrength"), VIDEO.opt("bloomStrength").p[VIDEO.defaultPreset()], "a range with text got in");
  delete LS.v["sc-laser-video"];
});

test("onChange receives the id that changed, and null when it was the whole preset", () => {
  const seen = [];
  VIDEO.onChange(id => seen.push(id));
  VIDEO.preset("ultra");
  VIDEO.set("haze", .1);
  VIDEO.set("haze", .1);            // same value: it does not warn again
  VIDEO.set("haze", 99);            // refused: it does not warn
  VIDEO.onChange(null);
  assert.deepStrictEqual(seen, [null, "haze"]);
});
