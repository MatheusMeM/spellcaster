/* Pino em 3D: um cabo DMX de verdade, plugado no DMX OUT do projetor, com a ponta macho (XLR-5) erguida em cima
   do flightcase. Os cinco pinos são botões (raycast), a trava manda ele embora, os olhos seguem o mouse, o balão
   Win98 é HTML ancorado na projeção da cabeça. */
window.Pino3D = (function () {
  "use strict";
  var PINS = [["ilda", "ILDA player", "LASER"], ["ndi", "NDI → ILDA", "FÓSFORO"], ["orq", "Orquestrador", "PATCHBAY"], ["cues", "Cenas e cues", "TEATRO DE PAPEL"], ["nfo", "Info", "N"]];
  function build(THREE, X, scene, pick, stage, cam, o) { var m = X.m, PI = Math.PI;
    function add(parent, g, mat, x, y, z, k, label) { var ob = new THREE.Mesh(g, mat); ob.position.set(x, y, z); ob.castShadow = ob.receiveShadow = true; if (k) { ob.userData = { key: k, label: label }; pick.push(ob); } parent.add(ob); return ob; }
    var g = new THREE.Group(); g.position.set(.28, .3, .16); scene.add(g);
    var head = new THREE.Group(); head.position.set(0, .06, 0); head.rotation.x = -.5; g.add(head); head.userData = { key: "pino", label: "Pino: cabo DMX, cinco pinos, zero paciência" }; pick.push(head);
    add(head, new THREE.CylinderGeometry(.010, .010, .06, 24), m.silver, 0, 0, 0); add(head, new THREE.CylinderGeometry(.0105, .0105, .006, 24), m.black, 0, .025, 0);
    add(head, new THREE.CylinderGeometry(.008, .008, .003, 24), m.plastic, 0, .0305, 0);
    var pins = []; PINS.forEach(function (p, i) { var a = PI / 2 + i * PI * 2 / 5 + PI / 5, pin = add(head, new THREE.CylinderGeometry(.0016, .0016, .009, 8), m.silver.clone(), Math.cos(a) * .0055, .0355, -Math.sin(a) * .0055, "pino." + p[0], "pino " + (i + 1) + " · " + p[1] + " · " + p[2]); pins.push(pin); });
    add(head, new THREE.BoxGeometry(.004, .012, .003), m.dark, .0105, .008, .004, "pino.bye", "trava: solta o Pino");
    add(head, new THREE.CylinderGeometry(.007, .0075, .022, 16), m.rubber, 0, -.038, 0);
    var eyes = [], pupils = []; [-.007, .007].forEach(function (x) { var e = add(head, new THREE.SphereGeometry(.006, 16, 12), m.white, x, .014, .0085); e.castShadow = false; eyes.push(e); var p = add(e, new THREE.SphereGeometry(.0032, 12, 8), m.black, 0, 0, .0045); p.castShadow = false; pupils.push(p); var b = add(head, new THREE.BoxGeometry(.009, .0015, .0015), m.dark, x, .0225, .009); b.rotation.z = x < 0 ? .3 : -.3; });
    /* Cabo: da bota, enrolado no case, até a fêmea plugada no DMX OUT — que fica em (.038, .338,
       .1503) no mundo (`body.L.dmxOut`, painel em z = D/2, corpo em y = .314). A fêmea estava presa
       nas coordenadas de um layout de painel antigo (−.015, .359, .19): o Pino aparecia plugado no
       ar, 5 cm ao lado da porta, e o painel DMX OUT dizia "o Pino está plugado aqui" sobre um
       conector vazio. */
    var pts = [[0, .0337, .0144], [.01, .012, .035], [.035, .004, .055], [-.01, .004, .07], [-.05, .004, .04], [-.04, .004, -.01], [-.12, .004, -.02], [-.2, .004, .02], [-.225, .006, .045], [-.242, .038, .053]];
    g.add(X.tube(pts, .003)); var fem = add(scene, new THREE.CylinderGeometry(.011, .011, .045, 20), m.silver, .038, .338, .173); fem.rotation.x = PI / 2; add(scene, new THREE.CylinderGeometry(.0075, .007, .016, 16), m.rubber, .038, .338, .203).rotation.x = PI / 2;
    // balão
    var bal = document.createElement("div"); bal.className = "bal"; stage.appendChild(bal); var talking = 0, t = 0, blink = 0, gone = false, cur = null, v = new THREE.Vector3();
    function say(text, items, hint) { bal.innerHTML = '<span class="x">×</span><b>' + text + "</b>" + (items ? "<ul>" + items.map(function (it) { return "<li data-a=\"" + it[0] + "\">" + it[1] + (it[2] ? " <small>" + it[2] + "</small>" : "") + "</li>"; }).join("") + "</ul>" : "") + (hint === false ? "" : '<div class="hint">' + (hint || "os pinos são o menu: 1 laser · 2 fósforo · 3 patchbay · 4 teatro · 5 info · trava = some") + "</div>"); bal.classList.add("on"); talking = 1.2; }
    function hide() { bal.classList.remove("on"); }
    bal.addEventListener("click", function (e) { if (e.target.classList.contains("x")) { hide(); return; } var li = e.target.closest("li"); if (li && o.on) o.on(li.dataset.a); });
    function current(k) { cur = k; pins.forEach(function (p, i) { p.material.emissive.setHex(PINS[i][0] === k ? 0x38ff5c : 0); p.material.emissiveIntensity = .8; }); }
    function bye() { gone = true; say("Tá, me solta. Puxa pelo cabo se precisar.", null, false); setTimeout(hide, 1600); }
    function back() { gone = false; }
    function update(dt, mouse, W, H, minTop) { t += dt; if (talking > 0) talking -= dt; var down = gone ? 1 : 0; head.rotation.x += ((down ? 1.1 : -.5) - head.rotation.x) * Math.min(1, dt * 3); head.position.y += ((down ? .012 : .06) - head.position.y) * Math.min(1, dt * 3);
      head.rotation.z = talking > 0 ? Math.sin(t * 28) * .06 : Math.sin(t * 1.3) * .02; head.position.x = Math.sin(t * .9) * .002;
      blink -= dt; if (blink < -3.4 - Math.random()) blink = .11; var sy = blink > 0 ? .15 : 1; eyes.forEach(function (e) { e.scale.y += (sy - e.scale.y) * Math.min(1, dt * 30); });
      v.set(0, .014, 0); head.localToWorld(v); v.project(cam); var sx = (v.x + 1) / 2 * W, sy2 = (1 - v.y) / 2 * H; if (mouse) { var dx = mouse[0] - sx, dy = mouse[1] - sy2, d = Math.max(1, Math.hypot(dx, dy)), k = Math.min(1, d / 200) * .0028; pupils.forEach(function (p) { p.position.x = dx / d * k; p.position.y = -dy / d * k; }); }
      v.set(0, .04, .012); head.localToWorld(v); v.project(cam); bal.style.left = Math.max(270, Math.min(W - 10, (v.x + 1) / 2 * W - 14)) + "px";
      // `minTop` e' a base da traseira do aparelho na tela: o balao desce para debaixo dela, senao
      // tapa o display e os controles — que sao o motivo de o programa existir.
      var top = Math.max((1 - v.y) / 2 * H + 16, Math.max(0, Math.min(H - 170, minTop || 0)));
      bal.style.top = Math.min(H - 40, top) + "px"; }
    return { group: g, head: head, say: say, hide: hide, current: current, bye: bye, back: back, update: update, PINS: PINS }; }
  return { build: build, PINS: PINS };
})();
