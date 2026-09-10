// Skins: skins/<name>/skin.json (vars + chrome) + skin.css (decoration). Swapped at runtime, without a reload,
// persisted in localStorage. Inspiration: the Windows Media Player .wmz (a zip with skin.xml, <THEME>/<VIEW>/
// <BUTTONGROUP>, bitmaps with a click map) -> here it is CSS only: variables plus one decoration file.
"use strict";

const Skins = {
  list: ["feiticaria", "corporate", "headspace", "bluesky", "quicksilver", "xp"],
  // ponytail: fixed list in the JS ; read skins/index.json once skins become user-installable
  DEFAULT: "feiticaria",
  current: null,

  async load(name) {
    const r = await fetch("skins/" + name + "/skin.json");
    if (!r.ok) throw new Error("skin " + name + ": " + r.status);
    const skin = await r.json();
    const root = document.documentElement;
    root.removeAttribute("style");                      // clears the vars of the previous skin
    for (const [k, v] of Object.entries(skin.vars || {})) root.style.setProperty(k, v);
    const chrome = skin.chrome || {};
    root.dataset.transport = chrome.transport || "square";
    root.dataset.visualizer = chrome.visualizer || "bars";
    root.dataset.skin = name;
    document.getElementById("skin-css").href = "skins/" + name + "/skin.css";   // swaps the <link>: no reload
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
