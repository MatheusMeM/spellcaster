"use strict";
// node --test spellgui/web/test/rope.test.js
// A corda de Verlet do cabo do Pino roda sem WebGL: e' aritmetica pura sobre um Float32Array.

const { test } = require("node:test");
const assert = require("node:assert");

global.window = {};
const Rope = require("../laser3d/rope.js");

const N = 24, REST = .02, DT = 1 / 60, G = -9.8, IT = 3, SUB = 4;  // o que o app usa
function seg(P, i) { const a = i * 6, b = a + 6;
  return Math.hypot(P[b] - P[a], P[b + 1] - P[a + 1], P[b + 2] - P[a + 2]); }

test("Rope.make: n nos numa reta entre as duas pontas, sem velocidade", () => {
  const P = Rope.make(N, [0, 0, 0], [.23, 0, 0]);
  assert.strictEqual(P.length, N * 6);
  assert.strictEqual(P[0], 0); assert.ok(Math.abs(P[(N - 1) * 6] - .23) < 1e-6);
  for (let i = 0; i < N; i++) assert.strictEqual(P[i * 6], P[i * 6 + 3], "posicao anterior = posicao: parte parada");
  assert.ok(Math.abs(seg(P, 0) - .23 / (N - 1)) < 1e-6, "nos igualmente espacados");
});

test("Rope.step: as duas pontas nao saem do lugar", () => {
  const A = [0, .3, 0], B = [.23, .34, .17];
  const P = Rope.make(N, A, B);
  for (let k = 0; k < 600; k++) Rope.step(P, REST, DT, G, IT, SUB);
  const last = (N - 1) * 6;
  assert.ok(Math.abs(P[0] - A[0]) < 1e-6 && Math.abs(P[1] - A[1]) < 1e-6 && Math.abs(P[2] - A[2]) < 1e-6, "ponta do Pino presa");
  assert.ok(Math.abs(P[last] - B[0]) < 1e-6 && Math.abs(P[last + 1] - B[1]) < 1e-6 && Math.abs(P[last + 2] - B[2]) < 1e-6, "ponta do DMX OUT presa");
});

test("Rope.step: o comprimento converge para o repouso (catenaria, nao elastico)", () => {
  // pontas a 0,25 m e 23 segmentos de 0,02 m = 0,46 m de cabo: sobra corda, entao ela pendura
  const P = Rope.make(N, [0, .3, 0], [.25, .3, 0]);
  for (let k = 0; k < 1200; k++) Rope.step(P, REST, DT, G, IT, SUB);
  let worst = 0, lowest = 1;
  for (let i = 0; i < N - 1; i++) { worst = Math.max(worst, Math.abs(seg(P, i) - REST) / REST); lowest = Math.min(lowest, P[i * 6 + 1]); }
  assert.ok(worst < .05, "todo segmento a menos de 5% do repouso, deu " + (worst * 100).toFixed(1) + "%");
  assert.ok(lowest < .28, "a gravidade fez barriga, o ponto mais baixo foi " + lowest.toFixed(3));
});

test("Rope.step: nao gera NaN com as pontas no mesmo ponto", () => {
  const P = Rope.make(N, [0, .3, 0], [0, .3, 0]);
  for (let k = 0; k < 300; k++) Rope.step(P, REST, DT, G, IT, SUB);
  for (let i = 0; i < P.length; i++) assert.ok(Number.isFinite(P[i]), "no " + i + " virou " + P[i]);
});

test("Rope.pin: mover a ponta arrasta a corda no passo seguinte", () => {
  const P = Rope.make(N, [0, .3, 0], [.25, .3, 0]);
  for (let k = 0; k < 300; k++) Rope.step(P, REST, DT, G, IT, SUB);
  const meio = P[12 * 6];
  for (let k = 0; k < 120; k++) { Rope.pin(P, 0, -.4, .3, 0); Rope.step(P, REST, DT, G, IT, SUB); }
  assert.ok(P[12 * 6] < meio, "a ponta andou para -x e o meio da corda foi junto");
});
