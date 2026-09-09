"use strict";
// face.js — runtime da Face (PRD §10): `faces/<nome>.face.json` declara quais widgets existem,
// onde e em que view; o comportamento vive no Graph, no engine. A pagina so' desenha estado e
// manda evento — nao ha segundo runtime aqui.
//
// Um widget liga por um destes dois caminhos, nunca por logica propria:
//   {"cmd": "cue_go", "args": {...}}   -> Registry::call pelo barramento
//   {"input": "widget:blackout"}       -> comando `input {key, value}`, que alimenta o Graph
//     (`in.widget` escuta a chave "widget:<id>")
// O caminho de volta e' o evento `widget` do barramento, que o `out.widget` do Graph emite:
//   {"event":"widget","data":{"id":"go","prop":"glow","value":1}}
//
// ponytail: catalogo de widget reduzido a button, toggle, fader e label ; o resto do catalogo
// do PRD §10 (cuelist, meter, universes, timecode, transport) entra com o editor de Face (R9).

const Face = {};

/// View pedida, a primeira declarada, ou uma view sintetica com todos os widgets.
Face.pick = function (face, name) {
  const vs = (face && face.views) || {};
  const k = name && vs[name] ? name : Object.keys(vs)[0];
  if (k) return Object.assign({ name: k }, vs[k]);
  return { name: "full", grid: "4x2", widgets: (face.widgets || []).map(w => w.id) };
};

/// Widgets da view, na ordem declarada por ela.
Face.widgets = function (face, view) {
  const by = {};
  for (const w of face.widgets || []) by[w.id] = w;
  return (view.widgets || Object.keys(by)).map(id => by[id]).filter(Boolean);
};

/// O que um toque no widget manda para o barramento. Puro: sem isto nao ha o que testar.
/// value so' importa para toggle/fader; botao e' sempre 1 (o pulso do `in.widget`).
Face.action = function (w, value) {
  const v = value === undefined ? 1 : +value;
  if (w.input) return { input: w.input, value: v };
  if (w.cmd) return { cmd: w.cmd, args: Object.assign({}, w.args) };
  return null;
};

/// "4x2" -> [4, 2]
Face.grid = function (g) {
  const m = /^(\d+)\s*[xX]\s*(\d+)$/.exec(String(g || ""));
  return m ? [+m[1], +m[2]] : [4, 2];
};

// ---- DOM ----------------------------------------------------------------
function el(tag, cls, txt) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (txt !== undefined) e.textContent = txt;
  return e;
}

/// Prop vinda do `out.widget`. O valor e' f64: prop de texto nao existe aqui de proposito.
Face.applyProp = function (node, prop, value) {
  const on = +value !== 0;
  if (prop === "glow" || prop === "on" || prop === "press") node.classList.toggle("on", on);
  else if (prop === "alert" || prop === "live") node.classList.toggle("alert", on);
  else if (prop === "enabled") node.classList.toggle("off", !on);
  else if (prop === "level") node.style.setProperty("--v", String(value));
  else node.dataset[prop] = String(value);
};

/// Monta a view no host. Devolve {view, nodes} para o chamador trocar de view.
Face.build = function (host, face, view, bus) {
  const [cols, rows] = Face.grid(view.grid);
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
    // a view pode remanejar sem duplicar o widget (PRD §10: mesma Face, arranjos diferentes)
    const at = (view.at && view.at[w.id]) || w.at || [0, 0, 1, 1];
    node.style.gridColumn = at[0] + 1 + " / span " + (at[2] || 1);
    node.style.gridRow = at[1] + 1 + " / span " + (at[3] || 1);
    node.appendChild(el("span", "w-lab", w.label || w.id));
    if (t === "fader") {
      const r = el("input", "w-fader");
      r.type = "range";
      r.min = w.min === undefined ? 0 : w.min;
      r.max = w.max === undefined ? 1 : w.max;
      r.step = "any";
      r.value = w.value === undefined ? 0 : w.value;
      r.oninput = () => fire(w, r.value);
      node.appendChild(r);
    } else if (t === "label") {
      // so' mostra: valor chega por out.widget em data-*
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

/// Carrega a face, monta e liga os eventos do barramento. `opts.view` escolhe a view.
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
      st.face = face;
      return st;
    });
};

if (typeof window !== "undefined") window.Face = Face;
if (typeof module !== "undefined") module.exports = Face;
