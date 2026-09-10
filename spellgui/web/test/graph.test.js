"use strict";
// node --test spellgui/web/test/graph.test.js
// Only the pure logic of the PATCHBAY: port type, patch ops and their inverse, Shift+Delete, group.
// No DOM: graph.js only touches the document inside PB.init().

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

test("port type: same links, trigger and bool link, number is no trigger", () => {
  const d = demo();
  assert.equal(GM.why(d, "space.down", "toggle.in"), "");             // trigger -> trigger
  assert.equal(GM.why(d, "toggle.out", "go.trigger"), "");            // bool -> trigger
  assert.equal(GM.why(d, "osc.out", "param.in"), "");                 // number -> number
  assert.match(GM.why(d, "osc.out", "toggle.in"), /does not link/);   // number -> trigger
  assert.match(GM.why(d, "space.nope", "toggle.in"), /output pin/);   // pin that does not exist
  assert.match(GM.why(d, "toggle.out", "toggle.in"), /into itself/);
});

test("catalogue: every node has cfg, ins and outs; module.json becomes a node", () => {
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
  assert.equal(CATALOG.search("toggle", {}).length, 1);
});

test("patch: it applies and the inverse gives the show back byte for byte", () => {
  const d = demo(), before = JSON.stringify(d);
  const ops = GM.opsAdd(d, { id: "new", type: "logic.not", x: 10, y: 10 });
  const r = GM.patch(d, ops);
  assert.equal(r.error, "");
  assert.equal(GM.g(r.doc).nodes.length, 6);
  assert.equal(JSON.stringify(d), before);                // patch does not touch the original
  const v = GM.patch(r.doc, r.undo);
  assert.equal(v.error, "");
  assert.equal(JSON.stringify(v.doc), before);
});

test("patch: an invalid op in the middle applies nothing", () => {
  const d = demo(), before = JSON.stringify(d);
  const r = GM.patch(d, [
    { op: "replace", path: "/graph/nodes/0/x", value: 99 },
    { op: "remove", path: "/graph/nodes/77" },
  ]);
  assert.match(r.error, /out of the list/);
  assert.equal(JSON.stringify(r.doc), before);
});

test("key, move and link ops", () => {
  let d = demo();
  d = GM.patch(d, GM.opsKey(d, "toggle", "mute", true)).doc;
  assert.equal(GM.node(d, "toggle").mute, true);
  d = GM.patch(d, GM.opsMove(d, [["toggle", 33.4, 44.6]])).doc;
  assert.equal(GM.node(d, "toggle").x, 33);
  assert.equal(GM.node(d, "toggle").y, 45);
  // an input takes ONE cable: linking into param.in replaces the two that already arrived there
  d = GM.patch(d, GM.opsLink(d, "space.down", "param.in")).doc;
  const arriving = GM.g(d).edges.filter(e => e[1] === "param.in");
  assert.deepEqual(arriving, [["space.down", "param.in"]]);
});

test("Delete drops the cables of the node; Shift+Delete reconnects input to output", () => {
  const d = demo();
  const plain = GM.patch(d, GM.opsDel(d, ["toggle"]));
  assert.equal(plain.error, "");
  assert.equal(GM.g(plain.doc).nodes.length, 4);
  assert.equal(GM.g(plain.doc).edges.filter(e => e.join().includes("toggle")).length, 0);
  assert.equal(GM.g(plain.doc).edges.length, 2);

  const re = GM.patch(d, GM.opsDelReconnect(d, ["toggle"]));
  assert.equal(re.error, "");
  assert.ok(GM.g(re.doc).edges.some(e => e[0] === "space.down" && e[1] === "go.trigger"));
  assert.equal(GM.g(re.doc).nodes.length, 4);
  // and the inverse of the reconnect gives the exact state back
  assert.equal(JSON.stringify(GM.patch(re.doc, re.undo).doc), JSON.stringify(d));
});

test("Shift+Delete does not reconnect when the type does not match", () => {
  const d = demo();
  // osc(number) -> param(number): deleting param has no output; deleting osc has no input.
  // real case: deleting `go` would link toggle.out(bool) into param.in(number): refused.
  const r = GM.patch(d, GM.opsDelReconnect(d, ["go"]));
  assert.equal(r.error, "");
  assert.ok(!GM.g(r.doc).edges.some(e => e[0] === "toggle.out" && e[1] === "param.in"));
});

test("group: it collapses and exposes only the pins that cross the border, in stable order", () => {
  const d = demo();
  const v = GM.visible(d, "");
  assert.deepEqual(v.nodes.map(n => n.id), ["space", "toggle", "go"]);
  assert.deepEqual([...v.groups.keys()], ["seg"]);

  const p = GM.groupPorts(d, v.groups.get("seg"));
  assert.deepEqual(p.ins.map(x => x.inner), ["param.in"]);
  assert.deepEqual(p.ins.map(x => x.outer), ["go.done"]);
  assert.deepEqual(p.outs, []);                       // no cable leaves the group

  const inside = GM.visible(d, "seg");
  assert.deepEqual(inside.nodes.map(n => n.id), ["osc", "param"]);
  assert.equal(inside.groups.size, 0);
});

test("Delete with a closed group selected drops the nodes of the group", () => {
  const d = demo();
  // the selection holds the NAME of the closed group, which is no node id: without GM.ids, Delete
  // does nothing
  assert.deepEqual(GM.ids(d, new Set(["seg"])), ["osc", "param"]);
  assert.deepEqual(GM.ids(d, new Set(["toggle", "seg"])), ["toggle", "osc", "param"]);
  const r = GM.patch(d, GM.opsDel(d, GM.ids(d, new Set(["seg"]))));
  assert.equal(r.error, "");
  assert.deepEqual(GM.g(r.doc).nodes.map(n => n.id), ["space", "toggle", "go"]);
  assert.equal(GM.g(r.doc).edges.length, 2);
});

test("show with no graph: the first edit creates /graph along with it, and the undo removes it", () => {
  const bare = { name: "medgrupo", tracks: [] }, before = JSON.stringify(bare);
  const node = { id: "toggle", type: "logic.toggle", x: 0, y: 0 };

  // without the /graph in front, neither here nor in the engine does the add find the parent
  assert.match(GM.patch(bare, GM.opsAdd(bare, node)).error, /no parent/);

  const ops = GM.withGraph(bare, GM.opsAdd(bare, node));
  assert.deepEqual(ops[0], { op: "add", path: "/graph", value: { nodes: [], edges: [] } });
  assert.equal(ops.length, 2);

  const r = GM.patch(bare, ops);
  assert.equal(r.error, "");
  assert.deepEqual(GM.g(r.doc).nodes, [node]);
  assert.deepEqual(r.undo[0], { op: "remove", path: "/graph/nodes/0" });
  assert.deepEqual(r.undo[1], { op: "remove", path: "/graph" });
  assert.equal(JSON.stringify(GM.patch(r.doc, r.undo).doc), before);   // back to the original show

  // a show that already has a graph passes the list through (same reference: nothing to add)
  const withG = demo(), o2 = GM.opsAdd(withG, node);
  assert.equal(GM.withGraph(withG, o2), o2);
});

// Shift+A: Enter and the click only create when the list has an item. Real typing loses the dot
// ("intimer"), swaps it for a space or arrives with Caps: the list came out EMPTY and the node was
// never born.
test("search finds the type with the wrong separator, a missing one or the case swapped", () => {
  const so = q => CATALOG.search(q, {});
  assert.deepEqual(so("in.timer"), ["in.timer"]);
  assert.deepEqual(so("intimer"), ["in.timer"]);
  assert.deepEqual(so("in timer"), ["in.timer"]);
  assert.deepEqual(so(" in.timer "), ["in.timer"]);
  assert.deepEqual(so("IN.TIMER"), ["in.timer"]);
  assert.deepEqual(so("in,timer"), ["in.timer"]);       // a pt-BR keypad sends a comma
  assert.deepEqual(so("mathmap"), ["math.map"]);
  assert.deepEqual(so("logictoggle"), ["logic.toggle"]);
  assert.deepEqual(so("doesnotexist"), []);             // what does not match keeps not matching
  assert.deepEqual(CATALOG.search("laser", { laser: {} }), ["module:laser"]);
  assert.equal(so("").length, Object.keys(CATALOG.CAT).length);
});

test("Inspector: the fields of the selected node and the patch of one field", () => {
  const d = demo();
  const c = GM.fields(d, "space");
  assert.deepEqual(c[0], ["key", "string", "Space"]);   // the cfg of the type comes before the universal keys
  assert.deepEqual(c.map(x => x[0]), ["key", "mute", "lock", "state", "group", "label"]);
  assert.deepEqual(GM.fields(d, "does_not_exist"), []);
  // math.curve: enum and json together, and the field the node does not have arrives undefined
  const curve = { graph: { nodes: [{ id: "c", type: "math.curve" }], edges: [] } };
  assert.deepEqual(GM.fields(curve, "c").slice(0, 2), [
    ["curve", "enum:linear|hold|in|out|inout|bezier", undefined],
    ["c", "json", undefined],
  ]);

  // the cfg lives FLAT on the node (graph.rs reads `n.get("every")`), and not in /cfg/<field>
  const ops = GM.opsKey(d, "space", "key", "Enter");
  assert.deepEqual(ops, [{ op: "replace", path: "/graph/nodes/0/key", value: "Enter" }]);
  assert.equal(GM.node(GM.patch(d, ops).doc, "space").key, "Enter");
});

test("Inspector: an invalid numeric or json field does not get stored", () => {
  assert.deepEqual(GM.value("number", "2.5"), { v: 2.5 });
  assert.deepEqual(GM.value("number", "0"), { v: 0 });
  assert.ok(GM.value("number", "").error);
  assert.ok(GM.value("number", "abc").error);           // NaN would become null in the JSON
  assert.deepEqual(GM.value("bool", true), { v: true });
  assert.deepEqual(GM.value("string", 12), { v: "12" });
  assert.deepEqual(GM.value("enum:a|b", "b"), { v: "b" });
  assert.deepEqual(GM.value("json", "[0,1]"), { v: [0, 1] });
  assert.ok(GM.value("json", "[0,").error);
});

test("newId does not repeat", () => {
  const d = demo();
  assert.equal(GM.newId(d, "logic.toggle"), "toggle2");
  assert.equal(GM.newId(d, "math.map"), "map");
});
