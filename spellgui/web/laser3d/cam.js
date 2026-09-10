/* Câmera do laser3d: três leis, uma por vista. O mapa e as URLs do manual estão em
   `design/FUNCOES/camera-solidworks.md`.

     free   (vista SHOW, o visualizador) — o SolidWorks inteiro: meio gira em torno do ponto
            clicado, Ctrl+meio pan, Shift+meio zoom, Alt+meio roll, roda dá zoom no cursor,
            esquerdo no vazio gira. Com limites: a câmera nunca entra no aparelho nem
            atravessa parede ou chão.
     rear   (vista TRÁS, que É o menu) — pose fixa calculada da normal do painel traseiro,
            enquadrando os 400 × 180 mm. Sem arrasto, sem meio, sem roda-zoom, sem setas.
            O único movimento é um respiro de ±2° que segue o mouse e NÃO muda a distância.
     inside (tampa aberta) — órbita presa ao centro da mesa óptica: yaw ±60° em torno da pose
            de entrada, pitch 20°..80°, distância fixa. Nada mais.

   Duas coisas que a versão anterior fazia e não se faz mais:
   - o movimento era `lerp` de fator fixo (`k = dt * 5`): depende do frame rate e nunca chega ao
     alvo, então a câmera "nadava" atrás do mouse e continuava andando depois do botão solto.
     Agora é mola criticamente amortecida, na forma analítica (estável em qualquer dt, chega e para).
   - um arrasto que começava EM CIMA de uma peça não virava arrasto de câmera (certo) mas também
     não contava como arrasto (errado): ao soltar, `app.js` tratava como clique e abria o painel
     daquela peça. Agora o gesto é registrado com modo "none" e `dragging()` responde por ele.

   `SWCam.clamp(mode, goal, env)` é a lei, pura e sem THREE: é o que `test/cam.test.js` verifica. */
(function () {
  "use strict";

  function lim(v, a, b) { return v < a ? a : v > b ? b : v; }
  function wrap(a) { return Math.atan2(Math.sin(a), Math.cos(a)); }
  // direção do alvo para a câmera, a mesma esférica de posOf()
  function dirX(y, p) { return Math.sin(y) * Math.cos(p); }
  function dirY(p) { return Math.sin(p); }
  function dirZ(y, p) { return Math.cos(y) * Math.cos(p); }

  /* A sala: chão em y = 0, parede em z = −5,01, projeção de 8 × 5 (body.js). O alvo anda dentro
     da caixa; quem não pode sair da sala é a POSIÇÃO da câmera, e essa é limitada por `d`. */
  var LAW = {
    free: { pit: [-1.4835, 1.4835],            // ±85°
            d: [.12, 6],
            box: { x: [-3.2, 3.2], y: [.05, 3], z: [-5.05, 2] },
            floor: .06, wall: -4.6 },
    rear: {},
    inside: { pit: [.349, 1.396], yaw: 1.047 } // 20°..80° ; ±60°
  };

  /// A lei de cada modo, aplicada ao ALVO do movimento (não ao estado corrente): função pura.
  /// `g` = { t:{x,y,z}, d, yaw, pit, roll } e é alterado no lugar. `env` = { center, R, d0, yaw0,
  /// pit0 } e vem da vista. Devolve `g`.
  function clamp(mode, g, env) {
    env = env || {};
    var c = env.center || { x: 0, y: .404, z: 0 };
    if (mode === "rear") {                                   // pose fixa: nada do usuário entra
      g.t.x = c.x; g.t.y = c.y; g.t.z = c.z; g.roll = 0;
      g.yaw = env.yaw0 || 0; g.pit = env.pit0 || 0;
      if (env.d0) g.d = env.d0;
      return g;
    }
    if (mode === "inside") {                                 // órbita presa, distância fixa
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
    // sala: a posição (t + d·n) não passa do chão nem da parede — estes só DIMINUEM d
    if (ny < 0 && g.t.y + g.d * ny < F.floor) g.d = (F.floor - g.t.y) / ny;
    if (nz < 0 && g.t.z + g.d * nz < F.wall) g.d = (F.wall - g.t.z) / nz;
    if (g.d < F.d[0]) g.d = F.d[0];   // alvo colado no chão: antes o limite de sala levava d a zero
    // aparelho: |t + d·n − c| ≥ R. Raiz maior da quadrática em d — só AUMENTA d, e por isso é a
    // última: a invariante "a câmera nunca entra no aparelho" vale sempre no fim.
    // ponytail: esfera, não a caixa do aparelho; R já é o raio da caixa mais a folga.
    var R = env.R || .34, ux = g.t.x - c.x, uy = g.t.y - c.y, uz = g.t.z - c.z;
    var b = ux * nx + uy * ny + uz * nz, q = ux * ux + uy * uy + uz * uz - R * R, disc = b * b - q;
    if (disc > 0) { var far = -b + Math.sqrt(disc); if (far > g.d) g.d = far; }
    return g;
  }

  window.SWCam = function (THREE, cam, dom, o) {
    var goal = { t: new THREE.Vector3(0, .4, 0), d: 1, yaw: 0, pit: .3, roll: 0 },
        cur = { t: goal.t.clone(), d: 1, yaw: 0, pit: .3, roll: 0 },
        vel = { x: 0, y: 0, z: 0, d: 0, yaw: 0, pit: 0, roll: 0 },
        drag = null, api, mode = "free", env = { R: .34 }, fixed = null, aspect0 = -1,
        br = { x: 0, y: 0 };                                   // respiro da vista fixa
    var Y = new THREE.Vector3(0, 1, 0), tmp = new THREE.Vector3();

    function posOf(s) { return new THREE.Vector3(s.t.x + s.d * dirX(s.yaw, s.pit), s.t.y + s.d * dirY(s.pit), s.t.z + s.d * dirZ(s.yaw, s.pit)); }
    function fromPos(s, pos, t) { s.t.copy(t); var v = pos.clone().sub(t); s.d = Math.max(.05, v.length()); s.pit = Math.asin(lim(v.y / s.d, -1, 1)); s.yaw = Math.atan2(v.x, v.z); }
    function setView(pos, t, snap) { fromPos(goal, new THREE.Vector3().fromArray(pos), new THREE.Vector3().fromArray(t)); goal.roll = 0; if (snap) { cur.t.copy(goal.t); cur.d = goal.d; cur.yaw = goal.yaw; cur.pit = goal.pit; cur.roll = 0; vel.x = vel.y = vel.z = vel.d = vel.yaw = vel.pit = vel.roll = 0; } }
    function basis() { var f = goal.t.clone().sub(posOf(goal)).normalize(), r = new THREE.Vector3().crossVectors(f, Y).normalize(), u = new THREE.Vector3().crossVectors(r, f).normalize(); return { f: f, r: r, u: u }; }
    // sensibilidade proporcional à distância: perto do aparelho o mesmo pixel anda menos mundo
    function pan(dx, dy) { var b = basis(), k = goal.d * .0014; goal.t.addScaledVector(b.r, -dx * k).addScaledVector(b.u, dy * k); }
    function zoomAt(s, p) { // p: ponto do mundo sob o cursor (ou null) — alvo e câmera vão juntos para ele
      if (p) goal.t.copy(p).add(tmp.copy(goal.t).sub(p).multiplyScalar(s));
      goal.d = goal.d * s; }

    /* Enquadramento de um retângulo w × h com folga, a partir do fov e do ASPECT corrente: é por
       isso que as poses fixas são recalculadas quando a janela muda de tamanho. */
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

    /* mola criticamente amortecida, forma analítica: y(t) = (A + B·t)·e^(−ωt), A = y0, B = v0 + ω·y0.
       Chega ao alvo sem passar dele e PARA — e não depende do frame rate como o lerp dependia. */
    function sp(s, v, k, g, w, dt) {
      var y = s[k] - g, A = y, B = v[k] + w * y, e = Math.exp(-w * dt);
      s[k] = g + (A + B * dt) * e; v[k] = (B - w * (A + B * dt)) * e;
    }

    function press(e) {
      var mid = e.button === 1, left = e.button === 0; if (!mid && !left) return;
      if (mode === "rear") return;                                   // a traseira é menu: não se arrasta
      // arrasto que começa numa peça não move a câmera, mas É um arrasto: sem isto o `pointerup`
      // de `app.js` virava clique e abria o painel da peça no fim de um giro.
      if (left && o.hit(e)) { drag = { mode: "none", x: e.clientX, y: e.clientY, moved: false }; return; }
      var m = mode === "inside" ? "rot"                              // dentro só gira
        : e.ctrlKey ? "pan" : e.shiftKey ? "zoom" : e.altKey ? "roll" : "rot";
      if (m === "rot" && mid && mode === "free") { var p = o.pick(e); if (p) { var pos = posOf(goal); fromPos(goal, pos, p); fromPos(cur, cam.position.clone(), p); } }
      drag = { mode: m, x: e.clientX, y: e.clientY, moved: false };
      dom.setPointerCapture(e.pointerId); e.preventDefault();
    }
    function moveAt(e) {
      var r = dom.getBoundingClientRect();                           // respiro: ±2° de paralaxe, sem mexer na distância
      br.x = lim(((e.clientX - r.left) / r.width) * 2 - 1, -1, 1); br.y = lim(((e.clientY - r.top) / r.height) * 2 - 1, -1, 1);
      if (!drag) return;
      var dx = e.clientX - drag.x, dy = e.clientY - drag.y; drag.x = e.clientX; drag.y = e.clientY;
      // 4 px: tremor de mão com o dedo no botão não pode virar arrasto e engolir o clique
      if (Math.abs(dx) + Math.abs(dy) > 4) drag.moved = true;
      if (drag.mode === "rot") { goal.yaw -= dx * .0045; goal.pit += dy * .0045; }
      else if (drag.mode === "pan") pan(dx, dy);
      else if (drag.mode === "zoom") zoomAt(Math.exp(dy * .0035), null);
      else if (drag.mode === "roll") goal.roll += dx * .005;
    }
    dom.addEventListener("pointerdown", press);
    dom.addEventListener("pointermove", moveAt);
    function up() { drag = null; }
    dom.addEventListener("pointerup", up); dom.addEventListener("pointercancel", up);
    dom.addEventListener("pointerleave", function () { br.x = br.y = 0; });
    dom.addEventListener("wheel", function (e) {
      e.preventDefault();
      if (mode !== "free") return;                                   // fixa e restrita: roda não dá zoom
      var dir = api.reverse ? -1 : 1, s = Math.exp(dir * Math.sign(e.deltaY) * .07);   // passo menor
      zoomAt(s, o.pick(e) || o.plane(e, goal.t));
    }, { passive: false });
    dom.addEventListener("auxclick", function (e) { if (e.button === 1) e.preventDefault(); });
    dom.addEventListener("contextmenu", function (e) { e.preventDefault(); });

    var STD = { front: [0, 0], back: [Math.PI, 0], left: [-Math.PI / 2, 0], right: [Math.PI / 2, 0], top: [0, 1.5], bottom: [0, -1.5], iso: [Math.PI / 4, .615] };
    api = { reverse: true, breathe: true, goal: goal, cur: cur,
      setView: setView,
      get: function () { return { pos: posOf(goal).toArray(), t: goal.t.toArray() }; },
      /// A lei desta vista. `e` traz center/normal/w/h/pad (fixa), yaw0/pit0 (entrada) e R.
      mode: function (m, e) { mode = LAW[m] ? m : "free"; env = e || { R: .34 }; repose(); },
      camMode: function () { return mode; },
      free: function () { return mode === "free"; },
      rotate: function (dy, dp) { if (mode !== "free") return; goal.yaw += dy; goal.pit += dp; },
      roll: function (dr) { if (mode === "free") goal.roll += dr; },
      pan: function (dx, dy) { if (mode === "free") pan(dx, dy); },
      zoom: function (s) { if (mode === "free") zoomAt(s, null); },
      fit: function (center, radius) { if (mode !== "free") return; goal.t.copy(center); goal.d = radius / Math.sin(cam.fov * Math.PI / 360) * 1.1; },
      std: function (k, center, radius) { var v = STD[k]; if (mode !== "free" || !v) return; goal.yaw = v[0]; goal.pit = v[1]; goal.roll = 0; if (center) api.fit(center, radius); },
      dragging: function () { return !!(drag && drag.moved); },
      update: function (dt) {
        if (cam.aspect !== aspect0) repose();                        // janela mudou: pose fixa refeita
        if (fixed) {
          env.d0 = fixed.d;
          env.yaw0 = fixed.yaw + (mode === "rear" && api.breathe ? br.x * .035 : 0);
          if (mode === "rear") env.pit0 = fixed.pit - (api.breathe ? br.y * .035 : 0);
        }
        clamp(mode, goal, env);
        var w = 18, dy = wrap(goal.yaw - cur.yaw);                   // yaw pelo caminho curto
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
