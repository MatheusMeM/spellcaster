/* Pino em 3D: um cabo DMX de verdade que é o menu do programa. Ele não fica mais em cima do
   flightcase, no mundo, onde a câmera o deixava para trás: o grupo é FILHO DA CÂMERA, no canto
   inferior esquerdo, ~120 px de altura em qualquer janela, com luz própria e desenhado por cima do
   aparelho (a sentinela `clr` limpa a profundidade antes da turma dele). Os cinco pinos continuam
   sendo botões (raycast), os olhos seguem o mouse, o balão Win98 é HTML ancorado na projeção da
   cabeça — agora abrindo para a DIREITA, porque o Pino mudou de lado.
   O cabo sai da BOTA (a ponta de trás, no eixo dele) e vai até o DMX OUT do aparelho como corda de
   Verlet (`rope.js`): como a ponta de cima anda com a câmera, o cabo acompanha a câmera até o laser
   de onde ela estiver. O botão `–` do balão manda o Pino embora: ele desce, encolhe, o cabo sai da
   cena e a corda PARA de ser simulada; no lugar fica o ícone `#pino-min`, que o traz de volta. */
window.Pino3D = (function () {
  "use strict";
  var PINS = [["ilda", "ILDA player", "LASER"], ["ndi", "NDI → ILDA", "FÓSFORO"], ["orq", "Orquestrador", "PATCHBAY"], ["cues", "Cenas e cues", "TEATRO DE PAPEL"], ["nfo", "Info", "N"]];
  var N = 24, RS = 6, RAD = .0032, RAD0 = .0011, DIST = .25, ALTO = 120, MARGX = 90, MARGY = 48, FOLGA = 1.03, GRAV = -3.5;
  function build(THREE, X, scene, pick, stage, cam, o) { var m = X.m, PI = Math.PI;
    var TGT = (o && o.target) || new THREE.Vector3(.038, .338, .173);
    function add(parent, g, mat, x, y, z, k, label) { var ob = new THREE.Mesh(g, mat); ob.position.set(x, y, z); ob.castShadow = ob.receiveShadow = true; ob.renderOrder = 999; if (k) { ob.userData = { key: k, label: label }; pick.push(ob); } parent.add(ob); return ob; }
    var g = new THREE.Group();
    var head = new THREE.Group(); head.position.set(0, .06, 0); head.rotation.x = -.32; g.add(head); head.userData = { key: "pino", label: "Pino: cabo DMX, cinco pinos, zero paciência" }; pick.push(head);
    add(head, new THREE.CylinderGeometry(.010, .010, .06, 24), m.silver, 0, 0, 0); add(head, new THREE.CylinderGeometry(.0105, .0105, .006, 24), m.black, 0, .025, 0);
    add(head, new THREE.CylinderGeometry(.008, .008, .003, 24), m.plastic, 0, .0305, 0);
    var pins = []; PINS.forEach(function (p, i) { var a = PI / 2 + i * PI * 2 / 5 + PI / 5, pin = add(head, new THREE.CylinderGeometry(.0016, .0016, .009, 8), m.silver.clone(), Math.cos(a) * .0055, .0355, -Math.sin(a) * .0055, "pino." + p[0], "pino " + (i + 1) + " · " + p[1] + " · " + p[2]); pins.push(pin); });
    add(head, new THREE.BoxGeometry(.004, .012, .003), m.dark, .0105, .008, .004, "pino.bye", "trava: solta o Pino");
    add(head, new THREE.CylinderGeometry(.007, .0075, .022, 16), m.rubber, 0, -.038, 0);
    var eyes = [], pupils = []; [-.007, .007].forEach(function (x) { var e = add(head, new THREE.SphereGeometry(.006, 16, 12), m.white, x, .014, .0085); e.castShadow = false; eyes.push(e); var p = add(e, new THREE.SphereGeometry(.0032, 12, 8), m.black, 0, 0, .0045); p.castShadow = false; pupils.push(p); var b = add(head, new THREE.BoxGeometry(.009, .0015, .0015), m.dark, x, .0225, .009); b.rotation.z = x < 0 ? .3 : -.3; });

    /* Desenhado por cima do aparelho sem `depthTest:false` peça a peça: os materiais do Pino são os
       mesmos objetos do resto do modelo (`X.m.silver` e companhia), e desligar o teste neles apagaria
       a profundidade do aparelho inteiro. A sentinela abaixo entra na lista de desenho logo antes da
       turma do Pino (renderOrder 998 < 999) e limpa o buffer de profundidade: dali para a frente ele
       é o único que existe, e continua se auto-ocultando direito (pino atrás de cabeça). */
    var clr = new THREE.Mesh(new THREE.PlaneGeometry(.001, .001), new THREE.MeshBasicMaterial({ colorWrite: false, depthWrite: false, depthTest: false }));
    clr.renderOrder = 998; clr.frustumCulled = false; clr.onBeforeRender = function (r) { r.clearDepth(); }; g.add(clr);
    g.rotation.set(0, .34, .05); g.updateMatrixWorld(true); // 3/4: os olhos ficam de frente para quem olha
    var bb = new THREE.Box3().setFromObject(g), H0 = (bb.max.y - bb.min.y) || .09, X0 = bb.min.x, Y0 = bb.min.y;
    // o Pino tem ALTO px de altura em qualquer janela, entao a largura dele tambem e' fixa em px:
    // o balao comeca depois dela, senao a caixa de fala cai em cima do proprio Pino.
    var BALX = MARGX + (bb.max.x - X0) / H0 * ALTO + 26;
    cam.add(g); if (!cam.parent) scene.add(cam);
    // luz propria: sem ela o Pino fica na sombra do aparelho, que e' onde a camera sempre esta'.
    // O alcance curto (0,4 m) e' o que faz dela luz DELE: com alcance de 1 m a mesma lampada
    // estourava a traseira do aparelho, que fica a meio metro da camera.
    var lamp = new THREE.PointLight(0xfff2e6, .1, .4, 2), lamp2 = new THREE.PointLight(0x9fc4ff, .045, .4, 2); cam.add(lamp); cam.add(lamp2);

    /* Cabo: corda de Verlet da bota até o DMX OUT, num tubo de topologia fixa (24 nós × 6 lados)
       cujos vértices são reescritos por quadro — sem `new TubeGeometry` e sem `dispose()` a cada
       quadro, que seria alocar e liberar 168 vértices 60 vezes por segundo para nada. */
    var P = Rope.make(N, [0, .3, 0], [TGT.x, TGT.y, TGT.z]), rest = .02, live = true;
    var tube = new THREE.Mesh(new THREE.TubeGeometry(new THREE.CatmullRomCurve3([new THREE.Vector3(0, .3, 0), new THREE.Vector3(TGT.x, TGT.y, TGT.z)]), N - 1, RAD, RS, false), m.rubber);
    tube.castShadow = false; tube.frustumCulled = false; scene.add(tube); // ponytail: cabo sem sombra; ele quase nunca encosta em superfície
    var plug = add(scene, new THREE.CylinderGeometry(.011, .011, .045, 20), m.silver, TGT.x, TGT.y, TGT.z + .022); plug.rotation.x = PI / 2;
    var relief = add(scene, new THREE.CylinderGeometry(.0075, .007, .016, 16), m.rubber, TGT.x, TGT.y, TGT.z + .052); relief.rotation.x = PI / 2;
    plug.renderOrder = relief.renderOrder = 0;
    var tp = tube.geometry.attributes.position, tn = tube.geometry.attributes.normal;
    var T = new THREE.Vector3(), Nr = new THREE.Vector3(1, 0, 0), Bi = new THREE.Vector3(), tmp = new THREE.Vector3(), BOOT = new THREE.Vector3(0, -.052, 0), boca = new THREE.Vector3();
    function retube() { var pa = tp.array, na = tn.array, i, j, k, a, ca, sa, nx, ny, nz, vi, rr;
      for (i = 0; i < N; i++) { k = i * 6;
        if (i === 0) T.set(P[6] - P[0], P[7] - P[1], P[8] - P[2]);
        else if (i === N - 1) T.set(P[k] - P[k - 6], P[k + 1] - P[k - 5], P[k + 2] - P[k - 4]);
        else T.set(P[k + 6] - P[k - 6], P[k + 7] - P[k - 5], P[k + 8] - P[k - 4]);
        if (T.lengthSq() < 1e-12) T.set(0, -1, 0); T.normalize();
        // transporte paralelo: a normal do nó anterior, projetada para fora da tangente. Sem isto o
        // tubo torce sozinho onde a corda dobra, e o cabo aparece com uma costura girando.
        tmp.copy(T).multiplyScalar(Nr.dot(T)); Nr.sub(tmp);
        if (Nr.lengthSq() < 1e-8) { Nr.set(0, 1, 0); tmp.copy(T).multiplyScalar(Nr.dot(T)); Nr.sub(tmp); if (Nr.lengthSq() < 1e-8) Nr.set(1, 0, 0); }
        Nr.normalize(); Bi.crossVectors(T, Nr);
        // o raio afina para o lado do Pino: ele esta' a 25 cm do olho e desenhado em escala de
        // canto de tela (~1/4), entao um cabo de raio unico viraria uma mangueira ali e um fio la'.
        rr = RAD0 + (RAD - RAD0) * (i / (N - 1));
        for (j = 0; j <= RS; j++) { a = j / RS * PI * 2; ca = -Math.cos(a); sa = Math.sin(a);
          nx = ca * Nr.x + sa * Bi.x; ny = ca * Nr.y + sa * Bi.y; nz = ca * Nr.z + sa * Bi.z;
          vi = (i * (RS + 1) + j) * 3;
          na[vi] = nx; na[vi + 1] = ny; na[vi + 2] = nz;
          pa[vi] = P[k] + nx * rr; pa[vi + 1] = P[k + 1] + ny * rr; pa[vi + 2] = P[k + 2] + nz * rr; } }
      tp.needsUpdate = tn.needsUpdate = true; }

    // balão + ícone minimizado
    var bal = document.createElement("div"); bal.className = "bal"; stage.appendChild(bal);
    // O icone e' um XLR-3 desenhado em CSS (aro + tres pinos), nao `brand/submark.svg`: aquele
    // arquivo, apesar do nome, e' o logotipo horizontal de 1009 x 305 e some num quadrado de 32 px.
    var mini = document.createElement("button"); mini.id = "pino-min"; mini.title = "traz o Pino de volta"; mini.setAttribute("aria-label", "traz o Pino de volta"); mini.innerHTML = "<i></i>"; stage.appendChild(mini);
    var talking = 0, t = 0, blink = 0, gone = false, cur = null, vis = 1, v = new THREE.Vector3(), pw = 0, ph = 0, ms = 0, msN = 0;
    // dispensado na sessao passada: entra ja' desmontado (sem animacao de saida) — marcar so' o
    // `gone` deixava o cabo na cena, pendurado no DMX OUT e vindo de um Pino invisivel.
    try { if (localStorage.getItem("sc-pino") === "0") { vis = 0; bye(); } } catch (e) {}
    function say(text, items, hint) { if (gone) return; bal.innerHTML = '<span class="x" title="fecha o balão">×</span><span class="m" title="manda o Pino embora">–</span><b>' + text + "</b>" + (items ? "<ul>" + items.map(function (it) { return "<li data-a=\"" + it[0] + "\">" + it[1] + (it[2] ? " <small>" + it[2] + "</small>" : "") + "</li>"; }).join("") + "</ul>" : "") + (hint === false ? "" : '<div class="hint">' + (hint || "os pinos são o menu: 1 laser · 2 fósforo · 3 patchbay · 4 teatro · 5 info · trava = some") + "</div>"); bal.classList.add("on"); talking = 1.2; }
    function hide() { bal.classList.remove("on"); }
    bal.addEventListener("click", function (e) { if (e.target.classList.contains("m")) { bye(); return; } if (e.target.classList.contains("x")) { hide(); return; } var li = e.target.closest("li"); if (li && o.on) o.on(li.dataset.a); });
    mini.addEventListener("click", back);
    function current(k) { cur = k; pins.forEach(function (p, i) { p.material.emissive.setHex(PINS[i][0] === k ? 0x38ff5c : 0); p.material.emissiveIntensity = .8; }); }
    function remember() { try { localStorage.setItem("sc-pino", gone ? "0" : "1"); } catch (e) {} }
    /* O cabo sai da cena no INSTANTE do dismiss, nao no fim da animacao de saida: enquanto o Pino
       encolhia e descia, o cabo continuava inteiro, preso no DMX OUT e pendurado no nada. */
    function bye() { if (gone) return; gone = true; live = false; scene.remove(tube); plug.visible = relief.visible = false; hide(); remember(); }
    /* Voltar recomeça a corda esticada entre as duas pontas de AGORA: religar com os nós de onde ela
       parou faria o cabo chicotear atravessando o aparelho no primeiro quadro (a câmera andou
       enquanto o Pino estava fora). */
    function back() { if (!gone) return; gone = false; live = true; scene.add(tube); plug.visible = relief.visible = true;
      boca.copy(BOOT); head.localToWorld(boca); reset(); remember(); say("Voltei. Os pinos continuam sendo o menu."); }
    function reset() { var i, k, f; for (i = 0; i < N; i++) { k = i * 6; f = i / (N - 1);
      P[k] = P[k + 3] = boca.x + (TGT.x - boca.x) * f; P[k + 1] = P[k + 4] = boca.y + (TGT.y - boca.y) * f; P[k + 2] = P[k + 5] = boca.z + (TGT.z - boca.z) * f; } }

    /* Onde ele fica: em coordenadas de câmera, a DIST metros na frente. A meia-altura visível ali é
       tan(fov/2)·DIST, então `u` é quanto vale um pixel em metros e o resto é conta de canto — o
       Pino tem ALTO px de altura e fica a MARGX px da borda esquerda em qualquer janela. */
    function place(W, H) { var u = 2 * Math.tan(cam.fov * PI / 360) * DIST / H, w2 = u * W / 2, h2 = u * H / 2, s = ALTO * u / H0;
      g.scale.setScalar(s * Math.max(.001, vis));
      g.position.set(-w2 + MARGX * u - X0 * s, -h2 + MARGY * u - Y0 * s - (1 - vis) * ALTO * 1.6 * u, -DIST);
      lamp.position.set(g.position.x + .05, g.position.y + .09, -DIST + .13); lamp2.position.set(g.position.x - .07, g.position.y + .02, -DIST + .09); }

    function update(dt, mouse, W, H, minTop) { t += dt; if (talking > 0) talking -= dt;
      vis += ((gone ? 0 : 1) - vis) * Math.min(1, dt * 6.5); if (vis < .02) vis = 0;
      if (W !== pw || H !== ph || vis !== 1) { pw = W; ph = H; place(W, H); }
      g.visible = vis > 0; mini.classList.toggle("on", gone && vis === 0);
      head.rotation.x += (-.32 - head.rotation.x) * Math.min(1, dt * 3);
      head.rotation.z = talking > 0 ? Math.sin(t * 28) * .06 : Math.sin(t * 1.3) * .02; head.position.x = Math.sin(t * .9) * .002;
      blink -= dt; if (blink < -3.4 - Math.random()) blink = .11; var sy = blink > 0 ? .15 : 1; eyes.forEach(function (e) { e.scale.y += (sy - e.scale.y) * Math.min(1, dt * 30); });
      if (!live) { hide(); return; }
      // corda: a ponta de cima é a bota do Pino (anda com a câmera), a de baixo é o DMX OUT
      var t0 = performance.now();
      boca.copy(BOOT); head.localToWorld(boca);
      /* ponytail: o comprimento de repouso segue a distância (com a folga FOLGA para dar catenária).
         Cabo de comprimento fixo esticaria reto na vista SHOW, onde a câmera fica a 2 m do aparelho,
         ou empilharia meio metro de corda na vista TRÁS. Trocar por comprimento fixo no dia em que o
         Pino puder ser arrastado pela tela. */
      // GRAV e' menor que 9,8: cabo DMX de 20 cm e' rigido, nao corrente de bicicleta. Com g real
      // e a mesma folga o cabo mergulhava 130 px e saia pela borda de baixo da tela.
      var want = Math.max(.008, boca.distanceTo(TGT) * FOLGA / (N - 1));
      rest += (want - rest) * Math.min(1, dt * 2);
      Rope.pin(P, 0, boca.x, boca.y, boca.z); Rope.pin(P, N - 1, TGT.x, TGT.y, TGT.z);
      Rope.step(P, rest, Math.min(dt, .033), GRAV, 3, 4); retube();
      ms += performance.now() - t0; msN++;
      v.set(0, .014, 0); head.localToWorld(v); v.project(cam); var sx = (v.x + 1) / 2 * W, sy2 = (1 - v.y) / 2 * H; if (mouse) { var dx = mouse[0] - sx, dy = mouse[1] - sy2, dd = Math.max(1, Math.hypot(dx, dy)), k = Math.min(1, dd / 200) * .0028; pupils.forEach(function (p) { p.position.x = dx / dd * k; p.position.y = -dy / dd * k; }); }
      /* O balão abre para a DIREITA da cabeça (o Pino mudou para o canto esquerdo) e nunca por cima
         do aparelho: `minTop` é a base da traseira na tela, e o balão fica dali para baixo — senão
         tapa o display e os controles, que são o motivo de o programa existir. Como o Pino agora
         está no rodapé, o balão sobe até caber inteiro na tela em vez de sair pela borda de baixo. */
      v.set(0, .04, .012); head.localToWorld(v); v.project(cam);
      var bh = bal.offsetHeight || 120, hy = (1 - v.y) / 2 * H;
      bal.style.left = Math.min(W - 262, BALX) + "px";
      bal.style.top = Math.max(8, Math.min(H - bh - 10, Math.max(hy - bh - 12, Math.max(0, minTop || 0)))) + "px"; }

    // custo médio da corda desde a última leitura, em ms por quadro (erro é dado: dá para medir na tela)
    function cost() { var r = msN ? ms / msN : 0; ms = 0; msN = 0; return r; }
    return { group: g, head: head, say: say, hide: hide, current: current, bye: bye, back: back, update: update, cost: cost, alive: function () { return live; }, PINS: PINS }; }
  return { build: build, PINS: PINS };
})();
