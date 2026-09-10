/* Pino in 3D: a real DMX cable that is the menu of the program. He no longer stands on top of the
   flightcase, out in the world, where the camera left him behind: the group is a CHILD OF THE CAMERA, in the
   lower left corner, ~120 px tall in any window, with its own light and drawn on top of the
   device (the `clr` sentinel clears the depth before his crew). The five pins are still
   buttons (raycast), the eyes follow the mouse, the Win98 balloon is HTML anchored on the projection of the
   head — now opening to the RIGHT, because the Pino changed sides.
   The cable leaves the BOOT (the rear tip, on his axis) and runs to the DMX OUT of the device as a Verlet
   rope (`rope.js`): since the top end travels with the camera, the cable follows the camera to the laser
   from wherever it is. The `–` button of the balloon sends the Pino away: he goes down, shrinks, the cable leaves the
   scene and the rope STOPS being simulated; in his place stays the `#pino-min` icon, which brings him back. */
window.Pino3D = (function () {
  "use strict";
  var PINS = [["ilda", "ILDA player", "LASER"], ["ndi", "NDI → ILDA", "FÓSFORO"], ["orq", "Orchestrator", "PATCHBAY"], ["cues", "Scenes and cues", "PAPER THEATER"], ["nfo", "Info", "N"]];
  var N = 24, RS = 6, RAD = .0032, RAD0 = .0011, DIST = .25, TALL = 120, MARGX = 90, MARGY = 66, SLACK = 1.014, GRAV = -3;
  function build(THREE, X, scene, pick, stage, cam, o) { var m = X.m, PI = Math.PI;
    /* Where the cable ends: the DMX OUT port of the chassis. If `o.port` comes in (the `B.dmxOut` group), the
       position AND the outgoing axis come from its matrix — the plug is born aligned with the panel, instead
       of me guessing the normal. `o.target` (contract `B.dmxOutWorld`) and the constant are the rung
       below. ponytail: no check that the port still exists afterwards; it does not vanish from the model. */
    var TGT = new THREE.Vector3(.038, .338, .173), AX = new THREE.Vector3(0, 0, 1);
    if (o && o.port) { o.port.updateWorldMatrix(true, false); o.port.getWorldPosition(TGT); o.port.getWorldDirection(AX); }
    else if (o && o.target) TGT.copy(o.target);
    function add(parent, g, mat, x, y, z, k, label) { var ob = new THREE.Mesh(g, mat); ob.position.set(x, y, z); ob.castShadow = ob.receiveShadow = true; ob.renderOrder = 999; if (k) { ob.userData = { key: k, label: label }; pick.push(ob); } parent.add(ob); return ob; }
    /* The Pino's materials, not the device's. The scene is physical light with a spot of ~50 cd: in it
       any diffuse dielectric saturates, and the Pino made of `m.silver` + `m.white` came out a white smear
       of 120 px where you could find neither eye nor pin. Dark metal solves the body (metal has almost
       no diffuse, it is the same trick as the panel's `m.alu`); the eyes and the eyebrows go with
       `MeshBasicMaterial`, which depends on no light and therefore does not blow out at any exposure
       — the face stays legible next to the lit device or in the dark of the inside view. */
    var body = new THREE.MeshStandardMaterial({ color: 0x24282d, metalness: .9, roughness: .78, envMapIntensity: .3 }),
      collar = new THREE.MeshStandardMaterial({ color: 0x9aa1a8, metalness: 1, roughness: .5, envMapIntensity: .3 }),
      cable = new THREE.MeshStandardMaterial({ color: 0x0a0b0d, metalness: .92, roughness: .42, envMapIntensity: .2 }),
      eye = new THREE.MeshBasicMaterial({ color: 0xdde2e6 }), ink = new THREE.MeshBasicMaterial({ color: 0x08090b });
    var g = new THREE.Group();
    var head = new THREE.Group(); head.position.set(0, .06, 0); head.rotation.x = -.32; g.add(head); head.userData = { key: "pino", label: "Pino: DMX cable, five pins, zero patience" }; pick.push(head);
    add(head, new THREE.CylinderGeometry(.010, .010, .06, 24), body, 0, 0, 0); add(head, new THREE.CylinderGeometry(.0106, .0106, .006, 24), collar, 0, .025, 0);
    add(head, new THREE.CylinderGeometry(.008, .008, .003, 24), body, 0, .0305, 0);
    var pins = []; PINS.forEach(function (p, i) { var a = PI / 2 + i * PI * 2 / 5 + PI / 5, pin = add(head, new THREE.CylinderGeometry(.0016, .0016, .009, 8), collar.clone(), Math.cos(a) * .0055, .0355, -Math.sin(a) * .0055, "pino." + p[0], "pin " + (i + 1) + " · " + p[1] + " · " + p[2]); pins.push(pin); });
    add(head, new THREE.BoxGeometry(.004, .012, .003), collar, .0105, .008, .004, "pino.bye", "latch: lets the Pino go");
    add(head, new THREE.CylinderGeometry(.007, .0075, .022, 16), cable, 0, -.038, 0);
    var eyes = [], pupils = []; [-.007, .007].forEach(function (x) { var e = add(head, new THREE.SphereGeometry(.006, 16, 12), eye, x, .014, .0085); e.castShadow = false; eyes.push(e); var p = add(e, new THREE.SphereGeometry(.0032, 12, 8), ink, 0, 0, .0045); p.castShadow = false; pupils.push(p); var b = add(head, new THREE.BoxGeometry(.009, .0015, .0015), ink, x, .0225, .009); b.rotation.z = x < 0 ? .3 : -.3; });

    /* Drawn on top of the device without `depthTest:false` part by part: turning the test off part by part
       kills his self-occlusion (a pin behind the head would start showing in front), and it is a flag
       per material — and he shares `collar` between body and plug. The sentinel below enters the drawing
       list right before the Pino's crew (renderOrder 998 < 999) and clears the depth buffer: from there on he
       is the only one that exists, and he keeps self-occluding properly (a pin behind the head). */
    var clr = new THREE.Mesh(new THREE.PlaneGeometry(.001, .001), new THREE.MeshBasicMaterial({ colorWrite: false, depthWrite: false, depthTest: false }));
    clr.renderOrder = 998; clr.frustumCulled = false; clr.onBeforeRender = function (r) { r.clearDepth(); }; g.add(clr);
    g.rotation.set(0, .34, .05); g.updateMatrixWorld(true); // 3/4: the eyes end up facing whoever is looking
    var bb = new THREE.Box3().setFromObject(g), H0 = (bb.max.y - bb.min.y) || .09, X0 = bb.min.x, Y0 = bb.min.y;
    // the Pino is TALL px high in any window, so his width is fixed in px as well:
    // the balloon starts after it, otherwise the speech box falls on top of the Pino himself.
    var BALX = MARGX + (bb.max.x - X0) / H0 * TALL + 26;
    cam.add(g); if (!cam.parent) scene.add(cam);
    // his own light: without it the Pino sits in the shadow of the device, which is where the camera always is.
    // The short range (0.4 m) is what makes it HIS light: with a range of 1 m the same lamp
    // blew out the rear of the device, which is half a metre from the camera.
    var lamp = new THREE.PointLight(0xfff2e6, .1, .4, 2), lamp2 = new THREE.PointLight(0x9fc4ff, .045, .4, 2); cam.add(lamp); cam.add(lamp2);

    /* Cable: a Verlet rope from the boot to the DMX OUT, in a tube of fixed topology (24 nodes × 6 sides)
       whose vertices are rewritten per frame — no `new TubeGeometry` and no `dispose()` every
       frame, which would be allocating and freeing 168 vertices 60 times a second for nothing. */
    var END = TGT.clone().addScaledVector(AX, .058); // it leaves through the strain relief of the plug, not through the panel
    var P = Rope.make(N, [0, .3, 0], [END.x, END.y, END.z]), rest = .02, live = true;
    var ropeOn = true;  // PINO CABLE row of the VIDEO menu: no Verlet and no retube when it is off
    var tube = new THREE.Mesh(new THREE.TubeGeometry(new THREE.CatmullRomCurve3([new THREE.Vector3(0, .3, 0), END.clone()]), N - 1, RAD, RS, false), cable);
    tube.castShadow = false; tube.frustumCulled = false; scene.add(tube); // ponytail: cable with no shadow; it almost never touches a surface
    /* Male XLR plug: black body, silver collar, rubber strain relief. All silver (as it was)
       turned into a white barrel on top of the panel, bigger than the device's own connector. */
    var plug = new THREE.Group(); plug.position.copy(TGT).addScaledVector(AX, .010); plug.lookAt(plug.position.clone().add(AX)); scene.add(plug);
    add(plug, new THREE.CylinderGeometry(.0098, .0098, .034, 20), m.alu, 0, 0, .017).rotation.x = PI / 2;
    add(plug, new THREE.CylinderGeometry(.0108, .0108, .006, 20), collar, 0, 0, .002).rotation.x = PI / 2;
    var relief = add(plug, new THREE.CylinderGeometry(.0072, .0045, .018, 14), cable, 0, 0, .042); relief.rotation.x = PI / 2;
    plug.traverse(function (x) { x.renderOrder = 0; });
    var tp = tube.geometry.attributes.position, tn = tube.geometry.attributes.normal;
    var T = new THREE.Vector3(), Nr = new THREE.Vector3(1, 0, 0), Bi = new THREE.Vector3(), tmp = new THREE.Vector3(), BOOT = new THREE.Vector3(0, -.052, 0), boot = new THREE.Vector3();
    function retube() { var pa = tp.array, na = tn.array, i, j, k, a, ca, sa, nx, ny, nz, vi, rr;
      for (i = 0; i < N; i++) { k = i * 6;
        if (i === 0) T.set(P[6] - P[0], P[7] - P[1], P[8] - P[2]);
        else if (i === N - 1) T.set(P[k] - P[k - 6], P[k + 1] - P[k - 5], P[k + 2] - P[k - 4]);
        else T.set(P[k + 6] - P[k - 6], P[k + 7] - P[k - 5], P[k + 8] - P[k - 4]);
        if (T.lengthSq() < 1e-12) T.set(0, -1, 0); T.normalize();
        // parallel transport: the normal of the previous node, projected out of the tangent. Without this the
        // tube twists on its own where the rope bends, and the cable shows up with a rotating seam.
        tmp.copy(T).multiplyScalar(Nr.dot(T)); Nr.sub(tmp);
        if (Nr.lengthSq() < 1e-8) { Nr.set(0, 1, 0); tmp.copy(T).multiplyScalar(Nr.dot(T)); Nr.sub(tmp); if (Nr.lengthSq() < 1e-8) Nr.set(1, 0, 0); }
        Nr.normalize(); Bi.crossVectors(T, Nr);
        // the radius thins towards the Pino's end: he is 25 cm from the eye and drawn at screen-corner
        // scale (~1/4), so a cable of a single radius would turn into a hose there and a thread over there.
        rr = RAD0 + (RAD - RAD0) * (i / (N - 1));
        for (j = 0; j <= RS; j++) { a = j / RS * PI * 2; ca = -Math.cos(a); sa = Math.sin(a);
          nx = ca * Nr.x + sa * Bi.x; ny = ca * Nr.y + sa * Bi.y; nz = ca * Nr.z + sa * Bi.z;
          vi = (i * (RS + 1) + j) * 3;
          na[vi] = nx; na[vi + 1] = ny; na[vi + 2] = nz;
          pa[vi] = P[k] + nx * rr; pa[vi + 1] = P[k + 1] + ny * rr; pa[vi + 2] = P[k + 2] + nz * rr; } }
      tp.needsUpdate = tn.needsUpdate = true; }

    // balloon + minimized icon
    var bal = document.createElement("div"); bal.className = "bal"; stage.appendChild(bal);
    // The icon is an XLR-3 drawn in CSS (collar + three pins), not `brand/submark.svg`: that
    // file, despite the name, is the horizontal logotype of 1009 x 305 and disappears in a 32 px square.
    var mini = document.createElement("button"); mini.id = "pino-min"; mini.title = "bring the Pino back"; mini.setAttribute("aria-label", "bring the Pino back"); mini.innerHTML = "<i></i>"; stage.appendChild(mini);
    var talking = 0, t = 0, blink = 0, gone = false, cur = null, vis = 1, v = new THREE.Vector3(), pw = 0, ph = 0, ms = 0, msN = 0;
    // dismissed in the last session: he comes in already taken apart (no exit animation) — marking only
    // `gone` left the cable in the scene, hanging off the DMX OUT and coming from an invisible Pino.
    try { if (localStorage.getItem("sc-pino") === "0") { vis = 0; bye(); } } catch (e) {}
    function say(text, items, hint) { if (gone) return; bal.innerHTML = '<span class="x" title="close the balloon">×</span><span class="m" title="send the Pino away">–</span><b>' + text + "</b>" + (items ? "<ul>" + items.map(function (it) { return "<li data-a=\"" + it[0] + "\">" + it[1] + (it[2] ? " <small>" + it[2] + "</small>" : "") + "</li>"; }).join("") + "</ul>" : "") + (hint === false ? "" : '<div class="hint">' + (hint || "the pins are the menu: 1 laser · 2 fósforo · 3 patchbay · 4 theater · 5 info · latch = he leaves") + "</div>"); bal.classList.add("on"); talking = 1.2; }
    function hide() { bal.classList.remove("on"); }
    bal.addEventListener("click", function (e) { if (e.target.classList.contains("m")) { bye(); return; } if (e.target.classList.contains("x")) { hide(); return; } var li = e.target.closest("li"); if (li && o.on) o.on(li.dataset.a); });
    mini.addEventListener("click", back);
    function current(k) { cur = k; pins.forEach(function (p, i) { p.material.emissive.setHex(PINS[i][0] === k ? 0x38ff5c : 0); p.material.emissiveIntensity = .8; }); }
    function remember() { try { localStorage.setItem("sc-pino", gone ? "0" : "1"); } catch (e) {} }
    /* The cable leaves the scene at the INSTANT of the dismiss, not at the end of the exit animation: while the Pino
       was shrinking and going down, the cable stayed whole, stuck to the DMX OUT and hanging off nothing. */
    function bye() { if (gone) return; gone = true; live = false; scene.remove(tube); plug.visible = relief.visible = false; hide(); remember(); }
    /* Coming back restarts the rope stretched between the two ends of NOW: reconnecting with the nodes from where it
       stopped would make the cable whip through the device on the first frame (the camera moved
       while the Pino was away). */
    function back() { if (!gone) return; gone = false; live = true; scene.add(tube); plug.visible = relief.visible = true;
      boot.copy(BOOT); head.localToWorld(boot); reset(); remember(); say("Back. The pins are still the menu."); }
    function reset() { var i, k, f; for (i = 0; i < N; i++) { k = i * 6; f = i / (N - 1);
      P[k] = P[k + 3] = boot.x + (END.x - boot.x) * f; P[k + 1] = P[k + 4] = boot.y + (END.y - boot.y) * f; P[k + 2] = P[k + 5] = boot.z + (END.z - boot.z) * f; } }

    /* Where he sits: in camera coordinates, DIST metres ahead. The visible half-height there is
       tan(fov/2)·DIST, so `u` is what a pixel is worth in metres and the rest is corner arithmetic — the
       Pino is TALL px high and sits MARGX px from the left edge in any window. */
    function place(W, H) { var u = 2 * Math.tan(cam.fov * PI / 360) * DIST / H, w2 = u * W / 2, h2 = u * H / 2, s = TALL * u / H0;
      g.scale.setScalar(s * Math.max(.001, vis));
      g.position.set(-w2 + MARGX * u - X0 * s, -h2 + MARGY * u - Y0 * s - (1 - vis) * TALL * 1.6 * u, -DIST);
      lamp.position.set(g.position.x + .05, g.position.y + .09, -DIST + .13); lamp2.position.set(g.position.x - .07, g.position.y + .02, -DIST + .09); }

    function update(dt, mouse, W, H, minTop) { t += dt; if (talking > 0) talking -= dt;
      vis += ((gone ? 0 : 1) - vis) * Math.min(1, dt * 6.5); if (vis < .02) vis = 0;
      if (W !== pw || H !== ph || vis !== 1) { pw = W; ph = H; place(W, H); }
      g.visible = vis > 0; mini.classList.toggle("on", gone && vis === 0);
      head.rotation.x += (-.32 - head.rotation.x) * Math.min(1, dt * 3);
      head.rotation.z = talking > 0 ? Math.sin(t * 28) * .06 : Math.sin(t * 1.3) * .02; head.position.x = Math.sin(t * .9) * .002;
      blink -= dt; if (blink < -3.4 - Math.random()) blink = .11; var sy = blink > 0 ? .15 : 1; eyes.forEach(function (e) { e.scale.y += (sy - e.scale.y) * Math.min(1, dt * 30); });
      if (!live) { hide(); return; }
      // rope: the top end is the Pino's boot (it travels with the camera), the bottom one is the DMX OUT
      var t0 = performance.now();
      boot.copy(BOOT); head.localToWorld(boot);
      if (ropeOn) {
      /* ponytail: the rest length follows the distance (with the SLACK margin to give a catenary).
         A cable of fixed length would stretch straight in the SHOW view, where the camera is 2 m from the device,
         or would pile up half a metre of rope in the REAR view. Swap it for a fixed length the day the
         Pino can be dragged around the screen. */
      // GRAV is smaller than 9.8: a 20 cm DMX cable is stiff, not a bicycle chain. With real g
      // and the same slack the cable dived 130 px and left through the bottom edge of the screen.
      var want = Math.max(.008, boot.distanceTo(END) * SLACK / (N - 1));
      rest += (want - rest) * Math.min(1, dt * 2);
      Rope.pin(P, 0, boot.x, boot.y, boot.z); Rope.pin(P, N - 1, END.x, END.y, END.z);
      Rope.step(P, rest, Math.min(dt, .033), GRAV, 3, 4); retube();
      ms += performance.now() - t0; msN++; }
      v.set(0, .014, 0); head.localToWorld(v); v.project(cam); var sx = (v.x + 1) / 2 * W, sy2 = (1 - v.y) / 2 * H; if (mouse) { var dx = mouse[0] - sx, dy = mouse[1] - sy2, dd = Math.max(1, Math.hypot(dx, dy)), k = Math.min(1, dd / 200) * .0028; pupils.forEach(function (p) { p.position.x = dx / dd * k; p.position.y = -dy / dd * k; }); }
      /* The balloon opens to the RIGHT of the head (the Pino moved to the left corner) and never on top
         of the device: `minTop` is the base of the rear panel on screen, and the balloon stays from there down — otherwise
         it covers the display and the controls, which are the reason the program exists. Since the Pino is now
         at the bottom, the balloon rises until it fits entirely on screen instead of leaving through the bottom edge. */
      v.set(0, .04, .012); head.localToWorld(v); v.project(cam);
      var bh = bal.offsetHeight || 120, hy = (1 - v.y) / 2 * H;
      bal.style.left = Math.min(W - 262, BALX) + "px";
      bal.style.top = Math.max(8, Math.min(H - bh - 10, Math.max(hy - bh - 12, Math.max(0, minTop || 0)))) + "px"; }

    // average cost of the rope since the last reading, in ms per frame (error is data: it can be measured on screen)
    function cost() { var r = msN ? ms / msN : 0; ms = 0; msN = 0; return r; }
    return { group: g, head: head, say: say, hide: hide, current: current, bye: bye, back: back, update: update, cost: cost, alive: function () { return live; },
      rope: function (on) { ropeOn = !!on; tube.visible = !!on && !gone; plug.visible = relief.visible = !!on && !gone; }, PINS: PINS }; }
  return { build: build, PINS: PINS };
})();
