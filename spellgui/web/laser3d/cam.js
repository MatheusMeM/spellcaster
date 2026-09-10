/* laser3d camera: three laws, one per view. The map and the manual URLs are in
   `design/FUNCOES/camera-solidworks.md`.

     free   (SHOW view, the visualizer) — the whole SolidWorks: middle orbits around the clicked
            point, Ctrl+middle pans, Shift+middle zooms, Alt+middle rolls, the wheel zooms at the
            cursor, left button on empty space orbits. With limits: the camera never enters the
            device and never goes through the wall or the floor.
     rear   (REAR view, which IS the menu) — fixed pose computed from the normal of the rear panel,
            framing the 400 x 180 mm. No dragging, no middle button, no wheel zoom, no arrows.
            The only movement is a +-2 deg breathe that follows the mouse and does NOT change the distance.
     inside (lid open) — orbit locked to the centre of the optical bench: yaw +-60 deg around the
            entry pose, pitch 20..80 deg, fixed distance. Nothing else.

   Two things the previous version did and are not done any more:
   - the movement was a `lerp` with a fixed factor (`k = dt * 5`): it depends on the frame rate and
     never reaches the target, so the camera "swam" behind the mouse and kept moving after the button
     was released. Now it is a critically damped spring, in analytic form (stable at any dt, it
     arrives and stops).
   - a drag that started ON TOP of a part did not become a camera drag (right) but also did not count
     as a drag (wrong): on release, `app.js` treated it as a click and opened the panel of that part.
     Now the gesture is recorded with mode "none" and `dragging()` answers for it — and keeps
     answering DURING the `pointerup`, which is when `app.js` asks (see `ended`).

   `SWCam.clamp(mode, goal, env)` is the law, pure and without THREE: it is what `test/cam.test.js`
   verifies. */
(function () {
  "use strict";

  function lim(v, a, b) { return v < a ? a : v > b ? b : v; }
  function wrap(a) { return Math.atan2(Math.sin(a), Math.cos(a)); }
  // direction from the target to the camera, the same spherical form as posOf()
  function dirX(y, p) { return Math.sin(y) * Math.cos(p); }
  function dirY(p) { return Math.sin(p); }
  function dirZ(y, p) { return Math.cos(y) * Math.cos(p); }

  /* The room: floor at y = 0, wall at z = -5.01, projection of 8 x 5 (body.js). The target moves
     inside the box; what cannot leave the room is the POSITION of the camera, and that one is
     limited by `d`. */
  var LAW = {
    free: { pit: [-1.4835, 1.4835],            // +-85 deg
            d: [.12, 6],
            box: { x: [-3.2, 3.2], y: [.05, 3], z: [-5.05, 2] },
            floor: .06, wall: -4.6 },
    rear: {},
    inside: { pit: [.349, 1.396], yaw: 1.047 } // 20..80 deg ; +-60 deg
  };

  /// The law of each mode, applied to the TARGET of the movement (not to the current state): a pure
  /// function. `g` = { t:{x,y,z}, d, yaw, pit, roll } and is changed in place. `env` = { center, R,
  /// d0, yaw0, pit0 } and comes from the view. Returns `g`.
  function clamp(mode, g, env) {
    env = env || {};
    var c = env.center || { x: 0, y: .404, z: 0 };
    if (mode === "rear") {                                   // fixed pose: nothing from the user gets in
      g.t.x = c.x; g.t.y = c.y; g.t.z = c.z; g.roll = 0;
      g.yaw = env.yaw0 || 0; g.pit = env.pit0 || 0;
      if (env.d0) g.d = env.d0;
      return g;
    }
    if (mode === "inside") {                                 // locked orbit, fixed distance
      var y0 = env.yaw0 || 0, L = LAW.inside;
      g.t.x = c.x; g.t.y = c.y; g.t.z = c.z; g.roll = 0;
      if (env.d0) g.d = env.d0;
      g.yaw = y0 + lim(wrap(g.yaw - y0), -L.yaw, L.yaw);
      g.pit = lim(g.pit, L.pit[0], L.pit[1]);
      return g;
    }
    var F = LAW.free;
    g.t.x = lim(g.t.x, F.box.x[0], F.box.x[1]);
    g.t.y = lim(g.t.y, F.box.y[0], F.box.y[1]);
    g.t.z = lim(g.t.z, F.box.z[0], F.box.z[1]);
    g.pit = lim(g.pit, F.pit[0], F.pit[1]);
    g.d = lim(g.d, F.d[0], F.d[1]);
    var nx = dirX(g.yaw, g.pit), ny = dirY(g.pit), nz = dirZ(g.yaw, g.pit);
    // room: the position (t + d*n) does not go past the floor or the wall — these only SHRINK d
    if (ny < 0 && g.t.y + g.d * ny < F.floor) g.d = (F.floor - g.t.y) / ny;
    if (nz < 0 && g.t.z + g.d * nz < F.wall) g.d = (F.wall - g.t.z) / nz;
    if (g.d < F.d[0]) g.d = F.d[0];   // target flat on the floor: the room limit used to take d to zero
    // device: |t + d*n - c| >= R. Larger root of the quadratic in d — it only GROWS d, and that is
    // why it comes last: the invariant "the camera never enters the device" always holds at the end.
    // ponytail: a sphere, not the box of the device; R is already the radius of the box plus the clearance.
    var R = env.R || .34, ux = g.t.x - c.x, uy = g.t.y - c.y, uz = g.t.z - c.z;
    var b = ux * nx + uy * ny + uz * nz, q = ux * ux + uy * uy + uz * uz - R * R, disc = b * b - q;
    if (disc > 0) { var far = -b + Math.sqrt(disc); if (far > g.d) g.d = far; }
    return g;
  }

  window.SWCam = function (THREE, cam, dom, o) {
    var goal = { t: new THREE.Vector3(0, .4, 0), d: 1, yaw: 0, pit: .3, roll: 0 },
        cur = { t: goal.t.clone(), d: 1, yaw: 0, pit: .3, roll: 0 },
        vel = { x: 0, y: 0, z: 0, d: 0, yaw: 0, pit: 0, roll: 0 },
        drag = null, ended = false, api, mode = "free", env = { R: .34 }, fixed = null, aspect0 = -1,
        br = { x: 0, y: 0 };                                   // breathe of the fixed view
    var Y = new THREE.Vector3(0, 1, 0), tmp = new THREE.Vector3();

    function posOf(s) { return new THREE.Vector3(s.t.x + s.d * dirX(s.yaw, s.pit), s.t.y + s.d * dirY(s.pit), s.t.z + s.d * dirZ(s.yaw, s.pit)); }
    function fromPos(s, pos, t) { s.t.copy(t); var v = pos.clone().sub(t); s.d = Math.max(.05, v.length()); s.pit = Math.asin(lim(v.y / s.d, -1, 1)); s.yaw = Math.atan2(v.x, v.z); }
    function setView(pos, t, snap) { fromPos(goal, new THREE.Vector3().fromArray(pos), new THREE.Vector3().fromArray(t)); goal.roll = 0; if (snap) { cur.t.copy(goal.t); cur.d = goal.d; cur.yaw = goal.yaw; cur.pit = goal.pit; cur.roll = 0; vel.x = vel.y = vel.z = vel.d = vel.yaw = vel.pit = vel.roll = 0; } }
    function basis() { var f = goal.t.clone().sub(posOf(goal)).normalize(), r = new THREE.Vector3().crossVectors(f, Y).normalize(), u = new THREE.Vector3().crossVectors(r, f).normalize(); return { f: f, r: r, u: u }; }
    // sensitivity proportional to the distance: near the device the same pixel moves less world
    function pan(dx, dy) { var b = basis(), k = goal.d * .0014; goal.t.addScaledVector(b.r, -dx * k).addScaledVector(b.u, dy * k); }
    function zoomAt(s, p) { // p: world point under the cursor (or null) — target and camera go to it together
      if (p) goal.t.copy(p).add(tmp.copy(goal.t).sub(p).multiplyScalar(s));
      goal.d = goal.d * s; }

    /* Framing of a w x h rectangle with clearance, from the fov and the current ASPECT: that is why
       the fixed poses are recomputed when the window changes size. */
    function frame(e) {
      var vf = 2 * Math.tan(cam.fov * Math.PI / 360), pad = e.pad || 1.4;
      return Math.max((e.h || .18) * pad / vf, (e.w || .4) * pad / (vf * (cam.aspect || 1.6)));
    }
    function repose() {
      aspect0 = cam.aspect;
      if (mode === "rear") { var n = env.normal || { x: 0, y: 0, z: 1 };
        fixed = { yaw: Math.atan2(n.x, n.z), pit: Math.asin(lim(n.y, -1, 1)), d: frame(env) }; }
      else if (mode === "inside") fixed = { yaw: env.yaw0 || 0, pit: env.pit0 || .9, d: frame(env) };
      else fixed = null;
    }

    /* critically damped spring, analytic form: y(t) = (A + B*t)*e^(-w*t), A = y0, B = v0 + w*y0.
       It reaches the target without overshooting and STOPS — and it does not depend on the frame
       rate the way the lerp did. */
    function sp(s, v, k, g, w, dt) {
      var y = s[k] - g, A = y, B = v[k] + w * y, e = Math.exp(-w * dt);
      s[k] = g + (A + B * dt) * e; v[k] = (B - w * (A + B * dt)) * e;
    }

    function press(e) {
      var mid = e.button === 1, left = e.button === 0; if (!mid && !left) return;
      if (mode === "rear") return;                                   // the rear is the menu: it is not dragged
      // a drag that starts on a part does not move the camera, but it IS a drag: without this the
      // `pointerup` in `app.js` turned into a click and opened the panel of the part at the end of an orbit.
      ended = false;
      if (left && o.hit(e)) { drag = { mode: "none", x: e.clientX, y: e.clientY, moved: false }; return; }
      var m = mode === "inside" ? "rot"                              // inside only orbits
        : e.ctrlKey ? "pan" : e.shiftKey ? "zoom" : e.altKey ? "roll" : "rot";
      if (m === "rot" && mid && mode === "free") { var p = o.pick(e); if (p) { var pos = posOf(goal); fromPos(goal, pos, p); fromPos(cur, cam.position.clone(), p); } }
      drag = { mode: m, x: e.clientX, y: e.clientY, moved: false };
      dom.setPointerCapture(e.pointerId); e.preventDefault();
    }
    function moveAt(e) {
      var r = dom.getBoundingClientRect();                           // breathe: +-2 deg of parallax, without touching the distance
      br.x = lim(((e.clientX - r.left) / r.width) * 2 - 1, -1, 1); br.y = lim(((e.clientY - r.top) / r.height) * 2 - 1, -1, 1);
      if (!drag) { ended = false; return; }                          // pointermove after the release: a new gesture
      var dx = e.clientX - drag.x, dy = e.clientY - drag.y; drag.x = e.clientX; drag.y = e.clientY;
      // 4 px: a hand tremor with the finger on the button must not become a drag and eat the click
      if (Math.abs(dx) + Math.abs(dy) > 4) drag.moved = true;
      if (drag.mode === "rot") { goal.yaw -= dx * .0045; goal.pit += dy * .0045; }
      else if (drag.mode === "pan") pan(dx, dy);
      else if (drag.mode === "zoom") zoomAt(Math.exp(dy * .0035), null);
      else if (drag.mode === "roll") goal.roll += dx * .005;
    }
    dom.addEventListener("pointerdown", press);
    dom.addEventListener("pointermove", moveAt);
    /* `up` runs BEFORE the `pointerup` in `app.js` (this listener was registered first), so clearing
       `drag` here made `dragging()` lie to exactly the caller that asks: the end of an orbit turned
       into a click and opened the panel of the part. `ended` keeps the "there was a drag" until the
       next gesture. */
    function up() { ended = !!(drag && drag.moved); drag = null; }
    dom.addEventListener("pointerup", up); dom.addEventListener("pointercancel", up);
    dom.addEventListener("pointerleave", function () { br.x = br.y = 0; });
    dom.addEventListener("wheel", function (e) {
      e.preventDefault();
      if (mode !== "free") return;                                   // fixed and restricted: the wheel does not zoom
      var dir = api.reverse ? -1 : 1, s = Math.exp(dir * Math.sign(e.deltaY) * .07);   // smaller step
      zoomAt(s, o.pick(e) || o.plane(e, goal.t));
    }, { passive: false });
    dom.addEventListener("auxclick", function (e) { if (e.button === 1) e.preventDefault(); });
    dom.addEventListener("contextmenu", function (e) { e.preventDefault(); });

    var STD = { front: [0, 0], back: [Math.PI, 0], left: [-Math.PI / 2, 0], right: [Math.PI / 2, 0], top: [0, 1.5], bottom: [0, -1.5], iso: [Math.PI / 4, .615] };
    api = { reverse: true, breathe: true, goal: goal, cur: cur,
      setView: setView,
      get: function () { return { pos: posOf(goal).toArray(), t: goal.t.toArray() }; },
      /// The law of this view. `e` carries center/normal/w/h/pad (fixed), yaw0/pit0 (entry) and R.
      mode: function (m, e) { mode = LAW[m] ? m : "free"; env = e || { R: .34 }; repose(); },
      camMode: function () { return mode; },
      free: function () { return mode === "free"; },
      rotate: function (dy, dp) { if (mode !== "free") return; goal.yaw += dy; goal.pit += dp; },
      roll: function (dr) { if (mode === "free") goal.roll += dr; },
      pan: function (dx, dy) { if (mode === "free") pan(dx, dy); },
      zoom: function (s) { if (mode === "free") zoomAt(s, null); },
      fit: function (center, radius) { if (mode !== "free") return; goal.t.copy(center); goal.d = radius / Math.sin(cam.fov * Math.PI / 360) * 1.1; },
      std: function (k, center, radius) { var v = STD[k]; if (mode !== "free" || !v) return; goal.yaw = v[0]; goal.pit = v[1]; goal.roll = 0; if (center) api.fit(center, radius); },
      dragging: function () { return !!(drag && drag.moved) || ended; },
      update: function (dt) {
        if (cam.aspect !== aspect0) repose();                        // window changed: fixed pose redone
        if (fixed) {
          env.d0 = fixed.d;
          env.yaw0 = fixed.yaw + (mode === "rear" && api.breathe ? br.x * .035 : 0);
          if (mode === "rear") env.pit0 = fixed.pit - (api.breathe ? br.y * .035 : 0);
        }
        clamp(mode, goal, env);
        var w = 18, dy = wrap(goal.yaw - cur.yaw);                   // yaw by the short path
        cur.yaw = goal.yaw - dy; sp(cur, vel, "yaw", goal.yaw, w, dt);
        sp(cur.t, vel, "x", goal.t.x, w, dt); sp(cur.t, vel, "y", goal.t.y, w, dt); sp(cur.t, vel, "z", goal.t.z, w, dt);
        sp(cur, vel, "d", goal.d, w, dt); sp(cur, vel, "pit", goal.pit, w, dt); sp(cur, vel, "roll", goal.roll, w, dt);
        cam.position.copy(posOf(cur)); cam.lookAt(cur.t); if (cur.roll) cam.rotateZ(cur.roll);
      } };
    repose();
    return api;
  };
  window.SWCam.clamp = clamp;
  window.SWCam.LAW = LAW;
  if (typeof module !== "undefined") module.exports = window.SWCam;
})();
