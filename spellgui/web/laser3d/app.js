/* Spellcaster LASER · a página principal do programa: estado do aparelho, parede (render 2D → textura), cena
   (mat/body/optics/beam/pino), splash (câmera na parede → contorno → brilho → voa para a traseira), câmera
   SolidWorks, ações com bindings de tecla e MIDI, painéis, OLED, e a ligação com o engine por `bus.js`.
   Sem engine a página continua inteira: o `Bus` responde em modo offline e a parede desenha igual. */
(function () {
  "use strict";
  var $ = function (s) { return document.querySelector(s); }, reduced = matchMedia("(prefers-reduced-motion: reduce)").matches;
  /* ---------- áudio ---------- */
  var AC = null; function ac() { if (!AC) AC = new (window.AudioContext || window.webkitAudioContext)(); if (AC.state === "suspended") AC.resume(); return AC; }
  function tone(f, t0, d, type, g) { var a = ac(), o = a.createOscillator(), v = a.createGain(); o.type = type || "square"; o.frequency.value = f; v.gain.setValueAtTime(0, t0); v.gain.linearRampToValueAtTime(g || .08, t0 + .01); v.gain.exponentialRampToValueAtTime(.0005, t0 + d); o.connect(v); v.connect(a.destination); o.start(t0); o.stop(t0 + d + .02); }
  function jingle() { try { var t = ac().currentTime + .05, seq = [[330, .09], [440, .09], [554, .09], [659, .09], [880, .18], [659, .09], [880, .3]], i; for (i = 0; i < seq.length; i++) { tone(seq[i][0], t, seq[i][1], "square", .07); tone(seq[i][0] / 2, t, seq[i][1], "triangle", .05); t += seq[i][1]; } for (i = 0; i < 24; i++) tone(1760 + Math.sin(i) * 300, t - .9 + i * .03, .04, "square", .02); } catch (e) {} }
  function blip(f) { try { var t = ac().currentTime; tone(f || 1200, t, .05, "square", .05); } catch (e) {} }
  function chord() { try { var t = ac().currentTime; [220, 277, 330, 440].forEach(function (f, i) { tone(f, t + i * .04, .7, "triangle", .06); }); } catch (e) {} }
  function click() { try { var t = ac().currentTime; tone(3000, t, .02, "square", .04); tone(180, t, .04, "triangle", .06); } catch (e) {} }

  /* ---------- estado ---------- */
  var S = { mode: "splash", t0: performance.now(), play: true, key: false, lock: true, power: true, kpps: 30000, buffer: 3, speed: 1, size: 1, fog: .85, frame: 0, pos: 0, show: null, name: "", cam: "rear", lim: { r: 1, g: 1, b: 1 }, gam: { r: 1, g: 1, b: 1 }, dmx: 1, univ: 1, net: { ndi: false, spout: false, artnet: false, sacn: true }, page: 0, field: 0, edit: false, err: null, dmxIn: null, temp: 31, mem: null, tip: {}, dim: 1, encT: 0 };
  try { S.mem = JSON.parse(localStorage.getItem("sc-laser") || "null"); if (S.mem && S.mem.kpps) S.kpps = S.mem.kpps; } catch (e) {}
  var demo = ILDA.parse(ILDA.write(ILDA.demo()).buffer); S.show = demo.frames; S.name = "demo.ild · " + demo.frames.length + " frames";
  function fps() { var f = S.show[S.frame]; return f && f.length ? S.kpps / f.length : 0; }
  function armed() { return S.power && S.key; } function live() { return armed() && S.lock; }
  /// Aparelho desligado responde só ao rocker (SISTEMA.md §5). Sem energia a chave não arma, o
  /// interlock não muda e o encoder não navega — antes tudo isso obedecia com o display apagado, e
  /// a chave chegava a mandar `laser_open` para o engine sem nada aceso na traseira para contar.
  function dead() { if (S.power) return false; if (pino) pino.say("Sem energia. O rocker POWER liga o aparelho.", null, false); blip(300); return true; }
  function remember() { try { localStorage.setItem("sc-laser", JSON.stringify({ kpps: S.kpps, name: S.name, when: Date.now() })); } catch (e) {} }

  /* ---------- engine ---------- */
  // O aparelho é mais um cliente do registry: quem tem lógica é o engine, aqui só passa comando.
  // `LaserEngine.cmdFor` é o mapa estado → comando; `push` manda e guarda a resposta.
  var bus = new Bus({ offline: location.protocol === "file:" || /(\?|&)offline=1/.test(location.search) }).connect();
  var ENG = { on: false, rev: 0, show: "", feed: null, dac: "etherdream", host: "", file: "", stats: null, dacs: null, files: [], tr: null, log: "", port: location.host || "offline", ver: "0.1.2", err: false };
  function ctx() { return { feed: ENG.feed, dac: ENG.dac, host: ENG.host, kpps: S.kpps, file: ENG.file, fps: fps() || 30, show: ENG.show }; }
  /// Erro é dado, não exceção silenciosa: vai para a página ERRO do display (que passa a ser a
  /// página corrente) e para a boca do Pino. Nada trava, nada some.
  function fail(msg) { S.err = { msg: String(msg), when: new Date().toTimeString().slice(0, 8) }; ENG.err = true; S.page = ERRPAGE; S.edit = false; S.field = 0; blip(260); if (pino) pino.say(msg, null, false); drawOled(); refresh(); }
  /// CLI ecoado (SISTEMA.md §10): o HUD âmbar mostra sempre o último comando que a peça mandou.
  /// Argumento vazio não vira `--host ` no eco: o que aparece no HUD é o que dá para digitar.
  function cliOf(c) { return "spell " + c.cmd + Object.keys(c.args).map(function (k) { return c.args[k] === "" ? "" : " --" + k + " " + c.args[k]; }).join(""); }
  function push(id, v) { var c = LaserEngine.cmdFor(id, v, ctx()); if (!c) return null;
    $("#cli").textContent = cliOf(c);
    return bus.call(c.cmd, c.args).then(function (r) { ENG.rev = bus.rev;
      if (c.cmd === "laser_open" && r && r.feed != null) { ENG.feed = r.feed; ENG.err = false; push("lock", S.lock); }
      if (c.cmd === "laser_close") { ENG.feed = null; ENG.stats = null; ENG.err = false; }
      drawOled(); refresh(); return r;
    }, function (e) { if (c.cmd === "laser_open") ENG.feed = null; var m = String(e.message); fail(m.indexOf(c.cmd) === 0 ? m : c.cmd + ": " + m); }); }
  // `full` traz também `inputs`: é de lá que o HUD sabe se existe entrada Art-Net ou sACN, e em
  // que universo. Sem isso a linha de entrada seria chute — e o HUD não chuta.
  function showName() { bus.call("show_get", { full: true }).then(function (r) { ENG.show = (r && r.name) || ""; ENG.rev = bus.rev; readInputs(r); drawOled(); refresh(); }, function () {}); }
  /// Firmware no chassi (o dono: "grava o firmware no próprio chassi"): a versão vem do engine, e
  /// só dele. Sem engine a placa escreve FIRMWARE OFFLINE — placa de aparelho não inventa número.
  function plate(fw) { if (B.plate) B.plate({ fw: fw || null, sn: "SC-0512" }); }
  // ponytail: `version` ainda não é comando do registry ; quando for, a placa passa a ler dele em
  // vez do `ver` que a página carrega.
  function fwPlate() { bus.call("version", {}).then(function (r) { plate((r && (r.version || r.ver)) || ENG.ver); }, function () { plate(ENG.ver); }); }
  function hello() { ENG.on = true; showName(); fwPlate();
    bus.call("laser_files", {}).then(function (r) { ENG.files = (r && r.files) || []; refresh(); }, function (e) { fail("laser_files: " + e.message); });
    drawOled(); refresh(); }
  bus.on("open", hello);
  bus.on("show", showName); // outra página renomeou ou editou o show: o display não pode mentir
  bus.on("transport", function (d) { ENG.tr = d; drawOled(); }); // página SHOW: t, estado e duração vêm daqui
  bus.on("log", function (d) { ENG.log = (d && d.text) || ""; if (/erro|error|falha|fail/i.test(ENG.log)) fail(ENG.log); else drawOled(); });
  // topic 1 é o frame que SAI, topic 2 é o que ENTRA (`show.inputs`): a entrada é que acende o LED
  // de Art-Net / sACN no HUD, e ela só acende com frame recebido de verdade.
  bus.on("dmx", function (d) { if (!d) return; if (d.topic === 2) INPUTS.forEach(function (i) { if (i.u === d.universe) i.last = performance.now(); });
    S.dmxIn = { universe: d.universe, value: d.data[Math.max(0, S.dmx - 1)] }; });
  bus.on("close", function () { ENG.on = false; ENG.feed = null; ENG.stats = null; ENG.tr = null; INPUTS = []; plate(null); drawOled(); refresh(); });
  // ponytail: `laser_stats` por polling de 1 s ; vira evento quando o engine publicar um tópico de laser
  setInterval(function () { if (ENG.feed == null) return; bus.call("laser_stats", { feed: ENG.feed }).then(function (r) { ENG.stats = r; drawOled(); }, function () {}); }, 1000);
  /// Toca um .ild que está no disco do engine: o engine manda para o DAC, a página lê o mesmo
  /// arquivo pelo HTTP do `serve` para desenhar na parede o que está saindo.
  function playFile(p, name) { ENG.file = p; S.name = name + " · engine";
    fetch("/" + p.replace(/\\/g, "/")).then(function (r) { return r.arrayBuffer(); }).then(function (ab) { var d = ILDA.parse(ab); if (d.frames.length) { S.show = d.frames; S.frame = 0; S.pos = 0; S.name = name + " · " + d.frames.length + " frames · engine"; } refresh(); }, function () {});
    push("play", true); }

  /* ---------- parede ----------
     A parede é um render target: o rastro mora NELE. Todo quadro entra um quad preto que esmaece o
     que já estava lá (o `fillRect` de antes, só que por tempo: `1 - FADE^(dt*60)`, para o rastro
     durar o mesmo tanto a 20 ou a 60 fps) e por cima os segmentos do quadro, num `LineSegments` de
     buffer pré-alocado. O traço vai em source-over, como o `stroke` do canvas: traço sobre traço
     troca a cor, não soma — aditivo saturava em branco onde a figura passa dezenas de vezes.
     Só o halo e o ponto do galvo somam. Antes era um canvas 2D 1024×640 com um `stroke` de `shadowBlur = 18` POR
     PONTO — a 30 kpps são 500 traços desfocados por quadro na CPU, mais a subida da textura inteira
     à GPU a cada quadro: com um .ild de 3000 pontos por frame o Chrome ficava em 1,8 fps.
     ponytail: a splash continua no canvas 2D (`fillDone` é `fill("evenodd")` de polígono de letra, e
     triangular fonte não cabe aqui); ela roda uma vez, por 8 s, com poucos traços. Ao sair do splash
     o `map` do plano troca do canvas para o render target, que nasce limpo. */
  var WC = document.createElement("canvas"); WC.width = 1024; WC.height = 640; var wx = WC.getContext("2d"), WW = 1024, WH = 640, galvo = null, lit = [], gpos = [0, 0];
  // ponytail: medida do portao numa linha, so com ?perf=1 ; sai quando houver um HUD de perf de verdade
  var PERF = /(\?|&)perf=1/.test(location.search) ? (window.__perf = { frames: 0, wallMs: 0 }) : null;
  // 4096 segmentos: o quadro desenha `min(pontos do frame, kpps × dt)` e dt está preso em .1 s, ou
  // seja no máximo 4000 pontos a 40 kpps. Passou disso, o resto do quadro cai fora (não realoca).
  var MAXSEG = 4096, segN = 0, dotN = 0;
  var wrt = new THREE.WebGLRenderTarget(WW, WH, { minFilter: THREE.LinearFilter, magFilter: THREE.LinearFilter, depthBuffer: false, stencilBuffer: false });
  wrt.texture.encoding = THREE.sRGBEncoding; wrt.texture.generateMipmaps = false;
  var wscene = new THREE.Scene(), wcam = new THREE.OrthographicCamera(0, WW, 0, WH, -1, 1); // coordenadas do canvas: y cresce para baixo
  /* O alfa do render target fica preso em 1: o plano `proj` é aditivo e multiplica a cor pelo alfa
     da textura, então um alfa que esmaece junto com o rastro apagaria a parede duas vezes. */
  function wmat(m, add) { m.transparent = true; m.depthTest = m.depthWrite = false; m.toneMapped = false; m.blending = THREE.CustomBlending;
    m.blendSrc = THREE.SrcAlphaFactor; m.blendDst = add ? THREE.OneFactor : THREE.OneMinusSrcAlphaFactor;
    m.blendSrcAlpha = THREE.ZeroFactor; m.blendDstAlpha = THREE.OneFactor; return m; }
  function attr(a, n) { var b = new THREE.BufferAttribute(a, n); b.setUsage(THREE.DynamicDrawUsage); return b; }
  var segPos = new Float32Array(MAXSEG * 6), segCol = new Float32Array(MAXSEG * 6), haloPos = new Float32Array(MAXSEG * 3), haloCol = new Float32Array(MAXSEG * 3), dotPos = new Float32Array(3);
  var segGeo = new THREE.BufferGeometry(); segGeo.setAttribute("position", attr(segPos, 3)); segGeo.setAttribute("color", attr(segCol, 3));
  var haloGeo = new THREE.BufferGeometry(); haloGeo.setAttribute("position", attr(haloPos, 3)); haloGeo.setAttribute("color", attr(haloCol, 3));
  var dotGeo = new THREE.BufferGeometry(); dotGeo.setAttribute("position", attr(dotPos, 3));
  // o halo do traço: `shadowBlur` virou um sprite de gradiente por ponto, que é o que a GPU faz de graça
  var GC = document.createElement("canvas"); GC.width = GC.height = 64; var gc = GC.getContext("2d"), gg = gc.createRadialGradient(32, 32, 0, 32, 32, 32);
  gg.addColorStop(0, "rgba(255,255,255,1)"); gg.addColorStop(.3, "rgba(255,255,255,.45)"); gg.addColorStop(1, "rgba(255,255,255,0)"); gc.fillStyle = gg; gc.fillRect(0, 0, 64, 64);
  var glowTex = new THREE.CanvasTexture(GC);
  var segMat = wmat(new THREE.LineBasicMaterial({ vertexColors: true, opacity: 1 }), false);
  var haloMat = wmat(new THREE.PointsMaterial({ size: 9, sizeAttenuation: false, map: glowTex, vertexColors: true, opacity: .12 }), true);
  var dotMat = wmat(new THREE.PointsMaterial({ size: 14, sizeAttenuation: false, map: glowTex, opacity: .9 }), true);
  var fadeMat = wmat(new THREE.MeshBasicMaterial({ color: 0x000000, opacity: reduced ? .6 : .3 }), false), FADE = reduced ? .4 : .7;
  var fade = new THREE.Mesh(new THREE.PlaneGeometry(WW, WH), fadeMat); fade.position.set(WW / 2, WH / 2, 0);
  var segs = new THREE.LineSegments(segGeo, segMat), halo = new THREE.Points(haloGeo, haloMat), dot = new THREE.Points(dotGeo, dotMat);
  [fade, segs, halo, dot].forEach(function (o, i) { o.frustumCulled = false; o.renderOrder = i; wscene.add(o); });
  /* Limite e curva viram tabela de 256 entradas: o quadro fazia 1500 `Math.pow` (três por ponto).
     A tabela guarda os dois valores da mesma cor — o byte, que é o contrato do `lit` (os feixes da
     cena leem `L[1][k] / 255`), e o valor LINEAR que a cor do material precisa, porque o render
     target é sRGB: entrando linear, sai gravado o mesmo byte que o canvas escrevia, e a soma dos
     traços acontece em sRGB como antes. */
  function s2l(v) { return v <= .04045 ? v / 12.92 : Math.pow((v + .055) / 1.055, 2.4); }
  var Lb = [new Uint8Array(256), new Uint8Array(256), new Uint8Array(256)], Ll = [new Float32Array(256), new Float32Array(256), new Float32Array(256)], Lk = "", PARKC = [s2l(1), s2l(42 / 255), s2l(26 / 255)];
  function lut() { var g = S.gam, l = S.lim, k = l.r + "," + l.g + "," + l.b + "," + g.r + "," + g.g + "," + g.b; if (k === Lk) return; Lk = k;
    ["r", "g", "b"].forEach(function (c, j) { for (var i = 0; i < 256; i++) { var v = Math.min(1, l[c] * Math.pow(i / 255, g[c])); Lb[j][i] = Math.round(255 * v); Ll[j][i] = s2l(v); } }); }
  function drawFrame(f, from, count) { var N = f.length, LG = Math.min(1, Math.max(.3, Math.min(1, 1.3 - S.kpps / 40000)) * S.speed), sc = S.size * WH * .3 / 32767, i, n = 0,
      q = f[(from - 1 + N) % N], px = galvo ? galvo[0] : WW * .5 + q.x * sc, py = galvo ? galvo[1] : WH * .5 - q.y * sc;
    lit.length = 0;
    for (i = 0; i < count; i++) { var p = f[(from + i) % N], cx = px + (WW * .5 + p.x * sc - px) * LG, cy = py + (WH * .5 - p.y * sc - py) * LG;
      if (!p.bl && (i > 0 || galvo)) { var r = Lb[0][p.r], g = Lb[1][p.g], b = Lb[2][p.b];
        if (r + g + b > 12 && n < MAXSEG) { var o = n * 6, h = n * 3, lr = Ll[0][p.r], lg = Ll[1][p.g], lb = Ll[2][p.b];
          segPos[o] = px; segPos[o + 1] = py; segPos[o + 3] = cx; segPos[o + 4] = cy;
          segCol[o] = segCol[o + 3] = lr; segCol[o + 1] = segCol[o + 4] = lg; segCol[o + 2] = segCol[o + 5] = lb;
          haloPos[h] = cx; haloPos[h + 1] = cy; haloCol[h] = lr; haloCol[h + 1] = lg; haloCol[h + 2] = lb;
          n++; lit.push([[cx, cy], [r, g, b]]); } }
      px = cx; py = cy; }
    segN = n; galvo = galvo || [0, 0]; galvo[0] = px; galvo[1] = py; gpos[0] = (px / WW - .5) * 2; gpos[1] = (.5 - py / WH) * 2;
    dotPos[0] = px; dotPos[1] = py; dotN = 1; dotMat.color.setRGB(1, 1, 1); dotMat.opacity = .9; dotMat.size = 10; }
  function parked(now) { galvo = null; gpos[0] = gpos[1] = 0; segN = 0;
    dotPos[0] = WW * .5; dotPos[1] = WH * .5; dotN = 1; dotMat.color.setRGB(PARKC[0], PARKC[1], PARKC[2]); dotMat.opacity = .6 + .4 * Math.sin(now / 90); dotMat.size = 12;
    lit.length = 0; lit.push([[WW * .5, WH * .5], [255, 42, 26]]); }
  function up(a, n) { if (!n) return; a.updateRange.offset = 0; a.updateRange.count = n; a.needsUpdate = true; }
  function wallDraw(dt) { fadeMat.opacity = 1 - Math.pow(FADE, dt * 60); up(segGeo.attributes.position, segN * 6); up(segGeo.attributes.color, segN * 6); up(haloGeo.attributes.position, segN * 3); up(haloGeo.attributes.color, segN * 3); up(dotGeo.attributes.position, dotN * 3);
    segGeo.setDrawRange(0, segN * 2); haloGeo.setDrawRange(0, segN); dotGeo.setDrawRange(0, dotN);
    if (proj.material.map !== wrt.texture) { var c0 = R.getClearColor(new THREE.Color()), a0 = R.getClearAlpha(); R.setRenderTarget(wrt); R.setClearColor(0x000000, 1); R.clear(true, false, false); R.setClearColor(c0, a0); proj.material.map = wrt.texture; proj.material.needsUpdate = true; }
    var ac = R.autoClear; R.autoClear = false; R.setRenderTarget(wrt); R.render(wscene, wcam); R.setRenderTarget(null); R.autoClear = ac; }
  function wallTick(now, dt) { lut(); var f = S.show[S.frame];
    if (!S.power) { lit.length = 0; segN = 0; dotN = 0; }
    else if (live() && f && f.length) { var N = f.length, adv = S.kpps * dt; if (S.play) { S.pos += adv; while (S.pos >= N) { S.pos -= N; S.frame = (S.frame + 1) % S.show.length; N = S.show[S.frame].length; f = S.show[S.frame]; } } else S.pos = (S.pos + adv) % N; var from = Math.floor(S.pos), count = Math.min(N, Math.ceil(adv)); drawFrame(f, (from - count + N) % N, count); }
    else parked(now);
    wallDraw(dt); }
  // splash: o galvo contorna SPELLCASTER LASER, laço a laço; cada letra fechada é revelada; no fim tudo brilha
  var SP = { loops: [], len: 0, pos: 0, li: 0, pi: 0, done: [], flew: false, last: null };
  // ponto do galvo na splash, que continua no canvas 2D como o resto dela
  function spDot(x, y, c, fill, blur, r) { wx.globalCompositeOperation = "lighter"; wx.shadowBlur = blur; wx.shadowColor = c; wx.fillStyle = fill; wx.beginPath(); wx.arc(x, y, r, 0, 7); wx.fill(); wx.shadowBlur = 0; wx.globalCompositeOperation = "source-over"; }
  function fillDone(alpha, glow) { if (!SP.done.length) return; wx.save(); wx.globalCompositeOperation = "lighter"; wx.beginPath(); SP.done.forEach(function (l) { wx.moveTo(l[0][0], l[0][1]); for (var i = 1; i < l.length; i++) wx.lineTo(l[i][0], l[i][1]); wx.closePath(); }); wx.shadowBlur = glow || 0; wx.shadowColor = "#38FF5C"; wx.fillStyle = "rgba(56,255,92," + alpha + ")"; wx.fill("evenodd"); wx.restore(); }
  function wallSplash(now, dt) { var t = (now - S.t0) / 1000, T = SP;
    if (t < .8) { wx.fillStyle = "rgba(0,0,0,.35)"; wx.fillRect(0, 0, WW, WH); parked(now); spDot(WW * .5, WH * .5, "#FF2A1A", "rgba(255,42,26," + (.6 + .4 * Math.sin(now / 90)) + ")", 34, 4); return; }
    if (t < 4.8 && T.loops.length) { if (!T.cleared) { T.cleared = true; wx.globalCompositeOperation = "source-over"; wx.fillStyle = "#000"; wx.fillRect(0, 0, WW, WH); } var target = Math.min(T.len, (t - .8) / 4 * T.len), pt = null; wx.globalCompositeOperation = "lighter"; wx.lineCap = "round"; wx.lineJoin = "round"; wx.lineWidth = 3; wx.strokeStyle = "rgba(56,255,92,.9)"; wx.shadowBlur = 14; wx.shadowColor = "#38FF5C";
      while (T.pos < target && T.li < T.loops.length) { var L = T.loops[T.li], a = L[T.pi], b = L[T.pi + 1]; if (!b) { T.done.push(L); T.li++; T.pi = 0; T.last = null; continue; } var sl = Math.hypot(b[0] - a[0], b[1] - a[1]), room = target - T.pos, k = Math.min(1, room / Math.max(1e-6, sl)); pt = [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k]; var from = T.last || a; wx.beginPath(); wx.moveTo(from[0], from[1]); wx.lineTo(pt[0], pt[1]); wx.stroke(); if (k >= 1) { T.pos += sl; T.pi++; T.last = null; } else { T.pos = target; T.last = pt; } }
      wx.shadowBlur = 0; wx.globalCompositeOperation = "source-over"; if (T.done.length > (T.filled || 0)) { fillDone(.14); T.filled = T.done.length; }
      if (pt) { spDot(pt[0], pt[1], "#fff", "rgba(255,255,255,.9)", 26, 3); lit = [[pt, [120, 255, 150]]]; gpos[0] = (pt[0] / WW - .5) * 2; gpos[1] = (.5 - pt[1] / WH) * 2; } return; }
    if (t < 5.8) { if (T.li < T.loops.length) { T.done = T.loops.slice(); T.li = T.loops.length; } var k = (t - 4.8); fillDone(.05 + .25 * Math.sin(Math.min(1, k) * Math.PI) * (1 + Math.sin(k * 40) * .3), 40 * Math.sin(Math.min(1, k) * Math.PI)); lit = []; return; }
    // laser apaga, a parede esfria, a luz da sala baixa, a câmera voa para a traseira
    wx.fillStyle = "rgba(0,0,0,.08)"; wx.fillRect(0, 0, WW, WH); lit = []; S.dim = .55; if (!T.flew) { T.flew = true; setCam("rear", 2.2); } if (t > 7.6) start(); }

  /* ---------- cena ---------- */
  var stage = $("#stage"), gl = $("#gl"), R = new THREE.WebGLRenderer({ canvas: gl, antialias: false, powerPreference: "high-performance" }), scene = new THREE.Scene(), cam = new THREE.PerspectiveCamera(42, 1, .02, 60), pick = [];
  R.setPixelRatio(Math.min(2, devicePixelRatio)); scene.background = new THREE.Color(0x020306); scene.fog = new THREE.FogExp2(0x03040a, .12);
  var X = MAT(THREE, R); scene.environment = X.env();
  var sun = new THREE.SpotLight(0xfff4e6, 90, 9, .42, .7, 1.4); sun.position.set(1.4, 3.2, 1.2); sun.target.position.set(0, .35, 0); sun.castShadow = true; sun.shadow.mapSize.set(2048, 2048); sun.shadow.bias = -.0004; sun.shadow.radius = 4; scene.add(sun, sun.target);
  var fill = new THREE.PointLight(0x38ff5c, 4, 4, 2); fill.position.set(-.9, .9, .5); scene.add(fill); var rim = new THREE.PointLight(0xa0c0ff, 8, 5, 2); rim.position.set(.4, 1.2, -1.6); scene.add(rim);
  var inLight = new THREE.PointLight(0xfff0dc, 0, 1.2, 2); inLight.position.set(.05, .62, 0); scene.add(inLight);
  var B = BODY(THREE, X, scene, pick), O = OPTICS(THREE, X, B.body, pick);
  var wallTex = new THREE.CanvasTexture(WC); wallTex.minFilter = THREE.LinearFilter; wallTex.encoding = THREE.sRGBEncoding; var proj = new THREE.Mesh(new THREE.PlaneGeometry(8, 5), new THREE.MeshBasicMaterial({ map: wallTex, transparent: true, blending: THREE.AdditiveBlending, depthWrite: false })); proj.position.set(0, 2.2, -5); scene.add(proj);
  var beamsOut = BEAM(THREE, scene, 160, .006, .028), beamsIn = BEAM(THREE, scene, 12, .0012, .0045);
  var FC = document.createElement("canvas"); FC.width = FC.height = 128; var fc = FC.getContext("2d"), fg = fc.createRadialGradient(64, 64, 0, 64, 64, 64); fg.addColorStop(0, "rgba(120,140,170,1)"); fg.addColorStop(1, "rgba(120,140,170,0)"); fc.fillStyle = fg; fc.fillRect(0, 0, 128, 128);
  var fogTex = new THREE.CanvasTexture(FC), puffs = []; for (var i = 0; i < 14; i++) { var sp = new THREE.Sprite(new THREE.SpriteMaterial({ map: fogTex, transparent: true, opacity: .05, blending: THREE.AdditiveBlending, depthWrite: false })); sp.position.set((Math.random() - .5) * 5, .4 + Math.random() * 2.4, -.5 - Math.random() * 4.5); var sc = 1.5 + Math.random() * 2.5; sp.scale.set(sc, sc, 1); sp.userData.v = [(Math.random() - .5) * .08, (Math.random() - .5) * .03]; scene.add(sp); puffs.push(sp); }
  var composer = new THREE.EffectComposer(R); composer.renderTarget1.texture.encoding = composer.renderTarget2.texture.encoding = THREE.sRGBEncoding; composer.addPass(new THREE.RenderPass(scene, cam)); var bloom = new THREE.UnrealBloomPass(new THREE.Vector2(1024, 760), .5, .45, .82); composer.addPass(bloom); var fxaa = new THREE.ShaderPass(THREE.FXAAShader); composer.addPass(fxaa);

  /* ---------- câmera ---------- */
  var ray = new THREE.Raycaster(), mv = new THREE.Vector2(-2, -2), mouse = null, hot = null, tip = $("#tip"), pressed = false;
  function ptr(e) { var r = gl.getBoundingClientRect(); mv.set(((e.clientX - r.left) / r.width) * 2 - 1, -((e.clientY - r.top) / r.height) * 2 + 1); mouse = [e.clientX - r.left, e.clientY - r.top]; return mouse; }
  /* A caixa é opaca: o raio para na primeira superfície sólida que encontra. Se essa superfície não
     é peça clicável (chapa, serigrafia, vidro do display), não há clique ali — antes o raycast só
     via a lista `pick` e a mesa óptica, a fonte e a placa DAC ficavam clicáveis ATRAVÉS da traseira
     fechada, com cursor de mão sobre chapa lisa. Névoa, feixe e a projeção da parede não contam
     como sólido (não escrevem profundidade), então não tapam nada. */
  function solid(o) { return o.visible && !o.isSprite && !o.isInstancedMesh && o.material && !o.material.transparent && o.material.depthWrite !== false; }
  function hitOf(e) { ptr(e); ray.setFromCamera(mv, cam); var hits = ray.intersectObjects(scene.children, true);
    for (var i = 0; i < hits.length; i++) { var o = hits[i].object; if (!solid(o)) continue;
      var h = o; while (h && !(h.userData && h.userData.key)) h = h.parent;
      return h ? { o: h, p: hits[i].point } : null; }
    return null; }
  function unhover() { glow(hot, false); hot = null; tip.style.display = "none"; gl.style.cursor = "grab"; }
  var CAM = SWCam(THREE, cam, gl, { hit: function (e) { return !!hitOf(e); }, pick: function (e) { ptr(e); ray.setFromCamera(mv, cam); var hs = ray.intersectObjects(scene.children, true).filter(function (h) { return h.object.visible && !h.object.isSprite && h.object.type !== "InstancedMesh"; }); return hs.length ? hs[0].point : null; }, plane: function (e, t) { ptr(e); ray.setFromCamera(mv, cam); var n = cam.getWorldDirection(new THREE.Vector3()), pl = new THREE.Plane().setFromNormalAndCoplanarPoint(n, t), p = new THREE.Vector3(); return ray.ray.intersectPlane(pl, p) ? p : null; } });
  var CENTER = new THREE.Vector3(0, .40, 0);
  var VIEWS = { show: [[1.5, .95, 1.25], [0, .8, -1.9]], rear: [[.015, .445, .60], [0, .40, .04]], inside: [[.09, .80, .27], [-.02, .38, -.03]], wall0: [[.25, .8, .35], [0, 1.5, -2.6]], wall: [[0, 2.0, .2], [0, 2.2, -5]] }, camSpeed = 5;
  /* A vista não é só uma pose: é a lei do mouse depois de chegar nela (FUNCOES/camera-solidworks.md
     §3). SHOW é o visualizador e ganha o SolidWorks inteiro; a TRÁS é o menu e por isso é pose FIXA
     relativa ao painel traseiro (sem arrasto, sem roda-zoom); DENTRO é órbita presa à mesa óptica,
     com a distância travada. As poses fixas são refeitas quando a janela muda de tamanho. */
  var LAWS = { show: { mode: "free", center: CENTER, R: .34 },
    rear: { mode: "rear", center: new THREE.Vector3(0, .314 + B.H / 2, B.D / 2), normal: new THREE.Vector3(0, 0, 1), w: B.W, h: B.H, pad: 1.55 },
    inside: { mode: "inside", center: new THREE.Vector3(-.02, .348, -.03), yaw0: .35, pit0: .92, w: .40, h: .26, pad: 1.6 },
    wall0: { mode: "free", center: CENTER, R: .34 }, wall: { mode: "free", center: CENTER, R: .34 } };
  // a câmera anda por baixo do cursor: a etiqueta da peça que estava sob ele fica mentindo na tela
  // (e a peça continua acesa) até o próximo movimento do mouse. Trocar de vista apaga as duas.
  function setCam(k, speed) { S.cam = k; unhover(); CAM.mode(LAWS[k].mode, LAWS[k]); CAM.setView(VIEWS[k][0], VIEWS[k][1]); camSpeed = speed || 5; document.querySelectorAll("#cams [data-a]").forEach(function (b) { b.classList.toggle("on", b.dataset.a === "cam." + k); }); if (S.mode === "play") blip(k === "inside" ? 700 : 1000); }
  window.SC = window.SC || {}; SC.camMode = function () { return CAM.camMode(); };   // contrato 4: hud-4 só lê

  /* ---------- painel: a gaveta da direita ----------
     Todo menu e toda configuração moram aqui (pedido do dono), numa gaveta de 360 px com abas, no
     lugar do painel flutuante que tapava o aparelho. Clicar numa peça abre a aba daquela peça: é a
     mesma navegação de antes, num lugar só. O puxador na borda abre e fecha, `Tab` também, `Esc`
     fecha. Dentro da gaveta o `Tab` volta a ser do teclado (foco entre os controles). */
  var drawer = $("#drawer"), dbody = $("#dbody"), dtabs = $("#dtabs"), dcur = "laser";
  var TABN = { laser: "LASER", dmx: "DMX", net: "NET", interlock: "INTERLOCK", bind: "BINDINGS", info: "INFO" };
  dtabs.innerHTML = Object.keys(TABN).map(function (t) { return "<button class='lb' data-tab='" + t + "'>" + TABN[t] + "</button>"; }).join("");
  function drawerOn() { return drawer.classList.contains("on"); }
  function openDrawer(tab) { if (tab) dcur = tab; drawer.classList.add("on"); paint(); }
  function closeDrawer() { drawer.classList.remove("on"); }
  function toggleDrawer() { if (drawerOn()) closeDrawer(); else openDrawer(); }
  function paint() { dbody.innerHTML = PANE[dcur](); var cv = dbody.querySelector("[data-cv]"); if (cv) cv.replaceWith(curveCanvas());
    dtabs.querySelectorAll("[data-tab]").forEach(function (b) { b.classList.toggle("on", b.dataset.tab === dcur); }); }
  function refresh() { if (drawerOn()) paint(); }
  $("#dpull").addEventListener("click", toggleDrawer);
  dtabs.addEventListener("click", function (e) { var b = e.target.closest("[data-tab]"); if (b) { blip(1000); openDrawer(b.dataset.tab); } });
  // dentro da gaveta o Tab é do teclado, não do aparelho: sem isto o binding global engolia o Tab
  // e não havia como andar de controle em controle com o teclado (SISTEMA.md §11).
  drawer.addEventListener("keydown", function (e) { if (e.key === "Tab") e.stopPropagation(); });
  /// `cam-4` desloca o enquadramento da vista TRÁS pelo que a gaveta ocupa: a gaveta não pode
  /// tapar o aparelho, e ela abre e fecha.
  window.SC = window.SC || {}; SC.drawerWidth = function () { return drawerOn() ? drawer.offsetWidth : 0; };
  function get(p) { var a = p.split("."), o = S; for (var i = 0; i < a.length; i++) o = o[a[i]]; return o; } function set(p, v) { var a = p.split("."), o = S; for (var i = 0; i < a.length - 1; i++) o = o[a[i]]; o[a[a.length - 1]] = v; }
  var FMT = { kpps: function (v) { return Math.round(v / 1000) + "k"; }, pct: function (v) { return Math.round(v * 100) + "%"; }, gam: function (v) { return "γ " + (+v).toFixed(2); }, n: function (v) { return v; }, x: function (v) { return "×" + (+v).toFixed(2); } }, fmtOf = {};
  function rg(label, path, min, max, step, f) { fmtOf[path] = FMT[f]; return "<div class='row'><span>" + label + "</span><input type='range' data-p='" + path + "' min='" + min + "' max='" + max + "' step='" + step + "' value='" + get(path) + "'><span class='v' data-v='" + path + "'>" + FMT[f](get(path)) + "</span></div>"; }
  /// Campo de texto (host do DAC, universo/canal do interlock, tópico MQTT) não redesenha a gaveta
  /// enquanto se digita: o campo sumiria debaixo do cursor.
  drawer.addEventListener("input", function (e) { var r = e.target;
    if (r.dataset.host !== undefined) { ENG.host = r.value.trim(); return; }
    if (r.dataset.ilk) { ILK[r.dataset.ilk] = r.dataset.ilk === "topic" ? r.value : Math.max(1, Math.min(512, +r.value || 1)); ilkSave(); return; }
    if (!r.dataset.p) return; set(r.dataset.p, +r.value); dbody.querySelector("[data-v='" + r.dataset.p + "']").textContent = fmtOf[r.dataset.p](+r.value); remember(); if (/^(lim|gam)\./.test(r.dataset.p)) { var old = dbody.querySelector(".cv"); if (old) old.replaceWith(curveCanvas()); } push(r.dataset.p, get(r.dataset.p)); Bind.syncAll(); });
  /// O mesmo valor aparece em três lugares no painel (fader, número e a linha `spell` do `<pre>`),
  /// e o `<pre>` ficava congelado no valor de quando o painel abriu — LIMITE 25% com
  /// `spell ilda limit --r 1.00` embaixo. Ao SOLTAR o fader o painel se redesenha inteiro (não
  /// durante o arrasto, que destruiria o fader na mão) e o foco volta para o mesmo fader.
  drawer.addEventListener("change", function (e) { var p = e.target.dataset.p; if (!p) return; refresh(); var el = dbody.querySelector("[data-p='" + p + "']"); if (el) el.focus(); });
  drawer.addEventListener("click", function (e) { if (Bind.click(e)) { paint(); return; }
    var f = e.target.closest("[data-f]"); if (f) { playFile(f.dataset.f, f.dataset.n); refresh(); return; }
    var d = e.target.closest("[data-d]"); if (d) { var p = d.dataset.d.split("|"); ENG.dac = p[0]; ENG.host = p[1]; refresh(); return; }
    var b = e.target.closest("[data-a]"); if (b) Bind.run(b.dataset.a); });
  function curveCanvas() { var c = document.createElement("canvas"); c.width = 268; c.height = 90; c.className = "cv"; var x = c.getContext("2d"); x.fillStyle = "#050708"; x.fillRect(0, 0, 268, 90); ["r", "g", "b"].forEach(function (k) { x.strokeStyle = { r: "#FF2A1A", g: "#38FF5C", b: "#3A6BFF" }[k]; x.lineWidth = 1.5; x.beginPath(); for (var i = 0; i <= 40; i++) { var t = i / 40, y = S.lim[k] * Math.pow(t, S.gam[k]); x.lineTo(4 + t * 260, 86 - y * 82); } x.stroke(); }); return c; }
  function onoff(a, on, y, n) { return "<button class='lb" + (on ? " on" : "") + "' data-a='" + a + "'>" + (on ? y : n) + "</button>"; }
  /// Botão de estado da NET: o rótulo diz ON/OFF sempre (estado não é só a cor — SISTEMA.md §11) e
  /// os dois rótulos têm quase a mesma largura, senão o botão do lado escapa de baixo do cursor.
  function netBtn(k) { var K = k.toUpperCase(); return onoff("net." + k, S.net[k], K + " · ON", K + " · OFF"); }
  /// Os .ild que estão no disco do engine (`laser_files`). O `<input type=file>` e o arrastar
  /// continuam: aquilo é o arquivo que veio na mão, isto é o que já está na máquina do show.
  function fileList() { if (!ENG.on) return "<div class='sub'>engine offline · arraste um .ild na porta ILDA IN</div>";
    if (!ENG.files.length) return "<div class='sub'>nenhum .ild em shows/ no disco do engine</div>";
    return "<div class='fl'>" + ENG.files.map(function (f) { return "<button class='lb" + (ENG.file === f.path ? " on" : "") + "' data-f=\"" + f.path.replace(/"/g, "&quot;") + "\" data-n=\"" + f.name.replace(/"/g, "&quot;") + "\">" + f.name + " <small>" + Math.round(f.bytes / 1024) + " kB</small></button>"; }).join("") + "</div>"; }
  /// A chave arma o aparelho, e armar é abrir um DAC: sem DAC escolhido não há o que armar.
  function dacList() { if (!ENG.on) return "<pre>engine offline · a chave arma só o modelo</pre>";
    return "<pre>DAC <b>" + ENG.dac + " " + (ENG.host || "(sem host)") + "</b>" + (ENG.feed != null ? " · feed " + ENG.feed : "") + "\nspell laser_open --dac " + ENG.dac + " --host " + (ENG.host || "&lt;ip&gt;") + " --kpps " + Math.round(S.kpps / 1000) + "</pre>"
      + "<div class='map'><span>HOST</span><input class='tx' data-host value=\"" + ENG.host.replace(/"/g, "&quot;") + "\" placeholder=\"169.254.207.140\"></div>"
      + "<div class='btns'><button class='lb" + (ENG.scan ? " on" : "") + "' data-a='dacs'>" + (ENG.scan ? "PROCURANDO…" : "PROCURAR DACS") + "</button>"
      + (ENG.dacs || []).map(function (d) { return "<button class='lb" + (ENG.host === d.host ? " on" : "") + "' data-d=\"" + d.type + "|" + d.host + "\">" + d.type.toUpperCase() + " " + d.host + "</button>"; }).join("") + "</div>"; }
  function h3(t, s) { return "<h3>" + t + "</h3><div class='sub'>" + s + "</div>"; }
  /// O interlock não é um botão de liga-desliga: é uma ENTRADA do aparelho. Quem o aciona é a fonte
  /// mapeada aqui, e a identidade dela é um endereço textual (FUNCOES/README regra 2), o mesmo em
  /// tecla, MIDI, OSC, Art-Net, MQTT e CLI. Só o que já existe no engine é ligado de verdade
  /// (tecla e MIDI, pelo `Bind`); o resto mostra o endereço e a linha de CLI, marcada "em breve".
  var ILK_ADDR = "laser/1/interlock", ILK = { univ: 1, chan: 512, topic: "spell/laser/1/interlock" };
  try { var im = JSON.parse(localStorage.getItem("sc-laser-ilk") || "null"); if (im) { ILK.univ = im.univ || ILK.univ; ILK.chan = im.chan || ILK.chan; ILK.topic = im.topic || ILK.topic; } } catch (e) {}
  function ilkSave() { try { localStorage.setItem("sc-laser-ilk", JSON.stringify(ILK)); } catch (e) {} }
  function learnBtn(src, ready) { var l = Bind.learnState(), on = l && l.id === "lock.toggle" && l.src === src;
    return "<button class='lb" + (on ? " learn" : "") + "' data-learn='" + src + "' data-id='lock.toggle'>" + (on ? ready : (Bind.keyOf("lock.toggle", src) || "MAPEAR")) + "</button>"; }
  var PANE = {
    /// Contador ao vivo é do HUD (canto de cima: frame, pontos, fps). Aqui ficava a mesma conta
    /// congelada no instante em que o painel abriu — número parado ao lado de um número andando é
    /// pior que número nenhum. A gaveta diz o que não muda: o arquivo, os limites, a curva.
    laser: function () { return h3("ILDA IN", "DB25 · " + S.name)
      + "<pre>arquivo <b>" + S.show.length + " frames</b> · frame, pontos e fps ao vivo no canto de cima\nspell ilda play show.ild --kpps " + Math.round(S.kpps / 1000) + "</pre>"
      + "<div class='btns'><button class='lb' data-a='file.open'>ESCOLHER .ILD</button><button class='lb' data-a='demo'>DEMO</button>" + onoff("play.toggle", S.play, "PAUSA", "PLAY") + "</div>"
      + rg("TAMANHO", "size", .3, 1.3, .01, "pct") + fileList()
      + h3("GALVOS X/Y", "30 kpps nominal · 40 máx") + rg("KPPS", "kpps", 5000, 40000, 500, "kpps") + rg("BUFFER", "buffer", 1, 8, 1, "n") + rg("VELOCIDADE", "speed", .4, 1.6, .01, "x")
      + "<pre>taxa de frame = kpps ÷ pontos · &lt; 25 fps pisca\n&gt; 32 kpps o galvo não acompanha: cantos viram curvas</pre>"
      + h3("MÓDULOS R/G/B", "638 / 520 / 445 nm · limite e curva de cada diodo")
      + ["r", "g", "b"].map(function (k) { return rg({ r: "VERMELHO", g: "VERDE", b: "AZUL" }[k], "lim." + k, 0, 1, .01, "pct") + rg("CURVA " + k.toUpperCase(), "gam." + k, .5, 2.5, .01, "gam"); }).join("")
      + "<span data-cv></span><pre>spell ilda limit --r " + S.lim.r.toFixed(2) + " --g " + S.lim.g.toFixed(2) + " --b " + S.lim.b.toFixed(2) + "</pre>"
      + h3("OBTURADOR", "solenoide · fecha sem chave ou sem interlock") + "<pre>estado: <b>" + (live() ? "ABERTO" : "FECHADO") + "</b> · tempo de fechamento 8 ms</pre>"
      + h3("ILDA OUT", "DB25 macho · passa o sinal para o próximo projetor") + "<pre>encadeado: <b>nenhum</b>\nspell ilda play show.ild --chain 2</pre>"; },
    dmx: function () { return h3("DMX IN / OUT", "XLR-3 · endereço, universo e modo")
      + rg("ENDEREÇO", "dmx", 1, 512, 1, "n") + rg("UNIVERSO", "univ", 1, 512, 1, "n")
      + "<pre>modo <b>16 canais</b>: shutter · padrão · tamanho · rotação · X · Y · cor · kpps...\nbus: " + (S.dmxIn ? "U" + S.dmxIn.universe + " ch" + S.dmx + " = " + S.dmxIn.value : "nenhum frame recebido") + "\nspell dmx addr " + S.dmx + "</pre>"
      + "<pre>o DMX OUT repete o universo para o próximo aparelho · o Pino está plugado nele</pre>"; },
    /// O DAC mora na porta NET: o EtherDream é um aparelho de rede. O host é editável — o beacon
    /// pode não chegar (Sitter aberto), e digitar o IP tem de ser possível sem procurar.
    net: function () { return h3("NET", "RJ45 · fontes de rede e o DAC")
      + "<div class='btns'>" + ["ndi", "spout", "artnet", "sacn"].map(netBtn).join("") + "</div>"
      + "<pre>NDI → ILDA e Spout → ILDA são o conversor <b>FÓSFORO</b>, que ainda não existe.\nspell ilda net --ndi \"RESOLUME (out)\"</pre>" + dacList(); },
    interlock: function () { return h3("INTERLOCK", "uma entrada do aparelho · endereço <b>" + ILK_ADDR + "</b>")
      + "<pre>estado agora: <b>" + (S.lock ? "PLUGUE POSTO · obturador liberado" : "PLUGUE FORA · SCAN FAIL") + "</b>\nquem muda esse estado é a fonte mapeada abaixo</pre>"
      + "<div class='sub'>O interlock é uma entrada. Mapeie para:</div>"
      + "<div class='map'><span>TECLA</span>" + learnBtn("key", "APERTE A TECLA…")
      + "<span>MIDI</span>" + learnBtn("midi", "MEXA NO CONTROLADOR…")
      + "<span>OSC</span><code>" + ILK_ADDR + "</code>"
      + "<span>ART-NET</span><span>univ <input class='tx n' data-ilk='univ' value='" + ILK.univ + "'> canal <input class='tx n' data-ilk='chan' value='" + ILK.chan + "'> · ≥ 128 = fechado</span>"
      + "<span>MQTT</span><input class='tx' data-ilk='topic' value=\"" + ILK.topic.replace(/"/g, "&quot;") + "\">"
      + "<span>CLI</span><code>spell laser_param --path shutter --value 1</code></div>"
      + "<pre>MIDI já mapeia no engine: <b>spell midi_map --key 176/1 --cmd laser_param</b>\nOSC, Art-Net e MQTT ainda não têm comando de mapa:\n<b class='soon'>spell map osc " + ILK_ADDR + "   (em breve)</b></pre>"
      + "<div class='btns'>" + onoff("lock.toggle", S.lock, "SIMULAR ABERTURA", "REPOR O PLUGUE") + "</div>"; },
    bind: function () { return h3("BINDINGS", "tecla e MIDI · clique, depois aperte a tecla ou mexa no controlador · Esc cancela")
      + "<div class='btns'><button class='lb' data-a='midi.connect'>MIDI: " + Bind.midi + "</button>" + onoff("cam.reverse", CAM.reverse, "RODA: SOLIDWORKS", "RODA: NORMAL") + "<button class='lb amb' data-a='bind.reset'>RESET</button></div>" + Bind.html(); },
    info: function () { return h3("SPELLCASTER LASER", "info · N fecha")
      + "<pre>o programa é o aparelho: portas atrás = menu, tampa aberta = preferências\nparede = kpps ÷ pontos, galvo passa-baixa; feixe em GLSL; câmera SolidWorks; bindings tecla + MIDI\nos cinco pinos do Pino são as telas do programa\n\nengine: <b>" + (ENG.on ? "ligado · rev " + ENG.rev + (ENG.show ? " · " + ENG.show : "") : "offline (a página roda local)") + "</b>" + (ENG.feed != null ? " · feed " + ENG.feed + " no " + ENG.dac : "") + "\nCLI: spell laser_open --dac etherdream --host &lt;ip&gt; · spell laser_play --file show.ild\n\nGREETZ: PANGOLIN · ETHER DREAM · LSX · KVANT · CHATAIGNE · MADMAPPER\nCRACKED BY FEITIÇARIA iNDUSTRIAL · NO SERIAL NEEDED</pre>"; } };
  /* A peça clicada abre a aba dela: é a navegação de sempre, agora num lugar só. Peça que não está
     aqui é peça inerte (`LaserEngine.inert`) e o clique nela não faz nada. */
  var TAB_OF = { ilda: "laser", ildathru: "laser", shutter: "laser", galvo: "laser", galvodrv: "laser", r: "laser", g: "laser", b: "laser",
    dmxin: "dmx", dmxout: "dmx", rj45: "net", interlock: "interlock", bind: "bind", nfo: "info" };
  var PANELS = {}; Object.keys(TAB_OF).forEach(function (k) { PANELS[k] = function () { openDrawer(TAB_OF[k]); }; });

  /* ---------- display do painel ----------
     O display é o instrumento do aparelho: 200 × 100 mm no meio da traseira, sete páginas, e a
     linha grande de cada uma é a resposta que o operador precisa ler do outro lado da sala. O texto
     vem inteiro de `LaserEngine.oledLines(S, ENG)` (função pura, testada sem navegador); aqui só se
     desenha. Uma cor só, `--oled`: alarme é inversão de bloco, nunca cor nova (SISTEMA.md §5).
     O encoder é um encoder: fora de campo gira páginas, dentro de campo gira valores, aperta entra
     e passa ao campo seguinte, BACK sai. Uma família de função, um mecanismo. */
  var PAGES = LaserEngine.PAGES, ERRPAGE = PAGES.indexOf("ERRO"), oc = B.oled.c, OLED = "#9FF5D0";
  function fieldsNow() { return LaserEngine.FIELDS[PAGES[S.page]] || []; }
  function fieldSet(d) { var f = fieldsNow()[S.field];
    if (f === "kpps") { S.kpps = Math.max(5000, Math.min(40000, S.kpps + d * 1000)); remember(); }
    else if (f === "addr") S.dmx = Math.max(1, Math.min(512, S.dmx + d));
    else if (f === "univ") S.univ = Math.max(1, Math.min(512, S.univ + d));
    else if (f && S.net[f] !== undefined) Bind.run("net." + f); }
  function oledTurn(d) { if (dead()) return; click(); S.encT = .1; B.knob.rotation.y -= d * .35; // gira no eixo do próprio cilindro
    if (S.edit) fieldSet(d); else { S.page = (S.page + d + PAGES.length) % PAGES.length; S.field = 0; }
    drawOled(); refresh(); }
  function oledOk() { if (dead()) return; click(); S.encT = .12; var f = fieldsNow();
    if (!S.edit) { if (!f.length) { pino.say("Essa página só informa. Gira o encoder até uma que tenha campo.", null, false); return; } S.edit = true; S.field = 0; }
    else if (PAGES[S.page] === "ERRO") { S.err = null; ENG.err = false; S.edit = false; }
    else if (++S.field >= f.length) { S.edit = false; S.field = 0; }
    drawOled(); refresh(); }
  function oledBack() { if (dead()) return; click(); S.backT = .12; if (S.edit) { S.edit = false; S.field = 0; } else S.page = (S.page + PAGES.length - 1) % PAGES.length; drawOled(); refresh(); }
  // linha que nao cabe na largura do vidro encolhe a fonte ate' caber: display de rack corta o
  // brilho, nunca a palavra (antes " ... OBTURADOR FECHADO" saia' do vidro pela direita)
  function fitText(s, x, y, bold, px, max) { var f = function (n) { return bold + n + "px 'Share Tech Mono'"; }; oc.font = f(px); var tw = oc.measureText(s).width; if (tw > max) oc.font = f(Math.floor(px * max / tw)); oc.fillText(s, x, y); }
  function drawOled() { var w = B.oled.w, h = B.oled.h; oc.fillStyle = "#020806"; oc.fillRect(0, 0, w, h);
    if (!S.power) { B.oled.tex.needsUpdate = true; return; }
    var LN = LaserEngine.oledLines(S, ENG), alarm = (S.power && S.key && !S.lock) || (S.page === ERRPAGE && !!S.err), i;
    oc.shadowColor = OLED; oc.shadowBlur = 5; oc.textBaseline = "middle"; oc.fillStyle = OLED;
    oc.font = "700 34px 'Share Tech Mono'"; oc.textAlign = "left"; oc.fillText(LN[0], 16, 28);
    oc.textAlign = "right"; oc.fillText(S.edit ? "GIRA: VALOR" : "GIRA: PAGINA", w - 16, 28);
    oc.shadowBlur = 0; oc.fillRect(12, 50, w - 24, 2); oc.shadowBlur = 5;
    // linha grande: bloco invertido quando é alarme (SCAN FAIL ou erro), como num display de rack
    oc.textAlign = "left";
    if (alarm) { oc.shadowBlur = 0; oc.fillRect(12, 68, w - 24, 100); oc.fillStyle = "#020806"; fitText(LN[1], 24, 118, "700 ", 92, w - 48); oc.fillStyle = OLED; oc.shadowBlur = 5; }
    else fitText(LN[1], 16, 118, "700 ", 92, w - 32);
    for (i = 2; i < LN.length && i < 6; i++) fitText(LN[i], 14, 200 + (i - 2) * 56, "", 44, w - 28);
    fitText("APERTA: " + (S.edit ? "PROXIMO CAMPO" : "ENTRA") + "   BACK: VOLTA", 16, 456, "", 30, w - 32);
    oc.shadowBlur = 0; B.oled.tex.needsUpdate = true; }
  // ponytail: redesenho por relógio de 4 Hz para o que muda sozinho (t do transporte, temperatura,
  // pontos) ; toda ação já chama `drawOled` na hora, então o feedback de clique não espera isto.
  drawOled(); setInterval(drawOled, 250);

  /* ---------- ações ---------- */
  function toggleKey() { if (dead()) return; S.key = !S.key; if (S.key) blip(1500); else chord(); camsEnabled(S.mode !== "splash"); push("key", S.key); drawOled(); refresh(); }
  Bind.def("cam.show", "vista SHOW", function () { if (!S.key) { pino.say("Arma a chave primeiro. Sem emissão não tem show; a chave está na traseira, à esquerda.", null, false); blip(300); return; } setCam("show"); }, { key: "1" });
  Bind.def("cam.rear", "vista TRÁS · menu", function () { setCam("rear"); }, { key: "2" });
  Bind.def("cam.inside", "vista DENTRO · preferências", function () { setCam("inside"); }, { key: "3" });
  Bind.def("key.toggle", "chave: arma", toggleKey, { key: "S", get: function () { return S.key; } });
  // ao repor o plugue o balão do SCAN FAIL some junto: aviso que descreve um estado não pode
  // continuar na tela depois que o estado acabou (o display já voltou a dizer LIVE)
  Bind.def("lock.toggle", "interlock", function () { if (dead()) return; S.lock = !S.lock; if (!S.lock) { chord(); pino.say("SCAN FAIL. Interlock aberto: obturador fechado, feixe estacionado.", null, false); } else { blip(1300); pino.hide(); } push("lock", S.lock); drawOled(); refresh(); }, { key: "I", get: function () { return S.lock; } });
  Bind.def("power.toggle", "energia", function () { S.power = !S.power; if (!S.power) chord(); else blip(900); drawOled(); refresh(); }, { key: "P", get: function () { return S.power; } });
  // um play só: o .ild no DAC e o transporte do show andam juntos (o show é quem manda no tempo)
  Bind.def("play.toggle", "play / pausa", function () { S.play = !S.play; blip(); push("play", S.play); push("transport", S.play); refresh(); }, { key: "Space", get: function () { return S.play; } });
  // ponytail: kpps local ate' `laser_param` aceitar `dev/pps` ; hoje o valor só chega ao DAC no
  // próximo `laser_open` (é argumento de abertura), então desarmar e armar aplica.
  Bind.def("kpps.down", "kpps −1k", function () { S.kpps = Math.max(5000, S.kpps - 1000); remember(); refresh(); }, { key: "[" });
  Bind.def("kpps.up", "kpps +1k", function () { S.kpps = Math.min(40000, S.kpps + 1000); remember(); refresh(); }, { key: "]" });
  Bind.def("kpps", "kpps (fader)", function (v) { S.kpps = Math.round(5000 + v * 35000); remember(); refresh(); }, { type: "cc", get: function () { return (S.kpps - 5000) / 35000; } });
  Bind.def("size", "tamanho (fader)", function (v) { S.size = .3 + v; push("size", S.size); refresh(); }, { type: "cc", get: function () { return S.size - .3; } });
  ["r", "g", "b"].forEach(function (k) { Bind.def("lim." + k, "limite " + { r: "vermelho", g: "verde", b: "azul" }[k] + " (fader)", function (v) { S.lim[k] = v; push("lim." + k, v); refresh(); }, { type: "cc", get: function () { return S.lim[k]; } }); });
  Bind.def("fog", "névoa (fader)", function (v) { S.fog = v; }, { type: "cc", get: function () { return S.fog; } });
  Bind.def("file.open", "abrir .ild", function () { $("#file").click(); }, { key: "O" });
  Bind.def("demo", "demo.ild", function () { S.show = demo.frames; S.frame = 0; S.pos = 0; S.name = "demo.ild · " + demo.frames.length + " frames"; refresh(); pino.say("Demo de volta: túnel, pentagrama e a fita.", null, false); }, { key: "D" });
  ["ndi", "spout", "artnet", "sacn"].forEach(function (k) { Bind.def("net." + k, "rede: " + k.toUpperCase(), function () { S.net[k] = !S.net[k]; blip(1000); drawOled(); refresh(); if (S.net[k] && (k === "ndi" || k === "spout")) pino.say(k.toUpperCase() + " ligado, mas o conversor para ILDA ainda não existe: o FÓSFORO, um monitor de rack nesta porta.", null, false); }, { get: function () { return S.net[k]; } }); });
  Bind.def("oled.up", "OLED: encoder +", function () { oledTurn(1); }); Bind.def("oled.down", "OLED: encoder −", function () { oledTurn(-1); }); Bind.def("oled.ok", "OLED: OK", oledOk); Bind.def("oled.back", "OLED: BACK", oledBack, { key: "Backspace" });
  function tabKey(t) { return function () { if (drawerOn() && dcur === t) closeDrawer(); else openDrawer(t); }; }
  Bind.def("nfo", "info", tabKey("info"), { key: "N" });
  Bind.def("bind", "bindings", tabKey("bind"), { key: "B" });
  Bind.def("drawer", "gaveta: abre e fecha", toggleDrawer, { key: "Tab", get: drawerOn });
  Bind.def("esc", "fecha a gaveta", function () { closeDrawer(); pino.hide(); }, { key: "Escape" });
  /// Procura que não responde nada é procura que parece quebrada: enquanto varre, o painel diz
  /// PROCURANDO; ao terminar diz quantos achou, e zero achados é resposta, não silêncio.
  Bind.def("dacs", "procurar DACs", function () { if (!ENG.on) { pino.say("Sem engine não há DAC para procurar. Sobe o spellcore serve e a chave passa a armar de verdade.", null, false); return; } ENG.dacs = []; ENG.scan = true; refresh(); bus.call("laser_dacs", { timeout: 2 }).then(function (r) { ENG.dacs = Array.isArray(r) ? r : []; ENG.scan = false; if (ENG.dacs.length) { if (!ENG.host) { ENG.dac = ENG.dacs[0].type; ENG.host = ENG.dacs[0].host; } pino.say(ENG.dacs.length + (ENG.dacs.length > 1 ? " DACs na rede." : " DAC na rede.") + " Clica no que vai receber o feixe.", null, false); } else pino.say("Nenhum DAC respondeu em 2 s. Confere o cabo no RJ45 e se o EtherDream está na mesma rede.", null, false); refresh(); }, function (e) { ENG.scan = false; refresh(); fail("laser_dacs: " + e.message); }); });
  Bind.def("midi.connect", "MIDI: conectar", function () { Bind.connect(); }); Bind.def("bind.reset", "bindings: reset", function () { Bind.reset(); });
  Bind.def("cam.reverse", "roda: sentido SolidWorks", function () { CAM.reverse = !CAM.reverse; try { localStorage.setItem("sc-laser-wheel", CAM.reverse ? "1" : "0"); } catch (e) {} refresh(); }, { get: function () { return CAM.reverse; } }); try { if (localStorage.getItem("sc-laser-wheel") === "0") CAM.reverse = false; } catch (e) {}
  // ±2° de paralaxe na vista fixa: é o único movimento que a traseira aceita, e é desligável
  Bind.def("cam.breathe", "TRÁS: paralaxe do mouse", function () { CAM.breathe = !CAM.breathe; try { localStorage.setItem("sc-laser-breathe", CAM.breathe ? "1" : "0"); } catch (e) {} refresh(); }, { get: function () { return CAM.breathe; } }); try { if (localStorage.getItem("sc-laser-breathe") === "0") CAM.breathe = false; } catch (e) {}
  var d15 = Math.PI / 12; [["L", "ArrowLeft", d15, 0], ["R", "ArrowRight", -d15, 0], ["U", "ArrowUp", 0, d15], ["D", "ArrowDown", 0, -d15]].forEach(function (a) { Bind.def("cam.rot" + a[0], "câmera: gira 15° " + a[0], function () { CAM.rotate(a[2], a[3]); }, { key: a[1] }); Bind.def("cam.rot90" + a[0], "câmera: gira 90° " + a[0], function () { CAM.rotate(a[2] * 6, a[3] * 6); }, { key: "Shift+" + a[1] }); Bind.def("cam.pan" + a[0], "câmera: pan " + a[0], function () { CAM.pan(a[2] * -400, a[3] * 400); }, { key: "Ctrl+" + a[1] }); });
  // Alt+setas = roll (t_roll_view.htm): girar a vista no plano da tela, que era o único gesto do manual sem par aqui
  [["L", "ArrowLeft", .12], ["R", "ArrowRight", -.12]].forEach(function (a) { Bind.def("cam.roll" + a[0], "câmera: roll " + a[0], function () { CAM.roll(a[2]); }, { key: "Alt+" + a[1] }); });
  Bind.def("cam.fit", "câmera: enquadra", function () { CAM.fit(CENTER, .32); }, { key: "F" });
  [["front", "1"], ["back", "2"], ["left", "3"], ["right", "4"], ["top", "5"], ["bottom", "6"], ["iso", "7"]].forEach(function (v) { Bind.def("cam." + v[0], "vista padrão: " + v[0], function () { CAM.std(v[0], CENTER, .32); }, { key: "Ctrl+" + v[1] }); });
  // no manual (t_zoom_in_out.htm) é `Z` que AFASTA e `Shift+Z` que aproxima; estava trocado aqui
  Bind.def("cam.zoomIn", "câmera: zoom +", function () { CAM.zoom(.8); }, { key: "Shift+Z" }); Bind.def("cam.zoomOut", "câmera: zoom −", function () { CAM.zoom(1.25); }, { key: "Z" });
  // O estado do MIDI (e o LEARN em curso) mora na gaveta e na linha MIDI dos stats: o canto de
  // baixo da tela some (pedido do dono), e um estado só pode ser lido num lugar.
  Bind.onChange(refresh);
  document.querySelectorAll("#cams [data-a]").forEach(function (b) { b.addEventListener("click", function () { if (S.mode === "splash") return; Bind.run(b.dataset.a); }); });
  /// Durante o splash esses botões engoliam o clique em silêncio, com cara de botão vivo. Botão que
  /// não faz nada agora fica desabilitado (cinza, sem cursor de mão) e volta quando o splash sai.
  function camsEnabled(on) { document.querySelectorAll("#cams [data-a]").forEach(function (b) { b.disabled = !on || (b.dataset.a === "cam.show" && !S.key); }); }
  camsEnabled(false);

  /* ---------- interação 3D ---------- */
  function glow(o, on) { if (!o) return; o.traverse(function (mm) { if (mm.material && mm.material.emissive) { if (on) { if (mm.userData.orig) return; mm.userData.orig = mm.material; mm.material = mm.material.clone(); mm.material.emissive.setHex(0x38ff5c); mm.material.emissiveIntensity = .45; } else if (mm.userData.orig) { mm.material = mm.userData.orig; mm.userData.orig = null; } } }); }
  /* Um clique, uma função. A família da peça (`LaserEngine.CONTROLS`) decide o que o clique faz, e
     nenhuma peça cai em dois ramos: `toggle` inverte o estado e acaba ali (não abre gaveta), a
     peça inerte (chapa, aleta, plugue, ventoinha) não faz nada, `mapear` abre onde se escolhe quem
     aciona aquela entrada, e o resto é navegação — abre a aba daquela peça na gaveta. */
  var ACT = { power: function () { Bind.run("power.toggle"); }, keyswitch: function () { Bind.run("key.toggle"); }, interlock: function () { openDrawer("interlock"); },
    enc: oledOk, back: oledBack, lid: function () { setCam("inside"); }, pino: function () { pinoMenu(); } };
  function cursorFor(k) { return LaserEngine.inert(k) ? "default" : "pointer"; }
  /* O knob do encoder no gesto do TouchDesigner: aperta e sobe o mouse = aumenta, desce = diminui,
     um passo a cada 6 px (Shift = 24 px, ajuste fino). A câmera não gira durante o arrasto em vista
     nenhuma (`cam.js` recusa arrasto do esquerdo que começa numa peça), e soltar sem andar 3 px
     continua sendo clique = OK. A roda em cima do knob continua girando o encoder, como antes.
     ponytail: só o knob; os faders do painel são `<input type=range>`, e o gesto deles é do HTML. */
  var knob = null;
  gl.addEventListener("pointermove", function (e) { var p = ptr(e);
    if (knob) { var up = knob.y - e.clientY; knob.y = e.clientY; knob.acc += up; var px = e.shiftKey ? 24 : 6;
      while (knob.acc >= px) { knob.acc -= px; oledTurn(1); } while (knob.acc <= -px) { knob.acc += px; oledTurn(-1); }
      tip.style.display = "none"; return; }
    if (CAM.dragging() || S.mode === "splash") { tip.style.display = "none"; return; } var h = hitOf(e), o = h ? h.o : null; if (o !== hot) { glow(hot, false); hot = o; if (hot && cursorFor(hot.userData.key) === "pointer") glow(hot, true); }
    if (hot) { tip.style.display = "block"; tip.textContent = hot.userData.label; tip.style.left = (p[0] + 14) + "px"; tip.style.top = (p[1] + 14) + "px"; gl.style.cursor = hot.userData.key === "enc" ? "ns-resize" : cursorFor(hot.userData.key); } else { tip.style.display = "none"; gl.style.cursor = "grab"; } });
  gl.addEventListener("pointerleave", unhover);
  gl.addEventListener("pointerdown", function (e) { pressed = e.button === 0;
    if (e.button !== 0 || S.mode === "splash") return;
    var h = hitOf(e); if (!h || h.o.userData.key !== "enc") return;                 // contrato 3: `B.knobHit` também é "enc"
    knob = { y: e.clientY, y0: e.clientY, acc: 0 }; gl.setPointerCapture(e.pointerId); e.preventDefault(); });
  // o alvo do clique sai do raycast do próprio `pointerup`, nunca do `hot` do hover: câmera que
  // anda por baixo do cursor (ou clique sem mexer o mouse antes) não pode fazer o painel errado abrir
  gl.addEventListener("pointerup", function (e) { if (!pressed || e.button !== 0) return; pressed = false;
    if (knob) { var moved = Math.abs(e.clientY - knob.y0) >= 3; knob = null; try { gl.releasePointerCapture(e.pointerId); } catch (x) {} if (moved) return; }
    if (CAM.dragging()) return; if (S.mode === "splash") { skipSplash(); return; }
    var h = hitOf(e); if (!h) return; var k = h.o.userData.key;
    if (LaserEngine.inert(k)) return; // chapa, aleta, plugue, ventoinha: o clique não faz nada
    if (/^pino\./.test(k)) { onPin(k.slice(5)); return; }
    blip(1100);
    if (ACT[k]) { ACT[k](); return; }
    if (/^(dmx|ilda|rj45)/.test(k) && S.cam === "show") setCam("rear"); if (/^(galvo|galvodrv|shutter|r|g|b)$/.test(k) && S.cam !== "inside") setCam("inside"); (PANELS[k] || function () {})(); });
  // roda do mouse em cima do encoder = girar o encoder (não é zoom de câmera)
  gl.addEventListener("wheel", function (e) { if (hot && hot.userData.key === "enc") { e.stopImmediatePropagation(); e.preventDefault(); oledTurn(e.deltaY < 0 ? 1 : -1); } }, { passive: false, capture: true });
  gl.addEventListener("dragover", function (e) { e.preventDefault(); stage.classList.add("dz"); }); gl.addEventListener("dragleave", function () { stage.classList.remove("dz"); });
  gl.addEventListener("drop", function (e) { e.preventDefault(); stage.classList.remove("dz"); loadFile(e.dataTransfer.files[0]); }); $("#file").addEventListener("change", function () { loadFile(this.files[0]); this.value = ""; });
  // Arquivo que veio na mão: o engine não tem esse caminho no disco dele, então `ENG.file` zera e o
  // play passa a mover só o transporte. Para sair no DAC, o .ild precisa estar em `shows/`.
  function loadFile(f) { if (!f) return; var r = new FileReader(); r.onload = function () { try { var d = ILDA.parse(r.result); if (!d.frames.length) throw 0; S.show = d.frames; S.frame = 0; S.pos = 0; ENG.file = ""; S.name = f.name + " · " + d.frames.length + " frames"; if (S.mode === "splash") skipSplash(); setCam("rear"); PANELS.ilda(); remember(); pino.say("Entrou pela ILDA IN: " + f.name + ", " + d.frames.length + " frames, " + d.frames[0].length + " pontos no primeiro. " + (d.frames[0].length > 1200 ? "Denso. Se piscar, abre a tampa e sobe os kpps no galvo." : "Leve. Vai voar.") + (S.key ? "" : " Arma a chave para ver na parede."), null, false); } catch (x) { pino.say("Isso não é ILDA. Formato 2 (só paleta) eu pulo, 0/1/4/5 eu leio.", null, false); } }; r.readAsArrayBuffer(f); }

  /* ---------- Pino ---------- */
  // O DMX OUT e' a ponta de baixo do cabo: quem sabe onde ele esta' e' o chassi. `B.dmxOut` (a
  // porta em si) da' posicao E normal do painel; `B.dmxOutWorld` e a constante ficam de reserva.
  var pino = Pino3D.build(THREE, X, scene, pick, stage, cam, { on: onPin, port: B.dmxOut, target: B.dmxOutWorld });
  Bind.def("pino.hide", "Pino: some da tela / volta", function () { if (pino.alive()) pino.bye(); else pino.back(); });
  // o custo da corda em ms por quadro fica legivel de fora (erro e' dado; a frente de usabilidade mede por aqui)
  window.SC = window.SC || {}; SC.pinoCost = function () { return pino.cost(); }; SC.pinoRopeLive = function () { return pino.alive(); };
  // O Pino é o menu do programa: cada pino é uma tela. Nada entra aqui por conveniência de
  // software — o item só existe se for peça do aparelho, e a frase do balão diz qual peça é.
  var MENU = [["ilda", "1 · LASER", "este aparelho"], ["ndi", "2 · FÓSFORO", "o conversor de rede que ainda não existe"], ["orq", "3 · PATCHBAY", "o painel de jacks: o que liga no quê"],
    ["cues", "4 · TEATRO DE PAPEL", "as cenas e os cues do DMX"], ["nfo", "5 · INFO", "N"],
    ["tl", "GRAVADOR", "a fita do aparelho: o que você mexe aqui fica gravado no tempo"], ["face", "MESA", "os botões grandes que o operador aperta durante o show"]];
  function pinoMenu() { pino.say("Cinco pinos, cinco telas. Puxa um.", MENU); }
  function onPin(k) { blip(1000);
    if (k === "ilda") { setCam("rear"); PANELS.ilda(); pino.say("ILDA IN, atrás. Solta o .ild na porta, escolhe no painel, ou pega um do disco do engine.", null, false); }
    else if (k === "ndi") { setCam("rear"); PANELS.rj45(); pino.say("A porta NET. NDI e Spout viram ILDA no FÓSFORO, que ainda não existe: por enquanto é só a porta.", null, false); }
    else if (k === "orq") { location.href = "../patchbay.html"; return; }
    else if (k === "cues") { location.href = "../teatro.html"; return; }
    else if (k === "tl") { location.href = "../index.html"; return; }
    else if (k === "face") { location.href = "../face.html?face=quatro"; return; }
    else if (k === "nfo") PANELS.nfo();
    else if (k === "bye") { pino.bye(); return; }
    pino.current(k); }
  pino.current("ilda");
  function tips() { var F = fps(), f = S.show[S.frame]; if (S.mode !== "play" || !f) return; var flick = live() && F > 0 && F < 25, slow = S.kpps > 32000;
    if (flick && !S.tip.flick) { S.tip.flick = true; pino.say("Tá piscando: " + f.length + " pontos a " + Math.round(S.kpps / 1000) + " kpps dá " + Math.round(F) + " fps. Abre a tampa e mexe no galvo.", null, false); } if (!flick) S.tip.flick = false;
    if (slow && !S.tip.slow) { S.tip.slow = true; pino.say("A " + Math.round(S.kpps / 1000) + " kpps o galvo não acompanha: os cantos viraram curvas.", null, false); } if (!slow) S.tip.slow = false; }

  /* ---------- splash / início ---------- */
  function start() { if (S.mode !== "splash") return; S.mode = "play"; $("#splash").classList.add("off"); S.dim = .55; camsEnabled(true); setCam("rear", 5); setTimeout(function () { pino.say(S.mem ? "Da última vez: " + Math.round(S.mem.kpps / 1000) + " kpps" + (S.mem.name ? ", " + S.mem.name.split(" · ")[0] : "") + ". A traseira é o menu: arma a chave e a vista SHOW libera. B abre os bindings." : "Parece que você está tentando fazer um show de laser. A traseira é o menu. Arma a chave (à esquerda) e a vista SHOW libera. Puxa um pino."); }, 900); }
  function skipSplash() { if (S.mode !== "splash") return; jingle(); start(); }
  document.addEventListener("keydown", function (e) { if (S.mode === "splash" && e.target.tagName !== "TEXTAREA") { e.stopPropagation(); skipSplash(); } }, true);

  /* ---------- HUD: os stats do canto de cima à direita ----------
     Uma linha por fato VIVO, e só aparece o que está ligado. Nada aqui é inventado: engine e DAC
     vêm do barramento, as entradas vêm do que o show declara em `inputs` (e acendem com o frame
     que chega, topic 2), o MIDI vem do `Bind`. Sem fato do engine, a linha não existe.
     Estado nunca é só cor (SISTEMA.md §11): cada LED anda colado ao rótulo que o nomeia. */
  var srows = $("#srows"), lastHud = "", INPUTS = [];
  function readInputs(sh) { INPUTS = ((sh && sh.inputs) || []).filter(function (i) { return i && i.type; }).map(function (i) {
    return { name: { sacn: "sACN", artnet: "ART-NET", midi: "MIDI IN" }[i.type] || String(i.type).toUpperCase(), u: i.universe == null ? 1 : i.universe, last: 0 }; }); }
  function row(cls, txt, led) { return "<div class='r " + cls + "'><span>" + txt + "</span><i class='led " + led + "'></i></div>"; }
  // os quatro estados do aparelho (SISTEMA.md §5), na ordem em que o operador precisa lê-los
  function stateRow() { return !S.power ? ["", "", "DESLIGADO"] : !S.key ? ["amb", "warn blink lg", "STANDBY · DESARMADO"]
    : !S.lock ? ["red", "err lg", "SCAN FAIL"] : ["las", "on lg", "LIVE · ARMADO"]; }
  function hudStats() {
    var f = S.show[S.frame], F = fps(), st = stateRow(), now = performance.now(), h = "";
    h += row("pri", ENG.on ? "ENGINE · rev " + ENG.rev + (ENG.show ? " · " + ENG.show : "") : "ENGINE OFFLINE", (ENG.on ? "on" : "") + " lg");
    if (ENG.on) h += row("", "DAC " + ENG.dac + " " + (ENG.host || "(sem host)") + (ENG.feed != null ? " · feed " + ENG.feed : ""),
      (ENG.feed != null ? "on" : ENG.scan ? "warn blink" : ENG.err ? "err" : "") + " lg");
    INPUTS.forEach(function (i) { h += row("", i.name + " · in " + i.u, now - i.last < 2000 ? "on" : ""); });
    if (/^[0-9]/.test(Bind.midi)) h += row("", "MIDI · " + Bind.midi, /^0 in/.test(Bind.midi) ? "" : "on");
    h += row(F && F < 25 && live() ? "red" : "las", (S.kpps / 1000).toFixed(0) + " kpps · " + (f ? f.length : 0) + " pts · " + (F ? F.toFixed(0) : "–") + " fps", "nil");
    h += row("", S.name.split(" · ")[0] + " · frame " + (S.frame + 1) + "/" + S.show.length + " · DMX " + S.dmx, "nil");
    h += row(st[0], st[2], st[1]);
    if (h !== lastHud) { lastHud = h; srows.innerHTML = h; }
  }

  /* ---------- tick ---------- */
  var last = performance.now(), W = 0, H = 0, T0 = performance.now(), lidT = 0, rearI = 0, segsW = [], tmpV = new THREE.Vector3();
  function size() { if (stage.clientWidth !== W || stage.clientHeight !== H) { W = stage.clientWidth; H = stage.clientHeight; var pr = R.getPixelRatio(); R.setSize(W, H, false); composer.setSize(W, H); bloom.resolution.set(W, H); fxaa.uniforms.resolution.value.set(1 / (W * pr), 1 / (H * pr)); cam.aspect = W / H; cam.updateProjectionMatrix(); } }
  /* dt nunca anda para trás. O `now` do requestAnimationFrame é o instante em que o QUADRO começou,
     e ele pode ser anterior ao `performance.now()` guardado em `last` na carga da página: nos
     primeiros quadros dt saía negativo (−0,23 s, medido no headless), `S.pos += S.kpps * dt` jogava
     a posição do galvo para −6848 e `f[índice negativo]` virava `undefined` — TypeError na parede a
     cada quadro até a posição voltar a subir, com a tarja vermelha de erro por cima da tela de quem
     abre `app.html#laser`. Era também a animação inteira (câmera, tampa, ventoinha) andando de ré. */
  function tick(now) { size(); var dt = Math.min(.1, Math.max(0, (now - last) / 1000)); last = now; var t = (now - T0) / 1000;
    var wt0 = PERF && performance.now(); if (S.mode === "splash") { wallSplash(now, dt); wallTex.needsUpdate = true; } else wallTick(now, dt); if (PERF) { PERF.wallMs += performance.now() - wt0; PERF.frames++; }
    var want = S.cam === "inside" ? 1 : 0; lidT += (want - lidT) * Math.min(1, dt * 3); var sT = Math.min(1, lidT / .45), lT = Math.max(0, (lidT - .4) / .6); B.screws.forEach(function (s, i) { s.position.y = .004 + sT * .05; s.rotation.y = sT * 12 + i; }); B.lid.rotation.x = -lT * 1.9;
    CAM.update(dt * camSpeed / 5); rearI += (((S.cam === "rear" && S.mode === "play") ? 14 : 0) - rearI) * Math.min(1, dt * 3); B.rearLight.intensity = rearI; inLight.intensity = .35 * lidT; sun.intensity = 90 * S.dim; B.wallLight.intensity = 14 * S.dim;
    // feixes externos: abertura → pontos acesos da parede
    var on = S.power && (S.mode === "splash" || live()), step = Math.max(1, Math.ceil(lit.length / 150)), gain = (.05 + .16 * S.fog); segsW.length = 0; if (on) for (var i = 0; i < lit.length; i += step) { var L = lit[i]; segsW.push([[B.APERT.x, B.APERT.y, B.APERT.z], [(L[0][0] / WW - .5) * 8, 2.2 + (.5 - L[0][1] / WH) * 5, -5], [L[1][0] / 255 * gain, L[1][1] / 255 * gain, L[1][2] / 255 * gain]]); } beamsOut.set(segsW, 1);
    // caminho óptico interno
    var arm = S.power && (S.key || S.mode === "splash"), opn = S.lock, segsI = O.segments(arm, opn, S.lim, gpos).map(function (s) { return [[s[0][0], s[0][1] + .314, s[0][2]], [s[1][0], s[1][1] + .314, s[1][2]], s[2]]; }); beamsIn.set(segsI, 1.2); beamsOut.tick(t); beamsIn.tick(t);
    O.shutter.rotation.y += (((arm && opn) ? 1.2 : 0) - O.shutter.rotation.y) * Math.min(1, dt * 12); O.mirX.rotation.y = -Math.PI / 4 + gpos[0] * .1; O.mirY.rotation.z = gpos[1] * .1;
    ["r", "g", "b"].forEach(function (k) { O.lens[k].m.material.color.setHex(arm ? O.lens[k].c : 0x111111); });
    proj.material.opacity = S.power ? 1 : 0;
    // LED de emissão com os quatro estados do aparelho (SISTEMA.md §5): apagado · âmbar lento em
    // STANDBY · vermelho fixo em SCAN FAIL · verde fixo em LIVE. Era vermelho para qualquer coisa
    // armada, e no LED o LIVE ficava igual ao SCAN FAIL. Os LEDs da NET só acendem com energia.
    B.emLed.material.color.setHex(!S.power ? 0x2a0a08 : !arm ? ((reduced || Math.floor(t * 1.2) % 2) ? 0xffb000 : 0x2a1e00) : !S.lock ? 0xff2a1a : 0x38ff5c);
    B.led1.material.color.setHex(S.power && (S.net.sacn || S.net.artnet) ? 0x38ff5c : 0x0a2a10); B.led2.material.color.setHex(S.power && (S.net.ndi || S.net.spout) && Math.floor(t * 6) % 2 ? 0xffb000 : 0x2a1e00);
    B.keyM.rotation.z += ((S.key ? Math.PI / 2 : 0) - B.keyM.rotation.z) * Math.min(1, dt * 8); B.lockPlug.position.z += ((S.lock ? 0 : .022) - B.lockPlug.position.z) * Math.min(1, dt * 6); B.rocker.rotation.x += ((S.power ? -.3 : .3) - B.rocker.rotation.x) * Math.min(1, dt * 18); B.blades.rotation.z += dt * (S.power ? 24 : 0);
    // encoder e BACK afundam quando apertados: botão físico que não anda não dá feedback
    S.encT = Math.max(0, S.encT - dt); S.backT = Math.max(0, (S.backT || 0) - dt);
    B.knob.position.z += ((S.encT > 0 ? .0072 : .009) - B.knob.position.z) * Math.min(1, dt * 22); B.backCap.position.z += ((S.backT > 0 ? .0015 : .003) - B.backCap.position.z) * Math.min(1, dt * 22);
    puffs.forEach(function (p) { p.position.x += p.userData.v[0] * dt; p.position.y += p.userData.v[1] * dt; if (p.position.x > 3) p.position.x = -3; if (p.position.x < -3) p.position.x = 3; p.material.opacity = .02 + .06 * S.fog; });
    S.temp += ((live() ? 42 : 31) - S.temp) * dt * .05;
    tmpV.set(0, .314, .15).project(cam); pino.update(dt, mouse, W, H, (1 - tmpV.y) / 2 * H + 14);
    // LED grande de ARMADO no chassi (contrato do `chassi-4`): apagado sem energia, vermelho forte
    // piscando desarmado pela chave, vermelho fixo em SCAN FAIL, verde fixo armado. É o mesmo
    // estado do bloco de stats — o operador lê no aparelho e na tela sem precisar comparar.
    if (B.armLed) B.armLed.material.color.setHex(!S.power ? 0x2a0a08 : !S.key ? ((reduced || Math.floor(t * 1.6) % 2) ? 0xff2a1a : 0x2a0a08) : !S.lock ? 0xff2a1a : 0x38ff5c);
    hudStats();
    if (S.mode === "play") tips(); composer.render(); requestAnimationFrame(tick); }

  /* ---------- boot: câmera mirada no output; o foco sai do ponto estático e vai para a parede ---------- */
  document.fonts.ready.then(function () { size(); plate(null); var ol = ILDA.outlines([["SPELLCASTER", "64px Michroma", 272], ["LASER", "64px Michroma", 372]], WW, WH); SP.loops = ol.loops; SP.len = ol.len;
    var h = location.hash; if (h === "#laser" || h === "#tras" || h === "#dentro") { CAM.setView(VIEWS.rear[0], VIEWS.rear[1], true); start(); if (h === "#laser" || h === "#dentro") { S.key = true; camsEnabled(true); if (h === "#laser") setCam("show"); } if (h === "#dentro") setCam("inside"); }
    else { CAM.setView(VIEWS.wall0[0], VIEWS.wall0[1], true); setTimeout(function () { setCam("wall", 2.6); }, 300); S.t0 = performance.now(); }
    requestAnimationFrame(tick); });
})();
