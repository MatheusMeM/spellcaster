// The pure function of the stage: DMX frame + patch + profile -> color and intensity of each fixture.
// node --test spellgui/web/test/teatro.test.js
"use strict";
const test = require("node:test");
const assert = require("node:assert");
const T = require("../teatro.js");

const par = {
  name: "PAR LED RGB 3ch",
  channels: [{ name: "r", offset: 0 }, { name: "g", offset: 1 }, { name: "b", offset: 2 }],
};
const dimmer = { name: "Dimmer 1ch", channels: [{ name: "dim", offset: 0 }] };
const wash = {
  name: "Wash 4ch",
  channels: [{ name: "r", offset: 0 }, { name: "g", offset: 1 },
             { name: "b", offset: 2 }, { name: "w", offset: 3 }],
};
const bsw = {
  name: "BSW 17ch",
  channels: [{ name: "pan", offset: 0, fine: 1 }, { name: "dim", offset: 9 },
             { name: "color", offset: 4, wheel: { branco: 0 } }],
};
const profiles = { par_rgb_3: par, dimmer_1: dimmer, wash_bx402_4: wash, bsw_scorpio_17: bsw };

// universe with values at 1-based addresses
function uni(pairs) {
  const a = new Array(512).fill(0);
  for (const [addr, v] of pairs) a[addr - 1] = v;
  return a;
}

test("color and intensity by profile type", () => {
  const patch = [
    { name: "par 1", profile: "par_rgb_3", universe: 1, address: 1 },
    { name: "footlight", profile: "wash_bx402_4", universe: 1, address: 10 },
    { name: "fresnel", profile: "dimmer_1", universe: 1, address: 20 },
    { name: "mover", profile: "bsw_scorpio_17", universe: 2, address: 1 },
  ];
  const dmx = {
    1: uni([[1, 255], [2, 0], [3, 0], [10, 10], [11, 20], [12, 30], [13, 200], [20, 128]]),
    2: uni([[1, 77], [10, 64]]),
  };
  const [p, w, f, m] = T.look(patch, profiles, dmx);

  assert.deepStrictEqual(p.rgb, [255, 0, 0], "pure RGB");
  assert.strictEqual(p.intensity, 1);
  assert.deepStrictEqual(p.channels, { r: 255, g: 0, b: 0 });

  // the w adds into the three channels and saturates at 255
  assert.deepStrictEqual(w.rgb, [210, 220, 230]);
  assert.strictEqual(w.intensity, 200 / 255, "with no dimmer, the brightest color channel rules");

  // a dimmer has no color: a white lamp with the intensity of the channel
  assert.deepStrictEqual(f.rgb, [255, 255, 255]);
  assert.strictEqual(f.intensity, 128 / 255);

  // universe 2, dimmer offset in the middle of the profile; an unnamed channel stays out
  assert.strictEqual(m.universe, 2);
  assert.strictEqual(m.channels.pan, 77);
  assert.strictEqual(m.intensity, 64 / 255, "dim rules even with the color channels missing");
  assert.deepStrictEqual(m.rgb, [255, 255, 255]);
});

test("white saturation and a missing frame", () => {
  const patch = [{ name: "w", profile: "wash_bx402_4", universe: 1, address: 1 }];
  const full = T.look(patch, profiles, { 1: uni([[1, 200], [4, 200]]) })[0];
  assert.deepStrictEqual(full.rgb, [255, 200, 200], "r + w saturates at 255");

  // a universe that has not arrived over the bus yet: everything dark, nothing breaks
  const dark = T.look(patch, profiles, {})[0];
  assert.deepStrictEqual(dark.rgb, [0, 0, 0]);
  assert.strictEqual(dark.intensity, 0);
});

test("a missing profile and an address past the universe do not break", () => {
  const patch = [
    { name: "no profile", profile: "does_not_exist", universe: 1, address: 1 },
    { name: "at the end", profile: "par_rgb_3", universe: 1, address: 511 },
  ];
  const [a, b] = T.look(patch, profiles, { 1: uni([[511, 90], [512, 91]]) });
  assert.deepStrictEqual(a.channels, {});
  assert.strictEqual(a.intensity, 0);
  assert.deepStrictEqual(a.rgb, [255, 255, 255], "no profile: white, dark");
  assert.deepStrictEqual(b.rgb, [90, 91, 0], "channel 513 does not exist and reads 0");
});

test("the default position spreads them in a row and a saved one rules", () => {
  const patch = [
    { name: "a", profile: "dimmer_1", universe: 1, address: 1 },
    { name: "b", profile: "dimmer_1", universe: 1, address: 2, pos: [0.5, 0.9] },
  ];
  const fs = T.look(patch, profiles, {});
  const [x0] = fs[0].pos;
  assert.ok(x0 > 0 && x0 < 1, "default position inside the screen: " + x0);
  assert.deepStrictEqual(fs[1].pos, [0.5, 0.9], "the saved position rules");
});
