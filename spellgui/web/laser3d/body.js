/* Sala, flightcase e o corpo do projetor (400 × 180 × 300 mm): chapas, aletas laterais, forquilha, pés, tampa com
   parafusos, abertura na frente e o painel traseiro completo (inspirado no Clubmax: powerCON + rocker, chave,
   um interlock só, LED de emissão, ILDA IN/OUT DB25, DMX IN/OUT XLR-5, NET, USB, display de rack + encoder + BACK,
   ventoinha axial de 60 mm com grade de arame, etiqueta de perigo, placa de série). Coordenadas em metros, origem
   no fundo do corpo.

   UM CONTROLE, UMA FUNÇÃO. A classificação de cada peça clicável é dado, não convenção: mora em
   `LaserEngine.CONTROLS` (engine.js), é de onde sai o rótulo do tooltip e é o que `app.js` consulta para decidir
   o que o clique faz. Nenhuma peça faz duas coisas, e nenhuma função tem dois pontos.

   | família     | o que o clique faz              | peças |
   |-------------|---------------------------------|-------|
   | toggle      | inverte um estado, e só          | `power` (rocker), `keyswitch`, `interlock` |
   | momentary   | age enquanto apertado            | — (nenhuma hoje) |
   | valor       | muda um número                   | encoder girando (roda do mouse sobre ele) e os faders dentro dos painéis |
   | conector    | nada: é plugue                   | `acin` (powerCON) |
   | navegação   | abre a tela daquela peça         | `enc`, `back`, portas (`ilda`, `ildathru`, `dmxin`, `dmxout`, `rj45`, `usb`), `lid`, `fan`, e as peças de dentro |

   O display não entra no raycast: display não é controle. A energia liga só no rocker (o powerCON é plugue), a
   emissão arma só na chave, e o interlock é um só — o obturador em `optics.js` reage a ele, mas é indicação. */
window.BODY = function (THREE, X, scene, pick) {
  "use strict";
  var m = X.m, PI = Math.PI, LBL = LaserEngine.labelOf;
  function add(parent, g, mat, x, y, z, k, label) { var o = new THREE.Mesh(g, mat); o.position.set(x, y, z); o.castShadow = o.receiveShadow = true; if (k) { o.userData = { key: k, label: label || LBL(k) }; pick.push(o); } parent.add(o); return o; }
  function cyl(r, h, seg) { return new THREE.CylinderGeometry(r, r, h, seg || 20); }
  function zcyl(parent, r, h, mat, x, y, z, k, label) { var o = add(parent, cyl(r, h), mat, x, y, z, k, label); o.rotation.x = PI / 2; return o; }
  /* sala */
  var floor = add(scene, new THREE.PlaneGeometry(14, 14), m.floor, 0, 0, -2); floor.rotation.x = -PI / 2; floor.castShadow = false;
  var wall = add(scene, new THREE.PlaneGeometry(8, 5), m.wall, 0, 2.2, -5.01); wall.castShadow = false;
  var wallLight = new THREE.PointLight(0x6f86b0, 14, 7, 2); wallLight.position.set(0, 2.6, -3.2); scene.add(wallLight);
  /* flightcase */
  add(scene, X.rbox(.62, .3, .46, .003), m.caseM, 0, .15, 0); [-.29, .29].forEach(function (x) { [-.21, .21].forEach(function (z) { add(scene, X.rbox(.03, .3, .03, .003), m.silver, x, .15, z); }); });
  [-.2, 0, .2].forEach(function (x) { [-.22, .22].forEach(function (z) { add(scene, new THREE.SphereGeometry(.004, 10, 8), m.steel, x, .29, z * 1.01); }); });
  /* corpo */
  var body = new THREE.Group(); body.position.y = .314; scene.add(body);
  var W = .40, H = .18, D = .30, T = .006;
  add(body, X.rbox(W, T, D, .003), m.alu, 0, T / 2, 0);
  add(body, X.rbox(W, H, T, .003), m.alu, 0, H / 2, -D / 2 + T / 2, "front");
  var rearPlate = add(body, X.rbox(W, H, T, .003), m.alu, 0, H / 2, D / 2 - T / 2);
  add(body, X.rbox(T, H, D, .003), m.alu, -W / 2 + T / 2, H / 2, 0); add(body, X.rbox(T, H, D, .003), m.alu, W / 2 - T / 2, H / 2, 0, "side");
  for (var i = 0; i < 8; i++) [-1, 1].forEach(function (s) { add(body, new THREE.BoxGeometry(.006, .0025, .24), m.alu, s * (W / 2 + .002), .03 + i * .016, -.01); });
  // forquilha (yoke) com manípulos, e pés de borracha
  add(body, X.rbox(.46, .008, .035, .002), m.dark, 0, -.010, 0); [-1, 1].forEach(function (s) { add(body, X.rbox(.008, .14, .035, .002), m.dark, s * .234, .06, 0); var k = add(body, cyl(.016, .012, 16), new THREE.MeshStandardMaterial({ color: 0x1e2226, metalness: .6, roughness: .5, flatShading: true }), s * .244, .10, 0); k.rotation.z = PI / 2; var sc = add(body, cyl(.004, .002, 12), m.silver, s * .251, .10, 0); sc.rotation.z = PI / 2; });
  [-.2, .2].forEach(function (x) { [-.09, .09].forEach(function (z) { add(body, cyl(.009, .006, 16), m.rubber, x, -.017, z); }); });
  // tampa (dobradiça atrás) com seis parafusos e etiqueta de perigo
  var lid = new THREE.Group(); lid.position.set(0, H, D / 2); body.add(lid); var lidM = add(lid, X.rbox(W, T, D, .003), m.alu, 0, 0, -D / 2, "lid");
  var screws = []; [[-.18, -.28], [.18, -.28], [-.18, -.02], [.18, -.02], [-.18, -.15], [.18, -.15]].forEach(function (p) { var s = add(lid, cyl(.0045, .004, 12), m.steel, p[0], .004, p[1], "lid"); var sk = add(s, X.hex(.002, .002), m.black, 0, .0015, 0); sk.rotation.y = .3; screws.push(s); });
  var lbl = X.tex(512, 256, function (x, w, h) { x.fillStyle = "#FFB000"; x.fillRect(0, 0, w, h); x.fillStyle = "#000"; x.fillRect(0, 0, w, 60); x.fillStyle = "#FFB000"; x.font = "700 40px 'Share Tech Mono'"; x.textAlign = "center"; x.fillText("PERIGO · RADIAÇÃO LASER", w / 2, 44); x.fillStyle = "#000"; x.font = "700 30px 'Share Tech Mono'"; x.fillText("EVITE EXPOSIÇÃO AO FEIXE", w / 2, 110); x.fillText("SAÍDA 10 W · 445-638 nm", w / 2, 160); x.fillText("PRODUTO LASER CLASSE 4", w / 2, 210); }, true);
  var lab = add(lid, new THREE.PlaneGeometry(.07, .035), new THREE.MeshStandardMaterial({ map: lbl, metalness: 0, roughness: .6 }), -.12, .0031, -.24); lab.rotation.x = -PI / 2; lab.castShadow = false;
  // frente: abertura com janela
  var BEAM_Y = .057; add(body, X.rbox(.04, .026, .004, .001), m.black, .095, BEAM_Y, -D / 2 - .001, "aperture");
  var apGlass = add(body, new THREE.PlaneGeometry(.03, .018), m.glassDark, .095, BEAM_Y, -D / 2 - .0035); apGlass.rotation.y = PI; apGlass.castShadow = false;
  var APERT = new THREE.Vector3(.095, .314 + BEAM_Y, -D / 2 - .005);
  /* painel traseiro */
  var Z = D / 2 + .0003, py = function (y) { return H / 2 + y; };
  function port(x, y, k, label) { var g = new THREE.Group(); g.position.set(x, py(y), Z); if (k) { g.userData = { key: k, label: label || LBL(k) }; pick.push(g); } body.add(g); return g; }
  /* Layout do painel: o display de rack ocupa a faixa central (200 × 100 mm — metade da largura do
     painel), energia e segurança ficam na coluna da esquerda, encoder e ventoinha na direita, e todas
     as portas descem para a régua de baixo. É a distribuição de um projetor de rack de verdade, e é o
     que faz o display ser lido de longe sem zoom. */
  var L = { title: [-.19, .078], brand: [.19, .078], rock: [-.180, .045], key: [-.180, -.005], lock: [-.145, .045], em: [-.145, .004],
    oled: [-.012, .008], enc: [.108, .038], back: [.108, -.012], fan: [.158, .012],
    pcon: [-.175, -.066], ildaIn: [-.108, -.066], ildaOut: [-.043, -.066], dmxIn: [.005, -.066], dmxOut: [.038, -.066], net: [.070, -.062], usb: [.098, -.062],
    warn: [.152, -.052], sn: [.152, -.080] };
  var silkT = X.tex(2048, 920, function (x, w, h) { x.fillStyle = "#101214"; x.fillRect(0, 0, w, h); var U = function (p) { return (p[0] + .2) * 5120; }, V = function (p) { return (.09 - p[1]) * 5111; };
    x.fillStyle = "#c9ced3"; x.textAlign = "left"; x.font = "700 58px Michroma"; x.fillText("SPELLCASTER", U(L.title), V(L.title) + 20); x.font = "700 30px 'Share Tech Mono'"; x.fillText("LASER 10 W RGB · 638 / 520 / 445 nm · 40 kpps", U(L.title) + 580, V(L.title) + 18);
    x.textAlign = "right"; x.fillText("FEITIÇARIA iNDUSTRIAL", U(L.brand), V(L.brand) + 18);
    x.textAlign = "center"; x.font = "700 34px 'Share Tech Mono'"; [["POWER", L.rock, -.024], ["AC 100-240 V", L.pcon, .022], ["KEY", L.key, -.019], ["INTERLOCK", L.lock, -.017], ["EMISSION", L.em, -.011], ["ILDA IN", L.ildaIn, .017], ["ILDA OUT", L.ildaOut, .017], ["DMX IN", L.dmxIn, .022], ["DMX OUT", L.dmxOut, .022], ["NET", L.net, .015], ["USB", L.usb, .014], ["MENU", L.enc, -.015], ["BACK", L.back, -.012], ["FAN", L.fan, -.040]].forEach(function (l) { x.fillText(l[0], U(l[1]), V([l[1][0], l[1][1] + l[2]]) + 12); });
    x.fillStyle = "#FFB000"; x.fillRect(U(L.warn) - 128, V(L.warn) - 51, 256, 102); x.fillStyle = "#000"; x.font = "700 19px 'Share Tech Mono'"; x.fillText("PERIGO · RADIAÇÃO LASER", U(L.warn), V(L.warn) - 18); x.fillText("EVITE EXPOSIÇÃO AO FEIXE", U(L.warn), V(L.warn) + 10); x.fillText("CLASSE 4 · 10 W · 445-638 nm", U(L.warn), V(L.warn) + 38);
    x.fillStyle = "#2a2e33"; x.fillRect(U(L.sn) - 128, V(L.sn) - 30, 256, 60); x.fillStyle = "#c9ced3"; x.font = "20px 'Share Tech Mono'"; x.fillText("S/N SC-0512 · 180 W · 50/60 Hz", U(L.sn), V(L.sn) - 6); x.fillText("IEC 60825-1 · MADE IN BR", U(L.sn), V(L.sn) + 20); }, true);
  var silk = add(body, new THREE.PlaneGeometry(W, H), new THREE.MeshStandardMaterial({ map: silkT, metalness: .85, roughness: .45, normalMap: X.brushN, normalScale: new THREE.Vector2(.2, .2), roughnessMap: X.brushR }), 0, H / 2, Z); silk.castShadow = false;
  function screws4(g, s, r) { [[-s, -s], [s, -s], [-s, s], [s, s]].forEach(function (p) { zcyl(g, r || .0015, .002, m.silver, p[0], p[1], .002); }); }
  // powerCON + rocker
  // powerCON: só plugue (a energia liga no rocker, e em lugar nenhum mais); rocker: só liga/desliga
  var pc = port(L.pcon[0], L.pcon[1], "acin"); add(pc, X.rbox(.032, .032, .003, .001), m.black, 0, 0, .0015); zcyl(pc, .012, .012, m.black, 0, 0, .006); zcyl(pc, .0095, .003, m.steel, 0, 0, .0125); add(pc, new THREE.BoxGeometry(.004, .003, .004), m.black, 0, .008, .012); screws4(pc, .013);
  var rk = port(L.rock[0], L.rock[1], "power"); add(rk, X.rbox(.022, .03, .003, .001), m.black, 0, 0, .0015);
  // o rocker bascula na aresta de cima da moldura, como um báscula de verdade: o grupo é o eixo
  var rocker = new THREE.Group(); rocker.position.z = .002; rk.add(rocker); add(rocker, X.rbox(.014, .022, .005, .001), m.plastic, 0, 0, .0035);
  // chave, interlock (um só), LED de emissão
  var ks = port(L.key[0], L.key[1], "keyswitch"); zcyl(ks, .011, .003, m.silver, 0, 0, .0015); zcyl(ks, .007, .006, m.black, 0, 0, .005); var keyM = add(ks, X.rbox(.003, .022, .014, .001), m.brass, 0, 0, .013);
  var il = port(L.lock[0], L.lock[1], "interlock"); var nut = add(il, X.hex(.0075, .003), m.silver, 0, 0, .0015); nut.rotation.x = PI / 2; zcyl(il, .0045, .004, m.black, 0, 0, .004); var lockPlug = new THREE.Group(); il.add(lockPlug); zcyl(lockPlug, .006, .016, m.plastic, 0, 0, .012); var loop = add(lockPlug, new THREE.TorusGeometry(.007, .0015, 8, 20), m.rubber, 0, -.007, .02); loop.rotation.y = PI / 2;
  var emLed = zcyl(body, .0025, .003, m.led(0x2a0a08), L.em[0], py(L.em[1]), Z + .0015);
  // ILDA IN/OUT (DB25), DMX IN/OUT (XLR-5)
  function db25(p, k, label, male) { var g = port(p[0], p[1], k, label); add(g, X.rbox(.055, .014, .004, .002), m.silver, 0, 0, .002); add(g, X.rbox(.046, .008, .002, .001), male ? m.silver : m.black, 0, 0, .0045); if (male) for (var i = 0; i < 13; i++) { zcyl(g, .0005, .002, m.brass, -.021 + i * .0035, .0018, .0055); if (i < 12) zcyl(g, .0005, .002, m.brass, -.0192 + i * .0035, -.0018, .0055); } [-.028, .028].forEach(function (x) { var s = add(g, X.hex(.0025, .005), m.silver, x, 0, .004); s.rotation.x = PI / 2; }); return g; }
  db25(L.ildaIn, "ilda", "", false); db25(L.ildaOut, "ildathru", "", true);
  function xlr5(p, k, label, male) { var g = port(p[0], p[1], k, label); add(g, X.rbox(.026, .026, .003, .001), m.black, 0, 0, .0015); screws4(g, .0105, .0012); zcyl(g, .011, .012, m.silver, 0, 0, .006); zcyl(g, .008, .002, m.black, 0, 0, .012); for (var i = 0; i < 5; i++) { var a = PI / 2 + i * PI * 2 / 5 + PI / 5; if (male) zcyl(g, .0012, .006, m.silver, Math.cos(a) * .0055, Math.sin(a) * .0055, .015); else zcyl(g, .0015, .001, m.rubber, Math.cos(a) * .0055, Math.sin(a) * .0055, .0131); } if (!male) add(g, X.rbox(.006, .004, .004, .001), m.black, 0, .011, .006); return g; }
  xlr5(L.dmxIn, "dmxin", "", false); var dmxOut = xlr5(L.dmxOut, "dmxout", "", true);
  // NET + USB
  var rj = port(L.net[0], L.net[1], "rj45"); add(rj, X.rbox(.017, .014, .003, .001), m.black, 0, 0, .0015); add(rj, new THREE.BoxGeometry(.012, .008, .002), m.plastic, 0, -.001, .003); var led1 = zcyl(rj, .001, .002, m.led(0x0a2a10), -.006, .0055, .003), led2 = zcyl(rj, .001, .002, m.led(0x2a1e00), .006, .0055, .003);
  var ub = port(L.usb[0], L.usb[1], "usb"); add(ub, X.rbox(.013, .011, .004, .001), m.black, 0, 0, .002); add(ub, new THREE.BoxGeometry(.009, .007, .002), m.silver, 0, 0, .004);
  /* Display de rack: 200 × 100 mm de moldura, 190 × 90 mm de vidro, textura de 1024 × 484 — resolução
     suficiente para a linha grande ser lida da vista `rear` sem zoom. Não entra em `pick`: display não
     é controle, e cursor de mão em cima do que não clica é mentira. */
  var OC = document.createElement("canvas"); OC.width = 1024; OC.height = 484; var oc = OC.getContext("2d"), oledTex = new THREE.CanvasTexture(OC); oledTex.minFilter = THREE.LinearFilter; oledTex.anisotropy = 8;
  var og = new THREE.Group(); og.position.set(L.oled[0], py(L.oled[1]), Z); body.add(og);
  add(og, X.rbox(.20, .10, .005, .002), m.black, 0, 0, .0025); add(og, X.rbox(.194, .094, .002, .001), m.dark, 0, 0, .0045);
  var oledM = add(og, new THREE.PlaneGeometry(.19, .09), new THREE.MeshBasicMaterial({ map: oledTex }), 0, 0, .0056); oledM.castShadow = false;
  var eg = port(L.enc[0], L.enc[1], "enc"); zcyl(eg, .003, .006, m.silver, 0, 0, .003); var knob = zcyl(eg, .007, .012, new THREE.MeshStandardMaterial({ color: 0x1e2226, metalness: .6, roughness: .45, flatShading: true }), 0, 0, .009);
  add(knob, new THREE.BoxGeometry(.0015, .001, .005), m.white, 0, .0061, .0035);
  var bk = port(L.back[0], L.back[1], "back"); zcyl(bk, .005, .002, m.silver, 0, 0, .001); var backCap = zcyl(bk, .004, .004, m.plastic, 0, 0, .003);
  // ventoinha axial 60 mm: aro com furo redondo, cubo, 7 pás inclinadas, grade de arame com 4 anéis e 4 raios
  var fg = port(L.fan[0], L.fan[1], "fan"); add(fg, X.ring(.062, .029, .004), m.black, 0, 0, .002); screws4(fg, .027, .0018);
  var blades = new THREE.Group(); blades.position.z = .005; fg.add(blades); zcyl(blades, .011, .008, m.plastic, 0, 0, 0); for (i = 0; i < 7; i++) { var bg = new THREE.Group(); bg.rotation.z = i * PI * 2 / 7; blades.add(bg); var b = add(bg, new THREE.PlaneGeometry(.018, .011), new THREE.MeshStandardMaterial({ color: 0x1a1d20, metalness: .1, roughness: .5, side: THREE.DoubleSide }), .019, 0, 0); b.rotation.x = .8; b.rotation.z = .25; }
  [.0085, .0155, .0225, .0295].forEach(function (r) { add(fg, new THREE.TorusGeometry(r, .0006, 6, 40), m.silver, 0, 0, .0075); }); for (i = 0; i < 4; i++) { var sp = add(fg, cyl(.0006, .06, 6), m.silver, 0, 0, .0075); sp.rotation.z = i * PI / 4; }
  var rearLight = new THREE.SpotLight(0xfff4e6, 0, 2.5, .6, .6, 1.2); rearLight.position.set(.3, .95, 1.1); rearLight.target = rearPlate; scene.add(rearLight);
  return { body: body, lid: lid, screws: screws, APERT: APERT, BEAM_Y: BEAM_Y, keyM: keyM, lockPlug: lockPlug, rocker: rocker, emLed: emLed, led1: led1, led2: led2, knob: knob, backCap: backCap, blades: blades, oled: { c: oc, tex: oledTex, w: 1024, h: 484 }, dmxOut: dmxOut, rearLight: rearLight, wallLight: wallLight, L: L, H: H, D: D, W: W, T: T };
};
