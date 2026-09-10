"use strict";
// face.js — Face runtime (PRD §10): `faces/<name>.face.json` declares which widgets exist, where
// and in which view; the behaviour lives in the Graph, in the engine. The page only draws state
// and sends events — there is no second runtime here.
//
// A widget fires through one of these two paths, never through logic of its own:
//   {"cmd": "cue_go", "args": {...}}   -> Registry::call over the bus
//   {"input": "widget:blackout"}       -> `input {key, value}` command, which feeds the Graph
//     (`in.widget` listens on the key "widget:<id>")
// The way back is the `widget` event of the bus, emitted by the Graph `out.widget`:
//   {"event":"widget","data":{"id":"go","prop":"glow","value":1}}
//
// ponytail: widget catalogue reduced to button, toggle, fader and label ; the rest of the PRD §10
// catalogue (cuelist, meter, universes, timecode, transport) comes with the Face editor (R9).

const Face = {};

/// The requested view, or the first declared one. `views` is required in the .face.json.
Face.pick = function (face, name) {
  const vs = face.views;
  const k = name && vs[name] ? name : Object.keys(vs)[0];
  return Object.assign({ name: k }, vs[k]);
};

/// Widgets of the view, in the order the view declares.
Face.widgets = function (face, view) {
  const by = {};
  for (const w of face.widgets || []) by[w.id] = w;
  return (view.widgets || Object.keys(by)).map(id => by[id]).filter(Boolean);
};

/// What a touch on the widget sends to the bus. Pure: without it there would be nothing to test.
/// value only matters for toggle/fader; a button is always 1 (the `in.widget` pulse).
Face.action = function (w, value) {
  const v = value === undefined ? 1 : +value;
  if (w.input) return { input: w.input, value: v };
  if (w.cmd) return { cmd: w.cmd, args: Object.assign({}, w.args) };
  return null;
};

// ---- DOM ----------------------------------------------------------------
function el(tag, cls, txt) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (txt !== undefined) e.textContent = txt;
  return e;
}

/// The prop coming from `out.widget` is the class name; the page decides what each one paints.
Face.applyProp = function (node, prop, value) {
  // `prop` comes from the network: `classList.toggle` THROWS with an empty name or with a space,
  // and the event arriving inside `emit` would take down the following subscribers.
  if (!/^[\w-]+$/.test(prop)) return;
  node.classList.toggle(prop, +value !== 0);
};

/// Builds the view in the host. Returns {view, nodes} so the caller can switch view.
Face.build = function (host, face, view, bus) {
  const [cols, rows] = view.grid;
  host.innerHTML = "";
  host.style.gridTemplateColumns = "repeat(" + cols + ", 1fr)";
  host.style.gridTemplateRows = "repeat(" + rows + ", 1fr)";
  const nodes = {};
  const fire = (w, value) => {
    const a = Face.action(w, value);
    if (!a) return;
    const p = a.input ? bus.input(a.input, a.value) : bus.call(a.cmd, a.args);
    p.catch(e => bus.emit("log", { text: w.id + ": " + e.message }));
  };
  for (const w of Face.widgets(face, view)) {
    const t = w.type || "button";
    const node = el("div", "w w-" + t + (w.tone ? " tone-" + w.tone : ""));
    // the view can rearrange without duplicating the widget (PRD §10: same Face, different layouts)
    const at = (view.at && view.at[w.id]) || w.at || [0, 0, 1, 1];
    node.style.gridColumn = at[0] + 1 + " / span " + (at[2] || 1);
    node.style.gridRow = at[1] + 1 + " / span " + (at[3] || 1);
    node.appendChild(el("span", "w-lab", w.label || w.id));
    if (t === "fader") {
      const r = el("input", "w-fader");
      Object.assign(r, {
        type: "range",
        step: "any",
        min: w.min ?? 0,
        max: w.max ?? 1,
        value: w.value ?? 0,
      });
      r.oninput = () => fire(w, r.value);
      node.appendChild(r);
    } else {
      node.tabIndex = 0;
      const press = ev => {
        ev.preventDefault();
        if (t === "toggle") {
          node.classList.toggle("on");
          fire(w, node.classList.contains("on") ? 1 : 0);
        } else {
          node.classList.add("hit");
          setTimeout(() => node.classList.remove("hit"), 120);
          fire(w, 1);
        }
      };
      node.onpointerdown = press;
      node.onkeydown = e => {
        if (e.key === "Enter" || e.key === " ") press(e);
      };
    }
    nodes[w.id] = node;
    host.appendChild(node);
  }
  return { view, nodes };
};

/// Loads the face, builds it and wires the bus events. `opts.view` picks the view.
Face.mount = function (host, url, bus, opts) {
  opts = opts || {};
  return fetch(url)
    .then(r => {
      if (!r.ok) throw new Error(url + ": " + r.status);
      return r.json();
    })
    .then(face => {
      const st = Face.build(host, face, Face.pick(face, opts.view), bus);
      bus.on("widget", d => {
        const n = d && st.nodes[d.id];
        if (n) Face.applyProp(n, d.prop || "glow", d.value);
      });
      return st;
    });
};

if (typeof window !== "undefined") window.Face = Face;
if (typeof module !== "undefined") module.exports = Face;
