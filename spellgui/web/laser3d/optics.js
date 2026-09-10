/* Dentro do aparelho: mesa óptica de alumínio natural com furação M4 a 12,5 mm de verdade, três módulos laser
   em bases parafusadas na grade, dois dicroicos e um espelho de dobra em suportes cinemáticos, obturador de
   solenoide com braço e ímã, bloco de galvos X/Y, placas de driver nas paredes e o DAC na traseira, fonte, e os
   cabos com rota por abraçadeiras. Nada solto e nada atravessa nada.

   Referências (só de proporção; tudo é procedural, nenhum asset baixado):
   mesa Newport SG / Thorlabs MB com furação M4 a 12,5 mm; suporte cinemático Thorlabs KM100;
   galvo Cambridge Technology 6215H (motor Ø 14,3 mm, espelho no fim do eixo) e Sino-Galvo SG-B2;
   módulos DPSS/diodo de 3 W com dissipador de aletas e ventoinha de 30 mm.

   Coordenadas locais do corpo (origem no fundo da chapa, y = 0 na chapa de baixo). O feixe corre a y = .057
   (BEAM_Y de body.js) e o interior útil é x ∈ [-.194, .194], z ∈ [-.144, .144].
   A grade da mesa cai em múltiplos de 12,5 mm: GX(i)/GZ(j) devolvem furo por furo, e todo suporte, módulo e
   parafuso está num furo. */
window.OPTICS = function (THREE, X, body, pick) {
  "use strict";
  var m = X.m, PI = Math.PI, BY = .057, LBL = LaserEngine.labelOf, V2 = THREE.Vector2;
  function add(parent, g, mat, x, y, z, k, label) { var o = new THREE.Mesh(g, mat); o.position.set(x, y, z); o.castShadow = o.receiveShadow = true; if (k) { o.userData = { key: k, label: label || LBL(k) }; pick.push(o); } parent.add(o); return o; }
  function cyl(r, h, seg) { return new THREE.CylinderGeometry(r, r, h, seg || 20); }
  function grp(x, y, z, k, label) { var g = new THREE.Group(); g.position.set(x, y, z); if (k) { g.userData = { key: k, label: label || LBL(k) }; pick.push(g); } body.add(g); return g; }
  function sub(parent, x, y, z, ry) { var g = new THREE.Group(); g.position.set(x, y, z); if (ry) g.rotation.y = ry; parent.add(g); return g; }

  /* ---------- materiais (locais: mat.js é de outra frente) ----------
     Medido no bench com o spot da cena (bench.html sun = 50, app.js 90*S.dim): superfície difusa satura e vira
     plástico branco por mais escuro que seja o albedo; superfície metálica (metalness >= .85) não satura porque
     só devolve o especular. Por isso todo alumínio aqui é metalness ~.9 com cor CLARA (é tinta de especular,
     não difusa) e o preto anodizado é metalness .85 com cor escura. O ganho de exposição do interior está
     anotado em design/DECISOES.md — a iluminação é de outra frente. */
  var mm = {
    aluTop: X.M(0xffffff, { metalness: 1, roughness: .55, envMapIntensity: .45 }),        // tom vem do mapa da grade
    aluSide: X.M(0x2e3339, { metalness: 1, roughness: .6, roughnessMap: X.brushR, normalMap: X.brushN, normalScale: new V2(.16, .16), envMapIntensity: .4 }),
    anod: X.M(0x161c22, { metalness: 1, roughness: .88, roughnessMap: X.grainR, envMapIntensity: .22 }),      // preto anodizado
    anodG: X.M(0x30353b, { metalness: 1, roughness: .62, roughnessMap: X.brushR, envMapIntensity: .38 }),      // alumínio usinado
    heat: X.M(0x101418, { metalness: 1, roughness: .92, roughnessMap: X.grainR, envMapIntensity: .2 }),      // dissipador anodizado
    steel: X.M(0x4a5058, { metalness: 1, roughness: .38, roughnessMap: X.brushR, envMapIntensity: .55 }),
    brass: X.M(0x6d5320, { metalness: 1, roughness: .46, roughnessMap: X.brushR, envMapIntensity: .5 }),
    rubber: X.M(0x030304, { metalness: 0, roughness: .95, roughnessMap: X.grainR }),
    mirror: X.M(0xc8d2db, { metalness: .5, roughness: .2, envMapIntensity: 2.4, side: THREE.DoubleSide }),
    ic: X.M(0x030405, { metalness: .04, roughness: .8, roughnessMap: X.grainR }),
    pin: X.M(0x35393d, { metalness: .9, roughness: .35 }),
    cap: X.M(0x070c17, { metalness: .3, roughness: .5 }),
    conn: X.M(0x040506, { metalness: .04, roughness: .64 }),
    green: X.M(0x05120a, { metalness: .05, roughness: .62 }),
    fr4: X.M(0x171609, { metalness: .05, roughness: .82 }),
    wK: X.M(0x020203, { metalness: 0, roughness: .86, roughnessMap: X.grainR }),
    wR: X.M(0x160504, { metalness: 0, roughness: .82, roughnessMap: X.grainR }),
    wG: X.M(0x0c1806, { metalness: 0, roughness: .82, roughnessMap: X.grainR }),
    wS: X.M(0x08090b, { metalness: 0, roughness: .8, roughnessMap: X.grainR }),
    clip: X.M(0x040506, { metalness: .05, roughness: .72 })
  };
  function bolts(parent, pts, y, r) { r = r || .003; pts.forEach(function (p) { var s = add(parent, cyl(r, .0028, 14), mm.steel, p[0], y, p[1]); add(s, X.hex(r * .52, .0018), mm.anod, 0, .0011, 0); }); }
  function label(w, h, txt, px) { return X.tex(w, h, function (x, cw, ch) { x.fillStyle = "#0a0b0d"; x.fillRect(0, 0, cw, ch); x.fillStyle = "#585d62"; x.font = "700 " + (px || 22) + "px 'Share Tech Mono'"; x.textBaseline = "middle"; x.fillText(txt, 8, ch / 2); }, true); }

  /* ---------- mesa óptica ----------
     Bloco único de 325 × 200 × 12 mm com aresta chanfrada, sobre quatro pés de isolamento. A furação M4 é
     textura no tampo (rebaixo + furo escuro) e a UV é corrigida: X.rbox é ExtrudeGeometry e as UV do tampo vêm
     em metros, então o repeat é 1/lado (era esse o bug dos "furos gigantes"). Grupo de materiais do extrude:
     0 = tampas (a grade), 1 = paredes e chanfro. */
  var BW = .325, BD = .20, BT = .012, BX = -.00625, BZ = -.03125, TOP = .022, PXM = 4000; // 4 px por mm
  function GX(i) { return BX - BW / 2 + .00625 + i * .0125; }
  function GZ(j) { return BZ - BD / 2 + .00625 + j * .0125; }
  var gridT = X.tex(Math.round(BW * PXM), Math.round(BD * PXM), function (x, w, h) {
    x.fillStyle = "#3d4349"; x.fillRect(0, 0, w, h);
    for (var i = 0; i < 9000; i++) { x.fillStyle = "rgba(255,255,255," + Math.random() * .06 + ")"; x.fillRect(Math.random() * w, Math.random() * h, 30 + Math.random() * 90, 1); }
    for (var a = 0; a * 50 < w; a++) for (var b = 0; b * 50 < h; b++) { var cx = 25 + a * 50, cy = 25 + b * 50;
      x.fillStyle = "#2c3137"; x.beginPath(); x.arc(cx, cy, 11, 0, 7); x.fill();          // rebaixo do furo
      x.fillStyle = "#171a1d"; x.beginPath(); x.arc(cx, cy, 9, 0, 7); x.fill();
      x.fillStyle = "#050607"; x.beginPath(); x.arc(cx, cy, 7, 0, 7); x.fill();           // furo M4 passante
      x.strokeStyle = "rgba(230,240,255,.30)"; x.lineWidth = 1.8; x.beginPath(); x.arc(cx, cy, 10, 3.4, 5.7); x.stroke(); }
  }, true);
  gridT.wrapS = gridT.wrapT = THREE.ClampToEdgeWrapping; gridT.repeat.set(1 / BW, 1 / BD); gridT.offset.set(.5, .5);
  mm.aluTop.map = gridT;
  var bench = new THREE.Mesh(X.rbox(BW, BD, BT, .0022), [mm.aluTop, mm.aluSide]);
  bench.rotation.x = -PI / 2; bench.position.set(BX, TOP - BT / 2, BZ); bench.castShadow = bench.receiveShadow = true;
  bench.userData = { key: "bench", label: LBL("bench") }; pick.push(bench); body.add(bench);
  var corners = [[GX(1), GZ(1)], [GX(24), GZ(1)], [GX(1), GZ(14)], [GX(24), GZ(14)]];
  corners.forEach(function (p) { add(body, cyl(.007, .004, 16), mm.rubber, p[0], .008, p[1]); });  // pés de isolamento
  bolts(body, corners, TOP + .0014, .0035);

  /* ---------- módulos laser ----------
     base na grade → bloco de subida → corpo anodizado com aletas → colimador de latão → lente.
     Local: emite em -z, lente em z = -.0386, ventoinha e conector atrás em z = +.035. */
  var lens = {}, fanT = X.tex(160, 160, function (x, w, h) {
    x.fillStyle = "#0a0b0d"; x.fillRect(0, 0, w, h); x.fillStyle = "#15181b"; x.beginPath(); x.arc(80, 80, 74, 0, 7); x.fill();
    x.strokeStyle = "#2f353b"; x.lineWidth = 9; x.lineCap = "round";
    for (var i = 0; i < 7; i++) { var a = i * PI * 2 / 7; x.beginPath(); x.moveTo(80 + Math.cos(a) * 20, 80 + Math.sin(a) * 20);
      x.quadraticCurveTo(80 + Math.cos(a + .55) * 48, 80 + Math.sin(a + .55) * 48, 80 + Math.cos(a + 1.05) * 70, 80 + Math.sin(a + 1.05) * 70); x.stroke(); }
    x.fillStyle = "#0c0e10"; x.beginPath(); x.arc(80, 80, 22, 0, 7); x.fill();
    x.strokeStyle = "#454b51"; x.lineWidth = 5; x.strokeRect(4, 4, w - 8, h - 8);
    x.fillStyle = "#586067"; x.font = "700 13px 'Share Tech Mono'"; x.textAlign = "center"; x.fillText("30x30", 80, 84);
  }, true);
  function module(k, x, z, dir, color, fan, txt) {
    var g = grp(x, BY, z, k), b = sub(g, 0, 0, 0, dir === "+x" ? -PI / 2 : 0);
    add(b, X.rbox(.046, .006, .062, .0015), mm.anodG, 0, -.032, .004);                       // base (fundo em -.035 = tampo da mesa)
    add(b, X.rbox(.030, .012, .050, .0015), mm.anodG, 0, -.023, .004);                       // bloco de subida
    add(b, X.rbox(.036, .034, .058, .0025), mm.anod, 0, 0, .004);                            // corpo
    add(b, X.rbox(.042, .0035, .050, .001), mm.heat, 0, .0172, .004);                        // base do dissipador
    for (var i = 0; i < 4; i++) add(b, new THREE.BoxGeometry(.040, .0015, .046), mm.heat, 0, .0215 + i * .0055, .004);
    var c = add(b, cyl(.008, .015, 24), mm.brass, 0, 0, -.0305); c.rotation.x = PI / 2;       // colimador
    add(b, cyl(.0088, .0022, 24), mm.anod, 0, 0, -.0377).rotation.x = PI / 2;                 // anel de trava
    var l = add(b, new THREE.CircleGeometry(.0048, 20), new THREE.MeshBasicMaterial({ color: 0x111111 }), 0, 0, -.0389);
    l.castShadow = false; lens[k] = { m: l, c: color };
    if (fan) { var f = add(b, new THREE.PlaneGeometry(.028, .028), X.M(0xffffff, { map: fanT, metalness: .1, roughness: .75 }), 0, .005, .0335); f.castShadow = false; }
    add(b, X.rbox(.012, .008, .005, .001), mm.conn, .011, -.011, .0345);                      // conector de potência
    var t = add(b, new THREE.PlaneGeometry(.026, .006), X.M(0xffffff, { map: label(256, 60, txt), metalness: 0, roughness: .7 }), 0, .0192, -.0175);
    t.rotation.x = -PI / 2; t.castShadow = false;
    return g;
  }
  module("g", GX(3), GZ(7), "+x", 0x38ff5c, true, "520 nm  3 W");    // x = -.125, z = -.0375
  module("r", GX(9), GZ(12), "-z", 0xff2a1a, false, "638 nm  2 W");  // x = -.05,  z = +.025
  module("b", GX(14), GZ(12), "-z", 0x3a6bff, false, "445 nm  4 W"); // x = .0125, z = +.025

  /* ---------- suportes cinemáticos ----------
     pé parafusado em dois furos da grade (o pé é axial à grade; só a torre gira 45°), poste, placa de trás e
     placa do espelho ligadas por dois parafusos de ajuste e uma mola, óptica de 1/2" com anel de retenção. As
     placas são anéis: dicroico transmite, então nada de chapa cheia atrás do vidro. */
  function mount(x, z, glassMat, k) {
    var g = grp(x, BY, z, k); g.rotation.y = PI / 4;
    var bs = sub(g, 0, 0, 0, -PI / 4);                                                        // pé alinhado à grade
    add(bs, X.rbox(.036, .006, .020, .0015), mm.anodG, 0, -.032, 0);
    bolts(bs, [[-.0125, 0], [.0125, 0]], -.0277, .003);
    add(g, X.rbox(.016, .030, .014, .0015), mm.anodG, 0, -.016, -.0095);                      // poste
    add(g, X.ring(.030, .0105, .005), mm.anodG, 0, 0, -.0095);                                // placa de trás
    add(g, X.ring(.026, .0105, .004), mm.anodG, 0, 0, -.002);                                 // placa do espelho
    [[-.0105, .0105], [.0105, -.0105]].forEach(function (p) {                                 // parafusos de ajuste
      var s = add(g, cyl(.0026, .0115, 12), mm.steel, p[0], p[1], -.0058); s.rotation.x = PI / 2;
      add(s, cyl(.0038, .0022, 16), mm.anod, 0, -.0058, 0); });
    add(g, cyl(.0016, .0075, 8), mm.steel, .0105, .0105, -.0058).rotation.x = PI / 2;         // mola
    var gl = add(g, new THREE.CircleGeometry(.0115, 32), glassMat, 0, 0, 0); gl.castShadow = false;
    add(g, X.ring(.026, .0105, .0016), mm.anod, 0, 0, .0009);                                 // anel de retenção
    return g;
  }
  mount(GX(9), GZ(7), m.dichro(0x9fffd8), "dichro");   // x = -.05,   junta G + R
  mount(GX(14), GZ(7), m.dichro(0xffd0a0), "dichro");  // x = .0125,  junta B
  mount(GX(19), GZ(7), mm.mirror, "fold");             // x = .075,   dobra para os galvos

  /* ---------- obturador ----------
     solenoide de pé na mesa em dois furos, braço no eixo do solenoide à altura do feixe, lâmina no feixe quando
     fechado e ímã de retenção do outro lado. app.js gira O.shutter.rotation.y de 0 (fechado) a 1.2 (aberto). */
  var sh = grp(GX(20), BY, GZ(5), "shutter");                                                 // pivô em x = .0875, z = -.0625
  add(sh, X.rbox(.030, .006, .020, .0015), mm.anodG, 0, -.032, 0);
  bolts(sh, [[-.0125, 0], [.0125, 0]], -.0277, .003);
  add(sh, cyl(.007, .022, 20), mm.anod, 0, -.018, 0);                                         // corpo do solenoide
  add(sh, cyl(.0082, .0035, 20), mm.steel, 0, -.0052, 0);                                     // flange
  add(sh, cyl(.0032, .0075, 12), mm.steel, 0, -.0018, 0);                                     // eixo
  add(sh, X.rbox(.010, .007, .010, .001), mm.anod, .013, -.0075, 0);                          // ímã de retenção
  add(sh, cyl(.004, .0025, 16), mm.steel, .013, -.0032, 0);
  var shArm = sub(sh, 0, 0, 0);
  add(shArm, new THREE.BoxGeometry(.0135, .0025, .005), mm.steel, -.0068, .0022, 0);          // braço
  add(shArm, new THREE.BoxGeometry(.0016, .014, .010), mm.anod, -.0125, .0022, 0);            // lâmina, no feixe quando fechado
  add(shArm, cyl(.0032, .0022, 12), mm.steel, .009, .0022, 0);                                // contra-peso no ímã

  /* ---------- bloco de galvos X/Y ----------
     Padrão Cambridge Technology 6215H / Sino-Galvo SG-B2: motor cilíndrico anodizado de Ø 14,3 mm com flange
     dianteiro, conector de 4 pinos atrás e o espelho colado na ponta do eixo. Os dois eixos ficam a 90° num
     bloco em L de alumínio: pé parafusado em quatro furos da grade, chapa vertical a 45° que carrega o motor Y
     e braço no alto de onde o motor X pende com o eixo para baixo.
     Corpo do motor: 32 mm no X (pendurado, tem altura livre) e 24 mm no Y (encurtado dos 32 mm reais para caber
     entre a mesa e o painel frontal) — está anotado aqui porque é a única licença de proporção do conjunto.
     Cinemática: o feixe chega em -z, o espelho X (eixo vertical) manda em +x, o espelho Y (eixo horizontal a 45°
     em planta, dentro do plano do espelho) manda em -z e sai pela abertura em x = .095.
     app.js gira O.mirX.rotation.y (-PI/4 + varredura) e O.mirY.rotation.z (varredura vertical). */
  var GBX = GX(20), GBZ = GZ(2), A45 = 3 * PI / 4;                  // pé do L em (.0875, -.10); eixo do Y = (1,0,-1)/raiz(2)
  var gb = grp(GBX, TOP, GBZ, "galvo");                             // y local 0 = tampo da mesa
  function motor(parent, len, z0) {                                 // eixo em +z local; z0 = face do flange
    add(parent, cyl(.0085, .003, 28), mm.anodG, 0, 0, z0 + .0015).rotation.x = PI / 2;            // flange dianteiro
    add(parent, cyl(.00715, len, 28), mm.anod, 0, 0, z0 + .003 + len / 2).rotation.x = PI / 2;    // corpo Ø 14,3
    for (var i = 0; i < 3; i++) add(parent, cyl(.00728, .0012, 28), mm.anodG, 0, 0, z0 + .008 + i * (len - .014) / 2).rotation.x = PI / 2;
    var cn = add(parent, X.rbox(.009, .006, .005, .0008), mm.conn, 0, .0092, z0 + len - .004);    // conector de 4 pinos
    for (var j = 0; j < 4; j++) add(cn, cyl(.0004, .005, 6), mm.pin, -.003 + j * .002, .0045, 0);
    return cn;
  }
  // --- bloco em L de alumínio
  add(gb, X.rbox(.076, .006, .038, .002), mm.anodG, 0, .003, 0);                                  // pé na grade
  bolts(gb, [[-.0125, -.0125], [.0125, -.0125], [-.0125, .0125], [.0125, .0125]], .0074, .0035);
  add(gb, X.rbox(.024, .100, .006, .002), mm.anodG, .0199, .050, -.0124).rotation.y = A45;        // chapa vertical a 45° (motor Y)
  add(gb, X.rbox(.046, .010, .013, .002), mm.anodG, .0037, .066, -.005);                          // braço lateral que segura o motor X (nada por cima: o motor fica à vista)
  // --- galvo X: motor pendurado do braço com o eixo para baixo
  var gxg = sub(gb, -.0125, 0, 0), xm = sub(gxg, 0, .0505, 0); xm.rotation.x = -PI / 2;            // +z local vira +y
  motor(xm, .032, 0);
  add(gxg, X.ring(.026, .00745, .011), mm.anodG, 0, .066, 0).rotation.x = PI / 2;                 // abraçadeira do motor
  var mirX = sub(gxg, 0, .035, 0); mirX.rotation.y = -PI / 4;                                     // O.mirX: normal local = +x
  add(mirX, cyl(.0015, .015, 10), mm.steel, 0, .0075, 0);                                          // eixo saindo do flange
  add(mirX, new THREE.BoxGeometry(.0030, .011, .006), mm.anodG, -.0023, 0, 0);                     // suporte colado no eixo
  add(mirX, new THREE.BoxGeometry(.0006, .010, .015), mm.mirror, .0011, 0, 0);                     // espelho X 15 x 10 mm
  // --- galvo Y: motor na chapa a 45°, eixo dentro do plano do espelho
  var yg = sub(gb, .0075, .035, 0, A45);
  motor(yg, .024, .0175);
  add(yg, X.ring(.026, .00745, .008), mm.anodG, 0, 0, .0135);                                      // abraçadeira na chapa
  var myp = sub(yg, 0, 0, 0);                                                                      // O.mirY: gira em torno de z local
  add(myp, cyl(.0015, .014, 10), mm.steel, 0, 0, .0105).rotation.x = PI / 2;
  add(myp, new THREE.BoxGeometry(.0030, .013, .006), mm.anodG, -.0023, 0, .0035);
  add(myp, new THREE.BoxGeometry(.0006, .012, .014), mm.mirror, .0011, 0, 0);                       // espelho Y 14 x 12 mm
  // --- plaquinhas de identificação no pé
  [["GALVO X", -.0125], ["GALVO Y", .017]].forEach(function (p) {
    var t = add(gb, new THREE.PlaneGeometry(.019, .0042), X.M(0xffffff, { map: label(256, 56, p[0], 30), metalness: 0, roughness: .7 }), p[1], .0062, .0148);
    t.rotation.x = -PI / 2; t.castShadow = false; });

  /* ---------- placas ---------- (refeitas no item 2) */
  function pcb(w, h, mat, x, y, z, ry, k) { var g = grp(x, y, z, k); g.rotation.y = ry; add(g, new THREE.BoxGeometry(w, h, .0016), mat, 0, 0, 0);
    [[-w / 2 + .004, -h / 2 + .004], [w / 2 - .004, h / 2 - .004]].forEach(function (p) { add(g, X.hex(.002, .007), mm.brass, p[0], p[1], -.0044).rotation.x = PI / 2; });
    for (var i = 0; i < 3; i++) add(g, new THREE.BoxGeometry(.008, .006, .002), mm.ic, -w / 2 + .012 + i * .014, h * .2, .0018);
    return g; }
  pcb(.045, .03, m.pcbA, -.186, .09, -.08, PI / 2, "pcb");
  pcb(.045, .03, m.pcbA, -.186, .09, -.03, PI / 2, "pcb");
  pcb(.045, .03, m.pcbA, -.186, .09, .03, PI / 2, "pcb");
  pcb(.065, .05, m.pcbB, .186, .09, -.06, -PI / 2, "galvodrv");
  pcb(.065, .05, m.pcbB, .186, .09, .02, -PI / 2, "galvodrv");
  pcb(.11, .055, m.pcbA, -.03, .09, .136, PI, "dac");

  /* ---------- fonte ---------- (atrás da mesa, junto da traseira) */
  var psu = grp(-.115, .029, .115, "psu"); add(psu, X.rbox(.10, .042, .045, .002), mm.anod, 0, 0, 0);
  bolts(psu, [[-.04, -.016], [.04, .016]], .0235, .0025);
  var psuL = X.tex(256, 128, function (x, w, h) { x.fillStyle = "#20242a"; x.fillRect(0, 0, w, h); x.fillStyle = "#111"; x.font = "700 22px 'Share Tech Mono'"; x.fillText("PSU 48V 5.2A", 12, 40); x.font = "16px 'Share Tech Mono'"; x.fillText("IN 100-240V~  OUT 48V", 12, 72); x.fillText("SPELLCASTER  SC-PS250", 12, 100); }, true);
  var lab = add(psu, new THREE.PlaneGeometry(.06, .03), X.M(0xffffff, { map: psuL, metalness: 0, roughness: .65 }), 0, .0215, 0); lab.rotation.x = -PI / 2; lab.castShadow = false;

  /* ---------- caminho óptico ---------- (todo ponto é o centro de uma peça de verdade) */
  var P = { g0: [GX(3) + .0389, BY, GZ(7)], d1: [GX(9), BY, GZ(7)], r0: [GX(9), BY, GZ(12) - .0389],
    d2: [GX(14), BY, GZ(7)], b0: [GX(14), BY, GZ(12) - .0389], m1: [GX(19), BY, GZ(7)],
    sh: [GX(19), BY, GZ(5)], gx: [GX(19), BY, GZ(2)], gy: [.095, BY, GZ(2)], out: [.095, BY, -.152] };
  function segments(armed, open, lim, g) { if (!armed) return []; var r = lim.r, gg = lim.g, b = lim.b, S = [[P.g0, P.d1, [0, gg, 0]], [P.r0, P.d1, [r, 0, 0]], [P.b0, P.d2, [0, 0, b]], [P.d1, P.d2, [r, gg, 0]], [P.d2, P.m1, [r, gg, b]], [P.m1, P.sh, [r, gg, b]]];
    if (open) { S.push([P.sh, P.gx, [r, gg, b]], [P.gx, P.gy, [r, gg, b]], [P.gy, [P.out[0] + g[0] * .009, P.out[1] + g[1] * .006, P.out[2]], [r, gg, b]]); } return S; }
  return { bench: bench, lens: lens, shutter: shArm, mirX: mirX, mirY: myp, segments: segments, P: P };
};
