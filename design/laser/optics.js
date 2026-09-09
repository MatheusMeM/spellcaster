/* Dentro: mesa óptica (um bloco só de alumínio natural com furação M4, parafusada no fundo), três módulos laser
   em bases, dois dicroicos e um espelho de dobra em suportes cinemáticos, obturador mecânico (solenoide) ligado à
   chave, bloco de galvos X/Y em cantoneira com os dois motores e espelhos, placas discretas nas paredes (drivers
   dos diodos à esquerda, drivers dos galvos com dissipador à direita, DAC ILDA atrás), fonte, e todos os cabos
   com rota. Nada solto. Coordenadas locais do corpo; feixe a y = .057. */
window.OPTICS = function (THREE, X, body, pick) {
  "use strict";
  var m = X.m, PI = Math.PI, BY = .057;
  function add(parent, g, mat, x, y, z, k, label) { var o = new THREE.Mesh(g, mat); o.position.set(x, y, z); o.castShadow = o.receiveShadow = true; if (k) { o.userData = { key: k, label: label }; pick.push(o); } parent.add(o); return o; }
  function cyl(r, h, seg) { return new THREE.CylinderGeometry(r, r, h, seg || 20); }
  function grp(x, y, z, k, label) { var g = new THREE.Group(); g.position.set(x, y, z); if (k) { g.userData = { key: k, label: label }; pick.push(g); } body.add(g); return g; }
  function bolts(parent, pts, y, r) { pts.forEach(function (p) { var s = add(parent, cyl(r || .003, .003, 12), m.steel, p[0], y, p[1]); add(s, X.hex(r ? r * .5 : .0015, .002), m.black, 0, .001, 0); }); }
  /* mesa óptica: 300 × 16 × 200 mm, quatro parafusos M6 nos cantos */
  var bench = add(body, X.rbox(.30, .016, .20, .004), m.bench, -.02, .014, -.03, "bench", "mesa óptica: alumínio 16 mm, furação M4 12,5 mm"); bolts(body, [[-.16, -.12], [.12, -.12], [-.16, .06], [.12, .06]], .0235, .004);
  /* módulos laser: base + corpo + colimador + lente; cabo atrás */
  var lens = {};
  function module(k, x, z, dir, color, gold, label) { // dir: "+x" ou "-z"
    var g = grp(x, BY, z, k, label), rot = dir === "+x" ? -PI / 2 : 0; g.rotation.y = rot; // local: emite em -z
    add(g, X.rbox(.05, .018, .085, .002), m.dark, 0, -.026, .005); bolts(g, [[-.019, -.03], [.019, -.03], [-.019, .038], [.019, .038]], -.016);
    add(g, X.rbox(.034, .034, .07, .003), gold ? m.brass : m.black, 0, 0, .005); if (!gold) for (var i = 0; i < 7; i++) add(g, new THREE.BoxGeometry(.04, .0015, .06), m.steel, 0, .0185 + i * .0035, .005);
    var c = add(g, cyl(.008, .016), m.brass, 0, 0, -.038); c.rotation.x = PI / 2; add(g, cyl(.0085, .002), m.black, 0, 0, -.045).rotation.x = PI / 2;
    var l = add(g, cyl(.004, .002, 16), new THREE.MeshBasicMaterial({ color: 0x111111 }), 0, 0, -.0465); l.rotation.x = PI / 2; l.castShadow = false; lens[k] = { m: l, c: color };
    add(g, cyl(.004, .006), m.rubber, 0, 0, .042).rotation.x = PI / 2; return g; }
  module("g", -.12, -.03, "+x", 0x38ff5c, true, "módulo verde 520 nm · 3 W: limite e curva");
  module("r", -.05, .05, "-z", 0xff2a1a, false, "módulo vermelho 638 nm · 2,5 W: limite e curva");
  module("b", .01, .05, "-z", 0x3a6bff, false, "módulo azul 445 nm · 4,5 W: limite e curva");
  /* suportes cinemáticos: base, poste, placa com dois parafusos de ajuste, vidro */
  function mount(x, z, glassMat, k, label) { var g = grp(x, BY, z, k, label); g.rotation.y = PI / 4; add(g, cyl(.009, .014), m.silver, 0, -.028, -.012); add(g, cyl(.005, .024), m.silver, 0, -.011, -.012);
    add(g, X.rbox(.026, .026, .005, .001), m.black, 0, 0, -.005); [[-.009, .009], [.009, -.009]].forEach(function (p) { add(g, cyl(.0025, .008, 10), m.steel, p[0], p[1], -.011).rotation.x = PI / 2; });
    var gl = add(g, new THREE.PlaneGeometry(.02, .02), glassMat, 0, 0, 0); gl.castShadow = false; return g; }
  mount(-.05, -.03, m.dichro(0x9fffd8), "dichro", "dicroico 1: reflete 638, passa 520");
  mount(.01, -.03, m.dichro(0xffd0a0), "dichro", "dicroico 2: reflete 445, passa 520 + 638");
  mount(.07, -.03, m.mirror, "fold", "espelho de dobra HR: manda o feixe para os galvos");
  /* obturador: solenoide na mesa, braço e lâmina no feixe; abre com a chave + interlock */
  var sh = grp(.082, .046, -.06, "shutter", "obturador: fecha sem chave ou sem interlock"); add(sh, cyl(.007, .022), m.dark, 0, -.007, 0); add(sh, X.rbox(.024, .006, .018, .001), m.dark, 0, -.021, 0); bolts(sh, [[-.009, .0], [.009, .0]], -.0165, .002);
  add(sh, new THREE.BoxGeometry(.018, .002, .005), m.steel, -.006, 0, 0); add(sh, new THREE.BoxGeometry(.0015, .018, .008), m.black, -.012, .008, 0);
  /* galvos: cantoneira à esquerda, bloco em cima do feixe, motor X vertical, motor Y a 45° na frente, espelhos */
  var gb = grp(.083, .080, -.09, "galvo", "galvos X/Y: kpps"); add(gb, X.rbox(.06, .03, .05, .003), m.dark, 0, 0, 0);
  add(gb, X.rbox(.006, .073, .05, .002), m.dark, -.033, -.0215, 0); add(gb, X.rbox(.03, .006, .05, .002), m.dark, -.045, -.055, 0); bolts(gb, [[-.05, -.018], [-.05, .018]], -.0515, .003);
  var mx = add(gb, cyl(.0075, .04, 24), m.steel, -.013, .035, 0); add(gb, cyl(.008, .004, 24), m.black, -.013, .056, 0); add(gb, cyl(.0015, .012, 8), m.silver, -.013, -.018, 0);
  var mirX = add(gb, new THREE.BoxGeometry(.001, .012, .007), m.mirror, -.013, -.023, 0); mirX.rotation.y = -PI / 4;
  var yg = new THREE.Group(); yg.position.set(.012, -.023, 0); yg.rotation.y = 3 * PI / 4; gb.add(yg); // eixo local +z = (1,0,-1)/√2
  var my = add(yg, cyl(.0075, .04, 24), m.steel, 0, 0, .032); my.rotation.x = PI / 2; add(yg, cyl(.0015, .012, 8), m.silver, 0, 0, .006).rotation.x = PI / 2; add(yg, cyl(.008, .004, 24), m.black, 0, 0, .054).rotation.x = PI / 2;
  add(yg, X.rbox(.02, .012, .014, .001), m.dark, 0, .011, .032); // sela que prende o motor Y ao bloco
  var myp = new THREE.Group(); yg.add(myp); var mirY = add(myp, new THREE.BoxGeometry(.012, .016, .001), m.mirror, 0, 0, 0); mirY.rotation.y = -PI / 2; // normal local -x → mundo (1,0,1)/√2; myp gira no eixo do motor
  /* placas: drivers dos diodos (esquerda), drivers dos galvos com dissipador (direita), DAC ILDA (atrás); fonte */
  function pcb(w, h, mat, x, y, z, ry, k, label, heat) { var g = grp(x, y, z, k, label); g.rotation.y = ry; add(g, new THREE.BoxGeometry(w, h, .0016), mat, 0, 0, 0); [[-w / 2 + .004, -h / 2 + .004], [w / 2 - .004, h / 2 - .004]].forEach(function (p) { add(g, X.hex(.002, .005), m.brass, p[0], p[1], -.0033).rotation.x = PI / 2; add(g, cyl(.0015, .001, 8), m.steel, p[0], p[1], .0013).rotation.x = PI / 2; });
    for (var i = 0; i < 3; i++) { var ic = add(g, new THREE.BoxGeometry(.008, .006, .002), m.plastic, -w / 2 + .012 + i * .014, h * .2, .0018); for (var j = 0; j < 4; j++) { add(g, new THREE.BoxGeometry(.0004, .0016, .0008), m.silver, ic.position.x - .003 + j * .002, ic.position.y + .0038, .0012); add(g, new THREE.BoxGeometry(.0004, .0016, .0008), m.silver, ic.position.x - .003 + j * .002, ic.position.y - .0038, .0012); } }
    for (i = 0; i < 2; i++) { var cap = add(g, cyl(.003, .008, 12), new THREE.MeshStandardMaterial({ color: 0x14245a, metalness: .3, roughness: .4 }), w / 2 - .01 - i * .009, -h * .2, .0045); cap.rotation.x = PI / 2; add(cap, cyl(.0028, .0005, 12), m.steel, 0, .0042, 0); }
    add(g, new THREE.BoxGeometry(.01, .006, .006), m.plastic, -w / 2 + .01, -h * .3, .0035);
    if (heat) { add(g, new THREE.BoxGeometry(.03, .022, .003), m.steel, w * .15, 0, .0025); for (i = 0; i < 6; i++) add(g, new THREE.BoxGeometry(.03, .0012, .012), m.steel, w * .15, -.01 + i * .004, .009); }
    return g; }
  pcb(.045, .03, m.pcbA, -.192, .09, -.08, PI / 2, "pcb", "driver do diodo: corrente e modulação");
  pcb(.045, .03, m.pcbA, -.192, .09, -.03, PI / 2, "pcb", "driver do diodo: corrente e modulação");
  pcb(.045, .03, m.pcbA, -.192, .09, .03, PI / 2, "pcb", "driver do diodo: corrente e modulação");
  pcb(.065, .05, m.pcbB, .192, .09, -.06, -PI / 2, "galvodrv", "driver dos galvos: buffer e velocidade", true);
  pcb(.065, .05, m.pcbB, .192, .09, .02, -PI / 2, "galvodrv", "driver dos galvos: buffer e velocidade", true);
  pcb(.12, .06, m.pcbA, -.03, .09, .142, PI, "dac", "placa DAC ILDA: conectores traseiros");
  var psu = grp(-.12, .026, .105, "psu", "fonte 48 V · 250 W"); add(psu, X.rbox(.10, .04, .06, .002), m.black, 0, 0, 0); bolts(psu, [[-.04, -.02], [.04, .02]], .0215, .002);
  var psuL = X.tex(256, 128, function (x, w, h) { x.fillStyle = "#d8dcdf"; x.fillRect(0, 0, w, h); x.fillStyle = "#111"; x.font = "700 22px 'Share Tech Mono'"; x.fillText("PSU 48V 5.2A", 12, 40); x.font = "16px 'Share Tech Mono'"; x.fillText("IN 100-240V~  OUT 48V", 12, 72); x.fillText("SPELLCASTER  SC-PS250", 12, 100); }, true);
  var lab = add(psu, new THREE.PlaneGeometry(.06, .03), new THREE.MeshStandardMaterial({ map: psuL, roughness: .6 }), 0, .0205, 0); lab.rotation.x = -PI / 2; lab.castShadow = false;
  /* cabos: cada um com origem e destino reais */
  [[[-.157, .057, -.03], [-.175, .05, -.03], [-.19, .07, -.04]],
   [[-.05, .057, .09], [-.05, .045, .105], [-.1, .03, .11], [-.17, .03, .06], [-.19, .07, .035]],
   [[.01, .057, .09], [0, .04, .108], [-.08, .028, .115], [-.16, .03, .07], [-.19, .075, .04]],
   [[.07, .137, -.09], [.09, .142, -.082], [.15, .115, -.072], [.19, .095, -.062]],
   [[.132, .057, -.127], [.152, .05, -.125], [.185, .06, -.09], [.19, .085, -.062]],
   [[-.07, .03, .105], [0, .022, .122], [.08, .03, .135], [.14, .05, .135], [.19, .07, .02]],
   [[-.09, .07, .135], [-.13, .04, .13], [-.17, .04, .1], [-.19, .07, .055]],
   [[-.165, .06, .14], [-.15, .05, .13], [-.14, .045, .118]],
   [[.03, .06, .138], [.03, .03, .13], [.02, .022, .1], [-.07, .022, .108]]].forEach(function (p, i) { body.add(X.tube(p, i === 7 ? .0025 : i < 3 ? .0015 : .002)); });
  /* caminho óptico: segmentos com cor (fatores de limite aplicados no app) */
  var P = { g0: [-.068, BY, -.03], d1: [-.05, BY, -.03], r0: [-.05, BY, -.001], d2: [.01, BY, -.03], b0: [.01, BY, -.001], m1: [.07, BY, -.03], sh: [.07, BY, -.06], gx: [.07, BY, -.09], gy: [.095, BY, -.09], out: [.095, BY, -.152] };
  function segments(armed, open, lim, g) { if (!armed) return []; var r = lim.r, gg = lim.g, b = lim.b, S = [[P.g0, P.d1, [0, gg, 0]], [P.r0, P.d1, [r, 0, 0]], [P.b0, P.d2, [0, 0, b]], [P.d1, P.d2, [r, gg, 0]], [P.d2, P.m1, [r, gg, b]], [P.m1, P.sh, [r, gg, b]]];
    if (open) { S.push([P.sh, P.gx, [r, gg, b]], [P.gx, P.gy, [r, gg, b]], [P.gy, [P.out[0] + g[0] * .009, P.out[1] + g[1] * .006, P.out[2]], [r, gg, b]]); } return S; }
  return { bench: bench, lens: lens, shutter: sh, mirX: mirX, mirY: myp, segments: segments, P: P };
};
