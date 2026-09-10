// GUI shell: WebSocket client (with reconnect), panel router, transport and visualizer.
// No product logic here: everything goes to App.rpc -> registry on the server.
"use strict";

const TOPIC_NAME = { 1: "dmx" };      // mirrors TOPICS in server.py
const BARS = 32, FPS = 30;
const el = {};                        // ids -> elements (getElementById: window.stop/status already exist)

const App = {
  ws: null, seq: 0, pending: new Map(), subs: {}, panels: {}, mounted: {}, cmds: new Set(),
  transport: { state: "stop", t: 0, dur: 60, loop: false },
  _anchor: 0,

  // ---- RPC -------------------------------------------------------------
  rpc(cmd, args) {
    if (!this.ws || this.ws.readyState !== 1) return Promise.reject(new Error("offline"));
    const id = ++this.seq;
    this.ws.send(JSON.stringify({ id, cmd, args: args || {} }));
    return new Promise((res, rej) => this.pending.set(id, { res, rej }));
  },
  on(topic, fn) { (this.subs[topic] = this.subs[topic] || []).push(fn); },
  emit(topic, ...a) { for (const fn of this.subs[topic] || []) fn(...a); },

  connect(delay = 500) {
    const ws = this.ws = new WebSocket((location.protocol === "https:" ? "wss://" : "ws://") + location.host + "/ws");
    ws.binaryType = "arraybuffer";
    ws.onopen = () => {
      this.online(true);
      this.rpc("schema").then(s => {
        for (const c of s) this.cmds.add(c.name);
        this.log("registry: " + s.length + " commands (" + s.map(c => c.name).join(", ") + ")");
      });
    };
    ws.onclose = () => {
      this.online(false);
      for (const p of this.pending.values()) p.rej(new Error("offline"));
      this.pending.clear();
      setTimeout(() => this.connect(Math.min(delay * 2, 5000)), delay);   // ponytail: backoff up to 5 s ; no attempt limit
    };
    ws.onmessage = e => {
      if (typeof e.data !== "string") return this.binary(e.data);
      const m = JSON.parse(e.data);
      if (m.topic !== undefined) {
        if (m.topic === "transport") Object.assign(this.transport, m.payload);
        return this.emit(m.topic, m.payload);
      }
      const p = this.pending.get(m.id);
      if (!p) return;
      this.pending.delete(m.id);
      if (m.error === undefined) p.res(m.result);
      else { this.log("error: " + m.error); p.rej(new Error(m.error)); }
    };
  },

  online(ok) {
    el.status.textContent = ok ? "online" : "offline";
    el.status.classList.toggle("on", ok);
    this.log(ok ? "ws connected" : "ws dropped; reconnecting");
  },

  binary(buf) {                                   // topic_id:u8 + universe:u16 + payload
    const head = new DataView(buf), topic = TOPIC_NAME[head.getUint8(0)], u = head.getUint16(1);
    const data = new Uint8Array(buf, 3);
    if (topic === "dmx") vu.feed(data);
    this.emit(topic, u, data);
  },

  // ---- transport -------------------------------------------------------
  now() {
    const tr = this.transport;
    return tr.state === "play" ? tr.t + (performance.now() - this._anchor) / 1000 : tr.t;
  },
  set(state, t) {
    const tr = this.transport;
    tr.t = t === undefined ? this.now() : t;
    tr.state = state;
    this._anchor = performance.now();
    el.play.classList.toggle("on", state === "play");
    if (this.cmds.has("transport")) this.rpc("transport", { state: state, t: tr.t }).catch(() => {});
  },
  play() { this.set(this.transport.state === "play" ? "pause" : "play"); },
  stop() { this.set("stop", 0); },
  locate(t) {
    this.transport.t = t;
    this._anchor = performance.now();
    if (this.cmds.has("locate")) this.rpc("locate", { t: t }).catch(() => {});
  },
  toggleLoop() {
    this.transport.loop = !this.transport.loop;
    el.loop.classList.toggle("on", this.transport.loop);
  },

  // ---- panels (contract described in index.html) -----------------------
  panel(name, def) { this.panels[name] = def; if ((location.hash.slice(1) || "timeline") === name) this.route(); },
  route() {
    const name = location.hash.slice(1) || "timeline";
    for (const s of document.querySelectorAll("#panels > section")) s.classList.toggle("on", s.dataset.panel === name);
    for (const a of el.tabs.children) a.classList.toggle("on", a.hash === "#" + name);
    const sec = document.querySelector('#panels > section[data-panel="' + name + '"]');
    if (sec && this.panels[name] && !this.mounted[name]) { this.mounted[name] = 1; this.panels[name].mount(sec); }
  },

  log(txt) {
    const p = el.log;
    p.textContent += new Date().toTimeString().slice(0, 8) + "  " + txt + "\n";
    if (p.textContent.length > 20000) p.textContent = p.textContent.slice(-16000);  // ponytail: cut by size ; ring buffer if the log ever becomes telemetry
  },
};

// ---- visualizer (draws on rAF, without allocating per frame) ------------
const vu = {
  cv: null, cx: null, accent: "#f2c300",
  lv: new Float32Array(BARS), peak: new Float32Array(BARS),
  feed(data) {                                    // 512 channels -> BARS bars (block peak)
    const n = (data.length / BARS) | 0 || 1;
    for (let b = 0; b < BARS; b++) {
      let m = 0;
      for (let i = b * n, e = Math.min(i + n, data.length); i < e; i++) if (data[i] > m) m = data[i];
      const v = m / 255;
      if (v > this.lv[b]) this.lv[b] = v;
    }
  },
  draw() {
    const mode = document.documentElement.dataset.visualizer;
    if (mode === "none") return;
    const c = this.cx, w = this.cv.width, h = this.cv.height, bw = w / BARS;
    c.clearRect(0, 0, w, h);
    if (mode === "scope") {
      c.strokeStyle = this.accent; c.lineWidth = 1.5;
      c.beginPath();
      for (let b = 0; b < BARS; b++) c.lineTo(b * bw + bw / 2, h - this.lv[b] * (h - 2) - 1);
      c.stroke();
    } else {
      c.fillStyle = this.accent;
      for (let b = 0; b < BARS; b++) {
        const bh = this.lv[b] * h;
        c.fillRect(b * bw + 1, h - bh, bw - 2, bh);
        c.fillRect(b * bw + 1, h - this.peak[b] * h - 1, bw - 2, 1);
      }
    }
    for (let b = 0; b < BARS; b++) {
      this.lv[b] *= 0.88;
      this.peak[b] = this.lv[b] > this.peak[b] ? this.lv[b] : this.peak[b] * 0.97;
    }
  },
};

// ---- single rAF: timecode, scrubber and VU ------------------------------
let lastFrame = -1, dragging = false;
const pad = n => (n < 10 ? "0" : "") + n;

function frame() {
  let t = App.now();
  if (App.transport.loop && t >= App.transport.dur) { App.locate(0); t = 0; }
  const f = Math.floor(t * FPS);
  if (f !== lastFrame) {
    lastFrame = f;
    el.tc.textContent = pad((t / 3600) | 0) + ":" + pad(((t / 60) | 0) % 60) + ":" + pad((t | 0) % 60) + ":" + pad(f % FPS);
    if (!dragging) el.scrub.value = t;
  }
  vu.draw();
  requestAnimationFrame(frame);
}

// ---- boot ---------------------------------------------------------------
addEventListener("DOMContentLoaded", () => {
  for (const id of ["status", "tabs", "log", "play", "stop", "loop", "tc", "scrub", "vu", "skin"]) el[id] = document.getElementById(id);
  vu.cv = el.vu; vu.cx = el.vu.getContext("2d");
  const readAccent = () => vu.accent = getComputedStyle(document.documentElement).getPropertyValue("--accent").trim() || "#f2c300";

  document.addEventListener("skin", () => { readAccent(); App.log("skin " + Skins.current); });
  Skins.menu(el.skin).then(readAccent);

  addEventListener("hashchange", () => App.route());
  App.route();

  el.play.onclick = () => App.play();
  el.stop.onclick = () => App.stop();
  el.loop.onclick = () => App.toggleLoop();
  el.scrub.max = App.transport.dur;
  el.scrub.oninput = () => { dragging = true; App.locate(+el.scrub.value); };
  el.scrub.onchange = () => { dragging = false; };

  addEventListener("keydown", e => {
    if (/^(INPUT|SELECT|TEXTAREA)$/.test(e.target.tagName) || e.ctrlKey || e.altKey) return;
    if (e.code === "Space") { e.preventDefault(); App.play(); }
    else if (e.code === "Home") { e.preventDefault(); App.locate(0); }
    else if (e.code === "KeyL") { App.toggleLoop(); }
  });

  App.connect();
  requestAnimationFrame(frame);
});
