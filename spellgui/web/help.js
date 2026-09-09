"use strict";
// help.js — a pagina de ajuda (help.html): ATALHOS, lidos de `design/SHORTCUTS.md`, e COMANDOS,
// lidos do registry vivo por `bus.commands()`. Nenhuma lista de atalho e nenhuma lista de
// comando mora aqui: os dois textos tem uma fonte so', e esta pagina os desenha.
//
// Duas regras testaveis (test/help.test.js): o parser da tabela markdown e a assinatura curta
// de um comando. O resto e' DOM.

const HELP = {};

HELP.MD = "../../design/SHORTCUTS.md";   // relativo: vale no `serve --dir .` e em file://
HELP.TABELA = "Mapa padrão";        // o titulo da tabela de atalhos em SHORTCUTS.md

/// Linhas `| a | b | c |` da PRIMEIRA tabela depois do titulo dado (sem titulo, a primeira do
/// texto). Devolve [[celula, ...], ...] com o cabecalho na posicao 0, sem a linha `|---|` e sem
/// exigir numero fixo de colunas.
// ponytail: parser de tabela, nao de markdown (nem enfase, nem link, nem celula com `|` dentro)
// ; se a ajuda precisar de mais que a tabela, gerar HTML no build — nao trazer biblioteca.
HELP.tabela = function (md, titulo) {
  const ls = String(md == null ? "" : md).split(/\r?\n/);
  let i = 0;
  if (titulo) {
    i = ls.findIndex(l => /^#{1,6}\s+/.test(l) && l.replace(/^#+\s+/, "").trim() === titulo);
    if (i < 0) return [];
  }
  const rows = [];
  for (; i < ls.length; i++) {
    const l = ls[i].trim();
    if (l[0] !== "|") {
      if (rows.length) break;   // a tabela acabou; o resto do arquivo nao interessa
      continue;
    }
    const cs = l.replace(/^\|/, "").replace(/\|$/, "").split("|").map(s => s.trim());
    if (cs.length && cs.every(c => /^:?-{2,}:?$/.test(c))) continue;   // separador
    rows.push(cs);
  }
  return rows;
};

/// Assinatura curta de um comando de `GET /commands`: os argumentos na ordem do schema, com `?`
/// no que nao e' obrigatorio. Comando sem argumento devolve lista vazia (e' um botao).
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

/// A tabela de atalhos dentro de `alvo`. Primeira linha vira <th>.
HELP.desenhaAtalhos = function (alvo, rows) {
  alvo.textContent = "";
  if (!rows.length) return void alvo.appendChild(el("p", "vazio", "sem " + HELP.MD));
  const t = el("table", "grade");
  rows.forEach((r, i) => {
    const tr = el("tr");
    for (const c of r) {
      const td = el(i ? "td" : "th", null, c.replace(/`/g, ""));
      if (i && /^(feito|falta|n\.a\.)$/.test(c)) td.className = "st-" + c.replace(/\./g, "");
      tr.appendChild(td);
    }
    t.appendChild(tr);
  });
  alvo.appendChild(t);
};

/// Um <details> por comando: resumo (nome, args, doc) e, aberto, o formulario que executa.
HELP.desenhaComandos = function (alvo, cmds, bus) {
  alvo.textContent = "";
  for (const c of cmds) {
    const d = el("details", "cmd");
    d.dataset.busca = (c.name + " " + (c.doc || "")).toLowerCase();
    const s = el("summary");
    s.appendChild(el("b", null, c.name));
    const a = HELP.args(c);
    s.appendChild(el("span", "args", a.length ? "{" + a.join(", ") + "}" : "{}"));
    s.appendChild(el("span", "doc", c.doc || ""));
    d.appendChild(s);
    // o formulario so' nasce quando o comando e' aberto: 46 formularios de uma vez e' desenho
    // que ninguem pediu
    d.ontoggle = () => {
      if (d.open && !d.dataset.pronto) {
        d.dataset.pronto = "1";
        d.appendChild(WG.form(c, bus));
      }
    };
    alvo.appendChild(d);
  }
};

/// `?` abre a ajuda de qualquer pagina que carregue este arquivo. Nao age em campo de texto e
/// nao recarrega a propria help.html.
// ponytail: quem carrega help.js hoje e' so' a help.html ; a frente gui-janela poe o `nav.js`
// em todas as paginas — e' de la' que este bind passa a valer em todas.
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
    .then(md => HELP.desenhaAtalhos(q("atalhos"), HELP.tabela(md, HELP.TABELA)))
    .catch(e => (q("atalhos").textContent = HELP.MD + ": " + e.message));

  bus
    .commands()
    .then(cs => {
      HELP.desenhaComandos(q("comandos"), cs, bus);
      q("n").textContent = cs.length + " comandos";
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
