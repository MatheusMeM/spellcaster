/* Camera in the SolidWorks standard: middle button orbits around the clicked point, Ctrl+middle pans, Shift+middle zooms,
   the wheel zooms at the cursor (SolidWorks direction by default, with a toggle), arrows 15°, Shift+arrows 90°, Ctrl+arrows pan,
   F frames, Ctrl+1..7 standard views. The left button on empty space orbits too (laptop with no middle button).
   ponytail: no inertia; no "around the model" rotation when the click lands on empty space (it uses the current target). */
window.SWCam = function (THREE, cam, dom, o) {
  "use strict";
  var goal = { t: new THREE.Vector3(0, .4, 0), d: 1, yaw: 0, pit: .3 }, cur = { t: goal.t.clone(), d: 1, yaw: 0, pit: .3 }, drag = null, api;
  var Y = new THREE.Vector3(0, 1, 0), tmp = new THREE.Vector3();
  function posOf(s) { return new THREE.Vector3(s.t.x + s.d * Math.sin(s.yaw) * Math.cos(s.pit), s.t.y + s.d * Math.sin(s.pit), s.t.z + s.d * Math.cos(s.yaw) * Math.cos(s.pit)); }
  function fromPos(s, pos, t) { s.t.copy(t); var d = pos.clone().sub(t); s.d = Math.max(.05, d.length()); s.pit = Math.asin(Math.max(-1, Math.min(1, d.y / s.d))); s.yaw = Math.atan2(d.x, d.z); }
  function setView(pos, t, snap) { fromPos(goal, new THREE.Vector3().fromArray(pos), new THREE.Vector3().fromArray(t)); if (snap) { cur.t.copy(goal.t); cur.d = goal.d; cur.yaw = goal.yaw; cur.pit = goal.pit; } }
  function clampPit(p) { return Math.max(-1.52, Math.min(1.52, p)); }
  function basis() { var f = goal.t.clone().sub(posOf(goal)).normalize(), r = new THREE.Vector3().crossVectors(f, Y).normalize(), u = new THREE.Vector3().crossVectors(r, f).normalize(); return { f: f, r: r, u: u }; }
  function pan(dx, dy) { var b = basis(), k = goal.d * .0016; goal.t.addScaledVector(b.r, -dx * k).addScaledVector(b.u, dy * k); }
  function zoomAt(s, p) { // p: world point under the cursor (or null) — camera and target move toward it together
    if (p) { goal.t.copy(p).add(tmp.copy(goal.t).sub(p).multiplyScalar(s)); } goal.d = Math.max(.08, Math.min(14, goal.d * s)); }
  dom.addEventListener("pointerdown", function (e) { var mid = e.button === 1, left = e.button === 0; if (!mid && !left) return; if (left && (o.hit(e) || !o.emptyRotate)) return;
    var mode = e.ctrlKey ? "pan" : e.shiftKey ? "zoom" : "rot"; if (mode === "rot" && mid) { var p = o.pick(e); if (p) { var pos = posOf(goal); fromPos(goal, pos, p); fromPos(cur, cam.position.clone(), p); } }
    drag = { mode: mode, x: e.clientX, y: e.clientY, moved: false }; dom.setPointerCapture(e.pointerId); e.preventDefault(); });
  dom.addEventListener("pointermove", function (e) { if (!drag) return; var dx = e.clientX - drag.x, dy = e.clientY - drag.y; drag.x = e.clientX; drag.y = e.clientY; if (Math.abs(dx) + Math.abs(dy) > 1) drag.moved = true;
    if (drag.mode === "rot") { goal.yaw -= dx * .006; goal.pit = clampPit(goal.pit + dy * .006); } else if (drag.mode === "pan") pan(dx, dy); else zoomAt(Math.exp(dy * .005), null); });
  function up() { drag = null; } dom.addEventListener("pointerup", up); dom.addEventListener("pointercancel", up);
  dom.addEventListener("wheel", function (e) { e.preventDefault(); var dir = api.reverse ? -1 : 1, s = Math.exp(dir * Math.sign(e.deltaY) * .12); zoomAt(s, o.pick(e) || o.plane(e, goal.t)); }, { passive: false });
  dom.addEventListener("auxclick", function (e) { if (e.button === 1) e.preventDefault(); });
  dom.addEventListener("contextmenu", function (e) { e.preventDefault(); });
  var STD = { front: [0, 0], back: [Math.PI, 0], left: [-Math.PI / 2, 0], right: [Math.PI / 2, 0], top: [0, 1.5], bottom: [0, -1.5], iso: [Math.PI / 4, .615] };
  api = { reverse: true, goal: goal, cur: cur,
    setView: setView, get: function () { return { pos: posOf(goal).toArray(), t: goal.t.toArray() }; },
    rotate: function (dy, dp) { goal.yaw += dy; goal.pit = clampPit(goal.pit + dp); }, pan: function (dx, dy) { pan(dx, dy); }, zoom: function (s) { zoomAt(s, null); },
    fit: function (center, radius) { goal.t.copy(center); goal.d = radius / Math.sin(cam.fov * Math.PI / 360) * 1.1; },
    std: function (k, center, radius) { var v = STD[k]; if (!v) return; goal.yaw = v[0]; goal.pit = v[1]; if (center) api.fit(center, radius); },
    dragging: function () { return !!(drag && drag.moved); },
    update: function (dt) { var k = Math.min(1, dt * (o.speed || 5)); cur.t.lerp(goal.t, k); cur.d += (goal.d - cur.d) * k; var dy = goal.yaw - cur.yaw; dy = Math.atan2(Math.sin(dy), Math.cos(dy)); cur.yaw += dy * k; cur.pit += (goal.pit - cur.pit) * k; cam.position.copy(posOf(cur)); cam.lookAt(cur.t); } };
  return api;
};
