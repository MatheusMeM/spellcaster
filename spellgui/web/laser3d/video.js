/* VIDEO menu of the visualizer: the TABLE of the options, and nothing else. No three.js in here.
   Each row is `{id, g, label, type, ...}` plus the value it takes in each preset. The VIDEO tab of the
   drawer draws itself from this table, `app.js` translates each `id` into three.js in `applyVideo`, and
   `Bind.def("video.<id>")` comes from the same place — the identity of each option is the textual address
   `video.<id>` (FUNCOES/README rule 2), the same one in key, MIDI and localStorage.
   The preset name is not stored: it is DERIVED from the values (`nameOf`), so touching one row turns it
   into CUSTOM by itself and putting the value back by hand returns the preset name. */
window.VIDEO = (function () {
  "use strict";
  var KEY = "sc-laser-video", PRESETS = ["low", "medium", "high", "ultra"];
  var NAMES = { low: "LOW", medium: "MEDIUM", high: "HIGH", ultra: "ULTRA", custom: "CUSTOM" };
  var YESNO = [["no", "NO"], ["yes", "YES"]];
  function sel(id, g, label, vals, b, m, a, u, note) { return { id: id, g: g, label: label, type: "select", vals: vals, p: { low: b, medium: m, high: a, ultra: u }, note: note }; }
  function rng(id, g, label, min, max, step, unit, b, m, a, u, note) { return { id: id, g: g, label: label, type: "range", min: min, max: max, step: step, unit: unit || "", p: { low: b, medium: m, high: a, ultra: u }, note: note }; }
  function bool(id, g, label, b, m, a, u, note) { return sel(id, g, label, YESNO, b, m, a, u, note); }

  var GROUPS = ["SCREEN", "QUALITY", "POST-PROCESSING", "LASER", "SCENE"];
  var SUB = {
    SCREEN: "how many pixels the program draws, and how many times per second",
    QUALITY: "aliasing, shadow and texture · this is where the GPU suffers",
    "POST-PROCESSING": "what the composer does after the scene, full screen",
    LASER: "the visualizer itself: the wall, the trail, the beams and the haze",
    SCENE: "the device, the Pino cable and whatever moves on its own"
  };
  /* The table. Order = order on screen. Nothing here is decorative: every id has a case in `applyVideo`. */
  var OPTS = [
    sel("scale", "SCREEN", "RESOLUTION SCALE", [[".5", "50 %"], [".75", "75 %"], ["1", "100 %"], ["1.5", "150 %"], ["2", "200 %"]], ".5", ".75", "1", "2",
      "% of the native resolution; 100 % = devicePixelRatio, capped at 2×"),
    rng("fov", "SCREEN", "FIELD OF VIEW", 30, 90, 1, "°", 42, 42, 42, 42, "only the SHOW view is free; the fixed ones reframe themselves"),
    sel("fpsMax", "SCREEN", "FPS CAP", [["30", "30"], ["60", "60"], ["120", "120"], ["0", "UNLIMITED"]], "0", "0", "0", "0"),
    bool("hudFps", "SCREEN", "HUD COUNTER", "no", "no", "no", "no"),

    sel("aa", "QUALITY", "ANTI-ALIASING", [["off", "OFF"], ["fxaa", "FXAA"], ["msaa4", "MSAA 4×"]], "off", "fxaa", "fxaa", "msaa4",
      "MSAA = multisampled target in the composer (needs WebGL 2); FXAA acts on the composer output"),
    sel("shadows", "QUALITY", "SHADOWS", [["0", "OFF"], ["1024", "1024"], ["2048", "2048"], ["4096", "4096"]], "0", "1024", "2048", "4096"),
    sel("shadowType", "QUALITY", "SHADOW FILTER", [["basic", "BASIC"], ["pcf", "PCF"], ["pcfsoft", "PCF SOFT"], ["vsm", "VSM"]], "basic", "pcf", "pcfsoft", "vsm"),
    sel("aniso", "QUALITY", "ANISOTROPY", [["1", "1×"], ["2", "2×"], ["4", "4×"], ["8", "8×"], ["16", "16×"]], "1", "4", "8", "16", "capped by the GPU maximum"),
    bool("reflections", "QUALITY", "ENVIRONMENT REFLECTIONS", "no", "yes", "yes", "yes"),
    rng("reflectionInt", "QUALITY", "REFLECTION INTENSITY", 0, 2, .05, "×", 1, 1, 1, 1),

    bool("bloom", "POST-PROCESSING", "BLOOM", "no", "yes", "yes", "yes"),
    rng("bloomStrength", "POST-PROCESSING", "BLOOM STRENGTH", 0, 2, .05, "", .5, .5, .5, .5),
    rng("bloomRadius", "POST-PROCESSING", "BLOOM RADIUS", 0, 1, .01, "", .45, .45, .45, .45),
    rng("bloomThreshold", "POST-PROCESSING", "BLOOM THRESHOLD", 0, 1, .01, "", .82, .82, .82, .82),
    sel("bloomRes", "POST-PROCESSING", "BLOOM RESOLUTION", [[".25", "¼ OF SCREEN"], [".5", "½ OF SCREEN"], ["1", "1× OF SCREEN"]], ".25", ".5", ".5", "1"),
    sel("tone", "POST-PROCESSING", "TONE MAPPING", [["none", "NONE"], ["linear", "LINEAR"], ["reinhard", "REINHARD"], ["cineon", "CINEON"], ["aces", "ACES"]], "aces", "aces", "aces", "aces"),
    rng("exposure", "POST-PROCESSING", "EXPOSURE", .2, 3, .05, "", .95, .95, .95, .95),

    sel("wall", "LASER", "WALL RESOLUTION", [["512x320", "512 × 320"], ["1024x640", "1024 × 640"], ["2048x1280", "2048 × 1280"]], "512x320", "1024x640", "1024x640", "2048x1280"),
    rng("trail", "LASER", "TRAIL PERSISTENCE", .3, .95, .01, "", .7, .7, .7, .7, "how much of the previous frame survives every 1/60 s"),
    rng("halo", "LASER", "STROKE HALO", 0, .4, .01, "", 0, .12, .12, .2),
    rng("haloPx", "LASER", "HALO SIZE", 4, 16, 1, "px", 9, 9, 9, 12),
    sel("beams", "LASER", "EXTERNAL BEAMS", [["40", "40"], ["80", "80"], ["160", "160"], ["320", "320"]], "40", "80", "160", "320"),
    bool("dust", "LASER", "BEAM DUST", "no", "yes", "yes", "yes"),
    rng("haze", "LASER", "HAZE", 0, 1, .01, "", .85, .85, .85, .85),
    sel("puffs", "LASER", "HAZE PUFFS", [["0", "NONE"], ["14", "14"], ["28", "28"]], "0", "14", "14", "28"),

    bool("rope", "SCENE", "PINO CABLE (VERLET)", "no", "yes", "yes", "yes"),
    bool("motion", "SCENE", "ANIMATIONS (FAN, LED)", "yes", "yes", "yes", "yes", "off by default with prefers-reduced-motion")
  ];
  var byId = {}; OPTS.forEach(function (o) { byId[o.id] = o; });

  var V = {}, onChange = function () {};
  /// Validation on the way in: out of range or out of the list is REFUSED (returns null), not clamped —
  /// a value the menu does not offer cannot become state, whether it comes from localStorage or from MIDI.
  function ok(o, v) {
    if (!o || v === undefined || v === null) return null;
    if (o.type === "range") { var n = +v; return (typeof v !== "boolean" && isFinite(n) && n >= o.min && n <= o.max) ? n : null; }
    var s = String(v); for (var i = 0; i < o.vals.length; i++) if (o.vals[i][0] === s) return s;
    return null;
  }
  function apply(name) { OPTS.forEach(function (o) { V[o.id] = o.p[name]; }); }
  /// The preset name is derived, never stored: if every value matches a preset, that is the name;
  /// otherwise it is CUSTOM.
  function nameOf() {
    for (var i = 0; i < PRESETS.length; i++) { var p = PRESETS[i], all = true;
      for (var j = 0; j < OPTS.length; j++) if (String(V[OPTS[j].id]) !== String(OPTS[j].p[p])) { all = false; break; }
      if (all) return p; }
    return "custom";
  }
  function reduced() { try { return typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches; } catch (e) { return false; } }
  // a weak machine starts at MEDIUM: four cores or fewer is almost always an office laptop
  function defaultPreset() { try { return (navigator.hardwareConcurrency || 8) <= 4 ? "medium" : "high"; } catch (e) { return "high"; } }
  function save() { try { localStorage.setItem(KEY, JSON.stringify(V)); } catch (e) {} }
  /// Broken JSON, an unknown key or a value out of range bring nothing down: every row that fails to
  /// validate keeps the default. One bad row does not cost the whole menu.
  function load() {
    apply(defaultPreset());
    if (reduced()) { V.motion = "no"; V.trail = .4; }   // this was the loose `reduced` of app.js
    var raw = null, d = null;
    try { raw = localStorage.getItem(KEY); } catch (e) {}
    if (raw) { try { d = JSON.parse(raw); } catch (e) { d = null; } }
    if (d && typeof d === "object") OPTS.forEach(function (o) { var c = ok(o, d[o.id]); if (c !== null) V[o.id] = c; });
    return V;
  }
  function set(id, v) { var o = byId[id], c = ok(o, v); if (c === null) return false;
    if (String(V[id]) === String(c)) return true;
    V[id] = c; save(); onChange(id); return true; }
  function preset(name) { if (PRESETS.indexOf(name) < 0) return false; apply(name); save(); onChange(null); return true; }

  var api = {
    OPTS: OPTS, GROUPS: GROUPS, SUB: SUB, PRESETS: PRESETS, NAMES: NAMES,
    opt: function (id) { return byId[id]; },
    load: load, save: save, reset: function () { return preset(defaultPreset()); },
    get: function (id) { return V[id]; },
    set: set, preset: preset, presetName: nameOf, defaultPreset: defaultPreset,
    onChange: function (f) { onChange = f || function () {}; }
  };
  load();
  return api;
})();
if (typeof module !== "undefined") module.exports = window.VIDEO;
