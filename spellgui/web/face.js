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

/// View pedida ou a primeira declarada. `views` e' obrigatorio no .face.json.
Face.pick = function (face, name) {
  const vs = face.views;
  const k = name && vs[name] ? name : Object.keys(vs)[0];
  return Object.assign({ name: k }, vs[k]);
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

// ---- DOM ----------------------------------------------------------------
function el(tag, cls, txt) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (txt !== undefined) e.textContent = txt;
  return e;
}

/// Prop vinda do `out.widget` e' o nome da classe; a pagina decide o que cada uma pinta.
Face.applyProp = function (node, prop, value) {
  node.classList.toggle(prop, +value !== 0);
};

/// Monta a view no host. Devolve {view, nodes} para o chamador trocar de view.
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
    // a view pode remanejar sem duplicar o widget (PRD §10: mesma Face, arranjos diferentes)
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
      return st;
    });
};

if (typeof window !== "undefined") window.Face = Face;
if (typeof module !== "undefined") module.exports = Face;
