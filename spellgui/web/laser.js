"use strict";
// ILDA player page. A registry client, never the owner of logic: every control is a `laser_*`
// command and the label on screen is the command name (rule 2 of design/FUNCOES).

const q = id => document.getElementById(id);
const msg = q("msg");

// `bus.js` is the bus client of every page: here only the event wiring is left.
const bus = new Bus({}).connect();
bus.on("close", () => { msg.textContent = "no server (spellcore serve)"; });

// ------------------------------------------------------------------ local state

let feed = null;                 // id of the open feed; null = closed
let playing = false;
let shuttered = false;

const err = e => { msg.textContent = e.message || String(e); };
const call = (cmd, args) => bus.call(cmd, args).catch(e => { err(e); throw e; });

function buttons() {
  q("close").disabled = q("play").disabled = q("shutter").disabled = feed === null;
  q("stop").disabled = feed === null || !playing;
  q("open").disabled = feed !== null;
  q("play").classList.toggle("on", playing);
  q("shutter").classList.toggle("live", shuttered);
}

// -------------------------------------------------------------------- commands

q("scan").onclick = () => {
  msg.textContent = "looking...";
  call("laser_dacs", { timeout: 2 }).then(l => {
    q("dacs").innerHTML = "";
    for (const d of l) {
      const o = document.createElement("option");
      o.value = d.host;
      o.dataset.type = d.type;
      o.textContent = `${d.type} ${d.host} ${d.id}`;
      q("dacs").append(o);
    }
    msg.textContent = l.length ? "" : "no DAC on the network";
    if (l.length) q("host").value = l[0].host;
  });
};

q("dacs").onchange = () => { q("host").value = q("dacs").value; };

q("ls").onclick = () => call("laser_files", { dir: q("dir").value }).then(r => {
  q("files").innerHTML = "";
  for (const f of r.files) {
    const o = document.createElement("option");
    o.value = f.path;
    o.textContent = `${f.name}  ${(f.bytes / 1024) | 0} kB`;
    q("files").append(o);
  }
  msg.textContent = r.files.length ? "" : `no .ild in ${r.dir}`;
});

q("open").onclick = () => {
  const t = q("dacs").selectedOptions[0]?.dataset.type || "etherdream";
  call("laser_open", { dac: t, host: q("host").value, kpps: +q("kpps").value }).then(r => {
    feed = r.feed;
    msg.textContent = "";
    buttons();
    for (const el of document.querySelectorAll("input[type=range]")) send(el);
  });
};

q("close").onclick = () => call("laser_close", { feed }).then(() => {
  feed = null; playing = false; shuttered = false;
  q("st").innerHTML = "<tbody><tr><td>feed</td><td>no feed open</td></tr></tbody>";
  buttons();
});

q("play").onclick = () => call("laser_play", {
  feed, file: q("files").value, fps: 30, loop: true,
}).then(() => { playing = true; buttons(); });

q("stop").onclick = () => call("laser_stop", { feed }).then(() => { playing = false; buttons(); });

q("shutter").onclick = () => call("laser_param", {
  feed, path: "shutter", value: shuttered ? 0 : 1,
}).then(r => { shuttered = r.shutter; buttons(); });

// -------------------------------------------------- sliders: id `p_a_b` = path `a/b`

function send(el) {
  const path = el.id.slice(2).replace("_", "/");   // p_geo_scale -> geo/scale
  el.nextElementSibling.textContent = el.step < 1 ? (+el.value).toFixed(2) : el.value;
  if (feed !== null) call("laser_param", { feed, path, value: +el.value }).catch(() => {});
}

for (const el of document.querySelectorAll("input[type=range]")) el.oninput = () => send(el);

// ------------------------------------------------------- live stats (4 Hz)

setInterval(() => {
  if (feed === null) return;
  bus.call("laser_stats", { feed }).then(s => {
    playing = !!s.playing;
    shuttered = !!s.shutter;
    buttons();
    const n = v => (typeof v === "number" && !Number.isInteger(v) ? v.toFixed(3) : v);
    q("st").innerHTML = "<tbody>" + Object.entries(s).map(
      ([k, v]) => `<tr><td>${k}</td><td>${v === null ? "-" : n(v)}</td></tr>`
    ).join("") + "</tbody>";
  }).catch(() => {});
}, 250);

buttons();
