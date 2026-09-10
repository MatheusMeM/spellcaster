/* Pure Verlet rope, no three.js and no DOM: the DMX cable of the Pino.
   The nodes live in a Float32Array [x, y, z, px, py, pz] per node (px = position of the previous step —
   in Verlet the velocity is the difference between the two, not a stored vector). Both ends are
   fixed: the caller writes them with `pin` before every `step`, and the constraint never moves them.
   ponytail: distance between nodes is the only constraint (no bending stiffness, no collision); if the
   cable ever has to hit the chassis, a collision pass goes in after the constraints. */
window.Rope = (function () {
  "use strict";
  function make(n, a, b) { var P = new Float32Array(n * 6), i, o, t;
    for (i = 0; i < n; i++) { o = i * 6; t = i / (n - 1);
      P[o] = P[o + 3] = a[0] + (b[0] - a[0]) * t;
      P[o + 1] = P[o + 4] = a[1] + (b[1] - a[1]) * t;
      P[o + 2] = P[o + 5] = a[2] + (b[2] - a[2]) * t; }
    return P; }
  function pin(P, i, x, y, z) { var o = i * 6; P[o] = P[o + 3] = x; P[o + 1] = P[o + 4] = y; P[o + 2] = P[o + 5] = z; }
  /* dt comes in fixed (the caller clamps it at 1/60): Verlet with a variable dt changes the energy of
     the system and the rope "explodes" on the first long frame. `damp` < 1 is the air drag; without it
     it never comes to rest. `sub` splits the frame into smaller steps, and that is what makes the rope
     inextensible: iterating the constraint more times inside the same step converges slowly on a
     24-node pendulum (the segment next to the pinned end carries the weight of all the others).
     Measured, a 0.46 m rope between ends 0.25 m apart, worst length error after 2000 frames:
     1x3 = 45%, 1x20 = 5.4%, 1x40 = 2.2%, 4x3 = 2.9% and costs half of 1x20. Hence the app runs
     sub 4, iters 3. */
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
