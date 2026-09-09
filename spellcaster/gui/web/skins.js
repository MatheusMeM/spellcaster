// Skins: skins/<nome>/skin.json (vars + cromo) + skin.css (decoracao). Troca em runtime, sem reload,
// persistida em localStorage. Inspiracao: .wmz do Windows Media Player (zip com skin.xml, <THEME>/<VIEW>/
// <BUTTONGROUP>, bitmaps com mapa de cliques) -> aqui e so CSS: variaveis + um arquivo de decoracao.
"use strict";

const Skins = {
  list: ["feiticaria", "corporate", "headspace", "bluesky", "quicksilver", "xp"],
  // ponytail: lista fixa no JS ; ler skins/index.json quando skins virarem instalaveis pelo usuario
  DEFAULT: "feiticaria",
  current: null,

  async load(name) {
    const r = await fetch("skins/" + name + "/skin.json");
    if (!r.ok) throw new Error("skin " + name + ": " + r.status);
    const skin = await r.json();
    const root = document.documentElement;
    root.removeAttribute("style");                      // limpa as vars da skin anterior
    for (const [k, v] of Object.entries(skin.vars || {})) root.style.setProperty(k, v);
    const chrome = skin.chrome || {};
    root.dataset.transport = chrome.transport || "square";
    root.dataset.visualizer = chrome.visualizer || "bars";
    root.dataset.skin = name;
    document.getElementById("skin-css").href = "skins/" + name + "/skin.css";   // troca o <link>: sem reload
    const sel = document.getElementById("skin");
    if (sel && sel.value !== name) sel.value = name;
    this.current = name;
    try { localStorage.skin = name; } catch (e) {}
    document.dispatchEvent(new CustomEvent("skin", { detail: skin }));
    return skin;
  },

  menu(sel) {
    for (const n of this.list) sel.add(new Option(n, n));
    let saved = this.DEFAULT;
    try { saved = localStorage.skin || saved; } catch (e) {}
    sel.value = this.list.includes(saved) ? saved : this.DEFAULT;
    sel.onchange = () => this.load(sel.value);
    return this.load(sel.value);
  },
};
