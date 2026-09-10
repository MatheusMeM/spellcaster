"use strict";
// nav.js — the bar that links the six pages and shows the open show. Every page includes it with
// two lines at the top of the <body>:
//
//   <div id="nav"></div>
//   <script src="nav.js"></script>
//
// Tabs TIMELINE / PATCHBAY / THEATER / FACE / LASER / HELP, `Shift+1`..`Shift+6`
// (design/SHORTCUTS.md, "Foco de painel"), ENGINE/OFFLINE indicator with the show `rev`, and the
// editable show name. The bar also loads `help.js`: that is what binds the `?` key on every
// page, and not only on the help page.
// Color and font only from design/tokens/spellcaster.css.

const NAV = {};

NAV.PAGES = [
  { lab: "TIMELINE", href: "index.html" },
  { lab: "PATCHBAY", href: "patchbay.html" },
  { lab: "THEATER", href: "teatro.html" },
  { lab: "FACE", href: "face.html?face=quatro" },
  { lab: "LASER", href: "laser.html" },
  { lab: "HELP", href: "help.html" },
];

// ---- rules (the rest is DOM) --------------------------------------------

/// File name of a path or href, without directory or query; the root is the index.
NAV.file = function (p) {
  return String(p || "").split("?")[0].split("/").pop() || "index.html";
};

/// The tabs, with the current one marked.
NAV.tabs = function (pathname) {
  const here = NAV.file(pathname);
  return NAV.PAGES.map(p => ({
    lab: p.lab,
    href: p.href,
    current: NAV.file(p.href) === here,
  }));
};

/// `Shift+1`..`Shift+6` -> href, or null. While typing in a field no key navigates: the show
/// name has digits. What counts is the key `code`, not the `key`: on an ABNT2 keyboard Shift+2
/// writes `"` and Shift+3 writes `#`. The `key` is only the fallback, for layouts where Shift
/// keeps the digit (and for synthetic events, which usually come without `code`).
NAV.target = function (e) {
  const t = e.target || {};
  if (t.tagName === "INPUT" || t.tagName === "TEXTAREA" || t.isContentEditable) return null;
  if (!e.shiftKey || e.ctrlKey || e.altKey || e.metaKey) return null;
  const p = NAV.PAGES.find((_, n) => e.code === "Digit" + (n + 1) || e.key === String(n + 1));
  return p ? p.href : null;
};

/// The `show_patch` for the name, or null when there is nothing to write (empty, or equal to what
/// the engine already has). The name is not edited by key: what calls this is Enter and blur.
NAV.rename = function (bus, before, value) {
  const v = String(value == null ? "" : value).trim();
  if (!v || v === before) return null;
  return bus.call("show_patch", { ops: [{ op: "replace", path: "/name", value: v }] });
};

// ---- bar ----------------------------------------------------------------

NAV.CSS = `
.sc-nav { flex: 0 0 auto; display: flex; align-items: center; gap: var(--sc-gap);
  padding: 4px 8px; background: var(--sc-panel); border-bottom: 1px solid var(--sc-line);
  font: var(--sc-text-xs) var(--sc-label); letter-spacing: var(--sc-tracking);
  text-transform: uppercase; color: var(--sc-fg-3); }
.sc-nav a { color: var(--sc-fg-2); text-decoration: none; padding: 3px 8px;
  border: 1px solid transparent; border-radius: var(--sc-radius); }
.sc-nav a:hover { color: var(--sc-fg); border-color: var(--sc-line); }
.sc-nav a.atual { color: var(--sc-accent); border-color: var(--sc-accent); }
/* No \`margin-left: auto\`: in a flex column the bar stretches to the width of the page content
   (the one above is wider than the window), and the name field would leave the screen. */
.sc-nav .sc-nav-nome { margin-left: var(--sc-gap); width: 220px; padding: 3px 6px;
  background: var(--sc-well); color: var(--sc-fg); border: 1px solid var(--sc-line);
  border-radius: var(--sc-radius); font: var(--sc-text-sm) var(--sc-mono);
  text-transform: none; letter-spacing: normal; }
.sc-nav .sc-nav-nome:focus { outline: none; border-color: var(--sc-accent); }
.sc-nav .sc-nav-estado { color: var(--sc-fg-3); }
.sc-nav .sc-nav-estado.on { color: var(--sc-go); }
.sc-nav .sc-nav-rev { color: var(--sc-fg-3); font-family: var(--sc-mono); }
`;

/// Builds the bar in `el` and wires it to the `bus`. `doc` is the document (the test passes a
/// fake one).
NAV.mount = function (doc, el, bus) {
  const make = (tag, cls) => {
    const n = doc.createElement(tag);
    if (cls) n.className = cls;
    return n;
  };
  const style = make("style");
  style.textContent = NAV.CSS;
  (doc.head || el).appendChild(style);

  el.className = "sc-nav";
  for (const a of NAV.tabs(doc.location && doc.location.pathname)) {
    const link = make("a", a.current ? "atual" : "");
    link.href = a.href;
    link.textContent = a.lab;
    el.appendChild(link);
  }
  const name = make("input", "sc-nav-nome");
  name.type = "text";
  name.id = "sc-nav-nome";
  name.title = "show name — Enter or leaving the field writes it";
  el.appendChild(name);
  const state = make("span", "sc-nav-estado");
  state.textContent = "OFFLINE";
  el.appendChild(state);
  const rev = make("span", "sc-nav-rev");
  rev.textContent = "rev 0";
  el.appendChild(rev);

  let current = ""; // the name as the engine has it
  const reload = () =>
    bus.call("show_get", {}).then(s => {
      current = (s && s.name) || "";
      rev.textContent = "rev " + (bus.rev || 0);
      // does not step on whoever is typing
      if (doc.activeElement !== name) name.value = current;
    });
  const write = () => {
    const p = NAV.rename(bus, current, name.value);
    if (!p) return void (name.value = current);
    current = String(name.value).trim(); // optimistic; the `show` event confirms
    p.catch(reload);
  };

  name.addEventListener("keydown", e => {
    e.stopPropagation(); // the page keys (Space plays, S snaps) do not apply inside the field
    if (e.key === "Enter") write();
  });
  name.addEventListener("blur", write);

  const opened = () => {
    state.textContent = "ENGINE";
    state.className = "sc-nav-estado on";
    reload();
  };
  bus.on("open", opened);
  bus.on("close", () => {
    state.textContent = "OFFLINE";
    state.className = "sc-nav-estado";
  });
  bus.on("show", reload);
  // the page may have connected before the bar mounted: in that case `open` has already passed
  if (bus.ws && bus.ws.readyState === 1) opened();

  doc.addEventListener("keydown", e => {
    const h = NAV.target(e);
    if (!h) return;
    if (e.preventDefault) e.preventDefault();
    doc.location = h;
  });
  return el;
};

// ---- start-up -----------------------------------------------------------

if (typeof module !== "undefined") module.exports = NAV;

if (typeof document !== "undefined") {
  // Reuses the page Bus when it has one (face, laser, patchbay); timeline and teatro still have
  // their own WS client, with another interface, and there the bar opens its own.
  // ponytail: two connections on the same page ; the bus treats each WS as an independent
  // client, and the cost is the DMX monitor going twice over loopback — it goes away on its own
  // once timeline.js and teatro.js use bus.js.
  const fromPage = () => {
    const b =
      window.bus ||
      (window.PB && window.PB.bus) ||
      (typeof bus !== "undefined" ? bus : null); // eslint-disable-line no-undef
    return window.Bus && b instanceof window.Bus ? b : null;
  };
  // The `?` key of SHORTCUTS.md only exists where help.js is loaded; the bar is on every page,
  // so it is the bar that brings it in.
  const help = () => {
    if (window.HELP) return window.HELP.bindKey();
    const s = document.createElement("script");
    s.src = "help.js";
    s.onload = () => window.HELP && window.HELP.bindKey();
    document.head.appendChild(s);
  };
  const start = () => {
    const el = document.getElementById("nav");
    if (!el) return;
    help();
    const ready = () => NAV.mount(document, el, fromPage() || new window.Bus({}).connect());
    if (window.Bus) return ready();
    const s = document.createElement("script");
    s.src = "bus.js";
    s.onload = ready;
    document.head.appendChild(s);
  };
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", start);
  else start();
}
