"use strict";
// Pagina do ILDA player. Cliente do registry, nunca dona de logica: cada controle e' um
// comando `laser_*` e o rotulo na tela e' o nome do comando (regra 2 de design/FUNCOES).

const q = id => document.getElementById(id);
const msg = q("msg");

// `bus.js` e' o cliente do barramento de todas as paginas: aqui so' sobra a ligacao dos eventos.
const bus = new Bus({}).connect();
bus.on("close", () => { msg.textContent = "sem servidor (spellcore serve)"; });

// ------------------------------------------------------------------ estado local

let feed = null;                 // id do feed aberto; null = fechado
let tocando = false;
let obturado = false;

const erro = e => { msg.textContent = e.message || String(e); };
const call = (cmd, args) => bus.call(cmd, args).catch(e => { erro(e); throw e; });

function botoes() {
  q("close").disabled = q("play").disabled = q("shutter").disabled = feed === null;
  q("stop").disabled = feed === null || !tocando;
  q("open").disabled = feed !== null;
  q("play").classList.toggle("on", tocando);
  q("shutter").classList.toggle("live", obturado);
}

// -------------------------------------------------------------------- comandos

q("scan").onclick = () => {
  msg.textContent = "procurando...";
  call("laser_dacs", { timeout: 2 }).then(l => {
    q("dacs").innerHTML = "";
    for (const d of l) {
      const o = document.createElement("option");
      o.value = d.host;
      o.dataset.type = d.type;
      o.textContent = `${d.type} ${d.host} ${d.id}`;
      q("dacs").append(o);
    }
    msg.textContent = l.length ? "" : "nenhum DAC na rede";
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
  msg.textContent = r.files.length ? "" : `nenhum .ild em ${r.dir}`;
});

q("open").onclick = () => {
  const t = q("dacs").selectedOptions[0]?.dataset.type || "etherdream";
  call("laser_open", { dac: t, host: q("host").value, kpps: +q("kpps").value }).then(r => {
    feed = r.feed;
    msg.textContent = "";
    botoes();
    for (const el of document.querySelectorAll("input[type=range]")) manda(el);
  });
};

q("close").onclick = () => call("laser_close", { feed }).then(() => {
  feed = null; tocando = false; obturado = false;
  q("st").innerHTML = "<tbody><tr><td>feed</td><td>sem feed aberto</td></tr></tbody>";
  botoes();
});

q("play").onclick = () => call("laser_play", {
  feed, file: q("files").value, fps: 30, loop: true,
}).then(() => { tocando = true; botoes(); });

q("stop").onclick = () => call("laser_stop", { feed }).then(() => { tocando = false; botoes(); });

q("shutter").onclick = () => call("laser_param", {
  feed, path: "shutter", value: obturado ? 0 : 1,
}).then(r => { obturado = r.shutter; botoes(); });

// -------------------------------------------------- sliders: id `p_a_b` = path `a/b`

function manda(el) {
  const path = el.id.slice(2).replace("_", "/");   // p_geo_scale -> geo/scale
  el.nextElementSibling.textContent = el.step < 1 ? (+el.value).toFixed(2) : el.value;
  if (feed !== null) call("laser_param", { feed, path, value: +el.value }).catch(() => {});
}

for (const el of document.querySelectorAll("input[type=range]")) el.oninput = () => manda(el);

// ------------------------------------------------------- stats ao vivo (4 Hz)

setInterval(() => {
  if (feed === null) return;
  bus.call("laser_stats", { feed }).then(s => {
    tocando = !!s.playing;
    obturado = !!s.shutter;
    botoes();
    const n = v => (typeof v === "number" && !Number.isInteger(v) ? v.toFixed(3) : v);
    q("st").innerHTML = "<tbody>" + Object.entries(s).map(
      ([k, v]) => `<tr><td>${k}</td><td>${v === null ? "-" : n(v)}</td></tr>`
    ).join("") + "</tbody>";
  }).catch(() => {});
}, 250);

botoes();
