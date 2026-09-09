"use strict";
// node --test spellgui/web/test/graph.test.js
// So' a logica pura do PATCHBAY: tipo de porta, ops de patch e o inverso, Shift+Delete, grupo.
// Sem DOM: graph.js so' toca o document dentro de PB.init().

const test = require("node:test");
const assert = require("node:assert");
const CATALOG = require("../catalog.js");
const { GM } = require("../graph.js");

function demo() {
  return {
    name: "t",
    graph: {
      nodes: [
        { id: "space", type: "in.key", key: "Space", x: 0, y: 0 },
        { id: "toggle", type: "logic.toggle", x: 200, y: 0 },
        { id: "go", type: "cmd", cmd: "cue_go", x: 400, y: 0 },
        { id: "osc", type: "in.osc", address: "/a", x: 0, y: 200, group: "seg" },
        { id: "param", type: "out.param", target: "laser/limit/r", x: 200, y: 200, group: "seg" },
      ],
      edges: [
        ["space.down", "toggle.in"],
        ["toggle.out", "go.trigger"],
        ["osc.out", "param.in"],
        ["go.done", "param.in"],
      ],
    },
  };
}

test("tipo de porta: igual liga, trigger e bool se ligam, number nao vira trigger", () => {
  const d = demo();
  assert.equal(GM.porQue(d, "space.down", "toggle.in"), "");        // trigger -> trigger
  assert.equal(GM.porQue(d, "toggle.out", "go.trigger"), "");       // bool -> trigger
  assert.equal(GM.porQue(d, "osc.out", "param.in"), "");            // number -> number
  assert.match(GM.porQue(d, "osc.out", "toggle.in"), /nao liga/);   // number -> trigger
  assert.match(GM.porQue(d, "space.nope", "toggle.in"), /saida/);   // pino que nao existe
  assert.match(GM.porQue(d, "toggle.out", "toggle.in"), /nele mesmo/);
});

test("catalogo: todo no tem cfg, ins e outs; module.json vira no", () => {
  for (const [t, d] of Object.entries(CATALOG.CAT)) {
    assert.ok(d.cfg && d.ins && d.outs, t);
  }
  const md = CATALOG.moduleDef({
    name: "laser",
    parameters: { "geo/scale": { type: "float" }, "limit/r": { type: "float" } },
    values: { "stats/pps": { type: "int" } },
    commands: { shutter: { context: "action" } },
  });
  assert.deepEqual(Object.keys(md.ins), ["geo/scale", "limit/r", "shutter"]);
  assert.equal(md.ins.shutter, "trigger");
  assert.deepEqual(Object.keys(md.outs), ["stats/pps"]);
  assert.equal(CATALOG.busca("toggle", {}).length, 1);
});

test("patch: aplica e o inverso devolve byte a byte", () => {
  const d = demo(), antes = JSON.stringify(d);
  const ops = GM.opsAdd(d, { id: "novo", type: "logic.not", x: 10, y: 10 });
  const r = GM.patch(d, ops);
  assert.equal(r.error, "");
  assert.equal(GM.g(r.doc).nodes.length, 6);
  assert.equal(JSON.stringify(d), antes);                 // patch nao mexe no original
  const v = GM.patch(r.doc, r.undo);
  assert.equal(v.error, "");
  assert.equal(JSON.stringify(v.doc), antes);
});

test("patch: op invalida no meio nao aplica nada", () => {
  const d = demo(), antes = JSON.stringify(d);
  const r = GM.patch(d, [
    { op: "replace", path: "/graph/nodes/0/x", value: 99 },
    { op: "remove", path: "/graph/nodes/77" },
  ]);
  assert.match(r.error, /fora da lista/);
  assert.equal(JSON.stringify(r.doc), antes);
});

test("ops de chave, movimento e ligacao", () => {
  let d = demo();
  d = GM.patch(d, GM.opsChave(d, "toggle", "mute", true)).doc;
  assert.equal(GM.no(d, "toggle").mute, true);
  d = GM.patch(d, GM.opsMove(d, [["toggle", 33.4, 44.6]])).doc;
  assert.equal(GM.no(d, "toggle").x, 33);
  assert.equal(GM.no(d, "toggle").y, 45);
  // uma entrada aceita UM cabo: ligar em param.in troca os dois que ja' chegavam la'
  d = GM.patch(d, GM.opsLiga(d, "space.down", "param.in")).doc;
  const entram = GM.g(d).edges.filter(e => e[1] === "param.in");
  assert.deepEqual(entram, [["space.down", "param.in"]]);
});

test("Delete apaga os cabos do no; Shift+Delete religa entrada na saida", () => {
  const d = demo();
  const so = GM.patch(d, GM.opsDel(d, ["toggle"]));
  assert.equal(so.error, "");
  assert.equal(GM.g(so.doc).nodes.length, 4);
  assert.equal(GM.g(so.doc).edges.filter(e => e.join().includes("toggle")).length, 0);
  assert.equal(GM.g(so.doc).edges.length, 2);

  const re = GM.patch(d, GM.opsDelReconecta(d, ["toggle"]));
  assert.equal(re.error, "");
  assert.ok(GM.g(re.doc).edges.some(e => e[0] === "space.down" && e[1] === "go.trigger"));
  assert.equal(GM.g(re.doc).nodes.length, 4);
  // e o inverso do religar devolve o estado exato
  assert.equal(JSON.stringify(GM.patch(re.doc, re.undo).doc), JSON.stringify(d));
});

test("Shift+Delete nao religa quando o tipo nao bate", () => {
  const d = demo();
  // osc(number) -> param(number): apagar param nao tem saida; apagar osc nao tem entrada.
  // caso real: apagar `go` ligaria toggle.out(bool) em param.in(number): recusado.
  const r = GM.patch(d, GM.opsDelReconecta(d, ["go"]));
  assert.equal(r.error, "");
  assert.ok(!GM.g(r.doc).edges.some(e => e[0] === "toggle.out" && e[1] === "param.in"));
});

test("grupo: colapsa e expoe so' os pinos que cruzam a borda, em ordem estavel", () => {
  const d = demo();
  const v = GM.visiveis(d, "");
  assert.deepEqual(v.nodes.map(n => n.id), ["space", "toggle", "go"]);
  assert.deepEqual([...v.grupos.keys()], ["seg"]);

  const p = GM.portasGrupo(d, v.grupos.get("seg"));
  assert.deepEqual(p.ins.map(x => x.interno), ["param.in"]);
  assert.deepEqual(p.ins.map(x => x.externo), ["go.done"]);
  assert.deepEqual(p.outs, []);                       // nenhum cabo sai do grupo

  const dentro = GM.visiveis(d, "seg");
  assert.deepEqual(dentro.nodes.map(n => n.id), ["osc", "param"]);
  assert.equal(dentro.grupos.size, 0);
});

test("Delete com grupo fechado selecionado apaga os nos do grupo", () => {
  const d = demo();
  // a selecao guarda o NOME do grupo fechado, que nao e' id de no: sem GM.ids, Delete nao faz nada
  assert.deepEqual(GM.ids(d, new Set(["seg"])), ["osc", "param"]);
  assert.deepEqual(GM.ids(d, new Set(["toggle", "seg"])), ["toggle", "osc", "param"]);
  const r = GM.patch(d, GM.opsDel(d, GM.ids(d, new Set(["seg"]))));
  assert.equal(r.error, "");
  assert.deepEqual(GM.g(r.doc).nodes.map(n => n.id), ["space", "toggle", "go"]);
  assert.equal(GM.g(r.doc).edges.length, 2);
});

test("show sem graph: a primeira edicao cria /graph junto, e o undo tira", () => {
  const sem = { name: "medgrupo", tracks: [] }, antes = JSON.stringify(sem);
  const node = { id: "toggle", type: "logic.toggle", x: 0, y: 0 };

  // sem o /graph na frente, nem local nem no engine o add acha o pai
  assert.match(GM.patch(sem, GM.opsAdd(sem, node)).error, /caminho sem pai/);

  const ops = GM.comGraph(sem, GM.opsAdd(sem, node));
  assert.deepEqual(ops[0], { op: "add", path: "/graph", value: { nodes: [], edges: [] } });
  assert.equal(ops.length, 2);

  const r = GM.patch(sem, ops);
  assert.equal(r.error, "");
  assert.deepEqual(GM.g(r.doc).nodes, [node]);
  assert.deepEqual(r.undo[0], { op: "remove", path: "/graph/nodes/0" });
  assert.deepEqual(r.undo[1], { op: "remove", path: "/graph" });
  assert.equal(JSON.stringify(GM.patch(r.doc, r.undo).doc), antes);   // volta ao show original

  // show que ja' tem graph passa a lista intacta (mesma referencia: nada a acrescentar)
  const com = demo(), o2 = GM.opsAdd(com, node);
  assert.equal(GM.comGraph(com, o2), o2);
});

// Shift+A: o Enter e o clique so' criam se a lista tiver item. Digitacao real perde o ponto
// ("intimer"), troca por espaco ou vem com Caps: a lista ficava VAZIA e o no' nunca nascia.
test("busca acha o tipo com o separador errado, faltando ou com caixa trocada", () => {
  const so = q => CATALOG.busca(q, {});
  assert.deepEqual(so("in.timer"), ["in.timer"]);
  assert.deepEqual(so("intimer"), ["in.timer"]);
  assert.deepEqual(so("in timer"), ["in.timer"]);
  assert.deepEqual(so(" in.timer "), ["in.timer"]);
  assert.deepEqual(so("IN.TIMER"), ["in.timer"]);
  assert.deepEqual(so("in,timer"), ["in.timer"]);       // numpad de teclado pt-BR manda virgula
  assert.deepEqual(so("mathmap"), ["math.map"]);
  assert.deepEqual(so("logictoggle"), ["logic.toggle"]);
  assert.deepEqual(so("naoexiste"), []);                // o que nao casa continua sem casar
  assert.deepEqual(CATALOG.busca("laser", { laser: {} }), ["module:laser"]);
  assert.equal(so("").length, Object.keys(CATALOG.CAT).length);
});

test("Inspector: os campos do no' selecionado e o patch de um campo", () => {
  const d = demo();
  const c = GM.campos(d, "space");
  assert.deepEqual(c[0], ["key", "string", "Space"]);   // cfg do tipo vem antes das universais
  assert.deepEqual(c.map(x => x[0]), ["key", "mute", "lock", "state", "group", "label"]);
  assert.deepEqual(GM.campos(d, "nao_existe"), []);
  // math.curve: enum e json juntos, e o campo que o no' nao tem chega undefined
  const curva = { graph: { nodes: [{ id: "c", type: "math.curve" }], edges: [] } };
  assert.deepEqual(GM.campos(curva, "c").slice(0, 2), [
    ["curve", "enum:linear|hold|in|out|inout|bezier", undefined],
    ["c", "json", undefined],
  ]);

  // o cfg mora PLANO no no' (graph.rs le `n.get("every")`), e nao em /cfg/<campo>
  const ops = GM.opsChave(d, "space", "key", "Enter");
  assert.deepEqual(ops, [{ op: "replace", path: "/graph/nodes/0/key", value: "Enter" }]);
  assert.equal(GM.no(GM.patch(d, ops).doc, "space").key, "Enter");
});

test("Inspector: campo numerico ou json invalido nao grava", () => {
  assert.deepEqual(GM.valor("number", "2.5"), { v: 2.5 });
  assert.deepEqual(GM.valor("number", "0"), { v: 0 });
  assert.ok(GM.valor("number", "").erro);
  assert.ok(GM.valor("number", "abc").erro);            // NaN viraria null no JSON
  assert.deepEqual(GM.valor("bool", true), { v: true });
  assert.deepEqual(GM.valor("string", 12), { v: "12" });
  assert.deepEqual(GM.valor("enum:a|b", "b"), { v: "b" });
  assert.deepEqual(GM.valor("json", "[0,1]"), { v: [0, 1] });
  assert.ok(GM.valor("json", "[0,").erro);
});

test("novoId nao repete", () => {
  const d = demo();
  assert.equal(GM.novoId(d, "logic.toggle"), "toggle2");
  assert.equal(GM.novoId(d, "math.map"), "map");
});
