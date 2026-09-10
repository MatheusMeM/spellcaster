"use strict";
// viewer.js — 2D previz of the timeline: what goes out at time `t`, in the bottom strip of the
// canvas (Alt+M).
//
// Three columns, each one a slice of the output at the playhead:
//   dmx    512 bars from the last binary frame of the universe of the focused lane: the OUTPUT
//          (bus topic 1), or the INPUT (topic 2) when the lane is armed for recording;
//   ilda   the .ild frame of each laser track at `t`, with the track `scale` and `rot` applied;
//   patch  one dot per patch fixture, painted with the output frame through the profile.
//
// It does not draw: 3D, beams, per-lane thumbnails, video (ROADMAP R6 is Godot, another thing).
//
// A separate file on purpose: the timeline only calls `VW.draw(ctx, rect, t)`, and the previz
// grows without touching timeline.js. It needs `window.TL` and the pure part of `teatro.js`
// (`TEATRO`) — index.html loads both before this script.

// IIFE because a classic script shares the global scope: `const TL` already belongs to timeline.js.
(function (window, TEATRO) {
  const TL = window.TL;

  const VW = {
    clip: new Map(),     // "file|index" -> points of the frame, or 1 while the request is in flight
    n: new Map(),        // file -> how many frames it has (only after the first response)
    prof: new Map(),     // profile name -> channels, or 1 in flight
    call: (cmd, args) => TL.call(cmd, args),
    // ponytail: ceiling of 512 cached frames, emptied whole ; make it an LRU if a long .ild flickers
    cap: 512,
  };
  window.VW = VW;

  // ---- model --------------------------------------------------------------

  // Value of a track parameter (`scale`, `rot`) at `t`, straight from the .spell keys. Same maths
  // as `TL.valueAt`: the curve applies to the segment that ARRIVES at the keyframe.
  VW.param = function (keys, t, def) {
    if (!Array.isArray(keys) || !keys.length) return def;
    if (t <= keys[0][0]) return +keys[0][1];
    let i = 0;
    while (i + 1 < keys.length && keys[i + 1][0] <= t) i++;
    if (i + 1 >= keys.length) return +keys[i][1];
    const a = keys[i], b = keys[i + 1], dt = b[0] - a[0];
    if (dt <= 0) return +b[1];
    return +a[1] + (+b[1] - +a[1]) * TL.ease(b[2])((t - a[0]) / dt);
  };

  // Frame of the clip at `t`: the player's own maths (`spellcaster/player/player.py::laser_frame`).
  // Before the first response nobody knows how many frames the file has; the engine takes the
  // modulo anyway, so the raw index only costs one extra cache entry.
  VW.index = function (clip, t, fps) {
    const i = Math.max(0, Math.floor(t * (fps > 0 ? fps : 30)));
    const n = VW.n.get(clip);
    return n ? i % n : i;
  };

  // Points of the frame, or null while they do not arrive. One request per (clip, frame).
  VW.frame = function (clip, index) {
    const k = clip + "|" + index;
    const v = VW.clip.get(k);
    if (v !== undefined) return v === 1 ? null : v;
    if (VW.clip.size >= VW.cap) VW.clip.clear();
    VW.clip.set(k, 1);
    VW.call("clip_frame", { clip: clip, index: index }).then(r => {
      VW.n.set(clip, r.frames);
      VW.clip.set(k, r.points || []);
      if (TL.k) TL.k.invalidate();
    }, () => VW.clip.set(k, []));      // the error stays cached: no point asking again every frame
    return null;
  };

  // Channels of the profile, or null while they do not arrive.
  VW.profile = function (name) {
    const v = VW.prof.get(name);
    if (v !== undefined) return v === 1 ? null : v;
    VW.prof.set(name, 1);
    VW.call("profile_get", { name: name }).then(p => {
      VW.prof.set(name, p.channels || []);
      if (TL.k) TL.k.invalidate();
    }, () => VW.prof.set(name, []));
    return null;
  };

  // Fixture color in the output frame: the SAME maths as the paper theater (`teatro.js`, pure
  // part, covered by test/teatro.test.js). A copy of its own disagreed with it on the same
  // fixture: it ignored the `w` channel (the WLED bar in pure white came out black here and white
  // there) and the `on` channel.
  // A dark fixture (intensity 0) returns null: the dot keeps only its outline.
  VW.color = function (chans, data, address) {
    if (!chans || !chans.length || !data) return null;
    const ch = TEATRO.channels({ channels: chans }, address, 1, { 1: data });
    const k = TEATRO.intensity(ch);
    return k ? TEATRO.color(ch).map(x => Math.round(x * k)) : null;
  };

  // ---- drawing ------------------------------------------------------------

  // An armed lane shows what COMES IN (bus topic 2, `TL.dmxIn`) in the live color; the others show
  // what goes out (topic 1, `TL.dmx`). The label says which of the two is on screen.
  // Universe and direction of the dmx column: an armed lane shows the input. Pure: this is what
  // the test covers.
  VW.source = function (sel) {
    const u = sel ? +(sel.spec.universe || 1) : 1, arm = !!(sel && sel.rec);
    return { u: u, arm: arm, lab: "dmx u" + u + (arm ? " in" : " out") };
  };

  function bars(c, b) {
    const { u, arm, lab } = VW.source(TL.lanes[TL.cur]);
    const d = (arm ? TL.dmxIn : TL.dmx).get(u);
    if (!d) return lab + "  no frame";
    const bw = b.w / 512;
    c.fillStyle = arm ? TL.col.live : TL.col.accent;
    for (let i = 0; i < 512; i++) {
      const v = d[i];
      if (v) c.fillRect(b.x + i * bw, b.y + b.h - v / 255 * b.h, Math.max(1, bw - 0.4), v / 255 * b.h);
    }
    return lab;
  }

  function frame(c, b, t) {
    const tracks = ((TL.show && TL.show.tracks) || []).filter(s => s.type === "laser" && s.clip);
    if (!tracks.length) return "no laser";
    const s = Math.min(b.w, b.h) / 2, cx = b.x + b.w / 2, cy = b.y + b.h / 2;
    let lab = "";
    for (const sp of tracks) {
      const i = VW.index(sp.clip, t, +sp.fps || TL.fps());
      const pts = VW.frame(sp.clip, i);
      lab = sp.clip + " #" + i;
      if (!pts) { lab += " ..."; continue; }
      // scale and rot as in the player: scale, rotate in degrees, and the ILDA Y grows upwards
      const k = VW.param(sp.scale, t, 1), a = VW.param(sp.rot, t, 0) * Math.PI / 180;
      const ca = Math.cos(a) * k * s, sa = Math.sin(a) * k * s;
      let cur = "", open = false, lx = 0, ly = 0;
      const close = () => { if (open) { c.strokeStyle = cur; c.stroke(); open = false; } };
      c.lineWidth = 1;
      for (let j = 0; j < pts.length; j++) {
        const p = pts[j], x = cx + p[0] * ca - p[1] * sa, y = cy - (p[0] * sa + p[1] * ca);
        const on = !p[5] && (p[2] | p[3] | p[4]);
        const q = on ? "rgb(" + p[2] + "," + p[3] + "," + p[4] + ")" : "";
        if (j && on) {
          if (q !== cur) { close(); cur = q; }
          if (!open) { c.beginPath(); c.moveTo(lx, ly); open = true; }
          c.lineTo(x, y);
        } else close();
        lx = x; ly = y;
      }
      close();
    }
    return lab;
  }

  function plan(c, b) {
    const ps = (TL.show && TL.show.patch) || [];
    if (!ps.length) return "no patch";
    // Grid in address order: the patch has no x/y (design/DECISOES.md, awaiting a vote).
    const fx = ps.slice().sort((x, y) =>
      (+x.universe || 1) - (+y.universe || 1) || (+x.address || 1) - (+y.address || 1));
    const cols = Math.ceil(Math.sqrt(fx.length)), rows = Math.ceil(fx.length / cols);
    const dx = b.w / cols, dy = b.h / rows, rad = Math.max(2, Math.min(dx, dy) / 2 - 3);
    for (let i = 0; i < fx.length; i++) {
      const f = fx[i];
      const x = b.x + dx * (i % cols) + dx / 2, y = b.y + dy * Math.floor(i / cols) + dy / 2;
      const rgb = VW.color(VW.profile(f.profile), TL.dmx.get(+f.universe || 1), +f.address || 1);
      c.beginPath();
      c.arc(x, y, rad, 0, Math.PI * 2);
      if (rgb) { c.fillStyle = "rgb(" + rgb + ")"; c.fill(); }
      c.strokeStyle = TL.col.fg3; c.lineWidth = 1; c.stroke();
    }
    return fx.length + " fixtures";
  }

  // `rect` = {x, y, w, h}: the strip the timeline reserves at the bottom of the canvas.
  VW.draw = function (c, r, t) {
    const col = TL.col, w = Math.floor(r.w / 3);
    c.fillStyle = col.panel;
    c.fillRect(r.x, r.y, r.w, r.h);
    c.strokeStyle = col.line; c.lineWidth = 1;
    c.beginPath();
    c.moveTo(r.x, r.y + 0.5); c.lineTo(r.x + r.w, r.y + 0.5);
    for (let i = 1; i < 3; i++) {
      c.moveTo(Math.round(r.x + i * w) + 0.5, r.y + 1);
      c.lineTo(Math.round(r.x + i * w) + 0.5, r.y + r.h);
    }
    c.stroke();
    c.font = "10px " + col.mono;          // textBaseline "middle" already comes from draw() in timeline.js
    const fs = [bars, frame, plan];
    for (let i = 0; i < 3; i++) {
      const b = { x: r.x + i * w + 8, y: r.y + 20, w: w - 16, h: r.h - 26 };
      c.save();
      c.beginPath();
      c.rect(b.x, b.y, b.w, b.h);
      c.clip();
      const lab = fs[i](c, b, t);
      c.restore();
      c.fillStyle = col.fg3;
      c.fillText(lab, r.x + i * w + 8, r.y + 10);
    }
  };

})(window, TEATRO);
