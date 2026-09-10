"use strict";
// node --test spellgui/web/test/timeline.test.js
// Covers the three pure functions the timeline gained when it connected to the engine: the
// translation of a local edit into registry calls (TL.ops), the reading of the binary monitor frame
// (TL.frameBin) and the decision to reload the show from the `rev` of the `show` event
// (TL.revEvent). timeline.js is a browser script: load it with a fake `window` and `CK`, no DOM.

const { test } = require("node:test");
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");

function loadTL() {
  const src = fs.readFileSync(path.join(__dirname, "..", "timeline.js"), "utf8");
  const win = {};
  const CK = {
    clamp: (v, a, b) => (v < a ? a : v > b ? b : v),
    bisect(ts, n, t) {
      let lo = 0, hi = n;
      while (lo < hi) { const m = (lo + hi) >> 1; if (ts[m] < t) lo = m + 1; else hi = m; }
      return lo;
    },
    near: () => -1,
    sel: () => ({}),
  };
  new Function("window", "CK", src)(win, CK);
  return win.TL;
}

const TL = loadTL();

test("a new keyframe becomes key_set with t, value and curve", () => {
  const c = TL.ops([{ k: "set", track: 2, t: 1.5, value: 200, curve: "inout" }]);
  assert.deepStrictEqual(c, [
    { cmd: "key_set", args: { track: 2, t: 1.5, value: 200, curve: "inout" } },
  ]);
});

test("a deleted keyframe becomes key_del", () => {
  assert.deepStrictEqual(TL.ops([{ k: "del", track: 0, t: 3.25 }]), [
    { cmd: "key_del", args: { track: 0, t: 3.25 } },
  ]);
});

test("a dragged keyframe becomes key_del of the old time plus key_set of the new one", () => {
  const c = TL.ops([{ k: "move", track: 1, from: 2, t: 4, value: 10, curve: "linear" }]);
  assert.deepStrictEqual(c, [
    { cmd: "key_del", args: { track: 1, t: 2 } },
    { cmd: "key_set", args: { track: 1, t: 4, value: 10, curve: "linear" } },
  ]);
});

// Two neighbouring keyframes swapping places: if the ops were interleaved, the key_del of the
// second would erase the key_set of the first. That is why every del goes out before every set.
test("every key_del goes out before every key_set", () => {
  const c = TL.ops([
    { k: "move", track: 0, from: 1, t: 2, value: 5, curve: "linear" },
    { k: "move", track: 0, from: 2, t: 3, value: 6, curve: "hold" },
  ]);
  assert.deepStrictEqual(c.map(x => x.cmd), ["key_del", "key_del", "key_set", "key_set"]);
  assert.deepStrictEqual(c.map(x => x.args.t), [1, 2, 2, 3]);
});

test("show fields become one single show_patch, in order", () => {
  const c = TL.ops([
    { k: "field", path: "/in", value: 2 },
    { k: "field", path: "/out", value: 8 },
    { k: "field", path: "/markers", value: [1, 2] },
  ]);
  assert.strictEqual(c.length, 1);
  assert.strictEqual(c[0].cmd, "show_patch");
  assert.deepStrictEqual(c[0].args.ops, [
    { op: "add", path: "/in", value: 2 },
    { op: "add", path: "/out", value: 8 },
    { op: "add", path: "/markers", value: [1, 2] },
  ]);
});

test("mixed: keyframes first, show_patch last", () => {
  const c = TL.ops([
    { k: "field", path: "/tracks/0/mute", value: true },
    { k: "set", track: 0, t: 0, value: 1, curve: "linear" },
    { k: "del", track: 0, t: 9 },
  ]);
  assert.deepStrictEqual(c.map(x => x.cmd), ["key_del", "key_set", "show_patch"]);
});

test("with no edit, no call", () => {
  assert.deepStrictEqual(TL.ops([]), []);
});

// Binary frame of the bus: topic:u8 | universe:u16 LE | 512 bytes.
test("binary monitor frame: universe in little endian and 512 channels", () => {
  const b = new Uint8Array(515);
  b[0] = 1;
  b[1] = 0x02; b[2] = 0x01;            // universe 258
  b[3] = 255; b[514] = 7;
  const f = TL.frameBin(b.buffer);
  assert.strictEqual(f.topic, 1);
  assert.strictEqual(f.universe, 258);
  assert.strictEqual(f.data.length, 512);
  assert.strictEqual(f.data[0], 255);
  assert.strictEqual(f.data[511], 7);
});

// topic 2 = INPUT dmx (show.inputs): same format, another destination in the monitor drawing.
test("a topic 2 frame is the input, not the output", () => {
  const b = new Uint8Array(515);
  b[0] = 2; b[1] = 1; b[3] = 99;
  const f = TL.frameBin(b.buffer);
  assert.strictEqual(f.topic, 2);
  assert.strictEqual(f.universe, 1);
  assert.strictEqual(f.data[0], 99);
});

test("a frame with no consumer for its topic, or too short, is ignored", () => {
  const other = new Uint8Array(515);
  other[0] = 3;
  assert.strictEqual(TL.frameBin(other.buffer), null);
  assert.strictEqual(TL.frameBin(new Uint8Array(10).buffer), null);
});

// ---- +Track: a type menu instead of dmx only ----------------------------
test("+Track dmx sends the usual fields", () => {
  assert.deepStrictEqual(TL.trackArgs("dmx"), {
    type: "dmx", universe: 1, address: 1, name: "",
  });
  assert.deepStrictEqual(TL.trackArgs(), { type: "dmx", universe: 1, address: 1, name: "" });
});

test("+Track laser carries the .ild clip", () => {
  assert.deepStrictEqual(TL.trackArgs("laser", "medgrupo_laser.ild"), {
    type: "laser", universe: 1, address: 1, name: "", clip: "medgrupo_laser.ild",
  });
});

test("+Track fx carries the .rhai script", () => {
  assert.deepStrictEqual(TL.trackArgs("fx", "medgrupo.rhai"), {
    type: "fx", universe: 1, address: 1, name: "", script: "medgrupo.rhai",
  });
});

// ---- record arm: the state comes from the engine, not from the .spell ----
test("rec_state marks only the lanes of the armed tracks", () => {
  TL.lanes = [{ si: 0, rec: true }, { si: 1, rec: false }, { si: 1, param: "scale", rec: true }];
  TL.recApply({ recording: true, tracks: [1] });
  assert.deepStrictEqual(TL.lanes.map(L => L.rec), [false, true, false]);
});

test("with nothing armed, every lane disarms", () => {
  TL.lanes = [{ si: 0, rec: true }, { si: 1, rec: true }];
  TL.recApply({ recording: false, tracks: [] });
  assert.deepStrictEqual(TL.lanes.map(L => L.rec), [false, false]);
  TL.recApply(null);
  assert.deepStrictEqual(TL.lanes.map(L => L.rec), [false, false]);
});

// `show` event: there is ONE counter, the engine one, and every bus response brings it. `TL.rev` is
// the highest `rev` ever seen in a response; reload only when the event goes past that number.
test("the echo of our own edit does not reload", () => {
  assert.deepStrictEqual(TL.revEvent(5, 5), { rev: 5, reload: false });
});

test("a rev above the expected one is another client editing: reload", () => {
  assert.deepStrictEqual(TL.revEvent(6, 5), { rev: 6, reload: true });
});

test("a late event with edits still in flight neither reloads nor sets the count back", () => {
  assert.deepStrictEqual(TL.revEvent(4, 6), { rev: 6, reload: false });
});

// The old counter (`expect++` per call) never came back from a drift (a command counted twice, a
// lost event, a page opened against an engine that already had revisions): it swallowed the
// outside reloads forever. With `rev` the drift costs one reload and the count goes back to the
// engine number.
test("a misaligned count fixes itself on the first event", () => {
  const r = TL.revEvent(9, 1);
  assert.deepStrictEqual(r, { rev: 9, reload: true });
  assert.deepStrictEqual(TL.revEvent(10, r.rev), { rev: 10, reload: true });
});

// Reconnection: the new process starts at rev = 0. `revEvent` only corrects the count upwards, so
// the onopen zeroes TL.rev before the reload; without that the first `show` of the new engine
// (rev 1) would fall below the old count and reload nothing.
test("after the reconnection reset, the first event of the new engine reloads", () => {
  assert.deepStrictEqual(TL.revEvent(1, 0), { rev: 1, reload: true });
});

test("with no reset, the restarted engine would be swallowed", () => {
  assert.deepStrictEqual(TL.revEvent(1, 37), { rev: 37, reload: false });
});

// ---- transport: loop and In-Out range -----------------------------------
// Fake `TL.k`: the functions below only mark `dirty` and draw nothing.
function mockShow() {
  TL.k = { dirty: false, view: { x: 0, y: 0, zoom: 40 } };
  TL.show = { name: "t", fps: 30, duration: 60, tracks: [], markers: [], in: 0, out: 60 };
  TL.lanes = [];
  TL.loop = false;
}

// The button lights up at once, but the one who decides is the engine: the `transport` event brings
// the loop back. Before, `TL.loop` was local only and the player kept playing while the page
// pretended to repeat.
test("the loop of the transport event rules the page state", () => {
  mockShow();
  TL.setLoop(true);
  assert.strictEqual(TL.loop, true, "optimistic reflection of the button");
  TL.onTransport({ state: "play", t: 1, loop: false });
  assert.strictEqual(TL.loop, false, "the engine contradicts the button");
  TL.onTransport({ state: "pause", t: 1, loop: true });
  assert.strictEqual(TL.loop, true);
  TL.onTransport({ state: "pause", t: 1 });
  assert.strictEqual(TL.loop, true, "a transport with no loop does not touch what already holds");
});

// The report: In and Out ended up 10 ms apart (31.42 / 31.43) after a drag on the ruler. Now the
// limit that crosses the other throws the other one to the end, and the range never collapses.
test("In and Out do not collapse: the one that was crossed goes to the end", () => {
  mockShow();
  assert.deepStrictEqual(TL.setInOut(10, null), [10, 60]);
  assert.deepStrictEqual(TL.setInOut(null, 20), [10, 20]);
  assert.deepStrictEqual(TL.setInOut(null, 5), [0, 5], "Out before In: In goes back to zero");
  assert.deepStrictEqual(TL.setInOut(30, null), [30, 60], "In after Out: Out goes to the end");
  TL.setInOut(31.42, null);
  assert.deepStrictEqual(TL.setInOut(null, 31.42), [0, 31.42], "I and O at the same instant");
  assert.deepStrictEqual(TL.setInOut(-5, null), [0, 31.42], "outside the show, it clamps at the edge");
  assert.deepStrictEqual(TL.setInOut(null, 999), [0, 60]);
});

// The ruler handle snapped to the other one and the `clamp` of the time left the two 10 ms apart:
// while one is being dragged, In and Out stay out of the snapping list.
test("while dragging the ruler handle, In and Out leave the snapping", () => {
  mockShow();
  TL.show.in = 2;
  TL.show.out = 4;
  TL.t = 1;
  TL.k.w = 800;
  TL.k.toWorld = x => x / 10;
  TL.k.sel = { m: new Map() };
  TL.k.drag = null;
  const free = TL.buildSnaps();
  assert.ok(free.includes(2) && free.includes(4), "with no drag, In and Out snap");
  TL.k.drag = { mode: "in" };
  const handle = TL.buildSnaps();
  assert.ok(!handle.includes(2) && !handle.includes(4), "with the handle in hand: " + handle);
  assert.ok(handle.includes(TL.t), "the playhead keeps snapping");
});

// Undo: a local stack of copies of the show, taken on each commit (the engine does not stack yet).
test("undo and redo give back the show from before the commit", () => {
  mockShow();
  TL.k.sel = { clear() {}, m: new Map() };
  TL.k.fit = () => {};
  TL.k.resize = () => {};
  TL.load({ name: "a", fps: 30, duration: 60, tracks: [], markers: [] });
  TL.show.name = "b";
  TL.commit([]);
  assert.strictEqual(TL.show.name, "b");
  TL.undo(-1);
  assert.strictEqual(TL.show.name, "a", "undo goes back to the show of the load");
  TL.undo(1);
  assert.strictEqual(TL.show.name, "b", "redo gives the commit back");
  TL.undo(1);
  assert.strictEqual(TL.show.name, "b", "with nothing to redo, nothing changes");
});
