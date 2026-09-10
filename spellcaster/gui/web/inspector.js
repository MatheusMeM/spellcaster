// Inspector: properties of the selected track and keyframe, with numeric editing.
// It stays docked in the right-hand column of the timeline (it is not a tab panel: whoever edits wants
// to see the timeline at the same time). It reads and writes the local TL model and asks for the commit there.
"use strict";

const Inspector = {
  el: null, box: null,

  mount(el) {
    this.el = el;
    el.innerHTML = '<h3>Inspector</h3><div class="insp-body" id="insp-body"></div>';
    this.box = el.querySelector("#insp-body");
    this.update();
  },

  row(label, input) {
    const d = document.createElement("label");
    d.className = "insp-row";
    d.innerHTML = "<span>" + label + "</span>";
    d.appendChild(input);
    this.box.appendChild(d);
    return input;
  },

  input(value, on, type) {
    const i = document.createElement("input");
    i.type = type || "text";
    i.value = value;
    i.onchange = () => on(i.value);
    return i;
  },

  sel(options, value, on) {
    const s = document.createElement("select");
    for (const o of options) s.add(new Option(o, o));
    s.value = value;
    s.onchange = () => on(s.value);
    return s;
  },

  head(txt) {
    const h = document.createElement("h4");
    h.textContent = txt;
    this.box.appendChild(h);
  },

  update() {
    if (!this.box) return;
    const L = TL.lanes[TL.cur];
    this.box.textContent = "";
    if (!L) { this.box.innerHTML = '<p class="insp-none">nothing selected</p>'; return; }
    const sp = L.spec;

    this.head("Track " + L.si + (L.param ? " . " + L.param : ""));
    this.row("name", this.input(sp.name || "", v => { sp.name = v; L.name = v + (L.param ? "." + L.param : ""); TL.commit(); }));
    this.row("type", this.input(sp.type || "", v => { sp.type = L.type = v; TL.commit(); }));
    this.row("universe", this.input(sp.universe === undefined ? 1 : sp.universe, v => { sp.universe = +v || 1; TL.commit(); }, "number"));
    this.row("address", this.input(sp.address === undefined ? 1 : sp.address, v => { sp.address = isNaN(+v) ? v : +v; TL.commit(); }));
    this.row("scale", this.input(L.vmax, v => { L.vmax = +v || 255; TL.dirty = true; }, "number"));
    const mute = this.row("mute", this.input("", () => {}, "checkbox"));
    mute.checked = L.mute;
    mute.onchange = () => { L.mute = mute.checked; TL.commit(); };
    const solo = this.row("solo", this.input("", () => {}, "checkbox"));
    solo.checked = L.solo;
    solo.onchange = () => { L.solo = solo.checked; TL.commit(); };

    const n = TL.selCount();
    if (!n) { this.head("Keyframe"); this.box.insertAdjacentHTML("beforeend", '<p class="insp-none">no keyframe</p>'); return; }
    if (n > 1) {
      this.head(n + " keyframes");
      this.row("curve", this.sel(["linear", "hold", "in", "out", "inout", "bezier"], "linear", v => {
        for (const [li, ks] of TL.sel) for (const i of ks) TL.lanes[li].cu[i] = CURVES.indexOf(v);
        TL.commit();
      }));
      return;
    }
    let li, ki;
    for (const [l, ks] of TL.sel) for (const i of ks) { li = l; ki = i; }
    const K = TL.lanes[li];
    this.head("Keyframe " + ki + " / " + K.n);
    this.row("t (s)", this.input(Math.round(K.ts[ki] * 1e4) / 1e4, v => { K.ts[ki] = Math.max(0, +v || 0); TL.resort(li); TL.commit(); }, "number"));
    this.row("value", this.input(K.raw[ki] !== null ? JSON.stringify(K.raw[ki]) : K.vs[ki], v => {
      let parsed;
      try { parsed = JSON.parse(v); } catch (e) { parsed = v; }
      if (typeof parsed === "number") { K.vs[ki] = parsed; K.raw[ki] = null; }
      else { K.raw[ki] = parsed; K.vs[ki] = Array.isArray(parsed) && typeof parsed[0] === "number" ? parsed[0] : 0; }
      TL.commit();
    }));
    this.row("curve", this.sel(["linear", "hold", "in", "out", "inout", "bezier"], CURVES[K.cu[ki]], v => {
      K.cu[ki] = CURVES.indexOf(v);
      TL.commit();
    }));
  },
};
window.Inspector = Inspector;
