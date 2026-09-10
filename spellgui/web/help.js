"use strict";
// help.js — the help page (help.html): SHORTCUTS, read from `design/SHORTCUTS.md`, and COMMANDS,
// read from the live registry through `bus.commands()`. No shortcut list and no command list lives
// here: the two texts have a single source, and this page draws them.
//
// Two testable rules (test/help.test.js): the markdown table parser and the short signature of a
// command. The rest is DOM.

const HELP = {};

HELP.MD = "../../design/SHORTCUTS.md";   // relative: works under `serve --dir .` and under file://
// Heading of the shortcut table in SHORTCUTS.md. A list because the file exists in two languages:
// whichever heading is there, the parser finds the table.
HELP.TABLE = ["Default map", "Mapa padrão"];

// Status word of SHORTCUTS.md -> CSS class of help.html. In both languages, for the same reason
// as HELP.TABLE; the classes themselves do not change.
HELP.ST = { "feito": "st-feito", "done": "st-feito", "falta": "st-falta", "missing": "st-falta",
            "n.a.": "st-na", "n/a": "st-na" };

/// `| a | b | c |` lines of the FIRST table after the given heading (with no heading, the first of
/// the text). `title` takes one heading or a list of accepted headings. Returns
/// [[cell, ...], ...] with the header row at position 0, without the `|---|` line and without
/// requiring a fixed number of columns.
// ponytail: a table parser, not a markdown one (no emphasis, no link, no cell with a `|` inside)
// ; if the help needs more than the table, generate HTML at build time — do not bring in a library.
HELP.table = function (md, title) {
  const ls = String(md == null ? "" : md).split(/\r?\n/);
  let i = 0;
  if (title) {
    const titles = Array.isArray(title) ? title : [title];
    i = ls.findIndex(l => /^#{1,6}\s+/.test(l) && titles.indexOf(l.replace(/^#+\s+/, "").trim()) >= 0);
    if (i < 0) return [];
  }
  const rows = [];
  for (; i < ls.length; i++) {
    const l = ls[i].trim();
    if (l[0] !== "|") {
      if (rows.length) break;   // the table ended; the rest of the file does not matter
      continue;
    }
    const cs = l.replace(/^\|/, "").replace(/\|$/, "").split("|").map(s => s.trim());
    if (cs.length && cs.every(c => /^:?-{2,}:?$/.test(c))) continue;   // separator
    rows.push(cs);
  }
  return rows;
};

/// Short signature of a command from `GET /commands`: the arguments in schema order, with `?` on
/// what is not required. A command with no argument returns an empty list (it is a button).
HELP.args = function (c) {
  const p = (c && c.params && c.params.properties) || {};
  const req = (c && c.params && c.params.required) || [];
  return Object.keys(p).map(k => (req.indexOf(k) < 0 ? k + "?" : k));
};

// ---- DOM ----------------------------------------------------------------

function el(tag, cls, txt) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (txt !== undefined) e.textContent = txt;
  return e;
}

/// The shortcut table inside `target`. The first row becomes <th>.
HELP.drawShortcuts = function (target, rows) {
  target.textContent = "";
  if (!rows.length) return void target.appendChild(el("p", "vazio", "no " + HELP.MD));
  const t = el("table", "grade");
  rows.forEach((r, i) => {
    const tr = el("tr");
    for (const c of r) {
      const td = el(i ? "td" : "th", null, c.replace(/`/g, ""));
      const st = HELP.ST[c.toLowerCase()];
      if (i && st) td.className = st;
      tr.appendChild(td);
    }
    t.appendChild(tr);
  });
  target.appendChild(t);
};

/// One <details> per command: summary (name, args, doc) and, when open, the form that runs it.
HELP.drawCommands = function (target, cmds, bus) {
  target.textContent = "";
  for (const c of cmds) {
    const d = el("details", "cmd");
    d.dataset.busca = (c.name + " " + (c.doc || "")).toLowerCase();
    const s = el("summary");
    s.appendChild(el("b", null, c.name));
    const a = HELP.args(c);
    s.appendChild(el("span", "args", a.length ? "{" + a.join(", ") + "}" : "{}"));
    s.appendChild(el("span", "doc", c.doc || ""));
    d.appendChild(s);
    // the form is only born when the command is opened: 46 forms at once is drawing nobody asked
    // for
    d.ontoggle = () => {
      if (d.open && !d.dataset.pronto) {
        d.dataset.pronto = "1";
        d.appendChild(WG.form(c, bus));
      }
    };
    target.appendChild(d);
  }
};

/// `?` opens the help from any page that loads this file. It does not act inside a text field and
/// does not reload help.html itself.
// ponytail: what loads help.js today is only help.html ; the gui-window frente puts `nav.js` on
// every page — from there this bind starts to hold everywhere.
HELP.bindKey = function () {
  addEventListener("keydown", e => {
    if (e.key !== "?" || e.ctrlKey || e.altKey) return;
    if (/^(INPUT|SELECT|TEXTAREA)$/.test(e.target.tagName) || e.target.isContentEditable) return;
    if (/help\.html$/.test(location.pathname)) return;
    e.preventDefault();
    location.href = "help.html";
  });
};

HELP.mount = function (doc) {
  const bus = new Bus({ offline: new URLSearchParams(location.search).get("offline") === "1" });
  bus.connect();
  const q = id => doc.getElementById(id);

  fetch(HELP.MD)
    .then(r => r.text())
    .then(md => HELP.drawShortcuts(q("atalhos"), HELP.table(md, HELP.TABLE)))
    .catch(e => (q("atalhos").textContent = HELP.MD + ": " + e.message));

  bus
    .commands()
    .then(cs => {
      HELP.drawCommands(q("comandos"), cs, bus);
      q("n").textContent = cs.length + " commands";
    })
    .catch(e => (q("comandos").textContent = "GET /commands: " + e.message));

  q("busca").oninput = () => {
    const t = q("busca").value.trim().toLowerCase();
    for (const d of q("comandos").children) {
      d.hidden = !!t && d.dataset.busca.indexOf(t) < 0;
    }
  };
  bus.on("open", () => (q("conn").textContent = "engine"));
  bus.on("close", () => (q("conn").textContent = "offline"));
  HELP.bindKey();
};

if (typeof window !== "undefined") window.HELP = HELP;
if (typeof module !== "undefined") module.exports = HELP;
