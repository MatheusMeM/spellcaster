/* Spellcaster LASER · the main page of the program: device state, wall (2D render → texture), scene
   (mat/body/optics/beam/pino), splash (camera on the wall → outline → glow → flies to the rear), SolidWorks
   camera, actions with key and MIDI bindings, panels, OLED, and the link to the engine through `bus.js`.
   Without an engine the page stays whole: the `Bus` answers in offline mode and the wall draws the same. */
(function () {
  "use strict";
  var $ = function (s) { return document.querySelector(s); }, reduced = matchMedia("(prefers-reduced-motion: reduce)").matches;
  /* ---------- audio ---------- */
  var AC = null; function ac() { if (!AC) AC = new (window.AudioContext || window.webkitAudioContext)(); if (AC.state === "suspended") AC.resume(); return AC; }
  function tone(f, t0, d, type, g) { var a = ac(), o = a.createOscillator(), v = a.createGain(); o.type = type || "square"; o.frequency.value = f; v.gain.setValueAtTime(0, t0); v.gain.linearRampToValueAtTime(g || .08, t0 + .01); v.gain.exponentialRampToValueAtTime(.0005, t0 + d); o.connect(v); v.connect(a.destination); o.start(t0); o.stop(t0 + d + .02); }
  function jingle() { try { var t = ac().currentTime + .05, seq = [[330, .09], [440, .09], [554, .09], [659, .09], [880, .18], [659, .09], [880, .3]], i; for (i = 0; i < seq.length; i++) { tone(seq[i][0], t, seq[i][1], "square", .07); tone(seq[i][0] / 2, t, seq[i][1], "triangle", .05); t += seq[i][1]; } for (i = 0; i < 24; i++) tone(1760 + Math.sin(i) * 300, t - .9 + i * .03, .04, "square", .02); } catch (e) {} }
  function blip(f) { try { var t = ac().currentTime; tone(f || 1200, t, .05, "square", .05); } catch (e) {} }
  function chord() { try { var t = ac().currentTime; [220, 277, 330, 440].forEach(function (f, i) { tone(f, t + i * .04, .7, "triangle", .06); }); } catch (e) {} }
  function click() { try { var t = ac().currentTime; tone(3000, t, .02, "square", .04); tone(180, t, .04, "triangle", .06); } catch (e) {} }

  /* ---------- state ---------- */
  var S = { mode: "splash", t0: performance.now(), play: true, key: false, lock: true, power: true, kpps: 30000, buffer: 3, speed: 1, size: 1, fog: .85, frame: 0, pos: 0, show: null, name: "", cam: "rear", lim: { r: 1, g: 1, b: 1 }, gam: { r: 1, g: 1, b: 1 }, dmx: 1, univ: 1, net: { ndi: false, spout: false, artnet: false, sacn: true }, page: 0, field: 0, edit: false, err: null, dmxIn: null, mem: null, tip: {}, dim: 1, encT: 0 };
  try { S.mem = JSON.parse(localStorage.getItem("sc-laser") || "null"); if (S.mem && S.mem.kpps) S.kpps = S.mem.kpps; } catch (e) {}
  var demo = ILDA.parse(ILDA.write(ILDA.demo()).buffer); S.show = demo.frames; S.name = "demo.ild · " + demo.frames.length + " frames";
  function fps() { var f = S.show[S.frame]; return f && f.length ? S.kpps / f.length : 0; }
  function armed() { return S.power && S.key; } function live() { return armed() && S.lock; }
  /// A device with no power answers the rocker only (SISTEMA.md §5). Without power the key does not
  /// arm, the interlock does not change and the encoder does not navigate — before, all of that obeyed
  /// with the display dark, and the key even sent `laser_open` to the engine with nothing lit on the
  /// rear panel to show for it.
  function dead() { if (S.power) return false; if (pino) pino.say("No power. The POWER rocker turns it on.", null, false); blip(300); return true; }
  function remember() { try { localStorage.setItem("sc-laser", JSON.stringify({ kpps: S.kpps, name: S.name, when: Date.now() })); } catch (e) {} }

  /* ---------- engine ---------- */
  // The device is one more client of the registry: the logic is in the engine, here only the command passes.
  // `LaserEngine.cmdFor` is the state → command map; `push` sends it and keeps the answer.
  var bus = new Bus({ offline: location.protocol === "file:" || /(\?|&)offline=1/.test(location.search) }).connect();
  var ENG = { on: false, rev: 0, show: "", feed: null, dac: "etherdream", host: "", file: "", stats: null, dacs: null, files: [], tr: null, log: "", port: location.host || "offline", ver: "0.1.2", err: false };
  function ctx() { return { feed: ENG.feed, dac: ENG.dac, host: ENG.host, kpps: S.kpps, file: ENG.file, fps: fps() || 30, show: ENG.show }; }
  /// An error is data, not a silent exception: it goes to the ERROR page of the display (which becomes
  /// the current page) and to the Pino's mouth. Nothing hangs, nothing vanishes.
  function fail(msg) { S.err = { msg: String(msg), when: new Date().toTimeString().slice(0, 8) }; ENG.err = true; S.page = ERRPAGE; S.edit = false; S.field = 0; blip(260); if (pino) pino.say(msg, null, false); drawOled(); refresh(); }
  /// Echoed CLI (SISTEMA.md §10): the amber HUD always shows the last command the part sent.
  /// An empty argument does not become `--host ` in the echo: what shows in the HUD is what can be typed.
  function cliOf(c) { return "spell " + c.cmd + Object.keys(c.args).map(function (k) { return c.args[k] === "" ? "" : " --" + k + " " + c.args[k]; }).join(""); }
  function push(id, v) { var c = LaserEngine.cmdFor(id, v, ctx()); if (!c) return null;
    $("#cli").textContent = cliOf(c);
    return bus.call(c.cmd, c.args).then(function (r) { ENG.rev = bus.rev;
      if (c.cmd === "laser_open" && r && r.feed != null) { ENG.feed = r.feed; ENG.err = false; push("lock", S.lock); }
      if (c.cmd === "laser_close") { ENG.feed = null; ENG.stats = null; ENG.err = false; }
      drawOled(); refresh(); return r;
    }, function (e) { if (c.cmd === "laser_open") ENG.feed = null; var m = String(e.message); fail(m.indexOf(c.cmd) === 0 ? m : c.cmd + ": " + m); }); }
  // `full` also brings `inputs`: that is where the HUD learns whether there is an Art-Net or sACN input,
  // and on which universe. Without it the input row would be a guess — and the HUD does not guess.
  function showName() { bus.call("show_get", { full: true }).then(function (r) { ENG.show = (r && r.name) || ""; ENG.rev = bus.rev; readInputs(r); drawOled(); refresh(); }, function () {}); }
  /// Firmware on the chassis (the owner: "engrave the firmware on the chassis itself"): the version
  /// comes from the engine, and from it alone. Without an engine the plate writes FIRMWARE OFFLINE — a
  /// device plate does not invent a number.
  function plate(fw) { if (B.plate) B.plate({ fw: fw || null }); }
  // ponytail: `version` is not a registry command yet ; when it is, the plate reads it from there
  // instead of the `ver` the page carries.
  function fwPlate() { bus.call("version", {}).then(function (r) { plate((r && (r.version || r.ver)) || ENG.ver); }, function () { plate(ENG.ver); }); }
  function hello() { ENG.on = true; showName(); fwPlate();
    bus.call("laser_files", {}).then(function (r) { ENG.files = (r && r.files) || []; refresh(); }, function (e) { fail("laser_files: " + e.message); });
    drawOled(); refresh(); }
  bus.on("open", hello);
  bus.on("show", showName); // another page renamed or edited the show: the display cannot lie
  bus.on("transport", function (d) { ENG.tr = d; drawOled(); }); // SHOW page: t, state and duration come from here
  bus.on("log", function (d) { ENG.log = (d && d.text) || ""; if (/error|fail/i.test(ENG.log)) fail(ENG.log); else drawOled(); });
  // topic 1 is the frame going OUT, topic 2 is the one coming IN (`show.inputs`): the input is what
  // lights the Art-Net / sACN LED in the HUD, and it only lights with a frame actually received.
  bus.on("dmx", function (d) { if (!d) return; if (d.topic === 2) INPUTS.forEach(function (i) { if (i.u === d.universe) i.last = performance.now(); });
    S.dmxIn = { universe: d.universe, value: d.data[Math.max(0, S.dmx - 1)] }; });
  bus.on("close", function () { ENG.on = false; ENG.feed = null; ENG.stats = null; ENG.tr = null; INPUTS = []; plate(null); drawOled(); refresh(); });
  // ponytail: `laser_stats` by 1 s polling ; it becomes an event when the engine publishes a laser topic
  setInterval(function () { if (ENG.feed == null) return; bus.call("laser_stats", { feed: ENG.feed }).then(function (r) { ENG.stats = r; drawOled(); }, function () {}); }, 1000);
  /// Plays an .ild that is on the engine disk: the engine sends it to the DAC, and the page reads the
  /// same file over the HTTP of `serve` to draw on the wall what is going out.
  function playFile(p, name) { ENG.file = p; S.name = name + " · engine";
    fetch("/" + p.replace(/\\/g, "/")).then(function (r) { return r.arrayBuffer(); }).then(function (ab) { var d = ILDA.parse(ab); if (d.frames.length) { S.show = d.frames; S.frame = 0; S.pos = 0; S.name = name + " · " + d.frames.length + " frames · engine"; } refresh(); }, function () {});
    push("play", true); }

  /* ---------- wall ----------
     The wall is a render target: the trail lives IN IT. Every frame a black quad comes in and fades what
     was already there (the `fillRect` of before, only by time: `1 - FADE^(dt*60)`, so the trail lasts the
     same at 20 or at 60 fps) and on top of it the segments of the frame, in a `LineSegments` with a
     pre-allocated buffer. The stroke goes in source-over, like the `stroke` of the canvas: stroke over
     stroke replaces the colour, it does not add — additive saturated to white where the figure passes
     dozens of times. Only the halo and the galvo dot add. Before it was a 1024×640 2D canvas with a
     `stroke` of `shadowBlur = 18` PER POINT — at 30 kpps that is 500 blurred strokes per frame on the CPU,
     plus uploading the whole texture to the GPU every frame: with a 3000-point-per-frame .ild Chrome sat
     at 1.8 fps.
     ponytail: the splash stays on the 2D canvas (`fillDone` is `fill("evenodd")` over letter polygons, and
     triangulating a font does not fit here); it runs once, for 8 s, with few strokes. On leaving the splash
     the `map` of the plane switches from the canvas to the render target, which is born clean. */
  var WC = document.createElement("canvas"); WC.width = 1024; WC.height = 640; var wx = WC.getContext("2d"), WW = 1024, WH = 640, galvo = null, lit = [], gpos = [0, 0];
  // ponytail: gate measurement in one line, only with ?perf=1 ; it goes when there is a real perf HUD
  var PERF = /(\?|&)perf=1/.test(location.search) ? (window.__perf = { frames: 0, wallMs: 0 }) : null;
  // 4096 segments: the frame draws `min(points of the frame, kpps × dt)` and dt is clamped at .1 s, that
  // is at most 4000 points at 40 kpps. Past that, the rest of the frame is dropped (no realloc).
  var MAXSEG = 4096, segN = 0, dotN = 0;
  var wrt = new THREE.WebGLRenderTarget(WW, WH, { minFilter: THREE.LinearFilter, magFilter: THREE.LinearFilter, depthBuffer: false, stencilBuffer: false });
  wrt.texture.encoding = THREE.sRGBEncoding; wrt.texture.generateMipmaps = false;
  var wscene = new THREE.Scene(), wcam = new THREE.OrthographicCamera(0, WW, 0, WH, -1, 1); // canvas coordinates: y grows downwards
  /* The alpha of the render target is pinned at 1: the `proj` plane is additive and multiplies the colour
     by the alpha of the texture, so an alpha that fades along with the trail would erase the wall twice. */
  function wmat(m, add) { m.transparent = true; m.depthTest = m.depthWrite = false; m.toneMapped = false; m.blending = THREE.CustomBlending;
    m.blendSrc = THREE.SrcAlphaFactor; m.blendDst = add ? THREE.OneFactor : THREE.OneMinusSrcAlphaFactor;
    m.blendSrcAlpha = THREE.ZeroFactor; m.blendDstAlpha = THREE.OneFactor; return m; }
  function attr(a, n) { var b = new THREE.BufferAttribute(a, n); b.setUsage(THREE.DynamicDrawUsage); return b; }
  var segPos = new Float32Array(MAXSEG * 6), segCol = new Float32Array(MAXSEG * 6), haloPos = new Float32Array(MAXSEG * 3), haloCol = new Float32Array(MAXSEG * 3), dotPos = new Float32Array(3);
  var segGeo = new THREE.BufferGeometry(); segGeo.setAttribute("position", attr(segPos, 3)); segGeo.setAttribute("color", attr(segCol, 3));
  var haloGeo = new THREE.BufferGeometry(); haloGeo.setAttribute("position", attr(haloPos, 3)); haloGeo.setAttribute("color", attr(haloCol, 3));
  var dotGeo = new THREE.BufferGeometry(); dotGeo.setAttribute("position", attr(dotPos, 3));
  // the halo of the stroke: `shadowBlur` became one gradient sprite per point, which is what the GPU does for free
  var GC = document.createElement("canvas"); GC.width = GC.height = 64; var gc = GC.getContext("2d"), gg = gc.createRadialGradient(32, 32, 0, 32, 32, 32);
  gg.addColorStop(0, "rgba(255,255,255,1)"); gg.addColorStop(.3, "rgba(255,255,255,.45)"); gg.addColorStop(1, "rgba(255,255,255,0)"); gc.fillStyle = gg; gc.fillRect(0, 0, 64, 64);
  var glowTex = new THREE.CanvasTexture(GC);
  var segMat = wmat(new THREE.LineBasicMaterial({ vertexColors: true, opacity: 1 }), false);
  var haloMat = wmat(new THREE.PointsMaterial({ size: 9, sizeAttenuation: false, map: glowTex, vertexColors: true, opacity: .12 }), true);
  var dotMat = wmat(new THREE.PointsMaterial({ size: 14, sizeAttenuation: false, map: glowTex, opacity: .9 }), true);
  var fadeMat = wmat(new THREE.MeshBasicMaterial({ color: 0x000000, opacity: .3 }), false), FADE = +VIDEO.get("trail");   // TRAIL PERSISTENCE, VIDEO menu
  var fade = new THREE.Mesh(new THREE.PlaneGeometry(WW, WH), fadeMat); fade.position.set(WW / 2, WH / 2, 0);
  var segs = new THREE.LineSegments(segGeo, segMat), halo = new THREE.Points(haloGeo, haloMat), dot = new THREE.Points(dotGeo, dotMat);
  [fade, segs, halo, dot].forEach(function (o, i) { o.frustumCulled = false; o.renderOrder = i; wscene.add(o); });
  /* Limit and curve become a 256-entry table: the frame was doing 1500 `Math.pow` (three per point).
     The table keeps both values of the same colour — the byte, which is the contract of `lit` (the beams
     of the scene read `L[1][k] / 255`), and the LINEAR value the material colour needs, because the render
     target is sRGB: going in linear, the same byte the canvas used to write comes out stored, and the sum
     of the strokes happens in sRGB as before. */
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
  // splash: the galvo outlines SPELLCASTER LASER, loop by loop; each closed letter is revealed; at the end everything glows
  var SP = { loops: [], len: 0, pos: 0, li: 0, pi: 0, done: [], flew: false, last: null };
  // the galvo dot in the splash, which stays on the 2D canvas like the rest of it
  function spDot(x, y, c, fill, blur, r) { wx.globalCompositeOperation = "lighter"; wx.shadowBlur = blur; wx.shadowColor = c; wx.fillStyle = fill; wx.beginPath(); wx.arc(x, y, r, 0, 7); wx.fill(); wx.shadowBlur = 0; wx.globalCompositeOperation = "source-over"; }
  function fillDone(alpha, glow) { if (!SP.done.length) return; wx.save(); wx.globalCompositeOperation = "lighter"; wx.beginPath(); SP.done.forEach(function (l) { wx.moveTo(l[0][0], l[0][1]); for (var i = 1; i < l.length; i++) wx.lineTo(l[i][0], l[i][1]); wx.closePath(); }); wx.shadowBlur = glow || 0; wx.shadowColor = "#38FF5C"; wx.fillStyle = "rgba(56,255,92," + alpha + ")"; wx.fill("evenodd"); wx.restore(); }
  function wallSplash(now, dt) { var t = (now - S.t0) / 1000, T = SP;
    if (t < .8) { wx.fillStyle = "rgba(0,0,0,.35)"; wx.fillRect(0, 0, WW, WH); parked(now); spDot(WW * .5, WH * .5, "#FF2A1A", "rgba(255,42,26," + (.6 + .4 * Math.sin(now / 90)) + ")", 34, 4); return; }
    if (t < 4.8 && T.loops.length) { if (!T.cleared) { T.cleared = true; wx.globalCompositeOperation = "source-over"; wx.fillStyle = "#000"; wx.fillRect(0, 0, WW, WH); } var target = Math.min(T.len, (t - .8) / 4 * T.len), pt = null; wx.globalCompositeOperation = "lighter"; wx.lineCap = "round"; wx.lineJoin = "round"; wx.lineWidth = 3; wx.strokeStyle = "rgba(56,255,92,.9)"; wx.shadowBlur = 14; wx.shadowColor = "#38FF5C";
      while (T.pos < target && T.li < T.loops.length) { var L = T.loops[T.li], a = L[T.pi], b = L[T.pi + 1]; if (!b) { T.done.push(L); T.li++; T.pi = 0; T.last = null; continue; } var sl = Math.hypot(b[0] - a[0], b[1] - a[1]), room = target - T.pos, k = Math.min(1, room / Math.max(1e-6, sl)); pt = [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k]; var from = T.last || a; wx.beginPath(); wx.moveTo(from[0], from[1]); wx.lineTo(pt[0], pt[1]); wx.stroke(); if (k >= 1) { T.pos += sl; T.pi++; T.last = null; } else { T.pos = target; T.last = pt; } }
      wx.shadowBlur = 0; wx.globalCompositeOperation = "source-over"; if (T.done.length > (T.filled || 0)) { fillDone(.14); T.filled = T.done.length; }
      if (pt) { spDot(pt[0], pt[1], "#fff", "rgba(255,255,255,.9)", 26, 3); lit = [[pt, [120, 255, 150]]]; gpos[0] = (pt[0] / WW - .5) * 2; gpos[1] = (.5 - pt[1] / WH) * 2; } return; }
    if (t < 5.8) { if (T.li < T.loops.length) { T.done = T.loops.slice(); T.li = T.loops.length; } var k = (t - 4.8); fillDone(.05 + .25 * Math.sin(Math.min(1, k) * Math.PI) * (1 + Math.sin(k * 40) * .3), 40 * Math.sin(Math.min(1, k) * Math.PI)); lit = []; return; }
    // the laser goes dark, the wall cools down, the room light drops, the camera flies to the rear
    wx.fillStyle = "rgba(0,0,0,.08)"; wx.fillRect(0, 0, WW, WH); lit = []; S.dim = .55; if (!T.flew) { T.flew = true; setCam("rear", 2.2); } if (t > 7.6) start(); }

  /* ---------- scene ---------- */
  var stage = $("#stage"), gl = $("#gl"), R = new THREE.WebGLRenderer({ canvas: gl, antialias: false, powerPreference: "high-performance" }), scene = new THREE.Scene(), cam = new THREE.PerspectiveCamera(42, 1, .02, 60), pick = [];
  R.setPixelRatio(Math.min(2, devicePixelRatio)); scene.background = new THREE.Color(0x020306); scene.fog = new THREE.FogExp2(0x03040a, .12);
  R.info.autoReset = false;   // the frame has several render() calls: the one that resets the counter is `tick`
  var X = MAT(THREE, R), ENVTEX = X.env(); scene.environment = ENVTEX;   // kept: ENVIRONMENT REFLECTIONS turns on and off without regenerating the PMREM
  var sun = new THREE.SpotLight(0xfff4e6, 90, 9, .42, .7, 1.4); sun.position.set(1.4, 3.2, 1.2); sun.target.position.set(0, .35, 0); sun.castShadow = true; sun.shadow.mapSize.set(2048, 2048); sun.shadow.bias = -.0004; sun.shadow.radius = 4; scene.add(sun, sun.target);
  var fill = new THREE.PointLight(0x38ff5c, 4, 4, 2); fill.position.set(-.9, .9, .5); scene.add(fill); var rim = new THREE.PointLight(0xa0c0ff, 8, 5, 2); rim.position.set(.4, 1.2, -1.6); scene.add(rim);
  var inLight = new THREE.PointLight(0xfff0dc, 0, 1.2, 2); inLight.position.set(.05, .62, 0); scene.add(inLight);
  var B = BODY(THREE, X, scene, pick), O = OPTICS(THREE, X, B.body, pick);
  var wallTex = new THREE.CanvasTexture(WC); wallTex.minFilter = THREE.LinearFilter; wallTex.encoding = THREE.sRGBEncoding; var proj = new THREE.Mesh(new THREE.PlaneGeometry(8, 5), new THREE.MeshBasicMaterial({ map: wallTex, transparent: true, blending: THREE.AdditiveBlending, depthWrite: false })); proj.position.set(0, 2.2, -5); scene.add(proj);
  var beamsOut = BEAM(THREE, scene, 320, .006, .028), beamsIn = BEAM(THREE, scene, 12, .0012, .0045);   // 320 = ceiling of the EXTERNAL BEAMS row
  var FC = document.createElement("canvas"); FC.width = FC.height = 128; var fc = FC.getContext("2d"), fg = fc.createRadialGradient(64, 64, 0, 64, 64, 64); fg.addColorStop(0, "rgba(120,140,170,1)"); fg.addColorStop(1, "rgba(120,140,170,0)"); fc.fillStyle = fg; fc.fillRect(0, 0, 128, 128);
  var fogTex = new THREE.CanvasTexture(FC), puffs = []; for (var i = 0; i < 28; i++) { var sp = new THREE.Sprite(new THREE.SpriteMaterial({ map: fogTex, transparent: true, opacity: .05, blending: THREE.AdditiveBlending, depthWrite: false })); sp.position.set((Math.random() - .5) * 5, .4 + Math.random() * 2.4, -.5 - Math.random() * 4.5); var sc = 1.5 + Math.random() * 2.5; sp.scale.set(sc, sc, 1); sp.userData.v = [(Math.random() - .5) * .08, (Math.random() - .5) * .03]; scene.add(sp); puffs.push(sp); }
  var composer = new THREE.EffectComposer(R); composer.renderTarget1.texture.encoding = composer.renderTarget2.texture.encoding = THREE.sRGBEncoding; composer.addPass(new THREE.RenderPass(scene, cam)); var bloom = new THREE.UnrealBloomPass(new THREE.Vector2(1024, 760), .5, .45, .82); composer.addPass(bloom); var fxaa = new THREE.ShaderPass(THREE.FXAAShader); composer.addPass(fxaa);

  /* ---------- camera ---------- */
  var ray = new THREE.Raycaster(), mv = new THREE.Vector2(-2, -2), mouse = null, hot = null, tip = $("#tip"), pressed = false;
  function ptr(e) { var r = gl.getBoundingClientRect(); mv.set(((e.clientX - r.left) / r.width) * 2 - 1, -((e.clientY - r.top) / r.height) * 2 + 1); mouse = [e.clientX - r.left, e.clientY - r.top]; return mouse; }
  /* The box is opaque: the ray stops at the first solid surface it meets. If that surface is not a
     clickable part (sheet metal, silkscreen, display glass), there is no click there — before, the raycast
     only saw the `pick` list and the optical bench, the PSU and the DAC board were clickable THROUGH the
     closed rear panel, with a hand cursor over plain sheet metal. Haze, beam and the wall projection do not
     count as solid (they do not write depth), so they hide nothing. */
  function solid(o) { return o.visible && !o.isSprite && !o.isInstancedMesh && o.material && !o.material.transparent && o.material.depthWrite !== false; }
  function hitOf(e) { ptr(e); ray.setFromCamera(mv, cam); var hits = ray.intersectObjects(scene.children, true);
    for (var i = 0; i < hits.length; i++) { var o = hits[i].object; if (!solid(o)) continue;
      var h = o; while (h && !(h.userData && h.userData.key)) h = h.parent;
      return h ? { o: h, p: hits[i].point } : null; }
    return null; }
  function unhover() { glow(hot, false); hot = null; tip.style.display = "none"; gl.style.cursor = "grab"; }
  var CAM = SWCam(THREE, cam, gl, { hit: function (e) { return !!hitOf(e); }, pick: function (e) { ptr(e); ray.setFromCamera(mv, cam); var hs = ray.intersectObjects(scene.children, true).filter(function (h) { return h.object.visible && !h.object.isSprite && h.object.type !== "InstancedMesh"; }); return hs.length ? hs[0].point : null; } });
  var CENTER = new THREE.Vector3(0, .40, 0);
  var VIEWS = { show: [[1.5, .95, 1.25], [0, .8, -1.9]], rear: [[.015, .445, .60], [0, .40, .04]], inside: [[.09, .80, .27], [-.02, .38, -.03]], wall0: [[.25, .8, .35], [0, 1.5, -2.6]], wall: [[0, 2.0, .2], [0, 2.2, -5]] }, camSpeed = 5;
  /* A view is not just a pose: it is the law of the mouse once you get there (FUNCOES/camera-solidworks.md
     §3). SHOW is the visualizer and gets the whole SolidWorks; REAR is the menu and that is why it is a
     FIXED pose relative to the rear panel (no dragging, no wheel zoom); INSIDE is an orbit locked to the
     optical bench, with the distance frozen. The fixed poses are redone when the window changes size. */
  var LAWS = { show: { mode: "free", center: CENTER, R: .34 },
    rear: { mode: "rear", center: new THREE.Vector3(0, .314 + B.H / 2, B.D / 2), normal: new THREE.Vector3(0, 0, 1), w: B.W, h: B.H, pad: 1.55 },
    inside: { mode: "inside", center: new THREE.Vector3(-.02, .348, -.03), yaw0: .35, pit0: .92, w: .40, h: .26, pad: 1.6 },
    wall0: { mode: "free", center: CENTER, R: .34 }, wall: { mode: "free", center: CENTER, R: .34 } };
  // the camera moves under the cursor: the label of the part that was beneath it keeps lying on the
  // screen (and the part stays lit) until the next mouse move. Changing view clears both.
  function setCam(k, speed) { S.cam = k; unhover(); CAM.mode(LAWS[k].mode, LAWS[k]); CAM.setView(VIEWS[k][0], VIEWS[k][1]); camSpeed = speed || 5; document.querySelectorAll("#cams [data-a]").forEach(function (b) { b.classList.toggle("on", b.dataset.a === "cam." + k); }); if (S.mode === "play") blip(k === "inside" ? 700 : 1000); }
  window.SC = window.SC || {}; SC.camMode = function () { return CAM.camMode(); };   // contract 4: hud-4 only reads

  /* ---------- panel: the drawer on the right ----------
     Every menu and every setting lives here (the owner asked for it), in a 360 px drawer with tabs, in
     place of the floating panel that covered the device. Clicking a part opens the tab of that part: it is
     the same navigation as before, in one place only. The pull handle on the edge opens and closes, `Tab`
     does too, `Esc` closes. Inside the drawer `Tab` goes back to being the keyboard's (focus between the
     controls). */
  var drawer = $("#drawer"), dbody = $("#dbody"), dtabs = $("#dtabs"), dcur = "laser";
  var TABN = { laser: "LASER", dmx: "DMX", net: "NET", interlock: "INTERLOCK", bind: "BINDINGS", video: "VIDEO", info: "INFO" };
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
  // inside the drawer Tab belongs to the keyboard, not to the device: without this the global binding
  // ate the Tab and there was no way to walk from control to control with the keyboard (SISTEMA.md §11).
  drawer.addEventListener("keydown", function (e) { if (e.key === "Tab") e.stopPropagation(); });
  function get(p) { var a = p.split("."), o = S; for (var i = 0; i < a.length; i++) o = o[a[i]]; return o; } function set(p, v) { var a = p.split("."), o = S; for (var i = 0; i < a.length - 1; i++) o = o[a[i]]; o[a[a.length - 1]] = v; }
  var FMT = { kpps: function (v) { return Math.round(v / 1000) + "k"; }, pct: function (v) { return Math.round(v * 100) + "%"; }, gam: function (v) { return "γ " + (+v).toFixed(2); }, n: function (v) { return v; }, x: function (v) { return "×" + (+v).toFixed(2); } }, fmtOf = {};
  function rg(label, path, min, max, step, f) { fmtOf[path] = FMT[f]; return "<div class='row'><span>" + label + "</span><input type='range' data-p='" + path + "' min='" + min + "' max='" + max + "' step='" + step + "' value='" + get(path) + "'><span class='v' data-v='" + path + "'>" + FMT[f](get(path)) + "</span></div>"; }
  /// A text field (DAC host, interlock universe/channel, MQTT topic) does not redraw the drawer while
  /// it is being typed into: the field would vanish from under the cursor.
  drawer.addEventListener("input", function (e) { var r = e.target;
    /// Fader of the VIDEO menu: applies right away and rewrites ONLY the number beside it. Repainting
    /// the drawer at every pixel of the drag would throw the scroll to the top and kill the fader under
    /// the cursor.
    if (r.dataset.vr) { if (VIDEO.set(r.dataset.vr, +r.value)) { var vv = dbody.querySelector("[data-vv='" + r.dataset.vr + "']"); if (vv) vv.textContent = vfmt(VIDEO.opt(r.dataset.vr), VIDEO.get(r.dataset.vr)); vpreset(); } return; }
    if (r.dataset.host !== undefined) { ENG.host = r.value.trim(); return; }
    if (r.dataset.ilk) { ILK[r.dataset.ilk] = r.dataset.ilk === "topic" ? r.value : Math.max(1, Math.min(512, +r.value || 1)); ilkSave(); return; }
    if (!r.dataset.p) return; set(r.dataset.p, +r.value); dbody.querySelector("[data-v='" + r.dataset.p + "']").textContent = fmtOf[r.dataset.p](+r.value); remember(); if (/^(lim|gam)\./.test(r.dataset.p)) { var old = dbody.querySelector(".cv"); if (old) old.replaceWith(curveCanvas()); } push(r.dataset.p, get(r.dataset.p)); Bind.syncAll(); });
  /// The same value shows up in three places in the panel (fader, number and the `spell` line of the
  /// `<pre>`), and the `<pre>` used to be frozen at the value from when the panel opened — LIMIT 25% with
  /// `spell ilda limit --r 1.00` underneath. On RELEASING the fader the panel redraws whole (not during
  /// the drag, which would destroy the fader in your hand) and the focus goes back to the same fader.
  drawer.addEventListener("change", function (e) {
    if (e.target.dataset.vs) { VIDEO.set(e.target.dataset.vs, e.target.value); vpreset(); return; }
    if (e.target.dataset.vpreset !== undefined) { VIDEO.preset(e.target.value); paint(); return; }
    var p = e.target.dataset.p; if (!p) return; refresh(); var el = dbody.querySelector("[data-p='" + p + "']"); if (el) el.focus(); });
  drawer.addEventListener("click", function (e) { if (Bind.click(e)) { paint(); return; }
    var f = e.target.closest("[data-f]"); if (f) { playFile(f.dataset.f, f.dataset.n); refresh(); return; }
    var d = e.target.closest("[data-d]"); if (d) { var p = d.dataset.d.split("|"); ENG.dac = p[0]; ENG.host = p[1]; refresh(); return; }
    var b = e.target.closest("[data-a]"); if (b) Bind.run(b.dataset.a); });
  function curveCanvas() { var c = document.createElement("canvas"); c.width = 268; c.height = 90; c.className = "cv"; var x = c.getContext("2d"); x.fillStyle = "#050708"; x.fillRect(0, 0, 268, 90); ["r", "g", "b"].forEach(function (k) { x.strokeStyle = { r: "#FF2A1A", g: "#38FF5C", b: "#3A6BFF" }[k]; x.lineWidth = 1.5; x.beginPath(); for (var i = 0; i <= 40; i++) { var t = i / 40, y = S.lim[k] * Math.pow(t, S.gam[k]); x.lineTo(4 + t * 260, 86 - y * 82); } x.stroke(); }); return c; }
  function onoff(a, on, y, n) { return "<button class='lb" + (on ? " on" : "") + "' data-a='" + a + "'>" + (on ? y : n) + "</button>"; }
  /// NET state button: the label always says ON/OFF (state is not only colour — SISTEMA.md §11) and both
  /// labels are almost the same width, otherwise the button next to it escapes from under the cursor.
  function netBtn(k) { var K = k.toUpperCase(); return onoff("net." + k, S.net[k], K + " · ON", K + " · OFF"); }
  /// The .ild files on the engine disk (`laser_files`). The `<input type=file>` and the drag stay: that is
  /// the file that came in your hand, this is what is already on the show machine.
  function fileList() { if (!ENG.on) return "<div class='sub'>engine offline · drop an .ild on the ILDA IN port</div>";
    if (!ENG.files.length) return "<div class='sub'>no .ild in shows/ on the engine disk</div>";
    return "<div class='fl'>" + ENG.files.map(function (f) { return "<button class='lb" + (ENG.file === f.path ? " on" : "") + "' data-f=\"" + f.path.replace(/"/g, "&quot;") + "\" data-n=\"" + f.name.replace(/"/g, "&quot;") + "\">" + f.name + " <small>" + Math.round(f.bytes / 1024) + " kB</small></button>"; }).join("") + "</div>"; }
  /// The key switch arms the device, and arming is opening a DAC: with no DAC chosen there is nothing to arm.
  function dacList() { if (!ENG.on) return "<pre>engine offline · the key arms the model only</pre>";
    return "<pre>DAC <b>" + ENG.dac + " " + (ENG.host || "(no host)") + "</b>" + (ENG.feed != null ? " · feed " + ENG.feed : "") + "\nspell laser_open --dac " + ENG.dac + " --host " + (ENG.host || "&lt;ip&gt;") + " --kpps " + Math.round(S.kpps / 1000) + "</pre>"
      + "<div class='map'><span>HOST</span><input class='tx' data-host value=\"" + ENG.host.replace(/"/g, "&quot;") + "\" placeholder=\"169.254.207.140\"></div>"
      + "<div class='btns'><button class='lb" + (ENG.scan ? " on" : "") + "' data-a='dacs'>" + (ENG.scan ? "SEARCHING…" : "FIND DACS") + "</button>"
      + (ENG.dacs || []).map(function (d) { return "<button class='lb" + (ENG.host === d.host ? " on" : "") + "' data-d=\"" + d.type + "|" + d.host + "\">" + d.type.toUpperCase() + " " + d.host + "</button>"; }).join("") + "</div>"; }
  function h3(t, s) { return "<h3>" + t + "</h3><div class='sub'>" + s + "</div>"; }
  /// The interlock is not an on-off button: it is an INPUT of the device. What drives it is the source
  /// mapped here, and the identity of that source is a textual address (FUNCOES/README rule 2), the same one
  /// in key, MIDI, OSC, Art-Net, MQTT and CLI. Only what already exists in the engine is really wired (key
  /// and MIDI, through `Bind`); the rest shows the address and the CLI line, marked "soon".
  var ILK_ADDR = "laser/1/interlock", ILK = { univ: 1, chan: 512, topic: "spell/laser/1/interlock" };
  try { var im = JSON.parse(localStorage.getItem("sc-laser-ilk") || "null"); if (im) { ILK.univ = im.univ || ILK.univ; ILK.chan = im.chan || ILK.chan; ILK.topic = im.topic || ILK.topic; } } catch (e) {}
  function ilkSave() { try { localStorage.setItem("sc-laser-ilk", JSON.stringify(ILK)); } catch (e) {} }
  function learnBtn(src, ready) { var l = Bind.learnState(), on = l && l.id === "lock.toggle" && l.src === src;
    return "<button class='lb" + (on ? " learn" : "") + "' data-learn='" + src + "' data-id='lock.toggle'>" + (on ? ready : (Bind.keyOf("lock.toggle", src) || "MAP")) + "</button>"; }
  /* ---------- VIDEO tab ----------
     The video menu of the visualizer in game-menu shape: PRESET at the top, blocks of `label … value` rows,
     RESTORE DEFAULTS at the bottom, and a read-only block with what the GPU is doing NOW. The whole list
     comes out of `VIDEO.OPTS`: whoever adds an option touches the table in `video.js`, not this file.
     Everything applies right away — there is no APPLY button because nothing needs one (even MSAA swaps the
     composer target at run time, without recreating the renderer). */
  function vfmt(o, v) { return o.type === "range" ? (o.step >= 1 ? (+v).toFixed(0) : (+v).toFixed(2)) + o.unit : ""; }
  function vrow(o) { var v = VIDEO.get(o.id), h;
    if (o.type === "select") h = "<div class='vrow sel'><label for='v_" + o.id + "'>" + o.label + "</label><select id='v_" + o.id + "' data-vs='" + o.id + "'>"
      + o.vals.map(function (x) { return "<option value=\"" + x[0] + "\"" + (String(v) === x[0] ? " selected" : "") + ">" + x[1] + "</option>"; }).join("") + "</select></div>";
    else h = "<div class='vrow'><label for='v_" + o.id + "'>" + o.label + "</label><input id='v_" + o.id + "' type='range' data-vr='" + o.id + "' min='" + o.min + "' max='" + o.max + "' step='" + o.step + "' value='" + v + "'><span class='v' data-vv='" + o.id + "'>" + vfmt(o, v) + "</span></div>";
    return h + (o.note ? "<div class='vnote'>" + o.note + "</div>" : ""); }
  /// Touching any row turns it into CUSTOM — and the name is derived from the values, so putting the value
  /// back by hand returns to the preset name on its own.
  function vpreset() { var s = dbody.querySelector("[data-vpreset]"); if (!s) return; var pn = VIDEO.presetName();
    if (pn === "custom" && !s.querySelector("option[value='custom']")) { var op = document.createElement("option"); op.value = "custom"; op.textContent = VIDEO.NAMES.custom; s.appendChild(op); }
    s.value = pn; }
  function videoPane() { var pn = VIDEO.presetName();
    var h = h3("VIDEO", "the cost of the frame is the operator's choice · every row applies right away")
      + "<div class='vpre'><div class='vrow sel'><label for='v_preset'>PRESET</label><select id='v_preset' data-vpreset>"
      + VIDEO.PRESETS.concat(pn === "custom" ? ["custom"] : []).map(function (k) { return "<option value='" + k + "'" + (k === pn ? " selected" : "") + ">" + VIDEO.NAMES[k] + "</option>"; }).join("")
      + "</select></div></div>";
    VIDEO.GROUPS.forEach(function (g) { h += h3(g, VIDEO.SUB[g]);
      VIDEO.OPTS.forEach(function (o) { if (o.g === g) h += vrow(o); });
      if (g === "SCREEN") h += "<div class='btns'><button class='lb' data-a='video.fullscreen'>FULLSCREEN</button></div>"; });
    return h + h3("PERFORMANCE", "measured, not estimated: it comes from R.info and the frame clock")
      + "<pre id='vperf'>" + perfText() + "</pre>"
      + "<div class='btns'><button class='lb amb' data-a='video.defaults'>RESTORE DEFAULTS</button></div>"; }
  var PANE = {
    video: videoPane,
    /// The live counter belongs to the HUD (top corner: frame, points, fps). Here that same count used to
    /// sit frozen at the instant the panel opened — a still number next to a moving number is worse than no
    /// number. The drawer says what does not change: the file, the limits, the curve.
    laser: function () { return h3("ILDA IN", "DB25 · " + S.name)
      + "<pre>file <b>" + S.show.length + " frames</b> · frame, points and fps live in the top corner\nspell ilda play show.ild --kpps " + Math.round(S.kpps / 1000) + "</pre>"
      + "<div class='btns'><button class='lb' data-a='file.open'>CHOOSE .ILD</button><button class='lb' data-a='demo'>DEMO</button>" + onoff("play.toggle", S.play, "PAUSE", "PLAY") + "</div>"
      + rg("SIZE", "size", .3, 1.3, .01, "pct") + fileList()
      + h3("X/Y GALVOS", "30 kpps nominal · 40 max") + rg("KPPS", "kpps", 5000, 40000, 500, "kpps") + rg("BUFFER", "buffer", 1, 8, 1, "n") + rg("SPEED", "speed", .4, 1.6, .01, "x")
      + "<pre>frame rate = kpps ÷ points · &lt; 25 fps flickers\n&gt; 32 kpps the galvo cannot keep up: corners become curves</pre>"
      + h3("R/G/B MODULES", "638 / 520 / 445 nm · limit and curve of each diode")
      + ["r", "g", "b"].map(function (k) { return rg({ r: "RED", g: "GREEN", b: "BLUE" }[k], "lim." + k, 0, 1, .01, "pct") + rg("CURVE " + k.toUpperCase(), "gam." + k, .5, 2.5, .01, "gam"); }).join("")
      + "<span data-cv></span><pre>spell ilda limit --r " + S.lim.r.toFixed(2) + " --g " + S.lim.g.toFixed(2) + " --b " + S.lim.b.toFixed(2) + "</pre>"
      + h3("SHUTTER", "solenoid · closes with no key or no interlock") + "<pre>state: <b>" + (live() ? "OPEN" : "CLOSED") + "</b> · closing time 8 ms</pre>"
      + h3("ILDA OUT", "male DB25 · passes the signal to the next projector") + "<pre>chained: <b>none</b>\nspell ilda play show.ild --chain 2</pre>"; },
    dmx: function () { return h3("DMX IN / OUT", "XLR-3 · address, universe and mode")
      + rg("ADDRESS", "dmx", 1, 512, 1, "n") + rg("UNIVERSE", "univ", 1, 512, 1, "n")
      + "<pre><b>16 channel</b> mode: shutter · pattern · size · rotation · X · Y · colour · kpps...\nbus: " + (S.dmxIn ? "U" + S.dmxIn.universe + " ch" + S.dmx + " = " + S.dmxIn.value : "no frame received") + "\nspell dmx addr " + S.dmx + "</pre>"
      + "<pre>the DMX OUT repeats the universe to the next device · the Pino is plugged into it</pre>"; },
    /// The DAC lives on the NET port: the EtherDream is a network device. The host is editable — the beacon
    /// may not arrive (Sitter open), and typing the IP has to be possible without a scan.
    net: function () { return h3("NET", "RJ45 · network sources and the DAC")
      + "<div class='btns'>" + ["ndi", "spout", "artnet", "sacn"].map(netBtn).join("") + "</div>"
      + "<pre>NDI → ILDA and Spout → ILDA are the <b>FÓSFORO</b> converter, which does not exist yet.\nspell ilda net --ndi \"RESOLUME (out)\"</pre>" + dacList(); },
    interlock: function () { return h3("INTERLOCK", "an input of the device · address <b>" + ILK_ADDR + "</b>")
      + "<pre>state now: <b>" + (S.lock ? "PLUG IN · shutter released" : "PLUG OUT · SCAN FAIL") + "</b>\nwhat changes this state is the source mapped below</pre>"
      + "<div class='sub'>The interlock is an input. Map it to:</div>"
      + "<div class='map'><span>KEY</span>" + learnBtn("key", "PRESS THE KEY…")
      + "<span>MIDI</span>" + learnBtn("midi", "MOVE THE CONTROLLER…")
      + "<span>OSC</span><code>" + ILK_ADDR + "</code>"
      + "<span>ART-NET</span><span>univ <input class='tx n' data-ilk='univ' value='" + ILK.univ + "'> channel <input class='tx n' data-ilk='chan' value='" + ILK.chan + "'> · ≥ 128 = closed</span>"
      + "<span>MQTT</span><input class='tx' data-ilk='topic' value=\"" + ILK.topic.replace(/"/g, "&quot;") + "\">"
      + "<span>CLI</span><code>spell laser_param --path shutter --value 1</code></div>"
      + "<pre>MIDI already maps in the engine: <b>spell midi_map --key 176/1 --cmd laser_param</b>\nOSC, Art-Net and MQTT still have no map command:\n<b class='soon'>spell map osc " + ILK_ADDR + "   (soon)</b></pre>"
      + "<div class='btns'>" + onoff("lock.toggle", S.lock, "SIMULATE OPENING", "PUT THE PLUG BACK") + "</div>"; },
    bind: function () { return h3("BINDINGS", "key and MIDI · click, then press the key or move the controller · Esc cancels")
      + "<div class='btns'><button class='lb' data-a='midi.connect'>MIDI: " + Bind.midi + "</button>" + onoff("cam.reverse", CAM.reverse, "WHEEL: SOLIDWORKS", "WHEEL: NORMAL") + "<button class='lb amb' data-a='bind.reset'>RESET</button></div>" + Bind.html(); },
    info: function () { return h3("SPELLCASTER LASER", "info · N closes")
      + "<pre>the program is the device: ports at the back = menu, lid open = preferences\nwall = kpps ÷ points, low-pass galvo; beam in GLSL; SolidWorks camera; key + MIDI bindings\nthe five pins of the Pino are the screens of the program\n\nengine: <b>" + (ENG.on ? "on · rev " + ENG.rev + (ENG.show ? " · " + ENG.show : "") : "offline (the page runs local)") + "</b>" + (ENG.feed != null ? " · feed " + ENG.feed + " on " + ENG.dac : "") + "\nCLI: spell laser_open --dac etherdream --host &lt;ip&gt; · spell laser_play --file show.ild\n\nGREETZ: PANGOLIN · ETHER DREAM · LSX · KVANT · CHATAIGNE · MADMAPPER\nCRACKED BY FEITIÇARIA iNDUSTRIAL · NO SERIAL NEEDED</pre>"; } };
  /* The clicked part opens its own tab: it is the usual navigation, now in one place only. A part that is
     not here is an inert part (`LaserEngine.inert`) and clicking it does nothing. */
  var TAB_OF = { ilda: "laser", ildathru: "laser", shutter: "laser", galvo: "laser", galvodrv: "laser", pcb: "laser", r: "laser", g: "laser", b: "laser",
    dmxin: "dmx", dmxout: "dmx", rj45: "net", interlock: "interlock", bind: "bind", nfo: "info" };
  var PANELS = {}; Object.keys(TAB_OF).forEach(function (k) { PANELS[k] = function () { openDrawer(TAB_OF[k]); }; });

  /* ---------- panel display ----------
     The display is the instrument of the device: 200 × 100 mm in the middle of the rear panel, seven pages,
     and the big line of each one is the answer the operator needs to read from across the room. The text
     comes whole from `LaserEngine.oledLines(S, ENG)` (a pure function, tested without a browser); here it is
     only drawn. One colour only, `--oled`: an alarm is an inverted block, never a new colour (SISTEMA.md §5).
     The encoder is an encoder: outside a field it turns pages, inside a field it turns values, pressing
     enters and moves to the next field, BACK leaves. One family of function, one mechanism. */
  var PAGES = LaserEngine.PAGES, ERRPAGE = PAGES.indexOf("ERROR"), oc = B.oled.c, OLED = "#9FF5D0";
  function fieldsNow() { return LaserEngine.FIELDS[PAGES[S.page]] || []; }
  function fieldSet(d) { var f = fieldsNow()[S.field];
    if (f === "kpps") { S.kpps = Math.max(5000, Math.min(40000, S.kpps + d * 1000)); remember(); }
    else if (f === "addr") S.dmx = Math.max(1, Math.min(512, S.dmx + d));
    else if (f === "univ") S.univ = Math.max(1, Math.min(512, S.univ + d));
    else if (f && S.net[f] !== undefined) Bind.run("net." + f); }
  function oledTurn(d) { if (dead()) return; click(); S.encT = .1; B.knob.rotation.y -= d * .35; // turns on the axis of its own cylinder
    if (S.edit) fieldSet(d); else { S.page = (S.page + d + PAGES.length) % PAGES.length; S.field = 0; }
    drawOled(); refresh(); }
  function oledOk() { if (dead()) return; click(); S.encT = .12; var f = fieldsNow();
    if (!S.edit) { if (!f.length) { pino.say("That page only informs. Turn the knob until one that has a field.", null, false); return; } S.edit = true; S.field = 0; }
    else if (PAGES[S.page] === "ERROR") { S.err = null; ENG.err = false; S.edit = false; }
    else if (++S.field >= f.length) { S.edit = false; S.field = 0; }
    drawOled(); refresh(); }
  function oledBack() { if (dead()) return; click(); S.backT = .12; if (S.edit) { S.edit = false; S.field = 0; } else S.page = (S.page + PAGES.length - 1) % PAGES.length; drawOled(); refresh(); }
  // a line that does not fit the width of the glass shrinks the font until it does: a rack display cuts
  // the glow, never the word (before, " ... SHUTTER CLOSED" ran off the glass on the right)
  function fitText(s, x, y, bold, px, max) { var f = function (n) { return bold + n + "px 'Share Tech Mono'"; }; oc.font = f(px); var tw = oc.measureText(s).width; if (tw > max) oc.font = f(Math.floor(px * max / tw)); oc.fillText(s, x, y); }
  function drawOled() { var w = B.oled.w, h = B.oled.h; oc.fillStyle = "#020806"; oc.fillRect(0, 0, w, h);
    if (!S.power) { B.oled.tex.needsUpdate = true; return; }
    var LN = LaserEngine.oledLines(S, ENG), alarm = (S.power && S.key && !S.lock) || (S.page === ERRPAGE && !!S.err), i;
    oc.shadowColor = OLED; oc.shadowBlur = 5; oc.textBaseline = "middle"; oc.fillStyle = OLED;
    oc.font = "700 34px 'Share Tech Mono'"; oc.textAlign = "left"; oc.fillText(LN[0], 16, 28);
    oc.textAlign = "right"; oc.fillText(S.edit ? "TURN: VALUE" : "TURN: PAGE", w - 16, 28);
    oc.shadowBlur = 0; oc.fillRect(12, 50, w - 24, 2); oc.shadowBlur = 5;
    // big line: inverted block when it is an alarm (SCAN FAIL or error), like on a rack display
    oc.textAlign = "left";
    if (alarm) { oc.shadowBlur = 0; oc.fillRect(12, 68, w - 24, 100); oc.fillStyle = "#020806"; fitText(LN[1], 24, 118, "700 ", 92, w - 48); oc.fillStyle = OLED; oc.shadowBlur = 5; }
    else fitText(LN[1], 16, 118, "700 ", 92, w - 32);
    for (i = 2; i < LN.length && i < 6; i++) fitText(LN[i], 14, 200 + (i - 2) * 56, "", 44, w - 28);
    fitText("PRESS: " + (S.edit ? "NEXT FIELD" : "ENTER") + "   BACK: RETURNS", 16, 456, "", 30, w - 32);
    oc.shadowBlur = 0; B.oled.tex.needsUpdate = true; }
  // ponytail: a 4 Hz clock redraw for what changes on its own (transport t, points) ; every
  // action already calls `drawOled` right away, so the click feedback does not wait for this.
  drawOled(); setInterval(drawOled, 250);

  /* ---------- actions ---------- */
  function toggleKey() { if (dead()) return; S.key = !S.key; if (S.key) blip(1500); else chord(); camsEnabled(S.mode !== "splash"); push("key", S.key); drawOled(); refresh(); }
  Bind.def("cam.show", "SHOW view", function () { if (!S.key) { pino.say("Arm the key first. No emission, no show; the key switch is on the rear panel, on the left.", null, false); blip(300); return; } setCam("show"); }, { key: "1" });
  Bind.def("cam.rear", "REAR view · menu", function () { setCam("rear"); }, { key: "2" });
  Bind.def("cam.inside", "INSIDE view · preferences", function () { setCam("inside"); }, { key: "3" });
  Bind.def("key.toggle", "key switch: arm", toggleKey, { key: "S", get: function () { return S.key; } });
  // putting the plug back takes the SCAN FAIL balloon with it: a warning that describes a state cannot
  // stay on the screen after the state is over (the display already says LIVE again)
  Bind.def("lock.toggle", "interlock", function () { if (dead()) return; S.lock = !S.lock; if (!S.lock) { chord(); pino.say("SCAN FAIL. Interlock open: shutter closed, beam parked.", null, false); } else { blip(1300); pino.hide(); } push("lock", S.lock); drawOled(); refresh(); }, { key: "I", get: function () { return S.lock; } });
  Bind.def("power.toggle", "power", function () { S.power = !S.power; if (!S.power) chord(); else blip(900); drawOled(); refresh(); }, { key: "P", get: function () { return S.power; } });
  // one play only: the .ild on the DAC and the show transport move together (the show is what rules the time)
  Bind.def("play.toggle", "play / pause", function () { S.play = !S.play; blip(); push("play", S.play); push("transport", S.play); refresh(); }, { key: "Space", get: function () { return S.play; } });
  // ponytail: local kpps until `laser_param` accepts `dev/pps` ; today the value only reaches the DAC on
  // the next `laser_open` (it is an opening argument), so disarming and arming applies it.
  Bind.def("kpps.down", "kpps −1k", function () { S.kpps = Math.max(5000, S.kpps - 1000); remember(); refresh(); }, { key: "[" });
  Bind.def("kpps.up", "kpps +1k", function () { S.kpps = Math.min(40000, S.kpps + 1000); remember(); refresh(); }, { key: "]" });
  Bind.def("kpps", "kpps (fader)", function (v) { S.kpps = Math.round(5000 + v * 35000); remember(); refresh(); }, { type: "cc", get: function () { return (S.kpps - 5000) / 35000; } });
  Bind.def("size", "size (fader)", function (v) { S.size = .3 + v; push("size", S.size); refresh(); }, { type: "cc", get: function () { return S.size - .3; } });
  ["r", "g", "b"].forEach(function (k) { Bind.def("lim." + k, { r: "red", g: "green", b: "blue" }[k] + " limit (fader)", function (v) { S.lim[k] = v; push("lim." + k, v); refresh(); }, { type: "cc", get: function () { return S.lim[k]; } }); });
  Bind.def("file.open", "open .ild", function () { $("#file").click(); }, { key: "O" });
  Bind.def("demo", "demo.ild", function () { S.show = demo.frames; S.frame = 0; S.pos = 0; S.name = "demo.ild · " + demo.frames.length + " frames"; refresh(); pino.say("Demo is back: tunnel, pentagram and the ribbon.", null, false); }, { key: "D" });
  ["ndi", "spout", "artnet", "sacn"].forEach(function (k) { Bind.def("net." + k, "network: " + k.toUpperCase(), function () { S.net[k] = !S.net[k]; blip(1000); drawOled(); refresh(); if (S.net[k] && (k === "ndi" || k === "spout")) pino.say(k.toUpperCase() + " is on, but the converter to ILDA does not exist yet: the FÓSFORO, a rack monitor on this port.", null, false); }, { get: function () { return S.net[k]; } }); });
  Bind.def("oled.up", "OLED: knob +", function () { oledTurn(1); }); Bind.def("oled.down", "OLED: knob −", function () { oledTurn(-1); }); Bind.def("oled.ok", "OLED: OK", oledOk); Bind.def("oled.back", "OLED: BACK", oledBack, { key: "Backspace" });
  function tabKey(t) { return function () { if (drawerOn() && dcur === t) closeDrawer(); else openDrawer(t); }; }
  Bind.def("nfo", "info", tabKey("info"), { key: "N" });
  Bind.def("bind", "bindings", tabKey("bind"), { key: "B" });
  Bind.def("drawer", "drawer: open and close", toggleDrawer, { key: "Tab", get: drawerOn });
  Bind.def("esc", "close the drawer", function () { closeDrawer(); pino.hide(); }, { key: "Escape" });
  /// A search that answers nothing is a search that looks broken: while it sweeps, the panel says
  /// SEARCHING; when it ends it says how many it found, and zero found is an answer, not silence.
  Bind.def("dacs", "find DACs", function () { if (!ENG.on) { pino.say("No engine, no DAC to find. Bring up spellcore serve and the key starts arming for real.", null, false); return; } ENG.dacs = []; ENG.scan = true; refresh(); bus.call("laser_dacs", { timeout: 2 }).then(function (r) { ENG.dacs = Array.isArray(r) ? r : []; ENG.scan = false; if (ENG.dacs.length) { if (!ENG.host) { ENG.dac = ENG.dacs[0].type; ENG.host = ENG.dacs[0].host; } pino.say(ENG.dacs.length + (ENG.dacs.length > 1 ? " DACs on the network." : " DAC on the network.") + " Click the one that gets the beam.", null, false); } else pino.say("No DAC answered in 2 s. Check the cable in the RJ45 and whether the EtherDream is on the same network.", null, false); refresh(); }, function (e) { ENG.scan = false; refresh(); fail("laser_dacs: " + e.message); }); });
  Bind.def("midi.connect", "MIDI: connect", function () { Bind.connect(); }); Bind.def("bind.reset", "bindings: reset", function () { Bind.reset(); });
  Bind.def("cam.reverse", "wheel: SolidWorks direction", function () { CAM.reverse = !CAM.reverse; try { localStorage.setItem("sc-laser-wheel", CAM.reverse ? "1" : "0"); } catch (e) {} refresh(); }, { get: function () { return CAM.reverse; } }); try { if (localStorage.getItem("sc-laser-wheel") === "0") CAM.reverse = false; } catch (e) {}
  // ±2° of parallax in the fixed view: it is the only movement the rear panel accepts, and it can be turned off
  Bind.def("cam.breathe", "REAR: mouse parallax", function () { CAM.breathe = !CAM.breathe; try { localStorage.setItem("sc-laser-breathe", CAM.breathe ? "1" : "0"); } catch (e) {} refresh(); }, { get: function () { return CAM.breathe; } }); try { if (localStorage.getItem("sc-laser-breathe") === "0") CAM.breathe = false; } catch (e) {}
  var d15 = Math.PI / 12; [["L", "ArrowLeft", d15, 0], ["R", "ArrowRight", -d15, 0], ["U", "ArrowUp", 0, d15], ["D", "ArrowDown", 0, -d15]].forEach(function (a) { Bind.def("cam.rot" + a[0], "camera: turn 15° " + a[0], function () { CAM.rotate(a[2], a[3]); }, { key: a[1] }); Bind.def("cam.rot90" + a[0], "camera: turn 90° " + a[0], function () { CAM.rotate(a[2] * 6, a[3] * 6); }, { key: "Shift+" + a[1] }); Bind.def("cam.pan" + a[0], "camera: pan " + a[0], function () { CAM.pan(a[2] * -400, a[3] * 400); }, { key: "Ctrl+" + a[1] }); });
  // Alt+arrows = roll (t_roll_view.htm): turning the view in the plane of the screen, which was the only gesture in the manual without a match here
  [["L", "ArrowLeft", .12], ["R", "ArrowRight", -.12]].forEach(function (a) { Bind.def("cam.roll" + a[0], "camera: roll " + a[0], function () { CAM.roll(a[2]); }, { key: "Alt+" + a[1] }); });
  Bind.def("cam.fit", "camera: fit", function () { CAM.fit(CENTER, .32); }, { key: "F" });
  [["front", "1"], ["back", "2"], ["left", "3"], ["right", "4"], ["top", "5"], ["bottom", "6"], ["iso", "7"]].forEach(function (v) { Bind.def("cam." + v[0], "standard view: " + v[0], function () { CAM.std(v[0], CENTER, .32); }, { key: "Ctrl+" + v[1] }); });
  // in the manual (t_zoom_in_out.htm) it is `Z` that ZOOMS OUT and `Shift+Z` that zooms in; it was swapped here
  Bind.def("cam.zoomIn", "camera: zoom +", function () { CAM.zoom(.8); }, { key: "Shift+Z" }); Bind.def("cam.zoomOut", "camera: zoom −", function () { CAM.zoom(1.25); }, { key: "Z" });
  // The MIDI state (and the LEARN in progress) lives in the drawer and in the MIDI row of the stats: the
  // bottom corner of the screen is gone (the owner asked for it), and a state can only be read in one place.
  Bind.onChange(refresh);
  document.querySelectorAll("#cams [data-a]").forEach(function (b) { b.addEventListener("click", function () { if (S.mode === "splash") return; Bind.run(b.dataset.a); }); });
  /// During the splash those buttons swallowed the click in silence, looking like live buttons. A button
  /// that does nothing is now disabled (grey, no hand cursor) and comes back when the splash leaves.
  function camsEnabled(on) { document.querySelectorAll("#cams [data-a]").forEach(function (b) { b.disabled = !on || (b.dataset.a === "cam.show" && !S.key); }); }
  camsEnabled(false);

  /* ---------- 3D interaction ---------- */
  function glow(o, on) { if (!o) return; o.traverse(function (mm) { if (mm.material && mm.material.emissive) { if (on) { if (mm.userData.orig) return; mm.userData.orig = mm.material; mm.material = mm.material.clone(); mm.material.emissive.setHex(0x38ff5c); mm.material.emissiveIntensity = .45; } else if (mm.userData.orig) { mm.material = mm.userData.orig; mm.userData.orig = null; } } }); }
  /* One click, one function. The family of the part (`LaserEngine.CONTROLS`) decides what the click does,
     and no part falls into two branches: `toggle` flips the state and ends there (it does not open the
     drawer), an inert part (sheet metal, fin, plug, fan) does nothing, `map` opens where you choose what
     drives that input, and the rest is navigation — it opens the tab of that part in the drawer. */
  var ACT = { power: function () { Bind.run("power.toggle"); }, keyswitch: function () { Bind.run("key.toggle"); }, interlock: function () { openDrawer("interlock"); },
    enc: oledOk, back: oledBack, lid: function () { setCam("inside"); }, pino: function () { pinoMenu(); } };
  function cursorFor(k) { return LaserEngine.inert(k) ? "default" : "pointer"; }
  /* The encoder knob with the TouchDesigner gesture: press and move the mouse up = increase, down =
     decrease, one step every 6 px (Shift = 24 px, fine adjustment). The camera does not turn during the drag
     in any view (`cam.js` refuses a left drag that starts on a part), and releasing without moving 3 px is
     still a click = OK. The wheel over the knob still turns the encoder, as before.
     ponytail: the knob only; the panel faders are `<input type=range>`, and their gesture belongs to HTML. */
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
    var h = hitOf(e); if (!h || h.o.userData.key !== "enc") return;                 // contract 3: `B.knobHit` is also "enc"
    knob = { y: e.clientY, y0: e.clientY, acc: 0 }; gl.setPointerCapture(e.pointerId); e.preventDefault(); });
  // the target of the click comes from the raycast of the `pointerup` itself, never from the `hot` of the
  // hover: a camera moving under the cursor (or a click without moving the mouse first) must not open the wrong panel
  gl.addEventListener("pointerup", function (e) { if (!pressed || e.button !== 0) return; pressed = false;
    if (knob) { var moved = Math.abs(e.clientY - knob.y0) >= 3; knob = null; try { gl.releasePointerCapture(e.pointerId); } catch (x) {} if (moved) return; }
    if (CAM.dragging()) return; if (S.mode === "splash") { skipSplash(); return; }
    var h = hitOf(e); if (!h) return; var k = h.o.userData.key;
    if (LaserEngine.inert(k)) return; // sheet metal, fin, plug, fan: the click does nothing
    if (/^pino\./.test(k)) { onPin(k.slice(5)); return; }
    blip(1100);
    if (ACT[k]) { ACT[k](); return; }
    if (/^(dmx|ilda|rj45)/.test(k) && S.cam === "show") setCam("rear"); if (/^(galvo|galvodrv|pcb|shutter|r|g|b)$/.test(k) && S.cam !== "inside") setCam("inside"); (PANELS[k] || function () {})(); });
  // mouse wheel over the encoder = turning the encoder (it is not a camera zoom)
  gl.addEventListener("wheel", function (e) { if (hot && hot.userData.key === "enc") { e.stopImmediatePropagation(); e.preventDefault(); oledTurn(e.deltaY < 0 ? 1 : -1); } }, { passive: false, capture: true });
  gl.addEventListener("dragover", function (e) { e.preventDefault(); stage.classList.add("dz"); }); gl.addEventListener("dragleave", function () { stage.classList.remove("dz"); });
  gl.addEventListener("drop", function (e) { e.preventDefault(); stage.classList.remove("dz"); loadFile(e.dataTransfer.files[0]); }); $("#file").addEventListener("change", function () { loadFile(this.files[0]); this.value = ""; });
  // A file that came in your hand: the engine does not have that path on its disk, so `ENG.file` clears and
  // play moves the transport only. To go out on the DAC, the .ild has to be in `shows/`.
  function loadFile(f) { if (!f) return; var r = new FileReader(); r.onload = function () { try { var d = ILDA.parse(r.result); if (!d.frames.length) throw 0; S.show = d.frames; S.frame = 0; S.pos = 0; ENG.file = ""; S.name = f.name + " · " + d.frames.length + " frames"; if (S.mode === "splash") skipSplash(); setCam("rear"); PANELS.ilda(); remember(); pino.say("Came in through the ILDA IN: " + f.name + ", " + d.frames.length + " frames, " + d.frames[0].length + " points in the first one. " + (d.frames[0].length > 1200 ? "Dense. If it flickers, open the lid and raise the kpps on the galvo." : "Light. It will fly.") + (S.key ? "" : " Arm the key to see it on the wall."), null, false); } catch (x) { pino.say("That is not ILDA. Format 2 (palette only) I skip, 0/1/4/5 I read.", null, false); } }; r.readAsArrayBuffer(f); }

  /* ---------- Pino ---------- */
  var pino = Pino3D.build(THREE, X, scene, pick, stage, cam, { on: onPin });
  Bind.def("pino.hide", "Pino: leaves the screen / comes back", function () { if (pino.alive()) pino.bye(); else pino.back(); });
  // The Pino is the menu of the program: each pin is a screen. Nothing gets in here out of software
  // convenience — an item only exists if it is a part of the device, and the balloon line says which part.
  var MENU = [["ilda", "1 · LASER", "this device"], ["ndi", "2 · FÓSFORO", "the network converter that does not exist yet"], ["orq", "3 · PATCHBAY", "the jack panel: what plugs into what"],
    ["cues", "4 · PAPER THEATER", "the DMX scenes and cues"], ["nfo", "5 · INFO", "N"],
    ["tl", "RECORDER", "the tape of the device: what you touch here gets recorded in time"], ["face", "CONSOLE", "the big buttons the operator presses during the show"]];
  function pinoMenu() { pino.say("Five pins, five screens. Pull one.", MENU); }
  function onPin(k) { blip(1000);
    if (k === "ilda") { setCam("rear"); PANELS.ilda(); pino.say("ILDA IN, at the back. Drop the .ild on the port, pick one in the panel, or take one from the engine disk.", null, false); }
    else if (k === "ndi") { setCam("rear"); PANELS.rj45(); pino.say("The NET port. NDI and Spout become ILDA in the FÓSFORO, which does not exist yet: for now it is just the port.", null, false); }
    else if (k === "orq") { location.href = "../patchbay.html"; return; }
    else if (k === "cues") { location.href = "../teatro.html"; return; }
    else if (k === "tl") { location.href = "../index.html"; return; }
    else if (k === "face") { location.href = "../face.html?face=quatro"; return; }
    else if (k === "nfo") PANELS.nfo();
    else if (k === "bye") { pino.bye(); return; }
    pino.current(k); }
  pino.current("ilda");
  function tips() { var F = fps(), f = S.show[S.frame]; if (S.mode !== "play" || !f) return; var flick = live() && F > 0 && F < 25, slow = S.kpps > 32000;
    if (flick && !S.tip.flick) { S.tip.flick = true; pino.say("It is flickering: " + f.length + " points at " + Math.round(S.kpps / 1000) + " kpps gives " + Math.round(F) + " fps. Open the lid and touch the galvo.", null, false); } if (!flick) S.tip.flick = false;
    if (slow && !S.tip.slow) { S.tip.slow = true; pino.say("At " + Math.round(S.kpps / 1000) + " kpps the galvo cannot keep up: the corners became curves.", null, false); } if (!slow) S.tip.slow = false; }

  /* ---------- splash / start ---------- */
  function start() { if (S.mode !== "splash") return; S.mode = "play"; $("#splash").classList.add("off"); S.dim = .55; camsEnabled(true); setCam("rear", 5); setTimeout(function () { pino.say(S.mem ? "Last time: " + Math.round(S.mem.kpps / 1000) + " kpps" + (S.mem.name ? ", " + S.mem.name.split(" · ")[0] : "") + ". The rear panel is the menu: arm the key and the SHOW view opens up. B opens the bindings." : "It looks like you are trying to run a laser show. The rear panel is the menu. Arm the key (on the left) and the SHOW view opens up. Pull a pin."); }, 900); }
  function skipSplash() { if (S.mode !== "splash") return; jingle(); start(); }
  document.addEventListener("keydown", function (e) { if (S.mode === "splash" && e.target.tagName !== "TEXTAREA") { e.stopPropagation(); skipSplash(); } }, true);

  /* ---------- HUD: the stats in the top right corner ----------
     One row per LIVE fact, and only what is on shows up. Nothing here is invented: engine and DAC come from
     the bus, the inputs come from what the show declares in `inputs` (and light up with the frame that
     arrives, topic 2), the MIDI comes from `Bind`. With no fact from the engine, the row does not exist.
     State is never colour alone (SISTEMA.md §11): every LED travels glued to the label that names it. */
  var srows = $("#srows"), lastHud = "", INPUTS = [];
  function readInputs(sh) { INPUTS = ((sh && sh.inputs) || []).filter(function (i) { return i && i.type; }).map(function (i) {
    return { name: { sacn: "sACN", artnet: "ART-NET", midi: "MIDI IN" }[i.type] || String(i.type).toUpperCase(), u: i.universe == null ? 1 : i.universe, last: 0 }; }); }
  function row(cls, txt, led) { return "<div class='r " + cls + "'><span>" + txt + "</span><i class='led " + led + "'></i></div>"; }
  // the four states of the device (SISTEMA.md §5), in the order the operator needs to read them
  function stateRow() { return !S.power ? ["", "", "OFF"] : !S.key ? ["amb", "warn blink lg", "STANDBY · DISARMED"]
    : !S.lock ? ["red", "err lg", "SCAN FAIL"] : ["las", "on lg", "LIVE · ARMED"]; }
  function hudStats() {
    var f = S.show[S.frame], F = fps(), st = stateRow(), now = performance.now(), h = "";
    h += row("pri", ENG.on ? "ENGINE · rev " + ENG.rev + (ENG.show ? " · " + ENG.show : "") : "ENGINE OFFLINE", (ENG.on ? "on" : "") + " lg");
    if (ENG.on) h += row("", "DAC " + ENG.dac + " " + (ENG.host || "(no host)") + (ENG.feed != null ? " · feed " + ENG.feed : ""),
      (ENG.feed != null ? "on" : ENG.scan ? "warn blink" : ENG.err ? "err" : "") + " lg");
    INPUTS.forEach(function (i) { h += row("", i.name + " · in " + i.u, now - i.last < 2000 ? "on" : ""); });
    if (/^[0-9]/.test(Bind.midi)) h += row("", "MIDI · " + Bind.midi, /^0 in/.test(Bind.midi) ? "" : "on");
    h += row(F && F < 25 && live() ? "red" : "las", (S.kpps / 1000).toFixed(0) + " kpps · " + (f ? f.length : 0) + " pts · " + (F ? F.toFixed(0) : "–") + " fps", "nil");
    h += row("", S.name.split(" · ")[0] + " · frame " + (S.frame + 1) + "/" + S.show.length + " · DMX " + S.dmx, "nil");
    if (VIDEO.get("hudFps") === "yes") h += row("", QFPS.toFixed(0) + " fps · " + QMSF.toFixed(1) + " ms · " + R.info.render.calls + " draw calls", "nil");
    h += row(st[0], st[2], st[1]);
    if (h !== lastHud) { lastHud = h; srows.innerHTML = h; }
  }

  /* ---------- tick ---------- */
  var last = performance.now(), W = 0, H = 0, T0 = performance.now(), lidT = 0, rearI = 0, fanW = 0, segsW = [], tmpV = new THREE.Vector3();
  function size() { if (stage.clientWidth !== W || stage.clientHeight !== H) { W = stage.clientWidth; H = stage.clientHeight; var pr = R.getPixelRatio(); R.setSize(W, H, false); composer.setSize(W, H); bloom.setSize(W * pr * +VIDEO.get("bloomRes"), H * pr * +VIDEO.get("bloomRes")); fxaa.uniforms.resolution.value.set(1 / (W * pr), 1 / (H * pr)); cam.aspect = W / H; cam.updateProjectionMatrix(); } }
  /* dt never runs backwards. The `now` of requestAnimationFrame is the instant the FRAME began, and it can
     be earlier than the `performance.now()` stored in `last` at page load: on the first frames dt came out
     negative (−0.23 s, measured in headless), `S.pos += S.kpps * dt` threw the galvo position to −6848 and
     `f[negative index]` became `undefined` — a TypeError on the wall every frame until the position climbed
     again, with the red error banner over the screen of whoever opens `app.html#laser`. It was also the whole
     animation (camera, lid, fan) running in reverse. */
  function tick(now) { requestAnimationFrame(tick);
    /* FPS CAP: a frame skipped by the clock, not by `setTimeout` — the rAF keeps the rhythm of the monitor
       and what gets cut is the work. `dt` does not vanish with it: `last` only moves on the frame that runs. */
    if (FPSCAP) { if (now < nextT) return; nextT = (nextT > now - 100 ? nextT : now) + 1000 / FPSCAP - .3; }
    var q0 = performance.now(); R.info.reset();
    size(); var dt = Math.min(.1, Math.max(0, (now - last) / 1000)); last = now; var t = (now - T0) / 1000;
    var wt0 = PERF && performance.now(); if (S.mode === "splash") { wallSplash(now, dt); wallTex.needsUpdate = true; } else wallTick(now, dt); if (PERF) { PERF.wallMs += performance.now() - wt0; PERF.frames++; }
    var want = S.cam === "inside" ? 1 : 0; lidT += (want - lidT) * Math.min(1, dt * 3); var sT = Math.min(1, lidT / .45), lT = Math.max(0, (lidT - .4) / .6); B.screws.forEach(function (s, i) { s.position.y = .004 + sT * .05; s.rotation.y = sT * 12 + i; }); B.lid.rotation.x = -lT * 1.9;
    CAM.update(dt * camSpeed / 5); rearI += (((S.cam === "rear" && S.mode === "play") ? 14 : 0) - rearI) * Math.min(1, dt * 3); B.rearLight.intensity = rearI; inLight.intensity = .35 * lidT; sun.intensity = 90 * S.dim; B.wallLight.intensity = 14 * S.dim;
    // external beams: aperture → lit points of the wall
    // not from behind: the fan of beams converges on the aperture, and the additive pile plus the bloom
    // painted a white slab over the chassis. The rear is the menu; the output stays on the wall.
    var on = S.power && S.cam !== "rear" && (S.mode === "splash" || live()), step = Math.max(1, Math.ceil(lit.length / BEAMN)), gain = (.05 + .16 * S.fog); segsW.length = 0; if (on) for (var i = 0; i < lit.length; i += step) { var L = lit[i]; segsW.push([[B.APERT.x, B.APERT.y, B.APERT.z], [(L[0][0] / WW - .5) * 8, 2.2 + (.5 - L[0][1] / WH) * 5, -5], [L[1][0] / 255 * gain, L[1][1] / 255 * gain, L[1][2] / 255 * gain]]); } beamsOut.set(segsW, 1);
    // internal optical path
    var arm = S.power && (S.key || S.mode === "splash"), opn = S.lock, segsI = O.segments(arm, opn, S.lim, gpos).map(function (s) { return [[s[0][0], s[0][1] + .314, s[0][2]], [s[1][0], s[1][1] + .314, s[1][2]], s[2]]; }); beamsIn.set(segsI, 1.2); beamsOut.tick(t); beamsIn.tick(t);
    O.shutter.rotation.y += (((arm && opn) ? 1.2 : 0) - O.shutter.rotation.y) * Math.min(1, dt * 12); O.mirX.rotation.y = -Math.PI / 4 + gpos[0] * .1; O.mirY.rotation.z = gpos[1] * .1;
    ["r", "g", "b"].forEach(function (k) { O.lens[k].m.material.color.setHex(arm ? O.lens[k].c : 0x111111); });
    proj.material.opacity = S.power ? 1 : 0;
    // emission LED with the four states of the device (SISTEMA.md §5): dark · slow amber in STANDBY ·
    // steady red in SCAN FAIL · steady green in LIVE. It used to be red for anything armed, and on the LED
    // LIVE looked the same as SCAN FAIL. The NET LEDs only light with power.
    B.emLed.material.color.setHex(!S.power ? 0x2a0a08 : !arm ? ((!MOVE || Math.floor(t * 1.2) % 2) ? 0xffb000 : 0x2a1e00) : !S.lock ? 0xff2a1a : 0x38ff5c);
    B.led1.material.color.setHex(S.power && (S.net.sacn || S.net.artnet) ? 0x38ff5c : 0x0a2a10); B.led2.material.color.setHex(S.power && (S.net.ndi || S.net.spout) && Math.floor(t * 6) % 2 ? 0xffb000 : 0x2a1e00);
    B.keyM.rotation.z += ((S.key ? Math.PI / 2 : 0) - B.keyM.rotation.z) * Math.min(1, dt * 8); B.lockPlug.position.z += ((S.lock ? 0 : .022) - B.lockPlug.position.z) * Math.min(1, dt * 6); B.rocker.rotation.x += ((S.power ? -.3 : .3) - B.rocker.rotation.x) * Math.min(1, dt * 18);
    /* fan: 7 rad/s, with spin-up and spin-down. At 24 rad/s a 60 Hz frame turned it 0.4 rad, almost half of
       the 7-blade period (0.9 rad): a stroboscope, it read as standing still. It follows the power switch
       only — a fan is a part of the device, not an animation the ANIMATIONS row switches off. */
    fanW += ((S.power ? 7 : 0) - fanW) * Math.min(1, dt * 1.5); B.blades.rotation.z -= dt * fanW;
    // the encoder and BACK sink when pressed: a physical button that does not move gives no feedback
    S.encT = Math.max(0, S.encT - dt); S.backT = Math.max(0, (S.backT || 0) - dt);
    B.knob.position.z += ((S.encT > 0 ? .0072 : .009) - B.knob.position.z) * Math.min(1, dt * 22); B.backCap.position.z += ((S.backT > 0 ? .0015 : .003) - B.backCap.position.z) * Math.min(1, dt * 22);
    puffs.forEach(function (p) { if (!p.visible) return; p.position.x += p.userData.v[0] * dt; p.position.y += p.userData.v[1] * dt; if (p.position.x > 3) p.position.x = -3; if (p.position.x < -3) p.position.x = 3; p.material.opacity = .02 + .06 * S.fog; });
    tmpV.set(0, .314, .15).project(cam); pino.update(dt, mouse, W, H, (1 - tmpV.y) / 2 * H + 14);
    // big ARMED LED on the chassis (contract of `chassi-4`): dark with no power, strong red blinking when
    // disarmed by the key, steady red in SCAN FAIL, steady green when armed. It is the same state as the
    // stats block — the operator reads it on the device and on the screen without having to compare.
    if (B.armLed) B.armLed.material.color.setHex(!S.power ? 0x2a0a08 : !S.key ? ((!MOVE || Math.floor(t * 1.6) % 2) ? 0xff2a1a : 0x2a0a08) : !S.lock ? 0xff2a1a : 0x38ff5c);
    hudStats();
    if (S.mode === "play") tips(); composer.render();
    // fps and ms/frame: MEASURED, and the same the VIDEO menu shows. A half-second window, so the number
    // does not shiver every frame — what the PERFORMANCE block says is what the clock counted.
    QN++; QMS += performance.now() - q0; QT += dt; if (QT >= .5) { QFPS = QN / QT; QMSF = QMS / QN; QT = QN = QMS = 0; }
    perfT += dt; if (perfT >= .25) { perfT = 0; vperf(); } }

  /* ---------- video: the table of `video.js` translated into three.js ----------
     One case per id, and nothing beyond that. If a row of the menu changes nothing in the render, it does
     not exist: a decorative option is a lie on the screen. `applyVideo(null)` applies the whole table (boot
     and preset change); `applyVideo(id)` applies one row only. */
  var FPSCAP = 0, nextT = 0, MOVE = true, BEAMN = 160, msRT = null, shKey = "";
  var QN = 0, QMS = 0, QT = 0, QFPS = 0, QMSF = 0, perfT = 0;
  var GLNAME = (function () { try { var g = R.getContext(), e = g.getExtension("WEBGL_debug_renderer_info");
    return String((e && g.getParameter(e.UNMASKED_RENDERER_WEBGL)) || g.getParameter(g.RENDERER) || "unknown renderer"); } catch (e) { return "unknown renderer"; } })();
  function mats(fn) { scene.traverse(function (o) { var m = o.material; if (!m) return; (Array.isArray(m) ? m : [m]).forEach(fn); }); }
  function matsDirty() { mats(function (m) { m.needsUpdate = true; }); }
  // 2x ceiling on the devicePixelRatio, as it already was: 100 % is the native resolution, 200 % is real supersampling
  function applyScale() { var pr = Math.min(2, devicePixelRatio) * +VIDEO.get("scale"); R.setPixelRatio(pr); composer.setPixelRatio(pr); W = 0; size(); }
  // the fixed view is framed FROM the fov (`frame` in cam.js): changing the fov without resetting the law
  // would leave the rear panel framed by the old fov. `CAM.mode` redoes the pose.
  function applyFov() { cam.fov = +VIDEO.get("fov"); cam.updateProjectionMatrix(); CAM.mode(LAWS[S.cam].mode, LAWS[S.cam]); }
  /* ANTI-ALIASING. What acts on the result is the composer, not the canvas: with EffectComposer, the
     `antialias:true` of the WebGLRenderer does not touch the passes (the scene goes to a render target). So
     MSAA here is a `WebGLMultisampleRenderTarget` in the composer — which the `composer.reset()` of r128
     swaps at run time, without recreating the renderer and without reloading the page. It needs WebGL 2;
     without it the row falls back to FXAA instead of lying. FXAA and MSAA do not add up: one replaces the
     other. */
  function applyAA() { var v = VIDEO.get("aa"), want = v === "msaa4";
    if (want && !R.capabilities.isWebGL2) { VIDEO.set("aa", "fxaa"); return; }
    fxaa.enabled = v === "fxaa";
    if (want === !!msRT) return;
    var pr = R.getPixelRatio(), w = Math.max(1, Math.round((W || 1440) * pr)), h = Math.max(1, Math.round((H || 900) * pr)),
      par = { minFilter: THREE.LinearFilter, magFilter: THREE.LinearFilter, format: THREE.RGBAFormat },
      rt = want ? new THREE.WebGLMultisampleRenderTarget(w, h, par) : new THREE.WebGLRenderTarget(w, h, par);
    if (want) rt.samples = 4;
    composer.reset(rt); msRT = want ? rt : null;
    composer.renderTarget1.texture.encoding = composer.renderTarget2.texture.encoding = THREE.sRGBEncoding;
    W = 0; size(); }
  // the shadow map is recreated when it changes size OR filter (VSM does not use the same format)
  function applyShadow() { var n = +VIDEO.get("shadows"), t = VIDEO.get("shadowType"), k = n + "/" + t;
    R.shadowMap.enabled = n > 0; sun.castShadow = n > 0;
    R.shadowMap.type = { basic: THREE.BasicShadowMap, pcf: THREE.PCFShadowMap, pcfsoft: THREE.PCFSoftShadowMap, vsm: THREE.VSMShadowMap }[t];
    if (n > 0) sun.shadow.mapSize.set(n, n);
    if (k !== shKey) { shKey = k; if (sun.shadow.map) { sun.shadow.map.dispose(); sun.shadow.map = null; } matsDirty(); }
    R.shadowMap.needsUpdate = true; }
  /* Only textures with a real image (canvas, <img>, ImageBitmap). The texture of a render target has
     `image = {width, height, depth}`: marking it `needsUpdate` makes three try `texImage2D` with that and
     the frame dies in a TypeError — and the wall (`proj`) uses one as its `map`. */
  function real(t) { return !!(t && t.image && (t.image.nodeName || (typeof ImageBitmap !== "undefined" && t.image instanceof ImageBitmap))); }
  function applyAniso() { var n = Math.min(+VIDEO.get("aniso"), R.capabilities.getMaxAnisotropy());
    mats(function (m) { ["map", "normalMap", "roughnessMap", "metalnessMap", "emissiveMap", "alphaMap"].forEach(function (k) {
      var t = m[k]; if (real(t) && t.anisotropy !== n) { t.anisotropy = n; t.needsUpdate = true; } }); }); }
  // the intensity multiplies the value the material brought from `mat.js` (the bench is .5, the floor .15):
  // one single value would flatten the tuning each surface already has. The original is kept in the material.
  function applyEnv() { scene.environment = VIDEO.get("reflections") === "yes" ? ENVTEX : null; var k = +VIDEO.get("reflectionInt");
    mats(function (m) { if (m.envMapIntensity === undefined) return; if (m.userData.ei0 === undefined) m.userData.ei0 = m.envMapIntensity; m.envMapIntensity = m.userData.ei0 * k; }); }
  function applyBloom() { bloom.enabled = VIDEO.get("bloom") === "yes"; bloom.strength = +VIDEO.get("bloomStrength"); bloom.radius = +VIDEO.get("bloomRadius"); bloom.threshold = +VIDEO.get("bloomThreshold"); }
  // exposure is a uniform (costs nothing); tone mapping is a #define, and only that one forces a recompile
  function applyTone() { var t = { none: THREE.NoToneMapping, linear: THREE.LinearToneMapping, reinhard: THREE.ReinhardToneMapping, cineon: THREE.CineonToneMapping, aces: THREE.ACESFilmicToneMapping }[VIDEO.get("tone")];
    if (R.toneMapping !== t) { R.toneMapping = t; matsDirty(); }
    R.toneMappingExposure = +VIDEO.get("exposure"); }
  /* WALL RESOLUTION: the render target where the trail lives. Changing its size is also changing the
     orthographic camera (which is in canvas coordinates), the fade quad and the 2D canvas of the splash —
     WW and WH are the unit of everything that draws there. The new target is born with GPU garbage: if it is
     not cleared here, the first frame of the wall comes with whatever was in memory. */
  function applyWall() { var q = String(VIDEO.get("wall")).split("x"), w = +q[0], h = +q[1];
    if (w === WW && h === WH) return;
    WW = w; WH = h; WC.width = WW; WC.height = WH; wrt.setSize(WW, WH);
    wcam.right = WW; wcam.bottom = WH; wcam.updateProjectionMatrix();
    fade.geometry.dispose(); fade.geometry = new THREE.PlaneGeometry(WW, WH); fade.position.set(WW / 2, WH / 2, 0);
    galvo = null; segN = 0; dotN = 0;
    var c0 = R.getClearColor(new THREE.Color()), a0 = R.getClearAlpha();
    R.setRenderTarget(wrt); R.setClearColor(0x000000, 1); R.clear(true, false, false); R.setClearColor(c0, a0); R.setRenderTarget(null); }
  function applyTrace() { haloMat.opacity = +VIDEO.get("halo"); haloMat.size = +VIDEO.get("haloPx"); }
  function applyPuffs() { var n = +VIDEO.get("puffs"); puffs.forEach(function (x, i) { x.visible = i < n; }); }
  var VAPP = {
    scale: applyScale, fov: applyFov,
    fpsMax: function () { FPSCAP = +VIDEO.get("fpsMax"); },
    hudFps: function () { lastHud = ""; },
    aa: applyAA, shadows: applyShadow, shadowType: applyShadow, aniso: applyAniso,
    reflections: applyEnv, reflectionInt: applyEnv,
    bloom: applyBloom, bloomStrength: applyBloom, bloomRadius: applyBloom, bloomThreshold: applyBloom,
    bloomRes: function () { W = 0; size(); },
    tone: applyTone, exposure: applyTone,
    wall: applyWall,
    trail: function () { FADE = +VIDEO.get("trail"); },
    halo: applyTrace, haloPx: applyTrace,
    beams: function () { BEAMN = +VIDEO.get("beams"); },
    dust: function () { var on = VIDEO.get("dust") === "yes"; beamsOut.dust(on); beamsIn.dust(on); },
    haze: function () { S.fog = +VIDEO.get("haze"); },
    puffs: applyPuffs,
    motion: function () { MOVE = VIDEO.get("motion") === "yes"; }
  };
  function applyVideo(id) { if (id) { if (VAPP[id]) VAPP[id](); } else Object.keys(VAPP).forEach(function (k) { VAPP[k](); }); }
  function perfText() { var r = R.info.render, m = R.info.memory, pr = R.getPixelRatio();
    return QFPS.toFixed(0) + " fps \u00b7 " + QMSF.toFixed(2) + " ms/frame" + (FPSCAP ? " \u00b7 cap " + FPSCAP : "")
      + "\n" + r.calls + " draw calls \u00b7 " + r.triangles + " triangles"
      + "\n" + m.geometries + " geometries \u00b7 " + m.textures + " textures"
      + "\nscreen " + Math.round(W * pr) + "\u00d7" + Math.round(H * pr) + " px (dpr " + pr.toFixed(2) + ") \u00b7 wall " + WW + "\u00d7" + WH
      + "\n" + (R.capabilities.isWebGL2 ? "WebGL 2" : "WebGL 1") + (msRT ? " \u00b7 MSAA " + msRT.samples + "\u00d7" : "") + " \u00b7 max aniso " + R.capabilities.getMaxAnisotropy()
      + "\n" + GLNAME; }
  // the PERFORMANCE block moves on its own, without repainting the drawer: only the text of the <pre> is rewritten
  function vperf() { if (!drawerOn() || dcur !== "video") return; var el = dbody.querySelector("#vperf"); if (el) el.textContent = perfText(); }
  /// Every row of the menu is also an address — `video.<id>`, of the right type (fader = cc, yes/no =
  /// button, list = cc that walks through the items). It comes out of the same table, so a new option gets a
  /// binding without anyone writing a binding. `Bind` already knows learn: no new fixed key is spent here.
  VIDEO.OPTS.forEach(function (o) { var id = "video." + o.id, lbl = "video: " + o.label.toLowerCase();
    if (o.type === "range") Bind.def(id, lbl, function (v) { VIDEO.set(o.id, o.min + v * (o.max - o.min)); },
      { type: "cc", get: function () { return (VIDEO.get(o.id) - o.min) / (o.max - o.min); } });
    else if (o.vals.length === 2) Bind.def(id, lbl, function () { VIDEO.set(o.id, VIDEO.get(o.id) === o.vals[1][0] ? o.vals[0][0] : o.vals[1][0]); },
      { get: function () { return VIDEO.get(o.id) === o.vals[1][0]; } });
    else Bind.def(id, lbl, function (v) { VIDEO.set(o.id, o.vals[Math.max(0, Math.min(o.vals.length - 1, Math.round(v * (o.vals.length - 1))))][0]); },
      { type: "cc", get: function () { var i = 0; o.vals.forEach(function (x, j) { if (x[0] === String(VIDEO.get(o.id))) i = j; }); return i / (o.vals.length - 1); } });
  });
  Bind.def("video", "video menu", tabKey("video"));
  Bind.def("video.preset", "video: preset", function (v) { VIDEO.preset(VIDEO.PRESETS[Math.max(0, Math.min(3, Math.round(v * 3)))]); },
    { type: "cc", get: function () { var i = VIDEO.PRESETS.indexOf(VIDEO.presetName()); return i < 0 ? 1 : i / 3; } });
  Bind.def("video.defaults", "video: restore defaults", function () { VIDEO.reset(); });
  Bind.def("video.fullscreen", "video: fullscreen", function () { try { if (document.fullscreenElement) document.exitFullscreen(); else stage.requestFullscreen(); } catch (e) {} });
  // the frame measurement is readable from outside: this is where the headless gate reads fps and draw calls
  window.SC = window.SC || {}; SC.video = function () { var r = R.info.render; return { fps: QFPS, ms: QMSF, calls: r.calls, tris: r.triangles, preset: VIDEO.presetName(), aa: VIDEO.get("aa"), msaa: msRT ? msRT.samples : 0, gl: GLNAME, dpr: R.getPixelRatio(), wall: WW + "x" + WH }; };
  VIDEO.onChange(function (id) { applyVideo(id); if (!id) refresh(); Bind.syncAll(); });
  applyVideo(null);

  /* ---------- boot: camera aimed at the output; the focus leaves the parked dot and goes to the wall ---------- */
  document.fonts.ready.then(function () { size(); plate(null); var ol = ILDA.outlines([["SPELLCASTER", "64px Michroma", 272], ["LASER", "64px Michroma", 372]], WW, WH); SP.loops = ol.loops; SP.len = ol.len;
    var h = location.hash; if (h === "#laser" || h === "#rear" || h === "#inside") { CAM.setView(VIEWS.rear[0], VIEWS.rear[1], true); start(); if (h === "#laser" || h === "#inside") { S.key = true; camsEnabled(true); if (h === "#laser") setCam("show"); } if (h === "#inside") setCam("inside"); }
    else { CAM.setView(VIEWS.wall0[0], VIEWS.wall0[1], true); setTimeout(function () { setCam("wall", 2.6); }, 300); S.t0 = performance.now(); }
    requestAnimationFrame(tick); });
})();
