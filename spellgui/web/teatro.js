// PAPER THEATER: patch, cues and the clickable stage. A bus client (`spellcore serve`): everything
// the page does is a `Registry::call` over WebSocket — no show rule lives here.
//
// The top half of the file is pure (DMX frame + patch + profile -> color and intensity) and runs
// under `node --test`; the bottom half is DOM and only exists in the browser.
"use strict";

// --------------------------------------------------------------- pure part

// Named channels of a fixture: {profile name: value 0..255}.
// `dmx` = {universe: array of 512 bytes}; a channel outside what arrived counts as 0.
function channels(profile, base, universe, dmx) {
  const buf = (dmx && dmx[universe]) || [];
  const out = {};
  for (const c of (profile && profile.channels) || []) {
    if (typeof c.offset !== "number") continue;
    out[c.name] = buf[base - 1 + c.offset] || 0;
  }
  return out;
}

// ponytail: color comes from r/g/b/w ; color wheel (`wheel`) and gel become color once the profile
// declares the channel type (design/FUNCOES/cenas-cues-dmx.md section 1, "Profile").
function color(ch) {
  const v = k => ch[k] || 0;
  const w = v("w");
  const has = "r" in ch || "g" in ch || "b" in ch;
  if (!has) return [255, 255, 255]; // dimmer, ellipsoidal: white lamp
  return [v("r") + w, v("g") + w, v("b") + w].map(x => Math.min(255, x));
}

// Intensity 0..1: the dimmer rules; with no dimmer, the brightest of the color channels; failing
// that, `on`.
function intensity(ch) {
  if ("dim" in ch) return ch.dim / 255;
  const c = ["r", "g", "b", "w"].filter(k => k in ch).map(k => ch[k]);
  if (c.length) return Math.max(...c) / 255;
  if ("on" in ch) return ch.on / 255;
  return 0;
}

// What each patch fixture is showing RIGHT NOW, from what went out on the network.
// `patch` = [{name, profile, universe, address, pos}], `profiles` = {file name: profile}.
// `pos` already comes resolved: whoever was never dragged falls into a row, like a light bar.
function look(patch, profiles, dmx) {
  const n = (patch || []).length;
  return (patch || []).map((f, i) => {
    const p = profiles[f.profile] || null;
    const u = f.universe || 1;
    const ch = channels(p, f.address || 1, u, dmx);
    return {
      i,
      name: f.name,
      profile: f.profile,
      universe: u,
      address: f.address || 1,
      pos: Array.isArray(f.pos) ? f.pos : [0.08 + (0.84 * (i + 0.5)) / n, 0.3],
      channels: ch,
      rgb: color(ch),
      intensity: intensity(ch),
    };
  });
}

const TEATRO = { channels, color, intensity, look };
if (typeof module !== "undefined") module.exports = TEATRO;

// --------------------------------------------------------------------- bus

// ponytail: a 40-line WS client ; swap it for `bus.js` (F5) once that exists — same
// `call(cmd, args)` / `on(event, fn)` signature.
TEATRO.bus = {
  ws: null,
  n: 0,
  pend: new Map(),
  ev: {},
  dmx: {},
  rev: 0, // last revision seen in a response: a `show` event with a lower rev is our own echo
  on(e, f) { (this.ev[e] = this.ev[e] || []).push(f); },
  emit(e, d) { for (const f of this.ev[e] || []) f(d); },
  open() {
    // ponytail: file:// has no host and the WebSocket throws ; the page stays mounted, only with no bus.
    let ws;
    try {
      ws = new WebSocket(`ws://${location.host}/ws`);
    } catch (e) {
      return this.emit("close");
    }
    ws.binaryType = "arraybuffer";
    this.ws = ws;
    ws.onopen = () => this.emit("open");
    ws.onclose = () => { this.emit("close"); setTimeout(() => this.open(), 2000); };
    ws.onmessage = m => {
      if (typeof m.data !== "string") return this.frame(m.data);
      const v = JSON.parse(m.data);
      if (v.event) return this.emit(v.event, v.data);
      if (typeof v.rev === "number") this.rev = Math.max(this.rev, v.rev);
      const p = this.pend.get(v.id);
      if (!p) return;
      this.pend.delete(v.id);
      v.error === undefined ? p.ok(v.result) : p.no(new Error(v.error));
    };
  },
  // topic:u8 | universe:u16 LE | 512 bytes  (topic 1 = output dmx)
  frame(buf) {
    const d = new DataView(buf);
    if (buf.byteLength < 515 || d.getUint8(0) !== 1) return;
    this.dmx[d.getUint16(1, true)] = new Uint8Array(buf, 3, 512);
    this.emit("dmx");
  },
  call(cmd, args) {
    if (!this.ws || this.ws.readyState !== 1) return Promise.reject(new Error("no bus"));
    const id = ++this.n;
    this.ws.send(JSON.stringify({ id, cmd, args: args || {} }));
    return new Promise((ok, no) => this.pend.set(id, { ok, no }));
  },
};

// -------------------------------------------------------------------- page

TEATRO.mount = function (el) {
  const bus = TEATRO.bus;
  const q = id => el.querySelector("#" + id);
  const st = {
    show: { patch: [], cues: [] },   // the whole show (show_get full)
    rows: [],                        // patch_check rows
    error: null,                     // patch_check error (on the row rows.length)
    profiles: {},                    // file name -> profile (profile_get)
    cue: -1,                         // current transport cue
    sel: -1,                         // fixture open on the stage
    drag: null,
  };
  const say = t => { q("msg").textContent = t; };
  const err = e => say(String((e && e.message) || e));

  // -------- loading
  async function reload() {
    st.show = await bus.call("show_get", { full: true });
    st.show.patch = st.show.patch || [];
    st.show.cues = st.show.cues || [];
    const g = await bus.call("patch_check", {});
    st.rows = g.rows;
    st.error = g.error;
    for (const f of st.show.patch) {
      if (f.profile && !(f.profile in st.profiles)) {
        st.profiles[f.profile] = await bus.call("profile_get", { name: f.profile }).catch(() => null);
      }
    }
    patch();
    cues();
  }

  // -------- patch
  function patch() {
    const t = q("patch");
    t.innerHTML = "";
    st.show.patch.forEach((f, i) => {
      const r = st.rows[i];
      const tr = document.createElement("tr");
      const cols = r
        ? [r.name, r.profile, r.universe, r.address, r.channels]
        : [f.name, f.profile, f.universe || 1, f.address || 1, "?"];
      for (const c of cols) {
        const td = document.createElement("td");
        td.textContent = c;
        tr.appendChild(td);
      }
      const td = document.createElement("td");
      const b = document.createElement("button");
      b.textContent = "x";
      b.title = "take out of the patch";
      b.onclick = () => bus.call("patch_del", { name: f.name }).then(reload, err);
      td.appendChild(b);
      tr.appendChild(td);
      if (i === st.rows.length && st.error) {
        tr.className = "alerta";
        tr.title = st.error;
      }
      tr.onclick = () => open(i);
      t.appendChild(tr);
    });
    q("patch-erro").textContent = st.error || "";
  }

  q("add").onclick = () => {
    bus
      .call("patch_add", {
        name: q("f-nome").value,
        profile: q("f-perfil").value,
        universe: +q("f-uni").value,
        address: +q("f-end").value,
      })
      .then(reload, e => { err(e); q("patch-erro").textContent = e.message; });
  };

  // -------- cues
  function cues() {
    const t = q("cues");
    t.innerHTML = "";
    st.show.cues.forEach((c, i) => {
      const tr = document.createElement("tr");
      if (i === st.cue) tr.className = "viva";
      const field = (k, type) => {
        const td = document.createElement("td");
        const inp = document.createElement("input");
        inp.type = type;
        if (type === "checkbox") inp.checked = !!c[k];
        else inp.value = c[k] === undefined ? "" : c[k];
        inp.onchange = () => {
          const v = type === "checkbox" ? inp.checked : type === "number" ? +inp.value : inp.value;
          bus
            .call("cue_set", {
              index: i,
              name: c.name || "",
              fade: c.fade || 0,
              wait: c.wait || 0,
              follow: !!c.follow,
              values: c.values || {},
              [k]: v,
            })
            .then(reload, err);
        };
        td.appendChild(inp);
        tr.appendChild(td);
      };
      const n = document.createElement("td");
      n.textContent = i;
      tr.appendChild(n);
      field("name", "text");
      field("fade", "number");
      field("wait", "number");
      field("follow", "checkbox");
      const nv = document.createElement("td");
      nv.textContent = Object.keys(c.values || {}).length + " ch";
      tr.appendChild(nv);
      const td = document.createElement("td");
      const b = document.createElement("button");
      b.textContent = "x";
      b.onclick = e => { e.stopPropagation(); bus.call("cue_del", { index: i }).then(reload, err); };
      td.appendChild(b);
      tr.appendChild(td);
      tr.ondblclick = () => bus.call("cue_go", { index: i }).catch(err);
      t.appendChild(tr);
    });
  }

  q("go").onclick = () => bus.call("cue_go", {}).then(() => say("GO"), err);
  q("capturar").onclick = () =>
    bus.call("cue_capture", { name: q("cena-nome").value || "scene" }).then(reload, err);
  q("solta").onclick = () => bus.call("level_clear", {}).then(n => say(n + " ch released"), err);

  // -------- stage
  const cv = q("cenario");
  const ctx = cv.getContext("2d");

  function draw() {
    const w = (cv.width = cv.clientWidth);
    const h = (cv.height = cv.clientHeight);
    const col = CK.colors(cv);
    ctx.fillStyle = col.well;
    ctx.fillRect(0, 0, w, h);
    ctx.strokeStyle = col.hair;
    ctx.beginPath();
    ctx.moveTo(0, h * 0.3);
    ctx.lineTo(w, h * 0.3);
    ctx.stroke();
    const fs = look(st.show.patch, st.profiles, bus.dmx);
    fs.forEach(f => {
      const [x, y] = f.pos;
      const [px, py] = [x * w, y * h];
      const [r, g, b] = f.rgb;
      const a = f.intensity;
      if (a > 0.01) {
        const grd = ctx.createRadialGradient(px, py, 4, px, py, 46);
        grd.addColorStop(0, `rgba(${r},${g},${b},${0.55 * a})`);
        grd.addColorStop(1, "rgba(0,0,0,0)");
        ctx.fillStyle = grd;
        ctx.fillRect(px - 46, py - 46, 92, 92);
      }
      ctx.beginPath();
      ctx.arc(px, py, 11, 0, 6.2832);
      ctx.fillStyle = `rgba(${r},${g},${b},${0.12 + 0.88 * a})`;
      ctx.fill();
      ctx.lineWidth = f.i === st.sel ? 2 : 1;
      ctx.strokeStyle = f.i === st.sel
        ? col.accent
        : col.fg3;
      ctx.stroke();
      ctx.fillStyle = col.fg3;
      ctx.font = "10px monospace";
      ctx.textAlign = "center";
      ctx.fillText(f.name || "?", px, py + 26);
    });
  }

  function find(e) {
    const b = cv.getBoundingClientRect();
    const p = [(e.clientX - b.left) / b.width, (e.clientY - b.top) / b.height];
    const fs = look(st.show.patch, st.profiles, bus.dmx);
    let best = -1;
    let d = 0.04;
    fs.forEach(f => {
      const [x, y] = f.pos;
      const dd = Math.hypot((x - p[0]) * b.width, (y - p[1]) * b.height);
      if (dd < d * b.width) { d = dd / b.width; best = f.i; }
    });
    return [best, p];
  }

  cv.onpointerdown = e => {
    const [i, p] = find(e);
    if (i < 0) return;
    st.drag = { i, p, moved: false };
    cv.setPointerCapture(e.pointerId);
  };
  cv.onpointermove = e => {
    if (!st.drag) return;
    const b = cv.getBoundingClientRect();
    const p = [(e.clientX - b.left) / b.width, (e.clientY - b.top) / b.height];
    st.drag.moved = st.drag.moved || Math.hypot(p[0] - st.drag.p[0], p[1] - st.drag.p[1]) > 0.005;
    st.show.patch[st.drag.i].pos = p;
    draw();
  };
  cv.onpointerup = () => {
    const d = st.drag;
    st.drag = null;
    if (!d) return;
    if (!d.moved) return open(d.i);
    // the position belongs to the show: it goes through show_patch (JSON Patch), the only path for
    // a partial edit
    bus
      .call("show_patch", {
        ops: [{ op: "add", path: `/patch/${d.i}/pos`, value: st.show.patch[d.i].pos }],
      })
      .catch(err);
  };

  // click on the fixture: the profile channels become widgets -> fixture_set
  async function open(i) {
    st.sel = i;
    const f = st.show.patch[i];
    const box = q("canais");
    box.innerHTML = "";
    if (!f) return;
    if (!st.profiles[f.profile]) {
      st.profiles[f.profile] = await bus.call("profile_get", { name: f.profile }).catch(() => null);
    }
    const p = st.profiles[f.profile];
    const head = document.createElement("div");
    head.className = "over";
    head.textContent = `${f.name} — ${(p && p.name) || f.profile} — u${f.universe || 1}/${f.address || 1}`;
    box.appendChild(head);
    const live = channels(p, f.address || 1, f.universe || 1, bus.dmx);
    for (const c of (p && p.channels) || []) {
      const l = document.createElement("label");
      l.textContent = c.name;
      const s = document.createElement("input");
      s.type = "range";
      s.min = 0;
      s.max = 255;
      s.value = live[c.name] || 0;
      const n = document.createElement("span");
      n.textContent = s.value;
      s.oninput = () => {
        n.textContent = s.value;
        bus.call("fixture_set", { name: f.name, channel: c.name, value: +s.value }).catch(err);
      };
      l.appendChild(s);
      l.appendChild(n);
      box.appendChild(l);
    }
    draw();
  }

  // -------- live bus
  bus.on("open", () => {
    say("connected");
    bus.call("profiles", {}).then(v => {
      q("f-perfil").innerHTML = "";
      for (const n of v) {
        const o = document.createElement("option");
        o.value = o.textContent = n;
        q("f-perfil").appendChild(o);
      }
    }, err);
    reload().catch(err);
  });
  bus.on("close", () => say("no bus — spellcore serve"));
  // the `show` only goes out when the engine `rev` changes; `bus.rev` has already risen in the
  // response of our own edit, so it only reloads what came from another client.
  bus.on("show", d => { if (!d || d.rev > bus.rev) reload().catch(err); });
  bus.on("transport", d => {
    if (d.cue !== st.cue) { st.cue = d.cue; cues(); }
    q("cue-viva").textContent = st.cue < 0 ? "—" : `${st.cue} ${(st.show.cues[st.cue] || {}).name || ""}`;
  });
  bus.on("dmx", draw);

  addEventListener("resize", draw);
  addEventListener("keydown", e => {
    if (e.key === "Enter" && e.target.tagName !== "INPUT") q("go").click();
  });
  draw();
  bus.open();
};
