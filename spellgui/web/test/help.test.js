"use strict";
// node --test spellgui/web/test/
// As duas regras do help.js: ler a tabela de `design/SHORTCUTS.md` e resumir os argumentos de um
// comando do registry. O resto da pagina e' DOM e nao tem regra.

const { test } = require("node:test");
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");
const HELP = require("../help.js");

const MD = fs.readFileSync(
  path.join(__dirname, "..", "..", "..", "design", "SHORTCUTS.md"),
  "utf8"
);
const CMDS = JSON.parse(
  fs.readFileSync(path.join(__dirname, "..", "dev", "commands.json"), "utf8")
);

test("tabela: cabecalho, sem separador, celulas aparadas", () => {
  const md = ["## X", "", "| a | b |", "|---|---|", "| 1 | 2 |", "|3|4|", "", "texto"].join("\n");
  assert.deepStrictEqual(HELP.tabela(md, "X"), [
    ["a", "b"],
    ["1", "2"],
    ["3", "4"],
  ]);
});

test("tabela: para na primeira tabela depois do titulo pedido", () => {
  const md = ["| z |", "|---|", "| antes |", "## X", "| a |", "|---|", "| 1 |", "", "| depois |"]
    .join("\n");
  assert.deepStrictEqual(HELP.tabela(md, "X"), [["a"], ["1"]]);
  assert.deepStrictEqual(HELP.tabela(md), [["z"], ["antes"]], "sem titulo = a primeira");
  assert.deepStrictEqual(HELP.tabela(md, "nao existe"), []);
  assert.deepStrictEqual(HELP.tabela(""), []);
  assert.deepStrictEqual(HELP.tabela(null), []);
});

test("tabela: o Mapa padrao do SHORTCUTS.md tem quatro colunas e a coluna estado", () => {
  const rows = HELP.tabela(MD, HELP.TABELA);
  assert.ok(rows.length > 30, "linhas: " + rows.length);
  assert.deepStrictEqual(rows[0], ["Ação", "Tecla", "Origem", "Estado"]);
  for (const r of rows) assert.strictEqual(r.length, 4, "linha irregular: " + r.join(" | "));
  const estados = new Set(rows.slice(1).map(r => r[3]));
  assert.deepStrictEqual([...estados].sort(), ["falta", "feito", "n.a."]);
  // a celula com a barra invertida do Resolve sobrevive ao split
  assert.ok(
    rows.some(r => r[1].indexOf("\\") >= 0),
    "a linha de zoom perdeu o `\\`"
  );
});

test("args: ordem do schema, `?` no que nao e' obrigatorio", () => {
  const c = n => CMDS.find(x => x.name === n);
  assert.deepStrictEqual(HELP.args(c("load")), ["file"]);
  assert.deepStrictEqual(HELP.args(c("locate")), ["t"]);
  assert.deepStrictEqual(HELP.args(c("show_get")), ["file?", "full?"]);
  assert.deepStrictEqual(HELP.args(c("pause")), [], "comando sem argumento e' botao");
  assert.deepStrictEqual(HELP.args(null), []);
});

test("todo comando do registry tem doc e todo argumento tem descricao", () => {
  for (const c of CMDS) {
    assert.ok(c.doc && c.doc.length > 10, c.name + ": sem doc");
    const p = (c.params && c.params.properties) || {};
    for (const k of Object.keys(p)) {
      assert.ok(p[k].description, c.name + "." + k + ": sem description");
    }
  }
});
