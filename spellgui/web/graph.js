"use strict";
// graph.js — PATCHBAY: o editor de rede de comando do graph (PRD §10, design/FUNCOES/orquestrador.md).
// Duas metades no mesmo arquivo:
//   GM  — modelo PURO (JSON Patch, ops de edicao, religar, grupo). Sem DOM: e o que o node --test roda.
//   PB  — a pagina (canvas sobre o canvaskit, Inspector, log, atalhos). So `PB.init()` toca o DOM.
//
// Toda edicao vira `show_patch {ops}` em /graph/... e empilha o `undo` devolvido (frente patch).
// Offline (sem engine) o mesmo caminho roda local: GM.patch aplica e devolve o inverso.
//
// ponytail: sem minimap, sem auto-layout, sem edicao colaborativa ; entram quando um show real
// passar de uma tela de nos.

// node --test: catalog.js e' modulo CommonJS; no navegador ele ja' declarou CATALOG global.
if (typeof require === "function") global.CATALOG = require("./catalog.js");

// ---------------------------------------------------------------- modelo puro (sem DOM)

const GM = {
  modules: {},          // module.json (frente module) ja' virado definicao de no, por nome

  def(n) { return CATALOG.nodeDef(n, GM.modules); },

  // JSON Pointer (RFC 6901). ponytail: sem unescape de ~0/~1 ; nenhum segmento do graph tem "/" ou "~".
  seg(p) { return p.split("/").slice(1); },

  get(doc, path) {
    let c = doc;
    for (const s of GM.seg(path)) {
      if (c === null || c === undefined) return undefined;
      c = Array.isArray(c) ? c[+s] : c[s];
    }
    return c;
  },

  pai(doc, path) {
    const p = GM.seg(path), last = p.pop();
    let c = doc;
    for (const s of p) {
      if (c === null || c === undefined) return null;
      c = Array.isArray(c) ? c[+s] : c[s];
    }
    return c === null || c === undefined ? null : { c, last };
  },

  clone(v) { return structuredClone(v); },

  // Aplica a lista inteira ou nada: devolve {doc, undo, error}. `undo` desfaz na ordem inversa.
  // ponytail: so' add, remove e replace do RFC 6902 ; `test` e `move` entram quando algum
  // construtor de ops precisar deles (reordenar hoje e' remove+add).
  patch(doc, ops) {
    const d = GM.clone(doc), undo = [];
    for (const op of ops) {
      const e = GM.uma(d, op, undo);
      if (e) return { doc, undo: [], error: e };
    }
    undo.reverse();
    return { doc: d, undo, error: "" };
  },

  uma(d, op, undo) {
    const p = op.path === undefined ? "" : op.path;
    const alvo = GM.pai(d, p);
    if (!alvo) return `caminho sem pai: ${p}`;
    const { c, last } = alvo;
    if (Array.isArray(c)) {
      const i = last === "-" ? c.length : +last;
      if (op.op === "add") {
        if (!(i >= 0 && i <= c.length)) return `indice fora da lista: ${p}`;
        c.splice(i, 0, GM.clone(op.value));
        undo.push({ op: "remove", path: p.replace(/\/[^/]*$/, "/" + i) });
        return "";
      }
      if (!(i >= 0 && i < c.length)) return `indice fora da lista: ${p}`;
      if (op.op === "remove") {
        undo.push({ op: "add", path: p.replace(/\/[^/]*$/, "/" + i), value: GM.clone(c[i]) });
        c.splice(i, 1);
        return "";
      }
      undo.push({ op: "replace", path: p, value: GM.clone(c[i]) });
      c[i] = GM.clone(op.value);
      return "";
    }
    if (typeof c !== "object") return `caminho nao e' objeto nem lista: ${p}`;
    const tinha = Object.prototype.hasOwnProperty.call(c, last);
    if (op.op === "remove") {
      if (!tinha) return `remove: ${p} nao existe`;
      undo.push({ op: "add", path: p, value: GM.clone(c[last]) });
      delete c[last];
      return "";
    }
    if (op.op === "replace" && !tinha) return `replace: ${p} nao existe`;
    undo.push(tinha ? { op: "replace", path: p, value: GM.clone(c[last]) } : { op: "remove", path: p });
    c[last] = GM.clone(op.value);
    return "";
  },

  // ------------------------------------------------------------ graph dentro do show

  g(doc) { return (doc && doc.graph) || { nodes: [], edges: [] }; },

  // "no.pino" -> ["no", "pino"] (mesmo rsplit_once('.') de graph.rs: o id pode ter ponto, o pino nao)
  lado(s) { const i = String(s).lastIndexOf("."); return [s.slice(0, i), s.slice(i + 1)]; },

  idx(doc, id) { return GM.g(doc).nodes.findIndex(n => n.id === id); },
  no(doc, id) { return GM.g(doc).nodes.find(n => n.id === id) || null; },

  // Id livre a partir do tipo: "logic.toggle" -> "toggle", "toggle2", ...
  novoId(doc, tipo) {
    const base = String(tipo).split(".").pop();
    const ids = new Set(GM.g(doc).nodes.map(n => n.id));
    if (!ids.has(base)) return base;
    for (let i = 2; ; i++) if (!ids.has(base + i)) return base + i;
  },

  opsAdd(doc, node) {
    return [{ op: "add", path: "/graph/nodes/-", value: node }];
  },

  // Show sem `graph` (o caso do medgrupo.spell): `add /graph/nodes/-` nao acha o pai nem aqui
  // nem no engine. A PRIMEIRA edicao cria o graph junto, na mesma lista de ops — e o undo do
  // add (`remove /graph`) devolve o show exatamente como estava.
  comGraph(doc, ops) {
    if (doc && doc.graph) return ops;
    return [{ op: "add", path: "/graph", value: { nodes: [], edges: [] } }].concat(ops);
  },

  // Apaga nos e todo cabo que os toca. Indices em ordem decrescente: o patch e' sequencial.
  opsDel(doc, ids) {
    const g = GM.g(doc), set = new Set(ids), ops = [];
    const toca = e => set.has(GM.lado(e[0])[0]) || set.has(GM.lado(e[1])[0]);
    g.edges.forEach((e, i) => { if (toca(e)) ops.push({ op: "remove", path: `/graph/edges/${i}` }); });
    g.nodes.forEach((n, i) => { if (set.has(n.id)) ops.push({ op: "remove", path: `/graph/nodes/${i}` }); });
    return ops.reverse();
  },

  // Shift+Delete: apaga religando a entrada do no na saida dele (Blender delete_reconnect).
  // Cada no apagado liga a origem do seu primeiro cabo de entrada a todos os destinos das saidas.
  opsDelReconecta(doc, ids) {
    const g = GM.g(doc), set = new Set(ids), novos = [];
    for (const id of ids) {
      const ent = g.edges.filter(e => GM.lado(e[1])[0] === id && !set.has(GM.lado(e[0])[0]));
      const sai = g.edges.filter(e => GM.lado(e[0])[0] === id && !set.has(GM.lado(e[1])[0]));
      if (!ent.length) continue;
      for (const s of sai) {
        const par = [ent[0][0], s[1]];
        if (GM.porQue(doc, par[0], par[1])) continue;                 // tipo incompativel: nao religa
        if (!novos.some(x => x[0] === par[0] && x[1] === par[1])) novos.push(par);
      }
    }
    return GM.opsDel(doc, ids).concat(
      novos.map(e => ({ op: "add", path: "/graph/edges/-", value: e })),
    );
  },

  // "" quando o cabo pode existir, senao o motivo.
  porQue(doc, from, to) {
    const [ai, ap] = GM.lado(from), [bi, bp] = GM.lado(to);
    const a = GM.no(doc, ai), b = GM.no(doc, bi);
    if (!a || !b) return "no nao existe";
    if (ai === bi) return "cabo do no nele mesmo";
    return CATALOG.compat(CATALOG.port(GM.def(a), ap, true), CATALOG.port(GM.def(b), bp, false));
  },

  // Uma entrada aceita UM cabo: ligar troca o que estava la (o "trocar a entrada" do TouchDesigner).
  opsLiga(doc, from, to) {
    const g = GM.g(doc), ops = [];
    g.edges.forEach((e, i) => { if (e[1] === to) ops.push({ op: "remove", path: `/graph/edges/${i}` }); });
    ops.reverse();
    ops.push({ op: "add", path: "/graph/edges/-", value: [from, to] });
    return ops;
  },

  opsChave(doc, id, chave, valor) {
    const i = GM.idx(doc, id);
    if (i < 0) return [];
    const tinha = Object.prototype.hasOwnProperty.call(GM.g(doc).nodes[i], chave);
    return [{ op: tinha ? "replace" : "add", path: `/graph/nodes/${i}/${chave}`, value: valor }];
  },

  opsMove(doc, mov) {
    const ops = [];
    for (const [id, x, y] of mov) {
      ops.push(...GM.opsChave(doc, id, "x", Math.round(x)));
      ops.push(...GM.opsChave(doc, id, "y", Math.round(y)));
    }
    return ops;
  },

  // ------------------------------------------------------------ grupos

  grupo(n) { return (n && n.group) || ""; },

  // A selecao guarda id de no OU nome de grupo fechado (o desenho trata o grupo como um no);
  // toda edicao trabalha com id de no, entao o nome de grupo vira a lista dos membros.
  ids(doc, sel) {
    return [...sel].flatMap(s => (GM.no(doc, s) ? [s]
      : GM.g(doc).nodes.filter(n => GM.grupo(n) === s).map(n => n.id)));
  },

  // Nos visiveis no contexto `ctx` e os grupos fechados. Ordem estavel: a do arquivo.
  // ponytail: grupo e' um nome PLANO (a chave `group` do no), sem aninhamento ; virar caminho
  // "a/b" quando um show real tiver grupo dentro de grupo.
  visiveis(doc, ctx) {
    const g = GM.g(doc), dentro = [], grupos = new Map();
    for (const n of g.nodes) {
      const gr = GM.grupo(n);
      if (gr === ctx) dentro.push(n);
      else if (ctx === "") grupos.set(gr, (grupos.get(gr) || []).concat([n]));
    }
    return { nodes: dentro, grupos };
  },

  // Pinos de um grupo fechado: os cabos que cruzam a borda, ordenados por id.pino (deterministico).
  portasGrupo(doc, membros) {
    const g = GM.g(doc), set = new Set(membros.map(n => n.id)), ins = [], outs = [];
    for (const e of g.edges) {
      const de = set.has(GM.lado(e[0])[0]), pa = set.has(GM.lado(e[1])[0]);
      if (de === pa) continue;
      (pa ? ins : outs).push({ interno: pa ? e[1] : e[0], externo: pa ? e[0] : e[1] });
    }
    const ord = (a, b) => a.interno.localeCompare(b.interno);
    ins.sort(ord); outs.sort(ord);
    return { ins, outs };
  },
};

// ---------------------------------------------------------------- a pagina

const NW = 176, TH = 22, PH = 14, CFGH = 14;   // largura do no, altura do titulo, passo do pino

const PB = {
  doc: { graph: { nodes: [], edges: [] } },
  ctx: "", sel: new Set(), selEdge: -1, undo: [], redo: [],
  erros: {}, k: null, bus: null, mx: 0, my: 0, el: {},

  /// Socket aberto? O `Bus` descarta request com o socket fechado; o patchbay prefere nem mandar.
  vivo() { return !!(PB.bus && PB.bus.ws && PB.bus.ws.readyState === 1); },

  // mundo -> tela
  sx(wx) { return (wx - PB.k.view.x) * PB.k.view.zoom; },
  sy(wy) { return wy * PB.k.view.zoom - PB.k.view.y; },

  // altura do no a partir do numero de pinos
  alt(n) {
    const d = GM.def(n);
    const p = d ? Math.max(Object.keys(d.ins).length, Object.keys(d.outs).length) : 1;
    return TH + Math.max(1, p) * PH + CFGH;
  },

  // Caixa do no em mundo. Durante o arrasto o delta e' do drag: o modelo so' muda no `up`.
  caixa(n) {
    const d = PB.k && PB.k.drag && PB.k.drag.mode === "node" ? PB.k.drag : null;
    const m = d && d.mov.get(n.id);
    return {
      x: m ? m[0] + d.dx : +n.x || 0,
      y: m ? Math.max(0, m[1] + d.dy) : +n.y || 0,
      w: NW, h: PB.alt(n),
    };
  },

  // ------------------------------------------------------------ edicao (um caminho so')

  // Manda a lista ao engine. Recusa volta o doc local ao estado de antes (`GM.patch` nao mexe
  // no original: a referencia guardada JA' e' a copia intacta) e `volta` desfaz a pilha — sem
  // doc local adiantado do engine. Aceita ou recusa, no fim revalida.
  manda(ops, antes, volta) {
    if (!PB.vivo()) return Promise.resolve(PB.checa());
    // O `rev` da resposta o proprio `Bus` guarda (`bus.rev`); e' contra ele que o evento decide.
    return PB.bus.call("show_patch", { ops })
      .catch(e => {
        PB.doc = antes;
        volta();
        PB.k.invalidate();
        PB.inspetor();
        PB.log("show_patch: " + e.message);
      })
      .then(PB.checa);
  },

  // Toda edicao passa por aqui: manda show_patch, empilha o undo, revalida com graph_check.
  aplica(ops, rotulo) {
    if (!ops.length) return Promise.resolve();
    ops = GM.comGraph(PB.doc, ops);
    const antes = PB.doc;
    const r = GM.patch(PB.doc, ops);
    if (r.error) { PB.log("erro: " + r.error); return Promise.resolve(); }
    PB.doc = r.doc;
    PB.redo.length = 0;
    PB.undo.push(r.undo);
    PB.k.invalidate();
    PB.inspetor();
    if (rotulo) PB.log(rotulo);
    return PB.manda(ops, antes, () => PB.undo.pop());
  },

  desfaz(pilhaA, pilhaB) {
    const ops = pilhaA.pop();
    if (!ops) return;
    const antes = PB.doc;
    const r = GM.patch(PB.doc, ops);
    if (r.error) { PB.log("undo: " + r.error); return; }
    PB.doc = r.doc;
    pilhaB.push(r.undo);
    PB.k.invalidate();
    PB.inspetor();
    PB.manda(ops, antes, () => { pilhaB.pop(); pilhaA.push(ops); });
  },

  // graph_check depois de cada edicao aceita e no recarrega: erro por no, marcado na caixa e no
  // Inspector. SEM argumento — o comando compila o graph do show ABERTO NO ENGINE, e e' esse o
  // ponto: depois de um show_patch aceito o doc da pagina e' o do engine, entao a fonte da
  // validacao e' o engine. Offline (sem bus vivo) nao ha' o que validar.
  // ponytail: o engine devolve UM erro ; vira contagem por no quando graph_check devolver lista.
  checa() {
    PB.erros = {};
    if (!PB.vivo()) {
      PB.el.aviso.textContent = "";
      PB.el.aviso.className = "over";
      PB.k.invalidate();
      return Promise.resolve();
    }
    return PB.bus.call("graph_check", {}).then(r => {
      const txt = r && r.error;
      if (txt) {
        const m = /"([^"]+)"/.exec(txt);              // graph.rs cita o no culpado entre aspas
        PB.erros[m ? m[1] : ""] = txt;
        PB.log("graph_check: " + txt);
      }
      PB.el.aviso.textContent = txt ? "1 erro" : `${(r && r.nodes) || 0} nos ok`;
      PB.el.aviso.className = "over" + (txt ? " ruim" : "");
      PB.k.invalidate();
    }).catch(() => {});
  },

  log(t) {
    const d = document.createElement("div");
    d.textContent = t;
    PB.el.log.appendChild(d);
    while (PB.el.log.childElementCount > 200) PB.el.log.firstChild.remove();
    PB.el.log.scrollTop = PB.el.log.scrollHeight;
  },

  // ------------------------------------------------------------ geometria

  // Ponto do pino em coordenadas de mundo; membro de grupo fechado sai pela borda do grupo.
  ancora(id, pino, saida) {
    const vis = PB.vis;
    const n = vis.mapa.get(id);
    if (n) {
      const c = PB.caixa(n), d = GM.def(n);
      const lista = Object.keys(saida ? d.outs : d.ins);
      const i = Math.max(0, lista.indexOf(pino));
      return { x: c.x + (saida ? c.w : 0), y: c.y + TH + i * PH + PH / 2 };
    }
    const gb = vis.deMembro.get(id);
    if (!gb) return null;
    const lista = saida ? gb.portas.outs : gb.portas.ins;
    const i = Math.max(0, lista.findIndex(p => GM.lado(p.interno)[0] === id));
    return { x: gb.x + (saida ? gb.w : 0), y: gb.y + TH + i * PH + PH / 2 };
  },

  // Recalcula o que esta' visivel no contexto atual (nos soltos + grupos fechados).
  monta() {
    const v = GM.visiveis(PB.doc, PB.ctx);
    const mapa = new Map(v.nodes.map(n => [n.id, n]));
    const caixas = [], deMembro = new Map();
    for (const [nome, membros] of v.grupos) {
      const cs = membros.map(n => PB.caixa(n));
      const portas = GM.portasGrupo(PB.doc, membros);
      const gb = {
        grupo: nome, membros, portas,
        x: Math.min(...cs.map(c => c.x)), y: Math.min(...cs.map(c => c.y)), w: NW,
        h: TH + Math.max(1, portas.ins.length, portas.outs.length) * PH + CFGH,
      };
      caixas.push(gb);
      for (const m of membros) deMembro.set(m.id, gb);
    }
    PB.vis = { nodes: v.nodes, mapa, grupos: caixas, deMembro };
  },

  // no ou grupo sob o ponto de mundo (grupo fechado por cima, como no desenho)
  achaNo(x, y) {
    const dentro = c => x >= c.x && x <= c.x + c.w && y >= c.y && y <= c.y + c.h;
    for (let i = PB.vis.grupos.length - 1; i >= 0; i--) if (dentro(PB.vis.grupos[i])) return PB.vis.grupos[i];
    for (let i = PB.vis.nodes.length - 1; i >= 0; i--) {
      if (dentro(PB.caixa(PB.vis.nodes[i]))) return PB.vis.nodes[i];
    }
    return null;
  },

  // pino sob o ponto (raio de 7 px de mundo)
  achaPino(x, y) {
    for (const n of PB.vis.nodes) {
      const d = GM.def(n);
      if (!d) continue;
      for (const saida of [false, true]) {
        const lista = Object.keys(saida ? d.outs : d.ins);
        for (const p of lista) {
          const a = PB.ancora(n.id, p, saida);
          if (a && Math.abs(a.x - x) < 8 && Math.abs(a.y - y) < 7) {
            return { id: n.id, pino: p, saida, x: a.x, y: a.y };
          }
        }
      }
    }
    return null;
  },

  // cabo sob o ponto de TELA (px do canvas): o proprio tracado do desenho responde pelo hit-test
  achaCabo(px, py) {
    const g = GM.g(PB.doc), cx = PB.k.cx;
    cx.save();                                                // o lineWidth abaixo e' do hit-test,
    cx.lineWidth = 10;                                        // tolerancia do clique, em px de tela
    let achou = -1;
    for (let i = g.edges.length - 1; i >= 0 && achou < 0; i--) {
      const p = PB.pontosCabo(g.edges[i]);
      if (p && cx.isPointInStroke(caminho(p), px * PB.k.dpr, py * PB.k.dpr)) achou = i;
    }
    cx.restore();                                             // nao do desenho do quadro seguinte.
    return achou;
  },

  pontosCabo(e) {
    const [ai, ap] = GM.lado(e[0]), [bi, bp] = GM.lado(e[1]);
    const a = PB.ancora(ai, ap, true), b = PB.ancora(bi, bp, false);
    if (!a || !b) return null;
    if (PB.vis.deMembro.get(ai) && PB.vis.deMembro.get(ai) === PB.vis.deMembro.get(bi)) return null;
    return { a, b };
  },
};

// Tracado do cabo em coordenadas de tela: desenho e hit-test usam o mesmo.
function caminho(p) {
  const d = Math.max(30, Math.abs(p.b.x - p.a.x) * 0.5), c = new Path2D();
  c.moveTo(PB.sx(p.a.x), PB.sy(p.a.y));
  c.bezierCurveTo(PB.sx(p.a.x + d), PB.sy(p.a.y), PB.sx(p.b.x - d), PB.sy(p.b.y),
                  PB.sx(p.b.x), PB.sy(p.b.y));
  return c;
}

// ---------------------------------------------------------------- desenho


function desenha(k) {
  const cx = k.cx, z = k.view.zoom;
  PB.monta();
  cx.clearRect(0, 0, k.w, k.h);
  cx.fillStyle = PB.col.well;
  cx.fillRect(0, 0, k.w, k.h);

  // grade de console (8 px do design system), so' quando da' para ver
  if (z > 0.35) {
    cx.strokeStyle = PB.col.hair;
    cx.lineWidth = 1;
    cx.beginPath();
    const p = 48 * z;
    for (let x = -((k.view.x * z) % p); x < k.w; x += p) { cx.moveTo(x | 0, 0); cx.lineTo(x | 0, k.h); }
    for (let y = -(k.view.y % p); y < k.h; y += p) { cx.moveTo(0, y | 0); cx.lineTo(k.w, y | 0); }
    cx.stroke();
  }

  // caixas de estado: envolvem os nos com a chave "state" apontando para o no state
  for (const s of PB.vis.nodes.filter(n => n.type === "state")) {
    const m = PB.vis.nodes.filter(n => n.state === s.id);
    if (!m.length) continue;
    const cs = m.map(n => PB.caixa(n));
    const x0 = Math.min(...cs.map(c => c.x)) - 12, y0 = Math.min(...cs.map(c => c.y)) - 12;
    const x1 = Math.max(...cs.map(c => c.x)) + NW + 12, y1 = Math.max(...cs.map(c => c.y + c.h)) + 12;
    cx.strokeStyle = PB.col.rehearsal;
    cx.setLineDash([6, 4]);
    cx.strokeRect(PB.sx(x0), PB.sy(y0), (x1 - x0) * z, (y1 - y0) * z);
    cx.setLineDash([]);
    cx.fillStyle = PB.col.rehearsal;
    cx.font = `${Math.max(9, 10 * z)}px ${PB.col.mono}`;
    cx.fillText(`estado ${s.id}`, PB.sx(x0) + 4, PB.sy(y0) - 3);
  }

  // cabos
  GM.g(PB.doc).edges.forEach((e, i) => {
    const p = PB.pontosCabo(e);
    if (!p) return;
    cx.strokeStyle = i === PB.selEdge ? PB.col.accent : PB.col.fg3;
    cx.lineWidth = i === PB.selEdge ? 2 : 1.5;
    cx.stroke(caminho(p));
  });

  // cabo em construcao
  if (k.drag && k.drag.mode === "wire") {
    cx.strokeStyle = PB.col.accent;
    cx.setLineDash([4, 3]);
    cx.beginPath();
    cx.moveTo(PB.sx(k.drag.x), PB.sy(k.drag.y));
    cx.lineTo(PB.mx, PB.my);
    cx.stroke();
    cx.setLineDash([]);
  }

  for (const n of PB.vis.nodes) noBox(cx, k, n);
  for (const gb of PB.vis.grupos) grupoBox(cx, k, gb);

  const m = k.rect();
  if (m) {
    cx.strokeStyle = PB.col.accent;
    cx.setLineDash([3, 3]);
    cx.strokeRect(m.x0, m.y0, m.x1 - m.x0, m.y1 - m.y0);
    cx.setLineDash([]);
  }

  cx.fillStyle = PB.col.fg3;
  cx.font = `11px ${PB.col.mono}`;
  cx.fillText(PB.ctx ? `grupo: ${PB.ctx}  (Ctrl+[ sai)` : "raiz", 8, k.h - 8);
}

function pino(cx, x, y, tipo, forte) {
  const s = CATALOG.PORT_SHAPE[tipo] || "circ";
  cx.fillStyle = forte ? PB.col.accent : PB.col.fg2;
  cx.beginPath();
  if (s === "tri") { cx.moveTo(x - 3, y - 4); cx.lineTo(x + 4, y); cx.lineTo(x - 3, y + 4); }
  else if (s === "sq") cx.rect(x - 3, y - 3, 6, 6);
  else if (s === "dia") { cx.moveTo(x, y - 4); cx.lineTo(x + 4, y); cx.lineTo(x, y + 4); cx.lineTo(x - 4, y); }
  else cx.arc(x, y, 3.5, 0, 6.284);
  cx.fill();
}

function resumo(n, d) {
  const k = Object.keys(d.cfg)[0];
  if (n.type === "module") return n.module || "";
  return k && n[k] !== undefined ? `${k}=${n[k]}` : "";
}

function noBox(cx, k, n) {
  const z = k.view.zoom, c = PB.caixa(n), d = GM.def(n);
  const x = PB.sx(c.x), y = PB.sy(c.y), w = c.w * z, h = c.h * z;
  if (x > k.w || y > k.h || x + w < 0 || y + h < 0) return;
  const sel = PB.sel.has(n.id), err = PB.erros[n.id];
  cx.globalAlpha = n.mute ? 0.4 : 1;
  cx.fillStyle = PB.col.panel2;
  cx.fillRect(x, y, w, h);
  cx.strokeStyle = err ? PB.col.live : sel ? PB.col.accent : PB.col.line;
  cx.lineWidth = sel || err ? 2 : 1;
  cx.strokeRect(x, y, w, h);
  cx.fillStyle = PB.col.panel;
  cx.fillRect(x, y, w, TH * z);
  if (z > 0.3) {
    const f = Math.max(7, 11 * z);
    cx.fillStyle = PB.col[d ? "no_" + d.fam : "live"] || PB.col.fg;
    cx.font = `${f}px ${PB.col.mono}`;
    cx.fillText(n.label || n.id, x + 6, y + TH * z * 0.68);
    cx.fillStyle = PB.col.fg3;
    cx.font = `${Math.max(6, 9 * z)}px ${PB.col.mono}`;
    cx.fillText(n.type, x + w - 6 - cx.measureText(n.type).width, y + TH * z * 0.68);
    if (d) cx.fillText(resumo(n, d), x + 6, y + h - 4);
    const marca = (n.mute ? "M" : "") + (n.lock ? "L" : "") + (err ? "!" : "");
    if (marca) { cx.fillStyle = PB.col.live; cx.fillText(marca, x + w - 14, y + h - 4); }
  }
  if (d) {
    Object.values(d.ins).forEach((t, i) => pino(cx, x, PB.sy(c.y + TH + i * PH + PH / 2), t, sel));
    Object.values(d.outs).forEach((t, i) => pino(cx, x + w, PB.sy(c.y + TH + i * PH + PH / 2), t, sel));
  }
  cx.globalAlpha = 1;
}

function grupoBox(cx, k, gb) {
  const z = k.view.zoom, x = PB.sx(gb.x), y = PB.sy(gb.y), w = gb.w * z, h = gb.h * z;
  cx.fillStyle = PB.col.panel;
  cx.fillRect(x, y, w, h);
  cx.strokeStyle = PB.sel.has(gb.grupo) ? PB.col.accent : PB.col.fg3;
  cx.lineWidth = 2;
  cx.strokeRect(x, y, w, h);
  if (z > 0.3) {
    cx.fillStyle = PB.col.fg;
    cx.font = `${Math.max(7, 11 * z)}px ${PB.col.mono}`;
    cx.fillText(`[${gb.grupo}]`, x + 6, y + TH * z * 0.68);
    cx.fillStyle = PB.col.fg3;
    cx.font = `${Math.max(6, 9 * z)}px ${PB.col.mono}`;
    cx.fillText(`${gb.membros.length} nos  Ctrl+]`, x + 6, y + h - 4);
  }
  gb.portas.ins.forEach((p, i) => pino(cx, x, PB.sy(gb.y + TH + i * PH + PH / 2), "number", false));
  gb.portas.outs.forEach((p, i) => pino(cx, x + w, PB.sy(gb.y + TH + i * PH + PH / 2), "number", false));
}

// ---------------------------------------------------------------- Inspector, catalogo, busca

function campo(rot, tipo, valor, onSet) {
  const l = document.createElement("label");
  l.textContent = rot;
  let i;
  if (tipo === "bool") {
    i = document.createElement("input");
    i.type = "checkbox";
    i.checked = !!valor;
    i.onchange = () => onSet(i.checked);
  } else if (tipo.startsWith("enum:")) {
    i = document.createElement("select");
    i.innerHTML = tipo.slice(5).split("|").map(o => `<option>${o}</option>`).join("");
    i.value = valor === undefined ? "" : String(valor);
    i.onchange = () => onSet(i.value);
  } else {
    i = document.createElement("input");
    i.type = tipo === "number" ? "number" : "text";
    i.value = tipo === "json" ? JSON.stringify(valor === undefined ? null : valor) : (valor === undefined ? "" : valor);
    i.onchange = () => {
      let v = i.value;
      if (tipo === "number") v = +v;
      if (tipo === "json") { try { v = JSON.parse(v); } catch (e) { PB.log("json invalido"); return; } }
      onSet(v);
    };
  }
  l.appendChild(i);
  return l;
}

PB.inspetor = function () {
  const box = PB.el.insp;
  box.textContent = "";
  const ids = [...PB.sel];
  if (ids.length !== 1) {
    box.appendChild(Object.assign(document.createElement("div"), {
      className: "over", textContent: ids.length ? `${ids.length} selecionados` : "nada selecionado",
    }));
    return;
  }
  const n = GM.no(PB.doc, ids[0]);
  if (!n) return;
  const d = GM.def(n);
  const t = Object.assign(document.createElement("div"), { className: "over", textContent: `${n.id} — ${n.type}` });
  box.appendChild(t);
  if (PB.erros[n.id]) {
    box.appendChild(Object.assign(document.createElement("div"), { className: "ruim", textContent: PB.erros[n.id] }));
  }
  const cfg = Object.assign({}, d ? d.cfg : {}, CATALOG.UNIVERSAL);
  for (const [chave, tipo] of Object.entries(cfg)) {
    box.appendChild(campo(chave, tipo, n[chave], v => PB.aplica(GM.opsChave(PB.doc, n.id, chave, v), `${n.id}.${chave} = ${v}`)));
  }
};

// A lista de tipos: a coluna da esquerda (q = "") e a busca do Shift+A sao a mesma.
function lista(box, q, cria) {
  box.textContent = "";
  for (const t of CATALOG.busca(q, GM.modules)) {
    const b = document.createElement("button");
    b.textContent = t;
    b.onclick = () => cria(t);
    box.appendChild(b);
  }
}

function catalogo() {
  lista(PB.el.cat, "", t => criaNo(t, PB.k.toWorld(PB.k.w / 2), (PB.k.view.y + PB.k.h / 2) / PB.k.view.zoom));
}

function criaNo(tipo, x, y) {
  const mod = tipo.startsWith("module:") ? tipo.slice(7) : "";
  const t = mod ? "module" : tipo;
  const n = { id: GM.novoId(PB.doc, mod || t), type: t, x: Math.round(x), y: Math.round(y) };
  if (mod) n.module = mod;
  if (PB.ctx) n.group = PB.ctx;
  PB.sel = new Set([n.id]);
  return PB.aplica(GM.opsAdd(PB.doc, n), `+ ${t} ${n.id}`);
}

// Shift+A: busca que filtra a partir do primeiro caractere (regra 10) e cria no cursor.
function abreBusca() {
  const el = PB.el.add, r = PB.el.cv.getBoundingClientRect();   // mx/my sao do canvas; o menu e' fixed
  el.style.display = "block";
  el.style.left = (r.left + PB.mx) + "px";
  el.style.top = (r.top + PB.my) + "px";
  const inp = el.querySelector("input"), caixa = el.querySelector("div");
  const wx = PB.k.toWorld(PB.mx), wy = (PB.my + PB.k.view.y) / PB.k.view.zoom;
  inp.value = "";
  const pinta = () => lista(caixa, inp.value, t => { el.style.display = "none"; criaNo(t, wx, wy); });
  inp.oninput = pinta;
  inp.onkeydown = e => {
    e.stopPropagation();
    if (e.key === "Escape") el.style.display = "none";
    if (e.key === "Enter" && caixa.firstChild) caixa.firstChild.click();
  };
  pinta();
  inp.focus();
}

// ---------------------------------------------------------------- atalhos

function onKey(e) {
  if (e.target.tagName === "INPUT" || e.target.tagName === "SELECT") return;
  const ids = GM.ids(PB.doc, PB.sel);          // nome de grupo fechado vira os ids dos membros
  // teclado sem caixa: "A" e "a" sao a mesma tecla; o modificador e' que manda (SHORTCUTS.md)
  const k = e.key.length === 1 ? e.key.toLowerCase() : e.key;
  if (k === "a" && e.shiftKey && !e.ctrlKey) { e.preventDefault(); return abreBusca(); }
  if (k === "Delete") {
    e.preventDefault();
    if (!ids.length) {                          // nada selecionado: Delete age no cabo selecionado
      const i = PB.selEdge;
      if (i < 0) return;
      PB.selEdge = -1;
      return void PB.aplica([{ op: "remove", path: `/graph/edges/${i}` }], "- cabo");
    }
    const ops = e.shiftKey ? GM.opsDelReconecta(PB.doc, ids) : GM.opsDel(PB.doc, ids);
    PB.sel.clear();
    return void PB.aplica(ops, (e.shiftKey ? "- religando " : "- ") + ids.join(" "));
  }
  if (e.ctrlKey && k === "z") {
    e.preventDefault();
    return e.shiftKey ? PB.desfaz(PB.redo, PB.undo) : PB.desfaz(PB.undo, PB.redo);
  }
  if (e.ctrlKey && k === "]") {
    const n = GM.no(PB.doc, ids[0]);
    const alvo = PB.vis.grupos.find(g => PB.sel.has(g.grupo)) || (n && GM.grupo(n) ? { grupo: GM.grupo(n) } : null);
    if (alvo) { PB.ctx = alvo.grupo; PB.sel.clear(); PB.k.invalidate(); }
    return;
  }
  if (e.ctrlKey && k === "[") {
    PB.ctx = "";
    PB.sel.clear();
    return PB.k.invalidate();
  }
  if (e.ctrlKey && k === "a") {
    e.preventDefault();
    PB.sel = new Set(PB.vis.nodes.map(n => n.id));
    PB.inspetor();
    return PB.k.invalidate();
  }
  if (k === "m" && !e.ctrlKey && ids.length) {
    const ops = ids.flatMap(id => GM.opsChave(PB.doc, id, "mute", !GM.no(PB.doc, id).mute));
    return void PB.aplica(ops, "mute " + ids.join(" "));
  }
  if (k === "l" && !e.ctrlKey && ids.length) {
    const ops = ids.flatMap(id => GM.opsChave(PB.doc, id, "lock", !GM.no(PB.doc, id).lock));
    return void PB.aplica(ops, "lock " + ids.join(" "));
  }
  if (k === "g" && e.shiftKey && ids.length) {
    const nome = prompt("grupo:", PB.ctx || "grupo");
    if (nome === null) return;
    return void PB.aplica(ids.flatMap(id => GM.opsChave(PB.doc, id, "group", nome)), "grupo " + nome);
  }
  if (k === "z" && e.shiftKey && !e.ctrlKey) return enquadra();
}

function enquadra() {
  PB.k.resize();
  if (PB.k.w < 2) return requestAnimationFrame(enquadra);   // o show chega antes do layout do canvas
  PB.monta();
  const ns = PB.vis.nodes;
  if (!ns.length) return;
  const x0 = Math.min(...ns.map(n => +n.x || 0)), x1 = Math.max(...ns.map(n => (+n.x || 0) + NW));
  PB.k.fit(x0 - 20, x1 + 20);
  PB.k.view.y = Math.min(...ns.map(n => +n.y || 0)) * PB.k.view.zoom - 20;
  PB.k.invalidate();
}

// ---------------------------------------------------------------- init

PB.init = function (opts) {
  PB.el = opts;
  const cv = opts.cv;
  PB.col = CK.cores(cv);

  const k = CK.attach(cv, desenha);
  PB.k = k;
  k.view.zoom = 1;
  PB.monta();

  const zoom0 = k.zoomAt;
  k.zoomAt = (px, f) => {                        // o kit ancora so' o X; o Y do graph tambem ancora
    const wy = (PB.my + k.view.y) / k.view.zoom;
    zoom0(px, f);
    k.view.y = Math.max(0, wy * k.view.zoom - PB.my);
  };

  const mundo = p => ({ x: k.toWorld(p.x), y: (p.y + k.view.y) / k.view.zoom });

  k.on.hover = p => { PB.mx = p.x; PB.my = p.y; };

  k.on.down = p => {
    PB.mx = p.x; PB.my = p.y;
    const w = mundo(p);
    const pin = PB.achaPino(w.x, w.y);
    if (pin) {
      k.drag = { mode: "wire", from: pin, x: pin.x, y: pin.y };
      return true;
    }
    const n = PB.achaNo(w.x, w.y);
    if (n) {
      const id = n.id || n.grupo;
      if (!p.shift && !PB.sel.has(id)) PB.sel.clear();
      PB.sel.add(id);
      PB.selEdge = -1;
      PB.inspetor();
      const mov = new Map();
      for (const i of GM.ids(PB.doc, PB.sel)) {
        const no = GM.no(PB.doc, i);
        if (no && !no.lock) mov.set(i, [+no.x || 0, +no.y || 0]);
      }
      k.drag = { mode: "node", mov, w, dx: 0, dy: 0 };
      k.dirty = true;
      return true;
    }
    PB.selEdge = PB.achaCabo(p.x, p.y);
    if (PB.selEdge >= 0) { PB.sel.clear(); PB.inspetor(); k.dirty = true; return true; }
    return false;
  };

  // Arrasto = delta no `drag` (o desenho soma em PB.caixa); o modelo so' muda no `up`, por patch.
  k.on.move = (p, d) => {
    PB.mx = p.x; PB.my = p.y;
    if (d.mode === "node") {
      const w = mundo(p);
      d.dx = w.x - d.w.x;
      d.dy = w.y - d.w.y;
    }
    k.dirty = true;
  };

  k.on.up = (p, d) => {
    PB.mx = p.x; PB.my = p.y;
    const w = mundo(p);
    if (d.mode === "wire") {
      const alvo = PB.achaPino(w.x, w.y);
      if (!alvo || alvo.saida === d.from.saida) return;
      const [a, b] = d.from.saida ? [d.from, alvo] : [alvo, d.from];
      const from = `${a.id}.${a.pino}`, to = `${b.id}.${b.pino}`;
      const por = GM.porQue(PB.doc, from, to);
      if (por) { PB.log(`cabo recusado: ${por}`); return; }
      PB.aplica(GM.opsLiga(PB.doc, from, to), `cabo ${from} -> ${to}`);
      return;
    }
    if (d.mode === "node" && (d.dx || d.dy)) {
      const mov = [...d.mov].map(([id, [x0, y0]]) => [id, x0 + d.dx, Math.max(0, y0 + d.dy)]);
      PB.aplica(GM.opsMove(PB.doc, mov), "");
    }
  };

  k.on.marquee = (r, add) => {
    if (!add) PB.sel.clear();
    for (const n of PB.vis.nodes) {
      const c = PB.caixa(n);
      const x = PB.sx(c.x), y = PB.sy(c.y);
      if (x < r.x1 && x + c.w * k.view.zoom > r.x0 && y < r.y1 && y + c.h * k.view.zoom > r.y0) PB.sel.add(n.id);
    }
    PB.inspetor();
  };

  addEventListener("keydown", onKey);
  addEventListener("pointerdown", e => {
    if (!PB.el.add.contains(e.target)) PB.el.add.style.display = "none";
  }, true);

  catalogo();
  PB.inspetor();
  k.loop();
};

// "Abrir". `path` e' o caminho como o ENGINE ve: "shows/patchbay_demo.spell", relativo a raiz
// do repo — e' a mesma string que `serve` e `python -m http.server` na raiz servem em "/".
// Com engine: `load` no engine e depois o show DELE por `show_get`. Sem engine: o arquivo.
// UM caminho de cada vez — o fetch correndo junto com o show_get era a corrida que deixava a
// pagina editando um documento e o engine outro.
PB.carrega = function (path) {
  if (PB.vivo()) {
    return PB.bus.call("load", { path })
      .then(() => PB.recarrega(true))
      .catch(e => PB.log(`load ${path}: ${e.message}`));
  }
  return fetch("/" + path).then(r => r.json()).then(d => {
    PB.doc = d;
    requestAnimationFrame(enquadra);
    PB.log(`show ${path}: ${GM.g(PB.doc).nodes.length} nos`);
    return PB.checa();
  }).catch(e => PB.log(`${path}: ${e.message}`));
};

// Boot: liga o bus e deixa o socket decidir a fonte. Abriu -> o show do engine; nao abriu (o
// `Bus` emite `close` mesmo no socket que nunca subiu) -> o arquivo, uma vez so'.
PB.liga = function (path) {
  PB.bus = new Bus({}).connect();
  PB.bus.on("log", d => PB.log(d && d.text ? d.text : JSON.stringify(d)));
  // `bus.rev` e' o maior rev ja' visto numa resposta: evento acima disso e' edicao de fora.
  PB.bus.on("show", d => { if (d && d.rev > PB.bus.rev) PB.recarrega(); });
  let arquivo = true;                              // o fetch ainda esta' na mesa?
  PB.bus.on("open", () => {
    arquivo = false;                               // engine achado: o arquivo nunca mais entra —
    PB.el.estado.textContent = "engine";           // socket que cai depois nao apaga o que o
    PB.recarrega(true);                            // engine mandou (ele volta em 1 s).
  });
  PB.bus.on("close", () => {
    PB.el.estado.textContent = "offline";
    if (arquivo) { arquivo = false; PB.carrega(path); }
  });
};

PB.recarrega = function (enquadrar) {
  PB.bus.call("show_get", { full: true }).then(s => {
    PB.doc = s;
    if (enquadrar) requestAnimationFrame(enquadra); else PB.k.invalidate();
    PB.checa();
  }).catch(e => PB.log("show_get: " + e.message));
  PB.bus.call("module_list", {}).then(l => {
    for (const m of l || []) {
      PB.bus.call("module_get", { name: m.name || m }).then(mo => {
        GM.modules[mo.name] = CATALOG.moduleDef(mo);
        catalogo();
      }).catch(() => {});
    }
  }).catch(() => {});     // frente module ainda nao em main: sem modulos vivos, so' o catalogo fechado
};

if (typeof module !== "undefined" && module.exports) module.exports = { GM };
if (typeof window !== "undefined") { window.GM = GM; window.PB = PB; }
