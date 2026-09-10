"use strict";
// node --test spellgui/web/test/viewer.test.js
// Covers what the previz has of logic: the interpolation of `scale`/`rot` from the track keys, the
// choice of the frame by `t` and `fps`, the cache per (clip, frame) and the fixture color from the
// profile. viewer.js is a browser script and depends on `window.TL`: load timeline.js first, with a
// fake `window` and `CK`, and no DOM.

const { test } = require("node:test");
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");

function load() {
  const win = {};
  const CK = {
    clamp: (v, a, b) => (v < a ? a : v > b ? b : v),
    bisect: () => 0,
    near: () => -1,
    sel: () => ({}),
  };
  const TEATRO = require("../teatro.js");           // pure part: the fixture color comes from it
  for (const f of ["timeline.js", "viewer.js"]) {
    const src = fs.readFileSync(path.join(__dirname, "..", f), "utf8");
    new Function("window", "CK", "TEATRO", src)(win, CK, TEATRO);
  }
  return win;
}

const win = load();
const VW = win.VW;

// ---- scale/rot interpolation -------------------------------------------

test("param with no keys returns the default", () => {
  assert.strictEqual(VW.param(undefined, 3, 1), 1);
  assert.strictEqual(VW.param([], 3, 0), 0);
});

test("param holds the first and the last value outside the range", () => {
  const k = [[1, 10], [3, 30]];
  assert.strictEqual(VW.param(k, 0, 0), 10);
  assert.strictEqual(VW.param(k, 9, 0), 30);
});

test("param interpolates linearly between two keyframes", () => {
  assert.strictEqual(VW.param([[0, 0], [2, 90]], 1, 0), 45);
  assert.strictEqual(VW.param([[0, 1], [4, 2]], 3, 1), 1.75);
});

// The curve applies to the segment that ARRIVES at the keyframe: `hold` holds the previous value
// to the end.
test("param respects the curve of the arriving keyframe", () => {
  assert.strictEqual(VW.param([[0, 0], [2, 100, "hold"]], 1.9, 0), 0);
  assert.strictEqual(VW.param([[0, 0], [2, 100, "in"]], 1, 0), 25);
  assert.strictEqual(VW.param([[0, 0], [2, 100, "out"]], 1, 0), 75);
});

test("param with three keys takes the right segment", () => {
  const k = [[0, 0], [1, 10], [2, 0]];
  assert.strictEqual(VW.param(k, 0.5, 0), 5);
  assert.strictEqual(VW.param(k, 1.5, 0), 5);
  assert.strictEqual(VW.param(k, 1, 0), 10);
});

// ---- frame by t and fps -------------------------------------------------

test("index is floor(t * fps) while the frame total has not arrived", () => {
  VW.n.clear();
  assert.strictEqual(VW.index("x.ild", 0, 30), 0);
  assert.strictEqual(VW.index("x.ild", 1, 30), 30);
  assert.strictEqual(VW.index("x.ild", 0.5, 10), 5);
  assert.strictEqual(VW.index("x.ild", -1, 30), 0);
  assert.strictEqual(VW.index("x.ild", 1, 0), 30, "fps zero falls back to the default 30");
});

test("with the total known, the clip repeats", () => {
  VW.n.set("x.ild", 4);
  assert.strictEqual(VW.index("x.ild", 1, 30), 2);
  assert.strictEqual(VW.index("x.ild", 0.1, 30), 3);
  VW.n.clear();
});

// ---- cache per (clip, frame) --------------------------------------------

test("the cache asks once per frame and returns the points when they arrive", async () => {
  const asked = [];
  let release;
  VW.clip.clear(); VW.n.clear();
  VW.call = (cmd, args) => {
    asked.push([cmd, args]);
    return new Promise(ok => { release = ok; });
  };
  assert.strictEqual(VW.frame("a.ild", 7), null, "the first request has no points yet");
  assert.strictEqual(VW.frame("a.ild", 7), null, "the second draw does not ask again");
  assert.strictEqual(asked.length, 1);
  assert.deepStrictEqual(asked[0], ["clip_frame", { clip: "a.ild", index: 7 }]);

  release({ frames: 12, points: [[0, 0, 255, 0, 0, 0]] });
  await Promise.resolve();
  assert.deepStrictEqual(VW.frame("a.ild", 7), [[0, 0, 255, 0, 0, 0]]);
  assert.strictEqual(asked.length, 1, "a cached frame does not ask again");
  assert.strictEqual(VW.n.get("a.ild"), 12, "the frame total came back with the response");
  assert.strictEqual(VW.frame("a.ild", 8), null, "another frame is another request");
  assert.strictEqual(asked.length, 2);
});

test("an engine error stays cached: the drawing does not become a flood of requests", async () => {
  let n = 0;
  VW.clip.clear();
  VW.call = () => { n++; return Promise.reject("no server"); };
  VW.frame("b.ild", 0);
  await Promise.resolve();
  await Promise.resolve();
  assert.deepStrictEqual(VW.frame("b.ild", 0), []);
  VW.frame("b.ild", 0);
  assert.strictEqual(n, 1);
});

test("the cache empties at the ceiling instead of growing without end", () => {
  VW.clip.clear();
  VW.call = () => new Promise(() => {});
  for (let i = 0; i <= VW.cap; i++) VW.frame("c.ild", i);
  assert.ok(VW.clip.size <= VW.cap, "size " + VW.clip.size);
});

// ---- fixture color from the profile ------------------------------------

test("the color comes from the r/g/b channels at the fixture address", () => {
  const ch = [{ name: "r", offset: 0 }, { name: "g", offset: 1 }, { name: "b", offset: 2 }];
  const d = new Uint8Array(512);
  d[9] = 255; d[10] = 128; d[11] = 0;                     // address 10 = index 9
  assert.deepStrictEqual(VW.color(ch, d, 10), [255, 128, 0]);
});

test("a profile with only a dimmer gives white times the dimmer", () => {
  const d = new Uint8Array(512);
  d[0] = 128;
  assert.deepStrictEqual(VW.color([{ name: "dim", offset: 0 }], d, 1), [128, 128, 128]);
});

// The color is the paper theater one: `w` joins the mix and `on` lights up, otherwise the two plans
// would show the same fixture in different colors.
test("the w channel and the on channel count, as in the theater", () => {
  const d = new Uint8Array(512);
  d[3] = 255;                                             // WLED bar in white only
  const wled = [{ name: "r", offset: 0 }, { name: "g", offset: 1 },
    { name: "b", offset: 2 }, { name: "w", offset: 3 }];
  assert.deepStrictEqual(VW.color(wled, d, 1), [255, 255, 255]);
  d[0] = 255;
  assert.deepStrictEqual(VW.color([{ name: "on", offset: 0 }], d, 1), [255, 255, 255]);
});

test("the dimmer scales the rgb, and with no color at all the fixture stays empty", () => {
  const d = new Uint8Array(512);
  d[0] = 200; d[3] = 0;
  const ch = [{ name: "r", offset: 0 }, { name: "dim", offset: 3 }];
  assert.strictEqual(VW.color(ch, d, 1), null, "dimmer at zero: dark, only the outline");
  assert.strictEqual(VW.color([{ name: "pan", offset: 0 }], d, 1), null);
  assert.strictEqual(VW.color(null, d, 1), null, "a profile that has not arrived yet");
  assert.strictEqual(VW.color([{ name: "r", offset: 0 }], null, 1), null, "no output frame");
});

// ---- dmx column: output, or input when the lane is armed ----------------

test("source: an armed lane reads the input; the other one, the output", () => {
  assert.deepStrictEqual(VW.source({ spec: { universe: 3 }, rec: true }),
    { u: 3, arm: true, lab: "dmx u3 in" });
  assert.deepStrictEqual(VW.source({ spec: { universe: 3 } }),
    { u: 3, arm: false, lab: "dmx u3 out" });
  assert.deepStrictEqual(VW.source({ spec: {} }), { u: 1, arm: false, lab: "dmx u1 out" });
  assert.deepStrictEqual(VW.source(undefined), { u: 1, arm: false, lab: "dmx u1 out" });
});
