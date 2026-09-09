/* Renderer, ambiente PMREM procedural, texturas procedurais e materiais do aparelho. Tudo sem asset externo. */
window.MAT = function (THREE, R) {
  "use strict";
  R.physicallyCorrectLights = true; R.outputEncoding = THREE.sRGBEncoding; R.toneMapping = THREE.ACESFilmicToneMapping; R.toneMappingExposure = .95; R.shadowMap.enabled = true; R.shadowMap.type = THREE.PCFSoftShadowMap;
  function env() { var es = new THREE.Scene(); es.add(new THREE.Mesh(new THREE.BoxGeometry(10, 10, 10), new THREE.MeshBasicMaterial({ color: 0x0a0c10, side: THREE.BackSide })));
    [[0, 4.9, 0, 3, 1.2, 1.5708, 0, 0xffffff, 3], [-4.9, 2, -1, 1, 3, 0, 1.5708, 0xdfe8ff, 2.5], [4.9, 2.5, 1, 1.4, 2.4, 0, -1.5708, 0xffe6cc, 1.6], [0, 1.5, 4.9, 2, .6, 0, 3.1416, 0x38ff5c, .5]].forEach(function (p) { var m = new THREE.Mesh(new THREE.PlaneGeometry(p[3], p[4]), new THREE.MeshBasicMaterial({ color: new THREE.Color(p[7]).multiplyScalar(p[8]) })); m.position.set(p[0], p[1], p[2]); m.rotation.set(p[5], p[6], 0); es.add(m); });
    var pm = new THREE.PMREMGenerator(R), t = pm.fromScene(es, .04).texture; pm.dispose(); return t; }
  function tex(w, h, fn, sRGB) { var c = document.createElement("canvas"); c.width = w; c.height = h; var x = c.getContext("2d"); fn(x, w, h); var t = new THREE.CanvasTexture(c); t.wrapS = t.wrapT = THREE.RepeatWrapping; t.anisotropy = 8; if (sRGB) t.encoding = THREE.sRGBEncoding; return t; }
  function noise(w, h, fn) { return tex(w, h, function (x) { var im = x.createImageData(w, h), d = im.data; for (var i = 0; i < w * h; i++) { var v = fn(i % w, Math.floor(i / w)); d[i * 4] = v[0]; d[i * 4 + 1] = v[1]; d[i * 4 + 2] = v[2]; d[i * 4 + 3] = 255; } x.putImageData(im, 0, 0); }); }
  var rows = []; for (var ri = 0; ri < 256; ri++) rows.push(Math.random());
  var brushN = noise(256, 256, function (x, y) { var n = ((rows[y] * .6 + Math.random() * .4) - .5) * 30; return [128, Math.max(0, Math.min(255, 128 + n)), 255]; }); brushN.repeat.set(3, 3);
  var brushR = noise(256, 256, function (x, y) { var v = 150 + (rows[y] - .5) * 60 + (Math.random() - .5) * 40; return [v, v, v]; }); brushR.repeat.set(3, 3);
  var grainR = noise(256, 256, function () { var v = 120 + Math.random() * 110; return [v, v, v]; }); grainR.repeat.set(20, 20);
  var anoN = noise(256, 256, function () { var v = (Math.random() - .5) * 10; return [128, 128 + v, 255]; }); anoN.repeat.set(8, 8);
  // mesa óptica: furação M4 a cada 12,5 mm (24 × 16 em 300 × 200 mm), textura de cor + rugosidade
  var benchC = tex(1200, 800, function (x, w, h) { x.fillStyle = "#b9bec4"; x.fillRect(0, 0, w, h); for (var i = 0; i < 6000; i++) { x.fillStyle = "rgba(255,255,255," + Math.random() * .08 + ")"; x.fillRect(Math.random() * w, Math.random() * h, 40 + Math.random() * 80, 1); } for (var a = 0; a < 24; a++) for (var b = 0; b < 16; b++) { var cx = 25 + a * 50, cy = 25 + b * 50; x.fillStyle = "#3a3d40"; x.beginPath(); x.arc(cx, cy, 8, 0, 7); x.fill(); x.fillStyle = "#101214"; x.beginPath(); x.arc(cx, cy, 5.5, 0, 7); x.fill(); } }, true);
  // PCB: máscara verde escura, trilhas manhattan, pads, silkscreen discreto
  function pcbTex(seed) { return tex(512, 384, function (x, w, h) { x.fillStyle = "#0b3b22"; x.fillRect(0, 0, w, h); var rnd = function () { seed = (seed * 9301 + 49297) % 233280; return seed / 233280; };
      x.strokeStyle = "#1c6b3c"; x.lineWidth = 3; for (var i = 0; i < 70; i++) { x.beginPath(); var px = rnd() * w, py = rnd() * h; x.moveTo(px, py); for (var j = 0; j < 4; j++) { if (rnd() < .5) px = rnd() * w; else py = rnd() * h; x.lineTo(px, py); } x.stroke(); }
      for (i = 0; i < 160; i++) { var cx = rnd() * w, cy = rnd() * h; x.fillStyle = "#c9cdd1"; x.beginPath(); x.arc(cx, cy, 5, 0, 7); x.fill(); x.fillStyle = "#0b3b22"; x.beginPath(); x.arc(cx, cy, 2.2, 0, 7); x.fill(); }
      x.fillStyle = "#d8dcdf"; x.font = "12px 'Share Tech Mono'"; for (i = 0; i < 26; i++) x.fillText(["R", "C", "U", "Q", "J", "D"][i % 6] + (i + 1), rnd() * w, rnd() * h); x.font = "700 14px 'Share Tech Mono'"; x.fillText("SPELLCASTER  " + (seed % 2 ? "GALVO DRV" : "LASER DRV") + "  v0.1", 12, h - 10); }, true); }
  var M = function (c, o) { return new THREE.MeshStandardMaterial(Object.assign({ color: c, metalness: .5, roughness: .5 }, o || {})); };
  var mats = {
    alu: M(0x0c0d10, { metalness: .85, roughness: .42, normalMap: brushN, normalScale: new THREE.Vector2(.25, .25), roughnessMap: brushR }),
    aluInner: M(0x15171a, { metalness: .8, roughness: .5, normalMap: anoN, normalScale: new THREE.Vector2(.3, .3) }),
    bench: M(0xffffff, { map: benchC, metalness: .9, roughness: .38, envMapIntensity: .5, normalMap: brushN, normalScale: new THREE.Vector2(.15, .15) }),
    silver: M(0xd2d6da, { metalness: 1, roughness: .22, roughnessMap: brushR }),
    steel: M(0x8f959b, { metalness: .9, roughness: .35, roughnessMap: brushR }),
    dark: M(0x1e2226, { metalness: .6, roughness: .5, normalMap: anoN, normalScale: new THREE.Vector2(.2, .2) }),
    brass: M(0xc9a24a, { metalness: 1, roughness: .3, roughnessMap: brushR }),
    black: M(0x0a0b0d, { metalness: 0, roughness: .55, roughnessMap: grainR }),
    rubber: M(0x0d0e0f, { metalness: 0, roughness: .9, roughnessMap: grainR }),
    plastic: M(0x121416, { metalness: .05, roughness: .35 }),
    white: M(0xe6e8ea, { metalness: 0, roughness: .5 }),
    amber: M(0xffb000, { metalness: 0, roughness: .4 }),
    pcbA: M(0xffffff, { map: pcbTex(11), metalness: .25, roughness: .6 }), pcbB: M(0xffffff, { map: pcbTex(42), metalness: .25, roughness: .6 }),
    floor: M(0x040507, { metalness: 0, roughness: .8, roughnessMap: grainR, envMapIntensity: .15 }),
    wall: M(0x0e1014, { metalness: 0, roughness: .95, roughnessMap: grainR, envMapIntensity: .3 }),
    caseM: M(0x040405, { roughness: .92, metalness: 0, roughnessMap: grainR, envMapIntensity: .12 }),
    glassDark: new THREE.MeshPhysicalMaterial({ color: 0x0a0c10, metalness: 0, roughness: .05, transmission: .4, thickness: .003, clearcoat: 1, clearcoatRoughness: .03, transparent: true, opacity: .9 }),
    dichro: function (tint) { return new THREE.MeshPhysicalMaterial({ color: tint, metalness: 0, roughness: .02, transmission: .82, thickness: .002, clearcoat: 1, clearcoatRoughness: .01, reflectivity: 1, transparent: true, opacity: .75, side: THREE.DoubleSide }); },
    mirror: M(0xf4f6f8, { metalness: 1, roughness: .02, side: THREE.DoubleSide }),
    led: function (c) { return new THREE.MeshBasicMaterial({ color: c }); } };
  // caixa com aresta arredondada (chapa dobrada tem raio) e chapa com furo redondo (aro da ventoinha)
  function rbox(w, h, d, r) { r = Math.min(r || .004, w / 2, h / 2, d / 2); var s = new THREE.Shape(); s.moveTo(-w / 2 + r, -h / 2); s.lineTo(w / 2 - r, -h / 2); s.absarc(w / 2 - r, -h / 2 + r, r, -1.5708, 0, false); s.lineTo(w / 2, h / 2 - r); s.absarc(w / 2 - r, h / 2 - r, r, 0, 1.5708, false); s.lineTo(-w / 2 + r, h / 2); s.absarc(-w / 2 + r, h / 2 - r, r, 1.5708, 3.1416, false); s.lineTo(-w / 2, -h / 2 + r); s.absarc(-w / 2 + r, -h / 2 + r, r, 3.1416, 4.7124, false); var g = new THREE.ExtrudeGeometry(s, { depth: d - 2 * r, bevelEnabled: true, bevelThickness: r, bevelSize: r, bevelSegments: 3, curveSegments: 6 }); g.center(); return g; }
  function ring(w, hole, d) { var s = new THREE.Shape(); s.moveTo(-w / 2, -w / 2); s.lineTo(w / 2, -w / 2); s.lineTo(w / 2, w / 2); s.lineTo(-w / 2, w / 2); s.closePath(); var p = new THREE.Path(); p.absarc(0, 0, hole, 0, Math.PI * 2, true); s.holes.push(p); var g = new THREE.ExtrudeGeometry(s, { depth: d, bevelEnabled: false, curveSegments: 32 }); g.center(); return g; }
  function hex(r, h) { return new THREE.CylinderGeometry(r, r, h, 6); }
  function tube(pts, r, m) { var c = new THREE.CatmullRomCurve3(pts.map(function (p) { return new THREE.Vector3(p[0], p[1], p[2]); })); var o = new THREE.Mesh(new THREE.TubeGeometry(c, Math.max(8, pts.length * 6), r, 8, false), m || mats.rubber); o.castShadow = true; return o; }
  return { env: env, tex: tex, M: M, m: mats, rbox: rbox, ring: ring, hex: hex, tube: tube, brushN: brushN, brushR: brushR, grainR: grainR };
};
