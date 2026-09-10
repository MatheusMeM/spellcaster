/* Sala, flightcase e o corpo do projetor (400 × 180 × 300 mm): chapas, aletas laterais, forquilha, pés, tampa com
   dobradiça de verdade e parafusos, abertura na frente e o painel traseiro completo (inspirado no Clubmax:
   powerCON + rocker, chave, um interlock só, LED de emissão, LED grande de ARMED, ILDA IN/OUT DB25,
   DMX IN/OUT em XLR-3 Neutrik D-shell, NET, display de rack + encoder + BACK, ventoinha axial de 60 mm com a
   hélice desenhada em textura atrás da grade de arame, etiqueta de perigo e placa de identificação).
   Coordenadas em metros, origem no fundo do corpo.

   Referências de forma (nada é baixado; tudo é modelado com as primitivas de mat.js):
   knob        Davies 1900H / Bourns PEC11 — corpo cônico estriado, colar cromado, indicador em ranhura
   DMX IN/OUT  Neutrik NC3FD-L-1 (fêmea) e NC3MD-L-1 (macho) — D-shell 24 mm, 2 furos na diagonal, 3 pinos a 120°
   ventoinha   Sunon/Delta 60 × 60 × 25 mm — 7 pás, grade de arame de 4 anéis
   etiqueta    IEC 60825-1 — campo amarelo, moldura preta e pictograma de radiação laser

   UM CONTROLE, UMA FUNÇÃO. A classificação de cada peça clicável é dado, não convenção: mora em
   `LaserEngine.CONTROLS` (engine.js), é de onde sai o rótulo do tooltip e é o que `app.js` consulta para decidir
   o que o clique faz. Nenhuma peça faz duas coisas, e nenhuma função tem dois pontos.

   | família     | o que o clique faz              | peças |
   |-------------|---------------------------------|-------|
   | toggle      | inverte um estado, e só          | `power` (rocker), `keyswitch`, `interlock` |
   | momentary   | age enquanto apertado            | — (nenhuma hoje) |
   | valor       | muda um número                   | encoder girando (roda do mouse ou arrasto sobre ele) e os faders dos painéis |
   | conector    | nada: é plugue                   | `acin` (powerCON) |
   | navegação   | abre a tela daquela peça         | `enc`, `back`, portas (`ilda`, `ildathru`, `dmxin`, `dmxout`, `rj45`), `lid`, `fan`, e as peças de dentro |

   O display não entra no raycast: display não é controle. A energia liga só no rocker (o powerCON é plugue), a
   emissão arma só na chave, e o interlock é um só — o obturador em `optics.js` reage a ele, mas é indicação. */
window.BODY = function (THREE, X, scene, pick) {
  "use strict";
  var m = X.m, PI = Math.PI, LBL = LaserEngine.labelOf;
  function add(parent, g, mat, x, y, z, k, label) { var o = new THREE.Mesh(g, mat); o.position.set(x, y, z); o.castShadow = o.receiveShadow = true; if (k) { o.userData = { key: k, label: label || LBL(k) }; pick.push(o); } parent.add(o); return o; }
  function cyl(r, h, seg) { return new THREE.CylinderGeometry(r, r, h, seg || 20); }
  function zcyl(parent, r, h, mat, x, y, z, k, label) { var o = add(parent, cyl(r, h), mat, x, y, z, k, label); o.rotation.x = PI / 2; return o; }
  // maior corpo de letra que cabe em `w` px: a fonte que o canvas usa pode não ser a pedida, então quem
  // decide o tamanho é o measureText, não uma constante. É o que fazia o texto estourar as placas.
  function fit(x, lines, w, px, font) { for (; px > 6; px--) { x.font = "700 " + px + "px " + font; var ok = true; for (var i = 0; i < lines.length; i++) if (x.measureText(lines[i]).width > w) ok = false; if (ok) break; } return px; }
  function textW(x, lines) { var w = 0; lines.forEach(function (l) { w = Math.max(w, x.measureText(l).width); }); return w; }
  // vetor da marca (brand/*.svg) desenhado no canvas: chega depois, então quem chama marca needsUpdate
  function svgInto(src, draw) { var im = new Image(); im.onload = function () { draw(im); }; im.src = src; return im; }
  function svgTex(src, cw, ch, alpha) { var c = document.createElement("canvas"); c.width = cw; c.height = ch; var t = new THREE.CanvasTexture(c); t.encoding = THREE.sRGBEncoding; svgInto(src, function (im) { var x = c.getContext("2d"); x.globalAlpha = alpha || 1; x.drawImage(im, 0, 0, cw, ch); t.needsUpdate = true; }); return t; }
  /* sala */
  var floor = add(scene, new THREE.PlaneGeometry(14, 14), m.floor, 0, 0, -2); floor.rotation.x = -PI / 2; floor.castShadow = false;
  var wall = add(scene, new THREE.PlaneGeometry(8, 5), m.wall, 0, 2.2, -5.01); wall.castShadow = false;
  var wallLight = new THREE.PointLight(0x6f86b0, 14, 7, 2); wallLight.position.set(0, 2.6, -3.2); scene.add(wallLight);
  /* flightcase: caixa, cantoneiras nos oito cantos e rebites nas bordas */
  add(scene, X.rbox(.62, .3, .46, .004), m.caseM, 0, .15, 0);
  var rivet = new THREE.SphereGeometry(.004, 10, 8);   // ponytail: a cantoneira é o próprio perfil de canto; cubo de canto embutido na chapa virava um adesivo branco
  [-.29, .29].forEach(function (x) { [-.21, .21].forEach(function (z) { add(scene, X.rbox(.03, .3, .03, .003), m.silver, x, .15, z); }); });
  [-.2, 0, .2].forEach(function (x) { [-.22, .22].forEach(function (z) { add(scene, rivet, m.steel, x, .29, z * 1.01); }); });
  [-.235, -.08, .08, .235].forEach(function (z) { [-.305, .305].forEach(function (x) { add(scene, rivet, m.steel, x, .29, z); }); });
  /* corpo */
  var body = new THREE.Group(); body.position.y = .314; scene.add(body);
  var W = .40, H = .18, D = .30, T = .006;
  add(body, X.rbox(W, T, D, .004), m.alu, 0, T / 2, 0);
  add(body, X.rbox(W, H, T, .004), m.alu, 0, H / 2, -D / 2 + T / 2, "front");
  var rearPlate = add(body, X.rbox(W, H, T, .004), m.alu, 0, H / 2, D / 2 - T / 2);
  add(body, X.rbox(T, H, D, .004), m.alu, -W / 2 + T / 2, H / 2, 0); add(body, X.rbox(T, H, D, .004), m.alu, W / 2 - T / 2, H / 2, 0, "side");
  // ponytail: aleta é caixa. rbox aqui custa ~7 k triângulos por um raio de 1 mm que nenhuma vista mostra.
  var finG = new THREE.BoxGeometry(.006, .0025, .24);
  for (var i = 0; i < 8; i++) [-1, 1].forEach(function (s) { add(body, finG, m.alu, s * (W / 2 + .002), .03 + i * .016, -.01); });
  // forquilha (yoke) com manípulos, e pés de borracha
  add(body, X.rbox(.46, .008, .035, .002), m.dark, 0, -.010, 0); [-1, 1].forEach(function (s) { add(body, X.rbox(.008, .14, .035, .002), m.dark, s * .234, .06, 0); var k = add(body, cyl(.016, .012, 16), new THREE.MeshStandardMaterial({ color: 0x1e2226, metalness: .6, roughness: .5, flatShading: true }), s * .244, .10, 0); k.rotation.z = PI / 2; var sc = add(body, cyl(.004, .002, 12), m.silver, s * .251, .10, 0); sc.rotation.z = PI / 2; });
  [-.2, .2].forEach(function (x) { [-.09, .09].forEach(function (z) { add(body, cyl(.009, .006, 16), m.rubber, x, -.017, z); }); });
  /* tampa com dobradiça de verdade.

     `app.js` abre a tampa com `B.lid.rotation.x = -lT * 1.9` — ângulo NEGATIVO. Com a dobradiça na traseira
     (como era) o negativo joga a chapa para BAIXO e para trás: ela atravessava o painel traseiro e o flightcase
     no caminho inteiro, e é isso que o dono viu clipar. O pivô certo para esse sinal é o da frente (z = −D/2):
     a tampa sobe e tomba para a frente, longe da câmera de dentro (que fica atrás, em z > 0). O pivô fica FORA
     da chapa, no centro do raio da aresta dianteira (r = 4,5 mm), e por isso a tampa gira sem varrer o canto do
     corpo: nenhum ângulo entre 0 e −1,9 rad atravessa nada, e não é preciso limitar o curso. Os parafusos são
     filhos da tampa e sobem com ela. */
  var HR = .0045, lid = new THREE.Group(); lid.position.set(0, H + T / 2, -D / 2 + HR); body.add(lid);
  var lidM = add(lid, X.rbox(W, T, D - HR, .004), m.alu, 0, 0, (D - HR) / 2, "lid");
  [-.13, .13].forEach(function (x) { var b = add(lid, cyl(.004, .05, 14), m.steel, x, 0, 0); b.rotation.z = PI / 2;   // canela
    add(lid, X.rbox(.05, .0016, .014, .0006), m.steel, x, .0036, .010);                                              // aba na tampa
    add(body, X.rbox(.05, .012, .0016, .0006), m.steel, x, H - .009, -D / 2 - .0011); });                            // aba no corpo
  var screws = []; [[-.18, .0455], [.18, .0455], [-.18, .1405], [.18, .1405], [-.18, .2455], [.18, .2455]].forEach(function (p) { var s = add(lid, cyl(.0045, .004, 14), m.steel, p[0], .004, p[1], "lid"); add(s, cyl(.0034, .0007, 14), m.dark, 0, .0019, 0); var sk = add(s, X.hex(.0021, .0022), m.black, 0, .0022, 0); sk.rotation.y = .3; screws.push(s); });
  // etiqueta de perigo da tampa: o campo amarelo É o canvas inteiro, e o texto é medido para caber nele
  var lbl = X.tex(512, 256, function (x, w, h) {
    x.fillStyle = "#FFB000"; x.fillRect(0, 0, w, h); x.fillStyle = "#000"; x.fillRect(0, 0, w, 9); x.fillRect(0, h - 9, w, 9); x.fillRect(0, 0, 9, h); x.fillRect(w - 9, 0, 9, h);
    var cx = 86, cy = 128, r = 60;   // pictograma IEC 60825-1: triângulo com o feixe irradiando
    x.beginPath(); x.moveTo(cx, cy - r); x.lineTo(cx + r * .92, cy + r * .72); x.lineTo(cx - r * .92, cy + r * .72); x.closePath(); x.lineWidth = r * .13; x.strokeStyle = "#000"; x.lineJoin = "round"; x.stroke();
    x.save(); x.translate(cx - r * .36, cy + r * .30); x.fillStyle = "#000"; x.beginPath(); x.arc(0, 0, r * .10, 0, 7); x.fill();
    for (var j = 0; j < 6; j++) { x.save(); x.rotate(-.62 + j * .21); x.fillRect(r * .13, -r * .028, r * .78, r * .056); x.restore(); } x.restore();
    var lines = ["PERIGO · RADIAÇÃO LASER", "EVITE EXPOSIÇÃO AO FEIXE", "SAÍDA 10 W · 445-638 nm", "PRODUTO LASER CLASSE 4"], px = fit(x, lines, 318, 32, "'Share Tech Mono'");
    x.textAlign = "left"; x.fillStyle = "#000"; lines.forEach(function (l, j2) { x.fillText(l, 170, 72 + j2 * (px + 13)); });
  }, true);
  var lab = add(lid, new THREE.PlaneGeometry(.07, .035), new THREE.MeshStandardMaterial({ map: lbl, metalness: 0, roughness: .62 }), -.12, .0032, .0955); lab.rotation.x = -PI / 2; lab.castShadow = false;
  var sub = add(lid, new THREE.PlaneGeometry(.042, .0127), new THREE.MeshStandardMaterial({ map: svgTex("brand/submark.svg", 512, 155, .45), transparent: true, metalness: .7, roughness: .45 }), .14, .0032, .2355); sub.rotation.x = -PI / 2; sub.castShadow = false;
  // frente: abertura com janela
  var BEAM_Y = .057; add(body, X.rbox(.04, .026, .004, .001), m.black, .095, BEAM_Y, -D / 2 - .001, "aperture");
  var apGlass = add(body, new THREE.PlaneGeometry(.03, .018), m.glassDark, .095, BEAM_Y, -D / 2 - .0035); apGlass.rotation.y = PI; apGlass.castShadow = false;
  var APERT = new THREE.Vector3(.095, .314 + BEAM_Y, -D / 2 - .005);
  /* painel traseiro */
  var Z = D / 2 + .0003, py = function (y) { return H / 2 + y; };
  function port(x, y, k, label) { var g = new THREE.Group(); g.position.set(x, py(y), Z); if (k) { g.userData = { key: k, label: label || LBL(k) }; pick.push(g); } body.add(g); return g; }
  /* Layout do painel: o display de rack ocupa a faixa central (200 × 100 mm — metade da largura do painel),
     energia e segurança ficam na coluna da esquerda, encoder e ventoinha na direita, e todas as portas descem
     para a régua de baixo. O display subiu 10 mm e a régua desceu 4 mm: a moldura tem 5 mm de profundidade e
     comia os rótulos das portas assim que a câmera saía do eixo. A USB saiu do aparelho. */
  var L = { title: [-.19, .078], brand: [.19, .078], rock: [-.180, .048], key: [-.180, -.004], lock: [-.145, .048], em: [-.145, .010], armed: [-.145, -.030],
    oled: [-.012, .018], enc: [.108, .040], back: [.108, -.010], fan: [.158, .014],
    pcon: [-.175, -.070], ildaIn: [-.108, -.070], ildaOut: [-.043, -.070], dmxIn: [.005, -.070], dmxOut: [.040, -.070], net: [.072, -.066],
    warn: [.152, -.052], sn: [.152, -.078] };
  var U = function (p) { return (p[0] + .2) * 5120; }, V = function (p) { return (.09 - p[1]) * 5111; };
  var PLATE = { w: 300, h: 86 };   // área da placa de identificação no canvas da serigrafia; B.plate redesenha só ela
  /* A serigrafia é desenhada duas vezes: em cor, e em cinza para o mapa de rugosidade — tinta é mais fosca que
     a chapa escovada, e é esse relevo que faz a serigrafia parecer impressa em vez de colada. */
  function silkDraw(x, rough) {
    var ink = rough ? "#e8e8e8" : "#c9ced3", base = rough ? "#3c3c3c" : "#101214";
    x.fillStyle = base; x.fillRect(0, 0, 2048, 920);
    x.fillStyle = ink; x.textAlign = "left"; x.font = "700 58px Michroma"; x.fillText("SPELLCASTER", U(L.title), V(L.title) + 20);
    x.textAlign = "center"; x.font = "700 34px 'Share Tech Mono'";
    [["POWER", L.rock, -.024], ["AC 100-240 V", L.pcon, .022], ["KEY", L.key, -.019], ["INTERLOCK", L.lock, .013], ["EMISSION", L.em, -.011], ["ARMED", L.armed, .011],
      ["ILDA IN", L.ildaIn, .017], ["ILDA OUT", L.ildaOut, .017], ["DMX IN", L.dmxIn, .022], ["DMX OUT", L.dmxOut, .022], ["NET", L.net, .015],
      ["MENU", L.enc, -.019], ["BACK", L.back, -.014], ["FAN", L.fan, -.040]].forEach(function (l) { x.fillText(l[0], U(l[1]), V([l[1][0], l[1][1] + l[2]]) + 12); });
    // etiqueta de perigo: o retângulo amarelo é dimensionado PELO texto, e o texto encolhe até caber em 48 mm
    var wl = ["PERIGO · RADIAÇÃO LASER", "EVITE EXPOSIÇÃO AO FEIXE", "CLASSE 4 · 10 W · 445-638 nm"], px = fit(x, wl, 290, 24, "'Share Tech Mono'"), ww = textW(x, wl) + 22, wh = wl.length * (px + 7) + 12;
    x.fillStyle = rough ? "#8c8c8c" : "#FFB000"; x.fillRect(U(L.warn) - ww / 2, V(L.warn) - wh / 2, ww, wh);
    x.fillStyle = rough ? "#dcdcdc" : "#000"; wl.forEach(function (l, j) { x.fillText(l, U(L.warn), V(L.warn) - wh / 2 + 12 + j * (px + 7) + px * .1); });
    x.fillStyle = rough ? "#565656" : "#2a2e33"; x.fillRect(U(L.sn) - PLATE.w / 2, V(L.sn) - PLATE.h / 2, PLATE.w, PLATE.h);
  }
  var silkT = X.tex(2048, 920, function (x) { silkDraw(x, false); }, true);
  var silkR = X.tex(1024, 460, function (x) { x.scale(.5, .5); silkDraw(x, true); });
  var sx = silkT.image.getContext("2d");
  /* Placa de identificação: FIRMWARE V<versão> vem do engine (`hud-4` chama `B.plate` ao conectar); sem engine
     a placa diz FIRMWARE OFFLINE, porque versão inventada é mentira gravada no chassi. Os 180 W / 50-60 Hz
     saíram: número copiado de datasheet alheio não serve a nenhuma função do software. */
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
  // a marca vetorial no lugar do texto "FEITIÇARIA iNDUSTRIAL" (e "LASER 10 W RGB · … · 40 kpps" saiu de vez)
  svgInto("brand/wordmark.svg", function (im) { var h = 46, w = h * (im.width && im.height ? im.width / im.height : 7.18); sx.drawImage(im, U(L.brand) - w, V(L.brand) - h / 2 + 4, w, h); silkT.needsUpdate = true; });
  var silk = add(body, new THREE.PlaneGeometry(W, H), new THREE.MeshStandardMaterial({ map: silkT, metalness: .85, roughness: 1, roughnessMap: silkR, normalMap: X.brushN, normalScale: new THREE.Vector2(.2, .2) }), 0, H / 2, Z); silk.castShadow = false;
  function screws4(g, s, r) { [[-s, -s], [s, -s], [-s, s], [s, s]].forEach(function (p) { zcyl(g, r || .0015, .002, m.silver, p[0], p[1], .002); }); }
  // powerCON: só plugue (a energia liga no rocker, e em lugar nenhum mais); rocker: só liga/desliga
  var pc = port(L.pcon[0], L.pcon[1], "acin"); add(pc, X.rbox(.032, .032, .003, .001), m.black, 0, 0, .0015); zcyl(pc, .012, .012, m.black, 0, 0, .006); zcyl(pc, .0095, .003, m.steel, 0, 0, .0125); add(pc, new THREE.BoxGeometry(.004, .003, .004), m.black, 0, .008, .012); screws4(pc, .013);
  var rk = port(L.rock[0], L.rock[1], "power"); add(rk, X.rbox(.022, .03, .003, .001), m.black, 0, 0, .0015);
  // o rocker bascula na aresta de cima da moldura, como um báscula de verdade: o grupo é o eixo
  var rocker = new THREE.Group(); rocker.position.z = .002; rk.add(rocker); add(rocker, X.rbox(.014, .022, .005, .001), m.plastic, 0, 0, .0035);
  // chave, interlock (um só), LED de emissão e o LED grande de ARMED
  var ks = port(L.key[0], L.key[1], "keyswitch"); zcyl(ks, .011, .003, m.silver, 0, 0, .0015); zcyl(ks, .007, .006, m.black, 0, 0, .005); var keyM = add(ks, X.rbox(.003, .022, .014, .001), m.brass, 0, 0, .013);
  var il = port(L.lock[0], L.lock[1], "interlock"); var nut = add(il, X.hex(.0075, .003), m.silver, 0, 0, .0015); nut.rotation.x = PI / 2; zcyl(il, .0045, .004, m.black, 0, 0, .004); var lockPlug = new THREE.Group(); il.add(lockPlug); zcyl(lockPlug, .006, .016, m.plastic, 0, 0, .012); var loop = add(lockPlug, new THREE.TorusGeometry(.007, .0015, 8, 20), m.rubber, 0, -.007, .02); loop.rotation.y = PI / 2;
  var emLed = zcyl(body, .0025, .003, m.led(0x2a0a08), L.em[0], py(L.em[1]), Z + .0015);
  /* LED de ARMED de 9 mm: cúpula, colar cromado e um halo aditivo — o indicador de armado tem de ser visível
     no modelo 3D de qualquer distância, não só no HUD. `hud-4` pinta `B.armLed.material.color` (SISTEMA.md §5:
     apagado sem energia, vermelho piscando desarmado, verde armado) e o halo usa a MESMA instância de Color,
     então acompanha sem ninguém pintar duas vezes. */
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
  /* DMX512 em XLR-3 (é o que este aparelho usa; XLR-5 é do padrão, não deste chassi). Neutrik D-shell: flange
     de 24 mm com dois furos na diagonal, colar de 19,6 mm, contatos a 120° com o pino 1 em cima à esquerda
     visto de frente. O inserto preto com "1 2 3" é uma textura só, compartilhada pelos dois conectores. */
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
    zcyl(g, .0098, .0092, m.silver, 0, 0, .0046);                                                    // colar
    zcyl(g, .0093, .0012, m.dark, 0, 0, .0098);                                                      // chanfro do colar
    var ins = add(g, new THREE.CircleGeometry(.0088, 28), male ? insM : insF, 0, 0, .0105); ins.castShadow = false;
    PIN.forEach(function (q) { var a = q[0] * PI / 180, cx = Math.cos(a) * PINR, cy = Math.sin(a) * PINR;
      if (male) zcyl(g, .00115, .0062, m.silver, cx, cy, .0135); else zcyl(g, .0018, .0016, m.black, cx, cy, .0099); });
    if (!male) add(g, X.rbox(.006, .0035, .0045, .0008), m.black, 0, .0072, .0075);                  // trava do NC3FD
    return g;
  }
  xlr3(L.dmxIn, "dmxin", "", false); var dmxOut = xlr3(L.dmxOut, "dmxout", "", true);
  // NET
  var rj = port(L.net[0], L.net[1], "rj45"); add(rj, X.rbox(.017, .014, .003, .001), m.black, 0, 0, .0015); add(rj, new THREE.BoxGeometry(.012, .008, .002), m.plastic, 0, -.001, .003); var led1 = zcyl(rj, .001, .002, m.led(0x0a2a10), -.006, .0055, .003), led2 = zcyl(rj, .001, .002, m.led(0x2a1e00), .006, .0055, .003);
  /* Display de rack: 200 × 100 mm de moldura, 190 × 90 mm de vidro, textura de 1024 × 484 — resolução
     suficiente para a linha grande ser lida da vista `rear` sem zoom. Não entra em `pick`: display não
     é controle, e cursor de mão em cima do que não clica é mentira. */
  var OC = document.createElement("canvas"); OC.width = 1024; OC.height = 484; var oc = OC.getContext("2d"), oledTex = new THREE.CanvasTexture(OC); oledTex.minFilter = THREE.LinearFilter; oledTex.anisotropy = 8;
  var og = new THREE.Group(); og.position.set(L.oled[0], py(L.oled[1]), Z); body.add(og);
  add(og, X.rbox(.20, .10, .005, .002), m.black, 0, 0, .0025); add(og, X.rbox(.194, .094, .002, .001), m.dark, 0, 0, .0045);
  var oledM = add(og, new THREE.PlaneGeometry(.19, .09), new THREE.MeshBasicMaterial({ map: oledTex }), 0, 0, .0056); oledM.castShadow = false;
  /* Encoder tipo Davies 1900H: bucha sextavada, eixo D à vista na folga, colar cromado, corpo cônico com 18
     estrias (uma Shape polar extrudada — cilindro com flatShading não faz estria) e o indicador branco na
     ranhura, do topo até a saia. `app.js` gira em `rotation.y` (o eixo do cilindro, porque o grupo já vem com
     rotation.x = π/2) e afunda em `position.z`. `B.knobHit` é o alvo grande e invisível do arrasto do `cam-4`. */
  var eg = port(L.enc[0], L.enc[1], "enc");
  var bush = add(eg, X.hex(.0055, .0035), m.silver, 0, 0, .0018); bush.rotation.x = PI / 2;
  zcyl(eg, .0028, .006, m.silver, 0, 0, .0045);
  add(eg, new THREE.BoxGeometry(.0009, .0056, .006), m.dark, .0023, 0, .0045);                       // o rebaixo do eixo "D"
  var knob = new THREE.Group(); knob.rotation.x = PI / 2; knob.position.set(0, 0, .009); eg.add(knob);
  var sh = new THREE.Shape(), SEG = 96;
  for (i = 0; i <= SEG; i++) { var th = i / SEG * PI * 2, rr = .0080 + .00055 * Math.cos(18 * th); if (i) sh.lineTo(Math.cos(th) * rr, Math.sin(th) * rr); else sh.moveTo(Math.cos(th) * rr, Math.sin(th) * rr); }
  var kg = new THREE.ExtrudeGeometry(sh, { depth: .0085, bevelEnabled: true, bevelThickness: .0009, bevelSize: .0009, bevelSegments: 2 }); kg.rotateX(-PI / 2);
  add(knob, kg, new THREE.MeshStandardMaterial({ color: 0x121417, metalness: .35, roughness: .48, roughnessMap: X.grainR, envMapIntensity: .5 }), 0, -.0042, 0);
  add(knob, cyl(.0092, .0018, 36), m.silver, 0, -.0051, 0);                                          // colar cromado
  add(knob, cyl(.0068, .0009, 32), m.black, 0, .0047, 0);                                            // tampa com chanfro
  add(knob, new THREE.BoxGeometry(.0014, .0011, .0056), m.white, 0, .0047, .0038);                   // indicador no topo
  add(knob, new THREE.BoxGeometry(.0014, .0072, .0013), m.white, 0, .0005, .0081);                   // indicador na saia
  var knobHit = add(eg, cyl(.016, .030, 12), new THREE.MeshBasicMaterial({ transparent: true, opacity: 0, depthWrite: false, colorWrite: false }), 0, 0, .012, "enc"); knobHit.rotation.x = PI / 2; knobHit.castShadow = knobHit.receiveShadow = false;
  var bk = port(L.back[0], L.back[1], "back"); zcyl(bk, .005, .002, m.silver, 0, 0, .001); var backCap = zcyl(bk, .004, .004, m.plastic, 0, 0, .003);
  /* Ventoinha axial de 60 mm: aro, grade de arame (malha) e a hélice em TEXTURA girando. Sete pás de malha
     giravam sete objetos por frame para virar um borrão que ninguém vê parado; um plano com as pás desenhadas
     (com o arrasto já pintado) dá a mesma leitura por 2 triângulos. `B.blades` continua sendo o grupo que
     `app.js` gira em `rotation.z`. */
  // a moldura é da mesma chapa do painel (m.alu), não plástico preto: com metalness 0 um albedo de 4 % sob o
  // refletor da sala sai CINZA-CLARO, e o quadrado da ventoinha era a peça mais clara do painel inteiro
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
  // centro da face do DMX OUT em coordenadas de mundo: é de onde o cabo do Pino sai (contrato com `pino-4`)
  body.updateMatrixWorld(true);
  var dmxOutWorld = dmxOut.localToWorld(new THREE.Vector3(0, 0, .0115));
  return { body: body, lid: lid, screws: screws, APERT: APERT, BEAM_Y: BEAM_Y, keyM: keyM, lockPlug: lockPlug, rocker: rocker, emLed: emLed, armLed: armLed, led1: led1, led2: led2, knob: knob, knobHit: knobHit, backCap: backCap, blades: blades, oled: { c: oc, tex: oledTex, w: 1024, h: 484 }, dmxOut: dmxOut, dmxOutWorld: dmxOutWorld, plate: plate, rearLight: rearLight, wallLight: wallLight, L: L, H: H, D: D, W: W, T: T };
};
