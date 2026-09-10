"use strict";
// node --test spellgui/web/test/rope.test.js
// The Verlet rope of the Pino cable runs without WebGL: it is pure arithmetic over a Float32Array.

const { test } = require("node:test");
const assert = require("node:assert");

global.window = {};
const Rope = require("../laser3d/rope.js");

const N = 24, REST = .02, DT = 1 / 60, G = -9.8, IT = 3, SUB = 4;  // what the app uses
function seg(P, i) { const a = i * 6, b = a + 6;
  return Math.hypot(P[b] - P[a], P[b + 1] - P[a + 1], P[b + 2] - P[a + 2]); }

test("Rope.make: n nodes on a straight line between the two ends, with no velocity", () => {
  const P = Rope.make(N, [0, 0, 0], [.23, 0, 0]);
  assert.strictEqual(P.length, N * 6);
  assert.strictEqual(P[0], 0); assert.ok(Math.abs(P[(N - 1) * 6] - .23) < 1e-6);
  for (let i = 0; i < N; i++) assert.strictEqual(P[i * 6], P[i * 6 + 3], "previous position = position: it starts at rest");
  assert.ok(Math.abs(seg(P, 0) - .23 / (N - 1)) < 1e-6, "evenly spaced nodes");
});

test("Rope.step: the two ends do not move", () => {
  const A = [0, .3, 0], B = [.23, .34, .17];
  const P = Rope.make(N, A, B);
  for (let k = 0; k < 600; k++) Rope.step(P, REST, DT, G, IT, SUB);
  const last = (N - 1) * 6;
  assert.ok(Math.abs(P[0] - A[0]) < 1e-6 && Math.abs(P[1] - A[1]) < 1e-6 && Math.abs(P[2] - A[2]) < 1e-6, "the Pino end stays pinned");
  assert.ok(Math.abs(P[last] - B[0]) < 1e-6 && Math.abs(P[last + 1] - B[1]) < 1e-6 && Math.abs(P[last + 2] - B[2]) < 1e-6, "the DMX OUT end stays pinned");
});

test("Rope.step: the length converges to the rest length (catenary, not elastic)", () => {
  // ends 0.25 m apart and 23 segments of 0.02 m = 0.46 m of cable: there is rope to spare, so it hangs
  const P = Rope.make(N, [0, .3, 0], [.25, .3, 0]);
  for (let k = 0; k < 1200; k++) Rope.step(P, REST, DT, G, IT, SUB);
  let worst = 0, lowest = 1;
  for (let i = 0; i < N - 1; i++) { worst = Math.max(worst, Math.abs(seg(P, i) - REST) / REST); lowest = Math.min(lowest, P[i * 6 + 1]); }
  assert.ok(worst < .05, "every segment within 5% of the rest length, got " + (worst * 100).toFixed(1) + "%");
  assert.ok(lowest < .28, "gravity made it sag, the lowest point was " + lowest.toFixed(3));
});

test("Rope.step: it does not produce NaN with both ends at the same point", () => {
  const P = Rope.make(N, [0, .3, 0], [0, .3, 0]);
  for (let k = 0; k < 300; k++) Rope.step(P, REST, DT, G, IT, SUB);
  for (let i = 0; i < P.length; i++) assert.ok(Number.isFinite(P[i]), "node " + i + " became " + P[i]);
});

test("Rope.pin: moving the end drags the rope on the next step", () => {
  const P = Rope.make(N, [0, .3, 0], [.25, .3, 0]);
  for (let k = 0; k < 300; k++) Rope.step(P, REST, DT, G, IT, SUB);
  const middle = P[12 * 6];
  for (let k = 0; k < 120; k++) { Rope.pin(P, 0, -.4, .3, 0); Rope.step(P, REST, DT, G, IT, SUB); }
  assert.ok(P[12 * 6] < middle, "the end moved towards -x and the middle of the rope followed");
});
