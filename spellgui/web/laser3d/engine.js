/* laser3d <-> engine: the three pure functions of the laser page — what each part of the device is
   (`CONTROLS`), the state -> registry command map (`cmdFor`) and what the display shows
   (`oledLines`). None of them touches DOM, THREE or the network: this is what the test in
   `spellgui/web/test/laser3d.test.js` runs without a browser.

   `cmdFor(id, value, st)`: `id` is the path of the device state (the same name the binding and the
   slider use), `value` is the absolute value already applied to `S`, `st` is the context of the open
   feed ({feed, dac, host, kpps, file, fps, show}). It returns `{cmd, args}` of the registry, or `null`
   when that state has no command today — the page keeps working locally and the drawing on the wall is
   the same. `app.js` is the caller; `bus.call` is what executes. */
(function () {
  "use strict";

  // ---------------------------------------------------------------- CONTROLS
  // One control, one function. `[family, label]` per `userData.key` of the parts in `body.js` and
  // `optics.js`; this is where the tooltip comes from and this is what `app.js` reads to decide what
  // the click does.
  // family: "toggle" (flips a state) | "momentary" (acts while held) | "value" (changes a number) |
  // "nav" (opens the tab of that part in the drawer) | "map" (the part is an INPUT of the device:
  // the click opens where you choose WHAT drives it) | "part" (plate, fin, plug, fan: it does not
  // light up, it has no tooltip and the click does nothing — the owner does not want hover menus on
  // those parts). No key can have two families, and no function can live in two keys.
  // Empty label = no tooltip. The label is screen text; the display PAGE names (`oledLines`) are the
  // ones kept in plain ASCII, because that is a display.
  var CONTROLS = {
    // rear panel — power and safety
    power:     ["toggle", "POWER: turns the device on and off"],
    keyswitch: ["toggle", "key switch: arms the emission"],
    interlock: ["map", "interlock: it is an input — click to map what drives it"],
    acin:      ["part", ""],
    // rear panel — display and navigation
    enc:       ["nav", "encoder: turn to navigate, press to enter"],
    back:      ["nav", "BACK: goes back one display page"],
    // rear panel — ports
    ilda:      ["nav", "ILDA IN: loads the .ild"],
    ildathru:  ["nav", "ILDA OUT: chains the next projector"],
    dmxin:     ["nav", "DMX IN: address and mode"],
    dmxout:    ["nav", "DMX OUT: repeats the universe"],
    rj45:      ["nav", "NET: sACN, Art-Net, NDI, Spout and the DACs on the network"],
    fan:       ["part", ""],
    // body — the lid still opens on click (that is nav), but with no tooltip
    lid:       ["nav", ""],
    front:     ["part", ""],
    aperture:  ["part", ""],
    side:      ["part", ""],
    // inside (optics.js)
    bench:     ["part", ""],
    r:         ["nav", "red module 638 nm · 2.5 W: limit and curve"],
    g:         ["nav", "green module 520 nm · 3 W: limit and curve"],
    b:         ["nav", "blue module 445 nm · 4.5 W: limit and curve"],
    dichro:    ["part", "dichroic mirror: joins the beams into one"],
    fold:      ["part", ""],
    shutter:   ["nav", "shutter: closes without key or without interlock"],
    galvo:     ["nav", "X/Y galvos: kpps"],
    galvodrv:  ["nav", "galvo driver: buffer and speed"],
    pcb:       ["nav", "laser driver: R/G/B limit and curve"],
    dac:       ["part", ""],
    psu:       ["part", ""],
    // Pino
    pino:      ["nav", "Pino: DMX cable, five pins, zero patience"]
  };
  function labelOf(k) { return (CONTROLS[k] && CONTROLS[k][1]) || ""; }
  function kindOf(k) { return (CONTROLS[k] && CONTROLS[k][0]) || ""; }
  // Inert part: does not light up, has no tooltip, normal cursor, the click does nothing. A key that
  // is not in the table lands here too (a model part with no declared function — that is how the
  // deleted USB disappears from the interface). A namespaced key (`pino.<pin>`) belongs to someone
  // else: it is not a part.
  function inert(k) { return String(k).indexOf(".") < 0 && (kindOf(k) === "" || kindOf(k) === "part"); }

  function cmdFor(id, v, st) {
    st = st || {};
    var feed = st.feed;
    switch (id) {
      // the key arms the device: it opens (or closes) the chosen DAC
      case "key":
        if (v) return { cmd: "laser_open", args: { dac: st.dac || "etherdream", host: st.host || "", kpps: (st.kpps || 30000) / 1000 } };
        return feed == null ? null : { cmd: "laser_close", args: { feed: feed } };
      case "play":
        if (feed == null) return null;
        if (!v) return { cmd: "laser_stop", args: { feed: feed } };
        return st.file ? { cmd: "laser_play", args: { feed: feed, file: st.file, fps: st.fps || 30, loop: true } } : null;
      // the same play moves the show transport, when there is a show loaded
      case "transport":
        return st.show ? { cmd: v ? "resume" : "pause", args: {} } : null;
      // interlock closed = shutter open; `shutter` 1 closes
      case "lock":
        return feed == null ? null : { cmd: "laser_param", args: { feed: feed, path: "shutter", value: v ? 0 : 1 } };
      case "size":
        return feed == null ? null : { cmd: "laser_param", args: { feed: feed, path: "geo/scale", value: +v } };
      case "lim.r": case "lim.g": case "lim.b":
        return feed == null ? null : { cmd: "laser_param", args: { feed: feed, path: "limit/" + id.slice(4), value: +v } };
      default:
        // ponytail: kpps, gam.*, dmx, buffer, speed, net.* and power stay local only until
        // `laser_param` accepts dev/pps, curve/r|g|b, dmx/addr ; then each one becomes one more case
        // here, without touching app.js.
        return null;
    }
  }

  // -------------------------------------------------------------- oledLines
  // The rear panel display is the instrument of the device, not decoration: seven pages, and each one
  // answers a question the operator asks from across the room. `oledLines(S, eng)` is pure and returns
  // the lines ready to draw: [0] header, [1] the big line (the one you read from the other side of the
  // room), [2..] up to four detail lines. A line that starts with ">" is the selected field.
  // Everything in ASCII: this is equipment hardware, not a web page.
  // ponytail: no TEMP page ; the device publishes no temperature. STATUS already shows key, interlock and shutter.
  var PAGES = ["STATUS", "SHOW", "DMX", "NET", "ENGINE", "ERROR"];
  // The fields the encoder edits on each page, in the order it walks through them.
  var FIELDS = { STATUS: ["kpps"], DMX: ["addr", "univ"], NET: ["sacn", "artnet", "ndi", "spout"], SHOW: [], ENGINE: [], ERROR: ["clear"] };
  function n3(v) { return ("00" + Math.round(v)).slice(-3); }
  function mmss(t) { t = Math.max(0, Math.round(t || 0)); return Math.floor(t / 60) + ":" + ("0" + (t % 60)).slice(-2); }
  function cut(s, n) { s = String(s == null ? "" : s).toUpperCase(); return s.length > n ? s.slice(0, n - 1) + "+" : s; }

  function oledLines(S, eng) {
    eng = eng || {}; S = S || {};
    var page = PAGES[S.page] || PAGES[0], f = FIELDS[page] || [];
    var head = page + "   " + ((S.page || 0) + 1) + "/" + PAGES.length + (S.edit ? "  EDIT" : "");
    var sel = function (i, s) { return (S.edit && S.field === i ? ">" : " ") + s; };
    if (!S.power) return [head, "OFF", " NO MAINS POWER", " THE POWER ROCKER TURNS IT ON"];
    var armed = !!(S.power && S.key), live = armed && S.lock;
    var fr = (S.show && S.show[S.frame]) || [], pts = fr.length, fps = pts ? Math.round(S.kpps / pts) : 0;
    var out = [head];
    if (page === "STATUS") {
      out.push(live ? "LIVE" : !S.key ? "DISARMED" : !S.lock ? "SCAN FAIL" : "STANDBY");
      out.push(" KEY " + (S.key ? "ARMED" : "OPEN") + "  ILK " + (S.lock ? "OK" : "OPEN") + "  SHUTTER " + (live ? "OPEN" : "CLOSED"));
      out.push(" DAC " + cut(eng.dac || "-", 12) + " " + (eng.feed != null ? "FEED " + eng.feed : eng.err ? "ERROR" : eng.host ? cut(eng.host, 15) : "NO HOST"));
      out.push(sel(0, "KPPS " + Math.round((S.kpps || 0) / 1000) + "  " + pts + " PTS  " + fps + " FPS"));
    } else if (page === "SHOW") {
      var tr = eng.tr || null;
      out.push(cut(eng.show || "NO SHOW", 18));
      out.push(" REV " + (eng.rev || 0) + "  " + (tr ? tr.state.toUpperCase() : "NO PLAYER") + "  T " + mmss(tr && tr.t) + " / " + mmss(tr && tr.duration));
      // ponytail: the `transport` event does not publish loop ; the only loop the page knows about is
      // the one from `laser_play --loop`, so that is the one shown. Goes away when the event carries loop.
      out.push(" ILDA " + cut(S.name || "-", 22));
      out.push(" FRAME " + ((S.frame || 0) + 1) + "/" + ((S.show && S.show.length) || 0) + "  " + (S.play ? "PLAY" : "PAUSE") + "  LOOP " + (eng.feed != null && eng.file ? "ON" : "-"));
    } else if (page === "DMX") {
      out.push("ADDR " + n3(S.dmx));
      out.push(sel(0, "ADDRESS " + n3(S.dmx)));
      out.push(sel(1, "UNIVERSE " + (S.univ || 1) + "   MODE 16CH"));
      out.push(" BUS " + (S.dmxIn ? "U" + S.dmxIn.universe + " CH" + n3(S.dmx) + " = " + S.dmxIn.value : "NO FRAME"));
    } else if (page === "NET") {
      var nets = f, on = nets.filter(function (k) { return S.net && S.net[k]; });
      out.push(on.length ? on.join(" ").toUpperCase() : "ALL OFF");
      nets.forEach(function (k, i) { if (i < 4) out.push(sel(i, k.toUpperCase() + (S.net && S.net[k] ? "  ON" : "  OFF") + (i === 0 ? "        DAC " + cut(eng.dac || "-", 10) : ""))); });
    } else if (page === "ENGINE") {
      out.push(eng.on ? "OK  REV " + (eng.rev || 0) : "OFFLINE");
      out.push(" PORT " + cut(eng.port || "-", 22) + "   SPELL " + cut(eng.ver || "0.1.2", 8));
      out.push(" FEED " + (eng.feed != null ? eng.feed + "  " + ((eng.stats && eng.stats["stat/sent"]) || 0) + " PT" : "CLOSED") + "   DAC " + cut(eng.dac || "-", 10));
      out.push(" LOG " + cut(eng.log || "-", 30));
    } else {
      out.push(S.err ? "ERROR" : "NO ERROR");
      if (S.err) { out.push(" " + cut(S.err.msg, 40)); if (S.err.msg.length > 40) out.push(" " + cut(S.err.msg.slice(39), 40)); out.push(" " + S.err.when); }
      else out.push(" NO ERROR SINCE POWER ON");
      out.push(sel(0, "CLEAR"));
    }
    return out;
  }

  var api = { cmdFor: cmdFor, CONTROLS: CONTROLS, labelOf: labelOf, kindOf: kindOf, inert: inert, PAGES: PAGES, FIELDS: FIELDS, oledLines: oledLines };
  if (typeof window !== "undefined") window.LaserEngine = api;
  if (typeof module !== "undefined") module.exports = api;
})();
