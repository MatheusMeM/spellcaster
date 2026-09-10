/* Room, flightcase and the body of the projector (400 × 180 × 300 mm): plates, side fins, yoke, feet, lid with a
   real hinge and screws, aperture at the front and the complete rear panel (inspired by the Clubmax:
   powerCON + rocker, key switch, a single interlock, emission LED, big ARMED LED, ILDA IN/OUT DB25,
   DMX IN/OUT in XLR-3 Neutrik D-shell, NET, rack display + encoder + BACK, 60 mm axial fan with the
   impeller drawn in a texture behind the wire grille, danger label and identification plate).
   Coordinates in metres, origin at the bottom of the body.

   Shape references (nothing is downloaded; everything is modelled with the primitives of mat.js):
   knob        Davies 1900H / Bourns PEC11 — fluted conical body, chrome collar, indicator in a groove
   DMX IN/OUT  Neutrik NC3FD-L-1 (female) and NC3MD-L-1 (male) — 24 mm D-shell, 2 holes on the diagonal, 3 pins at 120°
   fan         Sunon/Delta 60 × 60 × 25 mm — 7 blades, wire grille of 4 rings
   label       IEC 60825-1 — yellow field, black frame and the laser radiation pictogram

   ONE CONTROL, ONE FUNCTION. The classification of each clickable part is data, not convention: it lives in
   `LaserEngine.CONTROLS` (engine.js), it is where the tooltip label comes from and it is what `app.js` reads to
   decide what the click does. No part does two things, and no function has two places.

   | family      | what the click does             | parts |
   |-------------|---------------------------------|-------|
   | toggle      | flips a state, and that is all   | `power` (rocker), `keyswitch`, `interlock` |
   | momentary   | acts while held                  | — (none today) |
   | value       | changes a number                 | the encoder turning (mouse wheel or a drag over it) and the panel faders |
   | connector   | nothing: it is a plug            | `acin` (powerCON) |
   | nav         | opens the screen of that part    | `enc`, `back`, ports (`ilda`, `ildathru`, `dmxin`, `dmxout`, `rj45`), `lid`, `fan`, and the parts inside |

   The display does not enter the raycast: a display is not a control. Power turns on at the rocker only (the
   powerCON is a plug), the emission arms at the key switch only, and there is one interlock — the shutter in
   `optics.js` reacts to it, but that is indication. */
window.BODY = function (THREE, X, scene, pick) {
  "use strict";
  var m = X.m, PI = Math.PI, LBL = LaserEngine.labelOf;
  function add(parent, g, mat, x, y, z, k, label) { var o = new THREE.Mesh(g, mat); o.position.set(x, y, z); o.castShadow = o.receiveShadow = true; if (k) { o.userData = { key: k, label: label || LBL(k) }; pick.push(o); } parent.add(o); return o; }
  function cyl(r, h, seg) { return new THREE.CylinderGeometry(r, r, h, seg || 20); }
  function zcyl(parent, r, h, mat, x, y, z, k, label) { var o = add(parent, cyl(r, h), mat, x, y, z, k, label); o.rotation.x = PI / 2; return o; }
  // the largest type size that fits in `w` px: the font the canvas uses may not be the one asked for, so what
  // decides the size is measureText, not a constant. That is what made the text overflow the plates.
  function fit(x, lines, w, px, font) { for (; px > 6; px--) { x.font = "700 " + px + "px " + font; var ok = true; for (var i = 0; i < lines.length; i++) if (x.measureText(lines[i]).width > w) ok = false; if (ok) break; } return px; }
  function textW(x, lines) { var w = 0; lines.forEach(function (l) { w = Math.max(w, x.measureText(l).width); }); return w; }
  // the brand vector (brand/*.svg) drawn on the canvas: it arrives later, so the caller marks needsUpdate
  function svgInto(src, draw) { var im = new Image(); im.onload = function () { draw(im); }; im.src = src; return im; }
  function svgTex(src, cw, ch, alpha) { var c = document.createElement("canvas"); c.width = cw; c.height = ch; var t = new THREE.CanvasTexture(c); t.encoding = THREE.sRGBEncoding; svgInto(src, function (im) { var x = c.getContext("2d"); x.globalAlpha = alpha || 1; x.drawImage(im, 0, 0, cw, ch); t.needsUpdate = true; }); return t; }
  /* room */
  var floor = add(scene, new THREE.PlaneGeometry(14, 14), m.floor, 0, 0, -2); floor.rotation.x = -PI / 2; floor.castShadow = false;
  var wall = add(scene, new THREE.PlaneGeometry(8, 5), m.wall, 0, 2.2, -5.01); wall.castShadow = false;
  var wallLight = new THREE.PointLight(0x6f86b0, 14, 7, 2); wallLight.position.set(0, 2.6, -3.2); scene.add(wallLight);
  /* flightcase: box, corner pieces on the eight corners and rivets along the edges */
  add(scene, X.rbox(.62, .3, .46, .004), m.caseM, 0, .15, 0);
  var rivet = new THREE.SphereGeometry(.004, 10, 8);   // ponytail: the corner piece is the corner profile itself; a corner cube sunk into the plate looked like a white sticker
  [-.29, .29].forEach(function (x) { [-.21, .21].forEach(function (z) { add(scene, X.rbox(.03, .3, .03, .003), m.silver, x, .15, z); }); });
  [-.2, 0, .2].forEach(function (x) { [-.22, .22].forEach(function (z) { add(scene, rivet, m.steel, x, .29, z * 1.01); }); });
  [-.235, -.08, .08, .235].forEach(function (z) { [-.305, .305].forEach(function (x) { add(scene, rivet, m.steel, x, .29, z); }); });
  /* body */
  var body = new THREE.Group(); body.position.y = .314; scene.add(body);
  var W = .40, H = .18, D = .30, T = .006;
  add(body, X.rbox(W, T, D, .004), m.alu, 0, T / 2, 0);
  add(body, X.rbox(W, H, T, .004), m.alu, 0, H / 2, -D / 2 + T / 2, "front");
  var rearPlate = add(body, X.rbox(W, H, T, .004), m.alu, 0, H / 2, D / 2 - T / 2);
  add(body, X.rbox(T, H, D, .004), m.alu, -W / 2 + T / 2, H / 2, 0); add(body, X.rbox(T, H, D, .004), m.alu, W / 2 - T / 2, H / 2, 0, "side");
  // ponytail: a fin is a box. rbox here costs ~7 k triangles for a 1 mm radius that no view shows.
  var finG = new THREE.BoxGeometry(.006, .0025, .24);
  for (var i = 0; i < 8; i++) [-1, 1].forEach(function (s) { add(body, finG, m.alu, s * (W / 2 + .002), .03 + i * .016, -.01); });
  // yoke with handles, and rubber feet
  add(body, X.rbox(.46, .008, .035, .002), m.dark, 0, -.010, 0); [-1, 1].forEach(function (s) { add(body, X.rbox(.008, .14, .035, .002), m.dark, s * .234, .06, 0); var k = add(body, cyl(.016, .012, 16), new THREE.MeshStandardMaterial({ color: 0x1e2226, metalness: .6, roughness: .5, flatShading: true }), s * .244, .10, 0); k.rotation.z = PI / 2; var sc = add(body, cyl(.004, .002, 12), m.silver, s * .251, .10, 0); sc.rotation.z = PI / 2; });
  [-.2, .2].forEach(function (x) { [-.09, .09].forEach(function (z) { add(body, cyl(.009, .006, 16), m.rubber, x, -.017, z); }); });
  /* lid with a real hinge.

     `app.js` opens the lid with `B.lid.rotation.x = -lT * 1.9` — a NEGATIVE angle. With the hinge at the rear
     (as it was) the negative throws the plate DOWN and backwards: it went through the rear panel and the
     flightcase along the whole path, and that is what the owner saw clipping. The right pivot for that sign is
     the front one (z = −D/2): the lid rises and tips forward, away from the inside camera (which is behind, at
     z > 0). The pivot sits OUTSIDE the plate, at the centre of the radius of the front edge (r = 4.5 mm), and
     that is why the lid turns without sweeping the corner of the body: no angle between 0 and −1.9 rad goes
     through anything, and there is no need to limit the travel. The screws are children of the lid and rise
     with it. */
  var HR = .0045, lid = new THREE.Group(); lid.position.set(0, H + T / 2, -D / 2 + HR); body.add(lid);
  var lidM = add(lid, X.rbox(W, T, D - HR, .004), m.alu, 0, 0, (D - HR) / 2, "lid");
  [-.13, .13].forEach(function (x) { var b = add(lid, cyl(.004, .05, 14), m.steel, x, 0, 0); b.rotation.z = PI / 2;   // barrel
    add(lid, X.rbox(.05, .0016, .014, .0006), m.steel, x, .0036, .010);                                              // leaf on the lid
    add(body, X.rbox(.05, .012, .0016, .0006), m.steel, x, H - .009, -D / 2 - .0011); });                            // leaf on the body
  var screws = []; [[-.18, .0455], [.18, .0455], [-.18, .1405], [.18, .1405], [-.18, .2455], [.18, .2455]].forEach(function (p) { var s = add(lid, cyl(.0045, .004, 14), m.steel, p[0], .004, p[1], "lid"); add(s, cyl(.0034, .0007, 14), m.dark, 0, .0019, 0); var sk = add(s, X.hex(.0021, .0022), m.black, 0, .0022, 0); sk.rotation.y = .3; screws.push(s); });
  // danger label on the lid: the yellow field IS the whole canvas, and the text is measured to fit in it
  var lbl = X.tex(512, 256, function (x, w, h) {
    x.fillStyle = "#FFB000"; x.fillRect(0, 0, w, h); x.fillStyle = "#000"; x.fillRect(0, 0, w, 9); x.fillRect(0, h - 9, w, 9); x.fillRect(0, 0, 9, h); x.fillRect(w - 9, 0, 9, h);
    var cx = 86, cy = 128, r = 60;   // IEC 60825-1 pictogram: triangle with the beam radiating
    x.beginPath(); x.moveTo(cx, cy - r); x.lineTo(cx + r * .92, cy + r * .72); x.lineTo(cx - r * .92, cy + r * .72); x.closePath(); x.lineWidth = r * .13; x.strokeStyle = "#000"; x.lineJoin = "round"; x.stroke();
    x.save(); x.translate(cx - r * .36, cy + r * .30); x.fillStyle = "#000"; x.beginPath(); x.arc(0, 0, r * .10, 0, 7); x.fill();
    for (var j = 0; j < 6; j++) { x.save(); x.rotate(-.62 + j * .21); x.fillRect(r * .13, -r * .028, r * .78, r * .056); x.restore(); } x.restore();
    var lines = ["DANGER · LASER RADIATION", "AVOID EXPOSURE TO BEAM", "OUTPUT 10 W · 445-638 nm", "CLASS 4 LASER PRODUCT"], px = fit(x, lines, 318, 32, "'Share Tech Mono'");
    x.textAlign = "left"; x.fillStyle = "#000"; lines.forEach(function (l, j2) { x.fillText(l, 170, 72 + j2 * (px + 13)); });
  }, true);
  /* The bevel of the ExtrudeGeometry grows OUTWARDS: X.rbox(w, h, d, r) measures h + 2r in height, so the top
     face of the lid is at T / 2 + r, not at T / 2. With y = .0032 both labels sat 2.8 mm INSIDE the plate —
     invisible from the top view. Half a millimetre of clearance is what is left for the sticker. */
  var LBLY = T / 2 + .003 + .0005;
  var lab = add(lid, new THREE.PlaneGeometry(.07, .035), new THREE.MeshStandardMaterial({ map: lbl, metalness: 0, roughness: .62 }), -.12, LBLY, .0955); lab.rotation.x = -PI / 2; lab.castShadow = false;
  var sub = add(lid, new THREE.PlaneGeometry(.042, .0127), new THREE.MeshStandardMaterial({ map: svgTex("brand/submark.svg", 512, 155, .45), transparent: true, metalness: .7, roughness: .45 }), .14, LBLY, .2355); sub.rotation.x = -PI / 2; sub.castShadow = false;
  // front: aperture with a window
  var BEAM_Y = .057; add(body, X.rbox(.04, .026, .004, .001), m.black, .095, BEAM_Y, -D / 2 - .001, "aperture");
  var apGlass = add(body, new THREE.PlaneGeometry(.03, .018), m.glassDark, .095, BEAM_Y, -D / 2 - .0035); apGlass.rotation.y = PI; apGlass.castShadow = false;
  var APERT = new THREE.Vector3(.095, .314 + BEAM_Y, -D / 2 - .005);
  /* rear panel */
  var Z = D / 2 + .0003, py = function (y) { return H / 2 + y; };
  function port(x, y, k, label) { var g = new THREE.Group(); g.position.set(x, py(y), Z); if (k) { g.userData = { key: k, label: label || LBL(k) }; pick.push(g); } body.add(g); return g; }
  /* Panel layout: the rack display takes the central band (200 × 100 mm — half the width of the panel), power
     and safety are in the left column, encoder and fan on the right, and every port drops to the bottom rail.
     The display went up 10 mm and the rail went down 4 mm: the frame is 5 mm deep and it ate the port labels as
     soon as the camera left the axis. The USB left the device. */
  var L = { title: [-.19, .078], brand: [.19, .078], rock: [-.180, .048], key: [-.180, -.004], lock: [-.145, .048], em: [-.145, .010], armed: [-.145, -.030],
    oled: [-.012, .018], enc: [.108, .040], back: [.108, -.010], fan: [.158, .014],
    pcon: [-.175, -.070], ildaIn: [-.108, -.070], ildaOut: [-.043, -.070], dmxIn: [.005, -.070], dmxOut: [.040, -.070], net: [.072, -.066],
    warn: [.152, -.052], sn: [.152, -.078] };
  var U = function (p) { return (p[0] + .2) * 5120; }, V = function (p) { return (.09 - p[1]) * 5111; };
  var PLATE = { w: 300, h: 86 };   // area of the identification plate on the silkscreen canvas; B.plate redraws only it
  /* The silkscreen is drawn twice: in colour, and in grey for the roughness map — ink is more matte than the
     brushed plate, and it is that relief that makes the silkscreen look printed instead of glued on. */
  function silkDraw(x, rough) {
    var ink = rough ? "#e8e8e8" : "#c9ced3", base = rough ? "#3c3c3c" : "#101214";
    x.fillStyle = base; x.fillRect(0, 0, 2048, 920);
    x.fillStyle = ink; x.textAlign = "left"; x.font = "700 58px Michroma"; x.fillText("SPELLCASTER", U(L.title), V(L.title) + 20);
    x.textAlign = "center"; x.font = "700 34px 'Share Tech Mono'";
    [["POWER", L.rock, -.024], ["AC 100-240 V", L.pcon, .022], ["KEY", L.key, -.019], ["INTERLOCK", L.lock, .013], ["EMISSION", L.em, -.011], ["ARMED", L.armed, .011],
      ["ILDA IN", L.ildaIn, .017], ["ILDA OUT", L.ildaOut, .017], ["DMX IN", L.dmxIn, .022], ["DMX OUT", L.dmxOut, .022], ["NET", L.net, .015],
      ["MENU", L.enc, -.019], ["BACK", L.back, -.014], ["FAN", L.fan, -.040]].forEach(function (l) { x.fillText(l[0], U(l[1]), V([l[1][0], l[1][1] + l[2]]) + 12); });
    // danger label: the yellow rectangle is sized BY the text, and the text shrinks until it fits in 48 mm
    var wl = ["DANGER · LASER RADIATION", "AVOID EXPOSURE TO BEAM", "CLASS 4 · 10 W · 445-638 nm"], px = fit(x, wl, 290, 24, "'Share Tech Mono'"), ww = textW(x, wl) + 22, wh = wl.length * (px + 7) + 12;
    x.fillStyle = rough ? "#8c8c8c" : "#FFB000"; x.fillRect(U(L.warn) - ww / 2, V(L.warn) - wh / 2, ww, wh);
    x.fillStyle = rough ? "#dcdcdc" : "#000"; wl.forEach(function (l, j) { x.fillText(l, U(L.warn), V(L.warn) - wh / 2 + 12 + j * (px + 7) + px * .1); });
    x.fillStyle = rough ? "#565656" : "#2a2e33"; x.fillRect(U(L.sn) - PLATE.w / 2, V(L.sn) - PLATE.h / 2, PLATE.w, PLATE.h);
  }
  var silkT = X.tex(2048, 920, function (x) { silkDraw(x, false); }, true);
  var silkR = X.tex(1024, 460, function (x) { x.scale(.5, .5); silkDraw(x, true); });
  var sx = silkT.image.getContext("2d");
  /* Identification plate: FIRMWARE V<version> comes from the engine (`hud-4` calls `B.plate` on connecting);
     without an engine the plate says FIRMWARE OFFLINE, because an invented version is a lie engraved on the
     chassis. The 180 W / 50-60 Hz are gone: a number copied from someone else's datasheet serves no function of
     the software. */
  function plate(o) {
    o = o || {}; var fw = o.fw === undefined ? "0.1.2" : o.fw, sn = o.sn === undefined ? "SC-0512" : o.sn;
    var lines = [fw ? "FIRMWARE V" + String(fw).replace(/^[vV]/, "") : "FIRMWARE OFFLINE", sn ? "S/N " + sn : "", "IEC 60825-1 · MADE IN BRAZIL"].filter(Boolean);
    sx.save(); sx.fillStyle = "#2a2e33"; sx.fillRect(U(L.sn) - PLATE.w / 2, V(L.sn) - PLATE.h / 2, PLATE.w, PLATE.h);
    var px = fit(sx, lines, PLATE.w - 22, 22, "'Share Tech Mono'");
    sx.textAlign = "center"; sx.fillStyle = "#c9ced3";
    lines.forEach(function (l, j) { sx.fillText(l, U(L.sn), V(L.sn) - PLATE.h / 2 + 8 + j * (px + 6) + px * .85); });
    sx.restore(); silkT.needsUpdate = true;
  }
  plate();
  // the brand vector in place of the text "FEITIÇARIA iNDUSTRIAL" (and "LASER 10 W RGB · … · 40 kpps" is gone for good)
  svgInto("brand/wordmark.svg", function (im) { var h = 46, w = h * (im.width && im.height ? im.width / im.height : 7.18); sx.drawImage(im, U(L.brand) - w, V(L.brand) - h / 2 + 4, w, h); silkT.needsUpdate = true; });
  var silk = add(body, new THREE.PlaneGeometry(W, H), new THREE.MeshStandardMaterial({ map: silkT, metalness: .85, roughness: 1, roughnessMap: silkR, normalMap: X.brushN, normalScale: new THREE.Vector2(.2, .2) }), 0, H / 2, Z); silk.castShadow = false;
  function screws4(g, s, r) { [[-s, -s], [s, -s], [-s, s], [s, s]].forEach(function (p) { zcyl(g, r || .0015, .002, m.silver, p[0], p[1], .002); }); }
  // powerCON: a plug only (power turns on at the rocker, and nowhere else); rocker: on/off only
  var pc = port(L.pcon[0], L.pcon[1], "acin"); add(pc, X.rbox(.032, .032, .003, .001), m.black, 0, 0, .0015); zcyl(pc, .012, .012, m.black, 0, 0, .006); zcyl(pc, .0095, .003, m.steel, 0, 0, .0125); add(pc, new THREE.BoxGeometry(.004, .003, .004), m.black, 0, .008, .012); screws4(pc, .013);
  var rk = port(L.rock[0], L.rock[1], "power"); add(rk, X.rbox(.022, .03, .003, .001), m.black, 0, 0, .0015);
  // the rocker tilts on the top edge of the frame, like a real rocker: the group is the axis
  var rocker = new THREE.Group(); rocker.position.z = .002; rk.add(rocker); add(rocker, X.rbox(.014, .022, .005, .001), m.plastic, 0, 0, .0035);
  // key switch, interlock (a single one), emission LED and the big ARMED LED
  var ks = port(L.key[0], L.key[1], "keyswitch"); zcyl(ks, .011, .003, m.silver, 0, 0, .0015); zcyl(ks, .007, .006, m.black, 0, 0, .005); var keyM = add(ks, X.rbox(.003, .022, .014, .001), m.brass, 0, 0, .013);
  var il = port(L.lock[0], L.lock[1], "interlock"); var nut = add(il, X.hex(.0075, .003), m.silver, 0, 0, .0015); nut.rotation.x = PI / 2; zcyl(il, .0045, .004, m.black, 0, 0, .004); var lockPlug = new THREE.Group(); il.add(lockPlug); zcyl(lockPlug, .006, .016, m.plastic, 0, 0, .012); var loop = add(lockPlug, new THREE.TorusGeometry(.007, .0015, 8, 20), m.rubber, 0, -.007, .02); loop.rotation.y = PI / 2;
  var emLed = zcyl(body, .0025, .003, m.led(0x2a0a08), L.em[0], py(L.em[1]), Z + .0015);
  /* 9 mm ARMED LED: dome, chrome collar and an additive halo — the armed indicator has to be visible in the 3D
     model from any distance, not only in the HUD. `hud-4` paints `B.armLed.material.color` (SISTEMA.md §5: dark
     with no power, blinking red when disarmed, green when armed) and the halo uses the SAME Color instance, so
     it follows without anyone painting twice. */
  var armMat = new THREE.MeshBasicMaterial({ color: 0x2a0a08 });
  zcyl(body, .0062, .0035, m.silver, L.armed[0], py(L.armed[1]), Z + .0015);
  zcyl(body, .0045, .004, armMat, L.armed[0], py(L.armed[1]), Z + .002);
  var armLed = add(body, new THREE.SphereGeometry(.0045, 18, 12, 0, PI * 2, 0, PI / 2), armMat, L.armed[0], py(L.armed[1]), Z + .004); armLed.rotation.x = PI / 2; armLed.castShadow = false;
  var haloT = X.tex(64, 64, function (x, w) { var g = x.createRadialGradient(w / 2, w / 2, 0, w / 2, w / 2, w / 2); g.addColorStop(0, "rgba(255,255,255,1)"); g.addColorStop(.35, "rgba(255,255,255,.35)"); g.addColorStop(1, "rgba(255,255,255,0)"); x.fillStyle = g; x.fillRect(0, 0, w, w); });
  var halo = new THREE.Sprite(new THREE.SpriteMaterial({ map: haloT, blending: THREE.AdditiveBlending, depthWrite: false, transparent: true, opacity: .8 }));
  halo.material.color = armMat.color; halo.scale.set(.034, .034, 1); halo.position.set(L.armed[0], py(L.armed[1]), Z + .004); body.add(halo);
  // ILDA IN/OUT (DB25)
  function db25(p, k, label, male) { var g = port(p[0], p[1], k, label); add(g, X.rbox(.055, .014, .004, .002), m.silver, 0, 0, .002); add(g, X.rbox(.046, .008, .002, .001), male ? m.silver : m.black, 0, 0, .0045); if (male) for (var j = 0; j < 13; j++) { zcyl(g, .0005, .002, m.brass, -.021 + j * .0035, .0018, .0055); if (j < 12) zcyl(g, .0005, .002, m.brass, -.0192 + j * .0035, -.0018, .0055); } [-.028, .028].forEach(function (x) { var s = add(g, X.hex(.0025, .005), m.silver, x, 0, .004); s.rotation.x = PI / 2; }); return g; }
  db25(L.ildaIn, "ilda", "", false); db25(L.ildaOut, "ildathru", "", true);
  /* DMX512 on XLR-3 (that is what this device uses; XLR-5 belongs to the standard, not to this chassis).
     Neutrik D-shell: 24 mm flange with two holes on the diagonal, 19.6 mm collar, contacts at 120° with pin 1
     at the top left seen from the front. The black insert with "1 2 3" is a single texture, shared by both
     connectors. */
  var PIN = [[150, "1"], [30, "2"], [270, "3"]], PINR = .0042;
  function insTex(male) { return X.tex(256, 256, function (x, w) { var c = w / 2, s = c / .0095;
    x.fillStyle = "#0b0c0e"; x.beginPath(); x.arc(c, c, c - 2, 0, 7); x.fill();
    x.textAlign = "center"; x.font = "700 20px 'Share Tech Mono'";
    PIN.forEach(function (p) { var a = p[0] * PI / 180, px = c + Math.cos(a) * PINR * s, pz = c - Math.sin(a) * PINR * s;
      x.fillStyle = male ? "#2b2e31" : "#000"; x.beginPath(); x.arc(px, pz, .0019 * s, 0, 7); x.fill();
      x.fillStyle = "#8b9298"; x.fillText(p[1], px + Math.cos(a) * .0030 * s, pz - Math.sin(a) * .0030 * s + 7); }); }, true); }
  var insM = new THREE.MeshStandardMaterial({ map: insTex(true), metalness: .6, roughness: .55 }), insF = new THREE.MeshStandardMaterial({ map: insTex(false), metalness: .6, roughness: .55 });
  function xlr3(p, k, label, male) {
    var g = port(p[0], p[1], k, label);
    add(g, X.rbox(.024, .024, .0026, .0032), m.black, 0, 0, .0013);                                  // flange D-shell
    [[-.0088, .0088], [.0088, -.0088]].forEach(function (h) { zcyl(g, .0017, .0028, m.dark, h[0], h[1], .0014); var s = add(g, X.hex(.0016, .0014), m.silver, h[0], h[1], .0033); s.rotation.x = PI / 2; });
    zcyl(g, .0098, .0092, m.silver, 0, 0, .0046);                                                    // collar
    zcyl(g, .0093, .0012, m.dark, 0, 0, .0098);                                                      // chamfer of the collar
    var ins = add(g, new THREE.CircleGeometry(.0088, 28), male ? insM : insF, 0, 0, .0105); ins.castShadow = false;
    PIN.forEach(function (q) { var a = q[0] * PI / 180, cx = Math.cos(a) * PINR, cy = Math.sin(a) * PINR;
      if (male) zcyl(g, .00115, .0062, m.silver, cx, cy, .0135); else zcyl(g, .0018, .0016, m.black, cx, cy, .0099); });
    if (!male) add(g, X.rbox(.006, .0035, .0045, .0008), m.black, 0, .0072, .0075);                  // latch of the NC3FD
    return g;
  }
  xlr3(L.dmxIn, "dmxin", "", false); var dmxOut = xlr3(L.dmxOut, "dmxout", "", true);
  // NET
  var rj = port(L.net[0], L.net[1], "rj45"); add(rj, X.rbox(.017, .014, .003, .001), m.black, 0, 0, .0015); add(rj, new THREE.BoxGeometry(.012, .008, .002), m.plastic, 0, -.001, .003); var led1 = zcyl(rj, .001, .002, m.led(0x0a2a10), -.006, .0055, .003), led2 = zcyl(rj, .001, .002, m.led(0x2a1e00), .006, .0055, .003);
  /* Rack display: a 200 × 100 mm frame, 190 × 90 mm of glass, a 1024 × 484 texture — resolution enough for the
     big line to be read from the `rear` view without zoom. It does not go into `pick`: a display is not a
     control, and a hand cursor over what does not click is a lie. */
  var OC = document.createElement("canvas"); OC.width = 1024; OC.height = 484; var oc = OC.getContext("2d"), oledTex = new THREE.CanvasTexture(OC); oledTex.minFilter = THREE.LinearFilter; oledTex.anisotropy = 8;
  var og = new THREE.Group(); og.position.set(L.oled[0], py(L.oled[1]), Z); body.add(og);
  add(og, X.rbox(.20, .10, .005, .002), m.black, 0, 0, .0025); add(og, X.rbox(.194, .094, .002, .001), m.dark, 0, 0, .0045);
  var oledM = add(og, new THREE.PlaneGeometry(.19, .09), new THREE.MeshBasicMaterial({ map: oledTex }), 0, 0, .0056); oledM.castShadow = false;
  /* Davies 1900H style encoder: hex bushing, D shaft visible in the gap, chrome collar, conical body with 18
     flutes (an extruded polar Shape — a cylinder with flatShading does not make flutes) and the white indicator
     in the groove, from the top down to the skirt. `app.js` turns it on `rotation.y` (the axis of the cylinder,
     because the group already comes with rotation.x = π/2) and sinks it on `position.z`. `B.knobHit` is the
     big invisible target of the `cam-4` drag. */
  var eg = port(L.enc[0], L.enc[1], "enc");
  var bush = add(eg, X.hex(.0055, .0035), m.silver, 0, 0, .0018); bush.rotation.x = PI / 2;
  zcyl(eg, .0028, .006, m.silver, 0, 0, .0045);
  add(eg, new THREE.BoxGeometry(.0009, .0056, .006), m.dark, .0023, 0, .0045);                       // the flat of the "D" shaft
  var knob = new THREE.Group(); knob.rotation.x = PI / 2; knob.position.set(0, 0, .009); eg.add(knob);
  var sh = new THREE.Shape(), SEG = 96;
  for (i = 0; i <= SEG; i++) { var th = i / SEG * PI * 2, rr = .0080 + .00055 * Math.cos(18 * th); if (i) sh.lineTo(Math.cos(th) * rr, Math.sin(th) * rr); else sh.moveTo(Math.cos(th) * rr, Math.sin(th) * rr); }
  var kg = new THREE.ExtrudeGeometry(sh, { depth: .0085, bevelEnabled: true, bevelThickness: .0009, bevelSize: .0009, bevelSegments: 2 }); kg.rotateX(-PI / 2);
  add(knob, kg, new THREE.MeshStandardMaterial({ color: 0x121417, metalness: .35, roughness: .48, roughnessMap: X.grainR, envMapIntensity: .5 }), 0, -.0042, 0);
  add(knob, cyl(.0092, .0018, 36), m.silver, 0, -.0051, 0);                                          // chrome collar
  add(knob, cyl(.0068, .0009, 32), m.black, 0, .0047, 0);                                            // cap with a chamfer
  add(knob, new THREE.BoxGeometry(.0014, .0011, .0056), m.white, 0, .0047, .0038);                   // indicator on the top
  add(knob, new THREE.BoxGeometry(.0014, .0072, .0013), m.white, 0, .0005, .0081);                   // indicator on the skirt
  var knobHit = add(eg, cyl(.016, .030, 12), new THREE.MeshBasicMaterial({ transparent: true, opacity: 0, depthWrite: false, colorWrite: false }), 0, 0, .012, "enc"); knobHit.rotation.x = PI / 2; knobHit.castShadow = knobHit.receiveShadow = false;
  var bk = port(L.back[0], L.back[1], "back"); zcyl(bk, .005, .002, m.silver, 0, 0, .001); var backCap = zcyl(bk, .004, .004, m.plastic, 0, 0, .003);
  /* 60 mm axial fan: ring, wire grille (mesh) and the impeller as a spinning TEXTURE. Seven mesh blades turned
     seven objects per frame to become a blur nobody sees standing still; a plane with the blades drawn (with
     the smear already painted in) reads the same for 2 triangles. `B.blades` is still the group that `app.js`
     turns on `rotation.z`. */
  // the frame is of the same plate as the panel (m.alu), not black plastic: with metalness 0 a 4 % albedo under
  // the room reflector comes out LIGHT GREY, and the fan square was the brightest part of the whole panel
  var fg = port(L.fan[0], L.fan[1], "fan"); add(fg, X.ring(.060, .029, .004), m.alu, 0, 0, .002); screws4(fg, .026, .0018);
  var fanT = X.tex(256, 256, function (x, w) { var c = w / 2;
    for (var b = 0; b < 7; b++) for (var s = 0; s < 5; s++) { x.save(); x.translate(c, c); x.rotate(b * PI * 2 / 7 + s * .04);
      x.beginPath(); x.moveTo(30, -7); x.quadraticCurveTo(76, -30, 112, -5); x.quadraticCurveTo(94, 22, 33, 16); x.closePath();
      x.fillStyle = "rgba(19,22,25," + (s ? .10 : .95) + ")"; x.fill(); x.restore(); }
    var g = x.createRadialGradient(c, c, 3, c, c, 34); g.addColorStop(0, "#454a50"); g.addColorStop(1, "#16181b");
    x.beginPath(); x.arc(c, c, 34, 0, 7); x.fillStyle = g; x.fill();
    x.beginPath(); x.arc(c, c, 21, 0, 7); x.fillStyle = "#0c0e10"; x.fill();
    x.beginPath(); x.arc(c, c, 5, 0, 7); x.fillStyle = "#6a7076"; x.fill(); }, true);
  var blades = new THREE.Group(); blades.position.z = .005; fg.add(blades);
  var bl = add(blades, new THREE.PlaneGeometry(.058, .058), new THREE.MeshStandardMaterial({ map: fanT, transparent: true, depthWrite: false, metalness: .6, roughness: .55 }), 0, 0, 0); bl.castShadow = bl.receiveShadow = false;
  [.0085, .0155, .0225, .0295].forEach(function (r) { add(fg, new THREE.TorusGeometry(r, .0006, 6, 40), m.silver, 0, 0, .0075); }); for (i = 0; i < 4; i++) { var sp = add(fg, cyl(.0006, .06, 6), m.silver, 0, 0, .0075); sp.rotation.z = i * PI / 4; }
  var rearLight = new THREE.SpotLight(0xfff4e6, 0, 2.5, .6, .6, 1.2); rearLight.position.set(.3, .95, 1.1); rearLight.target = rearPlate; scene.add(rearLight);
  // centre of the DMX OUT face in world coordinates: it is where the Pino cable comes out of (contract with `pino-4`)
  body.updateMatrixWorld(true);
  var dmxOutWorld = dmxOut.localToWorld(new THREE.Vector3(0, 0, .0115));
  return { body: body, lid: lid, screws: screws, APERT: APERT, BEAM_Y: BEAM_Y, keyM: keyM, lockPlug: lockPlug, rocker: rocker, emLed: emLed, armLed: armLed, led1: led1, led2: led2, knob: knob, knobHit: knobHit, backCap: backCap, blades: blades, oled: { c: oc, tex: oledTex, w: 1024, h: 484 }, dmxOut: dmxOut, dmxOutWorld: dmxOutWorld, plate: plate, rearLight: rearLight, wallLight: wallLight, L: L, H: H, D: D, W: W, T: T };
};
