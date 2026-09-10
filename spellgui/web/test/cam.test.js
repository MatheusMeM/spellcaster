"use strict";
// node --test spellgui/web/test/
// The law of the laser3d camera (`SWCam.clamp`), which is the only part of it that runs without WebGL.
// Geometry is not what is tested: what is tested is the invariant of each view.
//   rear   = fixed pose: nothing the user does changes the distance
//   inside = locked orbit: yaw and pitch inside the range, distance locked
//   free   = free WITH a limit: the camera never gets closer than R to the centre of the device

const { test } = require("node:test");
const assert = require("node:assert");
global.window = global.window || {};
const SWCam = require("../laser3d/cam.js");
const clamp = SWCam.clamp;

function goal(t, d, yaw, pit) { return { t: { x: t[0], y: t[1], z: t[2] }, d: d, yaw: yaw, pit: pit, roll: .4 }; }
function pos(s) {
  return [s.t.x + s.d * Math.sin(s.yaw) * Math.cos(s.pit),
          s.t.y + s.d * Math.sin(s.pit),
          s.t.z + s.d * Math.cos(s.yaw) * Math.cos(s.pit)];
}
function far(p, c) { return Math.hypot(p[0] - c.x, p[1] - c.y, p[2] - c.z); }
// sweep: targets, distances and angles absurd on purpose
const TS = [[0, .4, 0], [0, .404, .15], [-.02, .35, -.03], [3, 2, -4], [-9, -9, 9], [.1, .41, .1]];
const DS = [0, .001, .05, .3, 1, 3.5, 40];
const AS = [-3.5, -1.6, -.35, 0, .35, 1.2, 1.6, 3.5, 8];

test("rear: the pose is fixed — no gesture changes distance, target or angle", () => {
  const env = { center: { x: 0, y: .404, z: .15 }, d0: .62, yaw0: .01, pit0: -.02 };
  for (const t of TS) for (const d of DS) for (const a of AS) {
    const g = clamp("rear", goal(t, d, a, a), env);
    assert.strictEqual(g.d, env.d0, "the distance of a fixed view does not change (d=" + d + ")");
    assert.strictEqual(g.yaw, env.yaw0);
    assert.strictEqual(g.pit, env.pit0);
    assert.strictEqual(g.roll, 0);
    assert.deepStrictEqual([g.t.x, g.t.y, g.t.z], [0, .404, .15], "the target is still the panel");
  }
});

test("inside: yaw +-60 degrees around the entry, pitch 20..80, distance locked", () => {
  const env = { center: { x: -.02, y: .348, z: -.03 }, d0: .46, yaw0: .35 };
  const L = SWCam.LAW.inside;
  for (const t of TS) for (const d of DS) for (const a of AS) {
    const g = clamp("inside", goal(t, d, a, a), env);
    assert.strictEqual(g.d, env.d0, "no wheel zoom: the distance is the one of the view");
    const dy = Math.atan2(Math.sin(g.yaw - env.yaw0), Math.cos(g.yaw - env.yaw0));
    assert.ok(Math.abs(dy) <= L.yaw + 1e-9, "yaw out of range: " + dy);
    assert.ok(g.pit >= L.pit[0] - 1e-9 && g.pit <= L.pit[1] + 1e-9, "pitch out of range: " + g.pit);
    assert.deepStrictEqual([g.t.x, g.t.y, g.t.z], [-.02, .348, -.03], "the target is still the bench");
    assert.strictEqual(g.roll, 0);
  }
});

test("free: the camera never enters the device, not even with an absurd target and wheel", () => {
  const c = { x: 0, y: .404, z: 0 }, R = .34, env = { center: c, R: R };
  for (const t of TS) for (const d of DS) for (const ya of AS) for (const pa of AS) {
    const g = clamp("free", goal(t, d, ya, pa), env);
    assert.ok(far(pos(g), c) >= R - 1e-9, "camera inside the device: " + far(pos(g), c).toFixed(4)
      + " (t=" + t + " d=" + d + " yaw=" + ya + " pit=" + pa + ")");
    assert.ok(g.d >= SWCam.LAW.free.d[0] - 1e-9 && g.pit >= SWCam.LAW.free.pit[0] - 1e-9 && g.pit <= SWCam.LAW.free.pit[1] + 1e-9);
  }
});

test("free: far from the device, the camera does not go past the wall or the floor", () => {
  const env = { center: { x: 0, y: .404, z: 0 }, R: .34 }, F = SWCam.LAW.free;
  // targets picked far from the device: only the room limit can act there
  for (const t of [[0, 2.2, -4], [2.5, 1.5, -3.5], [-2, .8, -4.4]]) for (const d of DS) for (const ya of AS) for (const pa of AS) {
    const p = pos(clamp("free", goal(t, d, ya, pa), env));
    assert.ok(p[1] >= F.floor - 1e-9, "camera below the floor: y=" + p[1].toFixed(3));
    assert.ok(p[2] >= F.wall - 1e-9, "camera behind the wall: z=" + p[2].toFixed(3));
  }
});
