// MIDI mapping: porta de entrada, tabela tecla -> comando, LEARN. Cliente do registry, nunca
// dona de logica: cada botao e' um comando `midi_*` e o mapa mora no `.spell`, no engine.
//
// A metade de cima e' pura (args do mapa) e roda no `node --test`; a de baixo e' DOM.
"use strict";

// --------------------------------------------------------------- parte pura

// Texto do campo -> objeto de args. Vazio = {}. So' objeto: o registry recebe um objeto.
function parseArgs(txt) {
  const s = (txt || "").trim();
  if (!s) return {};
  const v = JSON.parse(s); // erro sobe com a mensagem do JSON
  if (!v || typeof v !== "object" || Array.isArray(v)) throw new Error("esperava um objeto JSON");
  return v;
}

// O mesmo `expande` do engine (spellcore/engine/src/midi.rs): "$" vira o valor 0..1 e "$<n>"
// vira round(valor * n). E' o que a pagina mostra como previa do que vai ser enviado.
function preview(v, valor) {
  if (typeof v === "string") {
    if (v[0] !== "$") return v;
    const r = v.slice(1);
    if (r === "") return valor;
    const n = Number(r);
    return Number.isFinite(n) && r.trim() !== "" ? Math.round(valor * n) : v;
  }
  if (Array.isArray(v)) return v.map(x => preview(x, valor));
  if (v && typeof v === "object") {
    const o = {};
    for (const k of Object.keys(v)) o[k] = preview(v[k], valor);
    return o;
  }
  return v;
}

const MIDI = { parseArgs, preview };
if (typeof module !== "undefined") module.exports = MIDI;

// ------------------------------------------------------------------- pagina

if (typeof window !== "undefined") {
  window.MIDI = MIDI;
  const q = id => document.getElementById(id);
  const bus = new Bus({ offline: location.search.includes("offline=1") }).connect();
  const msg = () => q("msg");
  const erro = e => { msg().textContent = e.message || String(e); };
  const call = (cmd, args) => bus.call(cmd, args).catch(e => { erro(e); throw e; });

  let aberta = null;   // nome da porta aberta
  let ultima = 0;      // seq do ultimo evento visto

  bus.on("close", () => { msg().textContent = "sem servidor (spellcore serve)"; });
  bus.on("show", () => tabela());

  // -------------------------------------------------------------- portas

  function portas() {
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
        o.textContent = "(nenhuma porta MIDI)";
        s.append(o);
      }
      aberta = r.open || null;
      if (aberta) s.value = aberta;
      botoes();
    });
  }

  function botoes() {
    q("open").disabled = !!aberta;
    q("close").disabled = !aberta;
    q("porta").textContent = aberta || "fechada";
    q("porta").classList.toggle("on", !!aberta);
  }

  q("scan").onclick = portas;
  q("open").onclick = () => call("midi_open", { port: q("ports").value }).then(r => {
    aberta = r.open; msg().textContent = ""; botoes();
  });
  q("close").onclick = () => call("midi_close", {}).then(() => { aberta = null; botoes(); });

  // ------------------------------------------------------- ultima tecla e LEARN

  function mostra(r) {
    if (!r || !r.key) return;
    q("last").textContent = `${r.key}  v=${(+r.value).toFixed(3)}  raw=${r.raw}`;
    if (r.seq !== ultima) {
      ultima = r.seq;
      q("last").classList.remove("hit");
      void q("last").offsetWidth;
      q("last").classList.add("hit");
    }
  }

  // ponytail: sondagem a 5 Hz so' para o operador ver a tecla chegando ; virar evento do
  // barramento se o monitor de MIDI precisar de cada mensagem.
  setInterval(() => { if (aberta) bus.call("midi_last", {}).then(mostra, () => {}); }, 200);

  q("learn").onclick = () => {
    const b = q("learn");
    b.classList.add("live");
    b.textContent = "APERTE...";
    bus.call("midi_learn", {}).then(r => { q("key").value = r.key; mostra(r); }, erro)
      .finally(() => { b.classList.remove("live"); b.textContent = "APRENDER"; });
  };

  // ----------------------------------------------------------------- mapa

  function tabela() {
    return call("midi_maps", {}).then(m => {
      if (!m || m.offline) m = {}; // sem engine (offline=1) nao ha' mapa para mostrar
      const t = q("map");
      t.innerHTML = "";
      const chaves = Object.keys(m || {}).sort();
      for (const k of chaves) {
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
        b.onclick = () => call("midi_unmap", { key: k }).then(tabela);
        td.append(b);
        tr.append(td);
        t.append(tr);
      }
      q("n").textContent = chaves.length + " tecla(s)";
    });
  }

  q("map_add").onclick = () => {
    let args;
    try {
      args = parseArgs(q("args").value);
    } catch (e) {
      return erro(new Error("args: " + e.message));
    }
    call("midi_map", { key: q("key").value.trim(), cmd: q("cmd").value.trim(), args }).then(() => {
      msg().textContent = "";
      q("args").value = "";
      tabela();
    });
  };

  // previa do que o comando recebe com o fader no meio (valor 0,5)
  q("args").oninput = () => {
    try {
      q("prev").textContent = JSON.stringify(preview(parseArgs(q("args").value), 0.5));
    } catch (e) {
      q("prev").textContent = "args: " + e.message;
    }
  };

  // --------------------------------------------------------------- comandos

  bus.commands().then(cs => {
    const dl = q("cmds");
    for (const c of cs.map(c => c.name).sort()) {
      const o = document.createElement("option");
      o.value = c;
      dl.append(o);
    }
    const doc = Object.fromEntries(cs.map(c => [c.name, c.doc]));
    q("cmd").oninput = () => { q("doc").textContent = doc[q("cmd").value.trim()] || ""; };
  }, erro);

  // Request mandada com o socket fechado e' descartada pelo `bus.js`: a primeira leitura espera
  // o "open" (que tambem volta a cada reconexao).
  const carrega = () => portas().then(tabela, erro);
  if (bus.offline) carrega();
  else bus.on("open", carrega);
}
