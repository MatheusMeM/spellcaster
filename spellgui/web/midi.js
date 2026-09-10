// MIDI mapping: input port, key -> command table, LEARN. A registry client, never the owner of
// logic: every button is a `midi_*` command and the map lives in the `.spell`, in the engine.
//
// The top half is pure (map args) and runs under `node --test`; the bottom half is DOM.
"use strict";

// --------------------------------------------------------------- pure part

// Field text -> args object. Empty = {}. Object only: the registry takes an object.
function parseArgs(txt) {
  const s = (txt || "").trim();
  if (!s) return {};
  const v = JSON.parse(s); // the error comes up with the JSON message
  if (!v || typeof v !== "object" || Array.isArray(v)) throw new Error("expected a JSON object");
  return v;
}

// The same `expande` of the engine (spellcore/engine/src/midi.rs): "$" becomes the value 0..1 and
// "$<n>" becomes round(value * n). This is what the page shows as a preview of what will be sent.
function preview(v, value) {
  if (typeof v === "string") {
    if (v[0] !== "$") return v;
    const r = v.slice(1);
    if (r === "") return value;
    const n = Number(r);
    return Number.isFinite(n) && r.trim() !== "" ? Math.round(value * n) : v;
  }
  if (Array.isArray(v)) return v.map(x => preview(x, value));
  if (v && typeof v === "object") {
    const o = {};
    for (const k of Object.keys(v)) o[k] = preview(v[k], value);
    return o;
  }
  return v;
}

const MIDI = { parseArgs, preview };
if (typeof module !== "undefined") module.exports = MIDI;

// ------------------------------------------------------------------- page

if (typeof window !== "undefined") {
  window.MIDI = MIDI;
  const q = id => document.getElementById(id);
  const bus = new Bus({ offline: location.search.includes("offline=1") }).connect();
  const msg = () => q("msg");
  const err = e => { msg().textContent = e.message || String(e); };
  const call = (cmd, args) => bus.call(cmd, args).catch(e => { err(e); throw e; });

  let openPort = null;   // name of the open port
  let lastSeq = 0;       // seq of the last event seen

  bus.on("close", () => { msg().textContent = "no server (spellcore serve)"; });
  bus.on("show", () => table());

  // -------------------------------------------------------------- ports

  function ports() {
    return call("midi_ports", {}).then(r => {
      const s = q("ports");
      s.innerHTML = "";
      for (const n of r.ports || []) {
        const o = document.createElement("option");
        o.value = o.textContent = n;
        s.append(o);
      }
      if (!(r.ports || []).length) {
        const o = document.createElement("option");
        o.value = "";
        o.textContent = "(no MIDI port)";
        s.append(o);
      }
      openPort = r.open || null;
      if (openPort) s.value = openPort;
      buttons();
    });
  }

  function buttons() {
    q("open").disabled = !!openPort;
    q("close").disabled = !openPort;
    q("porta").textContent = openPort || "closed";
    q("porta").classList.toggle("on", !!openPort);
  }

  q("scan").onclick = ports;
  q("open").onclick = () => call("midi_open", { port: q("ports").value }).then(r => {
    openPort = r.open; msg().textContent = ""; buttons();
  });
  q("close").onclick = () => call("midi_close", {}).then(() => { openPort = null; buttons(); });

  // ------------------------------------------------------- last key and LEARN

  function show(r) {
    if (!r || !r.key) return;
    q("last").textContent = `${r.key}  v=${(+r.value).toFixed(3)}  raw=${r.raw}`;
    if (r.seq !== lastSeq) {
      lastSeq = r.seq;
      q("last").classList.remove("hit");
      void q("last").offsetWidth;
      q("last").classList.add("hit");
    }
  }

  // ponytail: polling at 5 Hz only so the operator sees the key arriving ; make it a bus event if
  // the MIDI monitor needs every message.
  setInterval(() => { if (openPort) bus.call("midi_last", {}).then(show, () => {}); }, 200);

  q("learn").onclick = () => {
    const b = q("learn");
    b.classList.add("live");
    b.textContent = "PRESS...";
    bus.call("midi_learn", {}).then(r => { q("key").value = r.key; show(r); }, err)
      .finally(() => { b.classList.remove("live"); b.textContent = "LEARN"; });
  };

  // ----------------------------------------------------------------- map

  function table() {
    return call("midi_maps", {}).then(m => {
      if (!m || m.offline) m = {}; // with no engine (offline=1) there is no map to show
      const t = q("map");
      t.innerHTML = "";
      const keys = Object.keys(m || {}).sort();
      for (const k of keys) {
        const e = m[k] || {};
        const tr = document.createElement("tr");
        const args = JSON.stringify(e.args || {});
        for (const txt of [k, e.cmd || "", args === "{}" ? "" : args]) {
          const td = document.createElement("td");
          td.textContent = txt;
          tr.append(td);
        }
        const td = document.createElement("td");
        const b = document.createElement("button");
        b.textContent = "x";
        b.title = "midi_unmap " + k;
        b.onclick = () => call("midi_unmap", { key: k }).then(table);
        td.append(b);
        tr.append(td);
        t.append(tr);
      }
      q("n").textContent = keys.length + " key(s)";
    });
  }

  q("map_add").onclick = () => {
    let args;
    try {
      args = parseArgs(q("args").value);
    } catch (e) {
      return err(new Error("args: " + e.message));
    }
    call("midi_map", { key: q("key").value.trim(), cmd: q("cmd").value.trim(), args }).then(() => {
      msg().textContent = "";
      q("args").value = "";
      table();
    });
  };

  // preview of what the command gets with the fader halfway (value 0.5)
  q("args").oninput = () => {
    try {
      q("prev").textContent = JSON.stringify(preview(parseArgs(q("args").value), 0.5));
    } catch (e) {
      q("prev").textContent = "args: " + e.message;
    }
  };

  // --------------------------------------------------------------- commands

  bus.commands().then(cs => {
    const dl = q("cmds");
    for (const c of cs.map(c => c.name).sort()) {
      const o = document.createElement("option");
      o.value = c;
      dl.append(o);
    }
    const doc = Object.fromEntries(cs.map(c => [c.name, c.doc]));
    q("cmd").oninput = () => { q("doc").textContent = doc[q("cmd").value.trim()] || ""; };
  }, err);

  // A request sent with the socket closed is dropped by `bus.js`: the first read waits for the
  // "open" (which also comes back on every reconnection).
  const load = () => ports().then(table, err);
  if (bus.offline) load();
  else bus.on("open", load);
}
