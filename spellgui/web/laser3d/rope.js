/* Corda de Verlet pura, sem three.js e sem DOM: o cabo DMX do Pino.
   Os nós vivem num Float32Array [x, y, z, px, py, pz] por nó (px = posição do passo anterior — em
   Verlet a velocidade é a diferença entre as duas, não um vetor guardado). As duas pontas são
   fixas: quem chama escreve nelas com `pin` antes de cada `step`, e a restrição nunca as move.
   ponytail: distância entre nós é a única restrição (sem rigidez à flexão, sem colisão); se o cabo
   precisar bater no chassi um dia, entra uma passada de colisão depois das restrições. */
window.Rope = (function () {
  "use strict";
  function make(n, a, b) { var P = new Float32Array(n * 6), i, o, t;
    for (i = 0; i < n; i++) { o = i * 6; t = i / (n - 1);
      P[o] = P[o + 3] = a[0] + (b[0] - a[0]) * t;
      P[o + 1] = P[o + 4] = a[1] + (b[1] - a[1]) * t;
      P[o + 2] = P[o + 5] = a[2] + (b[2] - a[2]) * t; }
    return P; }
  function pin(P, i, x, y, z) { var o = i * 6; P[o] = P[o + 3] = x; P[o + 1] = P[o + 4] = y; P[o + 2] = P[o + 5] = z; }
  /* dt entra fixo (o chamador prende em 1/60): Verlet com dt variável muda a energia do sistema e a
     corda "explode" no primeiro quadro longo. `damp` < 1 é o arrasto do ar; sem ele ela nunca para.
     `sub` divide o quadro em passos menores, e é ele que dá a corda inextensível: iterar a restrição
     mais vezes no mesmo passo converge devagar num pêndulo de 24 nós (o segmento junto da ponta
     presa carrega o peso de todos os outros). Medido, corda de 0,46 m entre pontas a 0,25 m, erro
     máximo de comprimento depois de 2000 quadros: 1×3 = 45%, 1×20 = 5,4%, 1×40 = 2,2%,
     4×3 = 2,9% e custa metade de 1×20. Por isso o app roda sub 4, iters 3. */
  function step(P, rest, dt, g, iters, sub) { var n = P.length / 6, s;
    for (s = (sub = sub || 1); s > 0; s--) step1(P, n, rest, dt / sub, g, iters);
    return P; }
  function step1(P, n, rest, dt, g, iters) { var damp = .96, i, k, o, dx, dy, dz, L, c, wa, wb, ma, mb;
    for (i = 1; i < n - 1; i++) { o = i * 6;
      dx = (P[o] - P[o + 3]) * damp; dy = (P[o + 1] - P[o + 4]) * damp; dz = (P[o + 2] - P[o + 5]) * damp;
      P[o + 3] = P[o]; P[o + 4] = P[o + 1]; P[o + 5] = P[o + 2];
      P[o] += dx; P[o + 1] += dy + g * dt * dt; P[o + 2] += dz; }
    for (k = 0; k < iters; k++) for (i = 0; i < n - 1; i++) { o = i * 6;
      ma = i > 0 ? 1 : 0; mb = i + 1 < n - 1 ? 1 : 0; if (!ma && !mb) continue;
      dx = P[o + 6] - P[o]; dy = P[o + 7] - P[o + 1]; dz = P[o + 8] - P[o + 2];
      L = Math.sqrt(dx * dx + dy * dy + dz * dz) || 1e-9; c = (L - rest) / L;
      wa = ma / (ma + mb) * c; wb = mb / (ma + mb) * c;
      if (ma) { P[o] += dx * wa; P[o + 1] += dy * wa; P[o + 2] += dz * wa; }
      if (mb) { P[o + 6] -= dx * wb; P[o + 7] -= dy * wb; P[o + 8] -= dz * wb; } } }
  return { make: make, pin: pin, step: step };
})();
if (typeof module !== "undefined") module.exports = window.Rope;
