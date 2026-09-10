"use strict";
// node --test spellgui/web/test/
// The two rules of help.js: reading the table of `design/SHORTCUTS.md` and summarizing the
// arguments of a registry command. The rest of the page is DOM and has no rule.

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

test("table: header, no separator, trimmed cells", () => {
  const md = ["## X", "", "| a | b |", "|---|---|", "| 1 | 2 |", "|3|4|", "", "text"].join("\n");
  assert.deepStrictEqual(HELP.table(md, "X"), [
    ["a", "b"],
    ["1", "2"],
    ["3", "4"],
  ]);
});

test("table: it stops at the first table after the requested heading", () => {
  const md = ["| z |", "|---|", "| before |", "## X", "| a |", "|---|", "| 1 |", "", "| after |"]
    .join("\n");
  assert.deepStrictEqual(HELP.table(md, "X"), [["a"], ["1"]]);
  assert.deepStrictEqual(HELP.table(md), [["z"], ["before"]], "no heading = the first one");
  assert.deepStrictEqual(HELP.table(md, "does not exist"), []);
  assert.deepStrictEqual(HELP.table(""), []);
  assert.deepStrictEqual(HELP.table(null), []);
});

// The heading comes as a list because SHORTCUTS.md exists in two languages: any of the accepted
// headings finds the table, and a list with none of them finds nothing.
test("table: a list of headings takes whichever one is in the file", () => {
  const md = ["## Default map", "| a |", "|---|", "| 1 |"].join("\n");
  const pt = ["## Mapa padrão", "| a |", "|---|", "| 1 |"].join("\n");
  assert.deepStrictEqual(HELP.table(md, HELP.TABLE), [["a"], ["1"]]);
  assert.deepStrictEqual(HELP.table(pt, HELP.TABLE), [["a"], ["1"]]);
  assert.deepStrictEqual(HELP.table(md, ["A", "B"]), []);
});

// Structural assertions: the SHORTCUTS.md text belongs to another file and may be translated, so
// what is checked here is the shape of the table, not its words.
test("table: the default map of SHORTCUTS.md has four columns and a status column", () => {
  const rows = HELP.table(MD, HELP.TABLE);
  assert.ok(rows.length > 30, "rows: " + rows.length);
  assert.strictEqual(rows[0].length, 4, "header: " + rows[0].join(" | "));
  for (const r of rows) assert.strictEqual(r.length, 4, "irregular row: " + r.join(" | "));
  const states = new Set(rows.slice(1).map(r => r[3]));
  assert.ok(states.size >= 2 && states.size <= 4, "states: " + [...states].join(", "));
  for (const s of states) assert.ok(s && s.length < 12, "state cell: " + s);
  // the cell with the Resolve backslash survives the split
  assert.ok(
    rows.some(r => r[1].indexOf("\\") >= 0),
    "the zoom row lost the `\\`"
  );
});

test("args: schema order, `?` on what is not required", () => {
  const c = n => CMDS.find(x => x.name === n);
  assert.deepStrictEqual(HELP.args(c("load")), ["file"]);
  assert.deepStrictEqual(HELP.args(c("locate")), ["t"]);
  assert.deepStrictEqual(HELP.args(c("show_get")), ["file?", "full?"]);
  assert.deepStrictEqual(HELP.args(c("pause")), [], "a command with no argument is a button");
  assert.deepStrictEqual(HELP.args(null), []);
});

test("every registry command has a doc and every argument has a description", () => {
  for (const c of CMDS) {
    assert.ok(c.doc && c.doc.length > 10, c.name + ": no doc");
    const p = (c.params && c.params.properties) || {};
    for (const k of Object.keys(p)) {
      assert.ok(p[k].description, c.name + "." + k + ": no description");
    }
  }
});
