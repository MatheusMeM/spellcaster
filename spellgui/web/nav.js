"use strict";
// nav.js — a barra que liga as seis paginas e mostra o show aberto. Toda pagina inclui com
// duas linhas no topo do <body>:
//
//   <div id="nav"></div>
//   <script src="nav.js"></script>
//
// Abas TIMELINE / PATCHBAY / TEATRO / FACE / LASER / AJUDA, `Shift+1`..`Shift+6`
// (design/SHORTCUTS.md, "Foco de painel"), indicador ENGINE/OFFLINE com o `rev` do show, e o
// nome do show editavel. A barra tambem carrega o `help.js`: e' o que liga a tecla `?` em
// toda pagina, e nao so' na de ajuda.
// Cor e fonte so' de design/tokens/spellcaster.css.

const NAV = {};

NAV.PAGINAS = [
  { rot: "TIMELINE", href: "index.html" },
  { rot: "PATCHBAY", href: "patchbay.html" },
  { rot: "TEATRO", href: "teatro.html" },
  { rot: "FACE", href: "face.html?face=quatro" },
  { rot: "LASER", href: "laser.html" },
  { rot: "AJUDA", href: "help.html" },
];

// ---- regras (o resto e' DOM) --------------------------------------------

/// Arquivo de um caminho ou href, sem diretorio nem query; a raiz e' o index.
NAV.arquivo = function (p) {
  return String(p || "").split("?")[0].split("/").pop() || "index.html";
};

/// As abas com a atual marcada.
NAV.abas = function (pathname) {
  const aqui = NAV.arquivo(pathname);
  return NAV.PAGINAS.map(p => ({
    rot: p.rot,
    href: p.href,
    atual: NAV.arquivo(p.href) === aqui,
  }));
};

/// `Shift+1`..`Shift+6` -> href, ou null. Digitando num campo, nenhuma tecla navega: o nome do
/// show tem digitos. Vale o `code` da tecla, nao o `key`: em teclado ABNT2 o Shift+2 escreve `"`
/// e o Shift+3 escreve `#`. O `key` so' entra como reserva, para layout em que o Shift mantem o
/// digito (e para evento sintetico, que costuma vir sem `code`).
NAV.destino = function (e) {
  const t = e.target || {};
  if (t.tagName === "INPUT" || t.tagName === "TEXTAREA" || t.isContentEditable) return null;
  if (!e.shiftKey || e.ctrlKey || e.altKey || e.metaKey) return null;
  const p = NAV.PAGINAS.find((_, n) => e.code === "Digit" + (n + 1) || e.key === String(n + 1));
  return p ? p.href : null;
};

/// O `show_patch` do nome, ou null quando nao ha' o que gravar (vazio, ou igual ao que ja' esta'
/// no engine). Nome nao se edita por tecla: quem chama isto e' o Enter e o blur.
NAV.renomeia = function (bus, antes, valor) {
  const v = String(valor == null ? "" : valor).trim();
  if (!v || v === antes) return null;
  return bus.call("show_patch", { ops: [{ op: "replace", path: "/name", value: v }] });
};

// ---- barra --------------------------------------------------------------

NAV.CSS = `
.sc-nav { flex: 0 0 auto; display: flex; align-items: center; gap: var(--sc-gap);
  padding: 4px 8px; background: var(--sc-panel); border-bottom: 1px solid var(--sc-line);
  font: var(--sc-text-xs) var(--sc-label); letter-spacing: var(--sc-tracking);
  text-transform: uppercase; color: var(--sc-fg-3); }
.sc-nav a { color: var(--sc-fg-2); text-decoration: none; padding: 3px 8px;
  border: 1px solid transparent; border-radius: var(--sc-radius); }
.sc-nav a:hover { color: var(--sc-fg); border-color: var(--sc-line); }
.sc-nav a.atual { color: var(--sc-accent); border-color: var(--sc-accent); }
/* Nada de \`margin-left: auto\`: numa coluna flex a barra estica ate' a largura do conteudo da
   pagina (a de cima e' mais larga que a janela), e o campo do nome sairia da tela. */
.sc-nav .sc-nav-nome { margin-left: var(--sc-gap); width: 220px; padding: 3px 6px;
  background: var(--sc-well); color: var(--sc-fg); border: 1px solid var(--sc-line);
  border-radius: var(--sc-radius); font: var(--sc-text-sm) var(--sc-mono);
  text-transform: none; letter-spacing: normal; }
.sc-nav .sc-nav-nome:focus { outline: none; border-color: var(--sc-accent); }
.sc-nav .sc-nav-estado { color: var(--sc-fg-3); }
.sc-nav .sc-nav-estado.on { color: var(--sc-go); }
.sc-nav .sc-nav-rev { color: var(--sc-fg-3); font-family: var(--sc-mono); }
`;

/// Monta a barra em `el` e liga no `bus`. `doc` e' o document (o teste passa um de mentira).
NAV.monta = function (doc, el, bus) {
  const cria = (tag, cls) => {
    const n = doc.createElement(tag);
    if (cls) n.className = cls;
    return n;
  };
  const estilo = cria("style");
  estilo.textContent = NAV.CSS;
  (doc.head || el).appendChild(estilo);

  el.className = "sc-nav";
  for (const a of NAV.abas(doc.location && doc.location.pathname)) {
    const link = cria("a", a.atual ? "atual" : "");
    link.href = a.href;
    link.textContent = a.rot;
    el.appendChild(link);
  }
  const nome = cria("input", "sc-nav-nome");
  nome.type = "text";
  nome.id = "sc-nav-nome";
  nome.title = "nome do show — Enter ou sair do campo grava";
  el.appendChild(nome);
  const estado = cria("span", "sc-nav-estado");
  estado.textContent = "OFFLINE";
  el.appendChild(estado);
  const rev = cria("span", "sc-nav-rev");
  rev.textContent = "rev 0";
  el.appendChild(rev);

  let atual = ""; // o nome como o engine tem
  const recarrega = () =>
    bus.call("show_get", {}).then(s => {
      atual = (s && s.name) || "";
      rev.textContent = "rev " + (bus.rev || 0);
      // nao pisa em cima de quem esta' digitando
      if (doc.activeElement !== nome) nome.value = atual;
    });
  const grava = () => {
    const p = NAV.renomeia(bus, atual, nome.value);
    if (!p) return void (nome.value = atual);
    atual = String(nome.value).trim(); // otimista; o evento `show` confirma
    p.catch(recarrega);
  };

  nome.addEventListener("keydown", e => {
    e.stopPropagation(); // as teclas da pagina (Space toca, S snap) nao valem dentro do campo
    if (e.key === "Enter") grava();
  });
  nome.addEventListener("blur", grava);

  const aberto = () => {
    estado.textContent = "ENGINE";
    estado.className = "sc-nav-estado on";
    recarrega();
  };
  bus.on("open", aberto);
  bus.on("close", () => {
    estado.textContent = "OFFLINE";
    estado.className = "sc-nav-estado";
  });
  bus.on("show", recarrega);
  // a pagina pode ter conectado antes da barra montar: nesse caso o `open` ja' passou
  if (bus.ws && bus.ws.readyState === 1) aberto();

  doc.addEventListener("keydown", e => {
    const h = NAV.destino(e);
    if (!h) return;
    if (e.preventDefault) e.preventDefault();
    doc.location = h;
  });
  return el;
};

// ---- arranque -----------------------------------------------------------

if (typeof module !== "undefined") module.exports = NAV;

if (typeof document !== "undefined") {
  // Reaproveita o Bus da pagina quando ela tem um (face, laser, patchbay); timeline e teatro
  // ainda tem cliente WS proprio, com outra interface, e ai' a barra abre o seu.
  // ponytail: duas conexoes na mesma pagina ; o barramento trata cada WS como um cliente
  // independente, e o custo e' o monitor DMX indo duas vezes em loopback — cai sozinho quando
  // timeline.js e teatro.js passarem a usar bus.js.
  const daPagina = () => {
    const b =
      window.bus ||
      (window.PB && window.PB.bus) ||
      (typeof bus !== "undefined" ? bus : null); // eslint-disable-line no-undef
    return window.Bus && b instanceof window.Bus ? b : null;
  };
  // A tecla `?` do SHORTCUTS.md so' existe em quem carrega o help.js; a barra esta' em todas as
  // paginas, entao e' ela quem o traz.
  const ajuda = () => {
    if (window.HELP) return window.HELP.bindKey();
    const s = document.createElement("script");
    s.src = "help.js";
    s.onload = () => window.HELP && window.HELP.bindKey();
    document.head.appendChild(s);
  };
  const arranca = () => {
    const el = document.getElementById("nav");
    if (!el) return;
    ajuda();
    const pronto = () => NAV.monta(document, el, daPagina() || new window.Bus({}).connect());
    if (window.Bus) return pronto();
    const s = document.createElement("script");
    s.src = "bus.js";
    s.onload = pronto;
    document.head.appendChild(s);
  };
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", arranca);
  else arranca();
}
