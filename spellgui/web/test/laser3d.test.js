"use strict";
// node --test spellgui/web/test/
// The part of the 3D laser page that runs without WebGL: the manifest of the actions (what the MIDI
// front end will consume), the state -> registry command map, and the ILDA reader.

const { test } = require("node:test");
const assert = require("node:assert");

// bind.js and ilda.js are browser files: two stubs are enough (DOM loopback).
// The stub keeps the keydown handler: that is how the keyboard test fires a key with no browser.
global.window = {};
const KEYDOWN = [];
global.document = { addEventListener(t, f) { if (t === "keydown") KEYDOWN.push(f); } };
function press(key, target) { let stopped = false; KEYDOWN[0]({ key: key, target: target || { tagName: "BODY" }, ctrlKey: false, shiftKey: false, altKey: false, metaKey: false, preventDefault() { stopped = true; } }); return stopped; }

const Bind = require("../laser3d/bind.js");
const ILDA = require("../laser3d/ilda.js");
const { cmdFor } = require("../laser3d/engine.js");

test("Bind.manifest: one row per action, with key, MIDI and type", () => {
  Bind.def("key.toggle", "key switch: arms", () => {}, { key: "S", get: () => true });
  Bind.def("kpps", "kpps (fader)", () => {}, { type: "cc" });
  Bind.def("mute", "no key", () => {});
  const m = Bind.manifest();
  assert.deepStrictEqual(m.map(a => a.id), ["key.toggle", "kpps", "mute"], "declaration order");
  assert.deepStrictEqual(m[0], { id: "key.toggle", label: "key switch: arms", key: "S", midi: "", type: "btn" });
  assert.strictEqual(m[1].type, "cc", "a fader is cc, not a button");
  assert.strictEqual(m[2].key, "", "an action with no key does not invent one");
});

test("cmdFor: the key switch opens and closes the DAC", () => {
  assert.deepStrictEqual(cmdFor("key", true, { dac: "etherdream", host: "10.0.0.9", kpps: 30000 }),
    { cmd: "laser_open", args: { dac: "etherdream", host: "10.0.0.9", kpps: 30 } });
  assert.deepStrictEqual(cmdFor("key", false, { feed: 7 }), { cmd: "laser_close", args: { feed: 7 } });
  assert.strictEqual(cmdFor("key", false, {}), null, "disarming with no open feed sends nothing");
});

test("cmdFor: play plays the .ild on the feed, and the transport moves with the show", () => {
  assert.deepStrictEqual(cmdFor("play", true, { feed: 2, file: "shows/a.ild", fps: 25 }),
    { cmd: "laser_play", args: { feed: 2, file: "shows/a.ild", fps: 25, loop: true } });
  assert.deepStrictEqual(cmdFor("play", false, { feed: 2 }), { cmd: "laser_stop", args: { feed: 2 } });
  assert.strictEqual(cmdFor("play", true, { feed: 2 }), null, "with no file there is nothing to play");
  assert.strictEqual(cmdFor("play", true, {}), null, "with no feed there is nowhere to play");
  assert.deepStrictEqual(cmdFor("transport", true, { show: "medgrupo" }), { cmd: "resume", args: {} });
  assert.deepStrictEqual(cmdFor("transport", false, { show: "medgrupo" }), { cmd: "pause", args: {} });
  assert.strictEqual(cmdFor("transport", true, {}), null, "with no show loaded the transport stays quiet");
});

test("cmdFor: the parameters laser_param accepts today, and only those", () => {
  const st = { feed: 1 };
  assert.deepStrictEqual(cmdFor("size", 1.25, st), { cmd: "laser_param", args: { feed: 1, path: "geo/scale", value: 1.25 } });
  assert.deepStrictEqual(cmdFor("lim.g", 0.5, st), { cmd: "laser_param", args: { feed: 1, path: "limit/g", value: 0.5 } });
  // interlock closed = shutter open
  assert.deepStrictEqual(cmdFor("lock", true, st), { cmd: "laser_param", args: { feed: 1, path: "shutter", value: 0 } });
  assert.deepStrictEqual(cmdFor("lock", false, st), { cmd: "laser_param", args: { feed: 1, path: "shutter", value: 1 } });
  for (const id of ["kpps", "gam.r", "dmx", "buffer", "speed", "net.sacn", "power", "cam.show"]) {
    assert.strictEqual(cmdFor(id, 1, st), null, id + " has no command yet: it stays local");
  }
  assert.strictEqual(cmdFor("size", 1, {}), null, "with no open feed there is nothing to parametrize");
});

test("ILDA: reads back the frame it wrote (format 5, RGB)", () => {
  const frame = [
    { x: 0, y: 0, r: 255, g: 0, b: 0, bl: true },
    { x: 10000, y: -10000, r: 0, g: 255, b: 0, bl: false },
    { x: -32768, y: 32767, r: 1, g: 2, b: 3, bl: false },
  ];
  const d = ILDA.parse(ILDA.write([frame]).buffer);
  assert.strictEqual(d.frames.length, 1);
  assert.strictEqual(d.name, "SPELL", "the header carries the name of whoever wrote it");
  assert.deepStrictEqual(d.frames[0], frame);
});

test("ILDA: data that is not ILDA gives back zero frames, without blowing up", () => {
  assert.deepStrictEqual(ILDA.parse(new Uint8Array(64).buffer).frames, []);
});

// ---- display and controls (ui-3d-usab front) ----
const { oledLines, CONTROLS, kindOf, PAGES } = require("../laser3d/engine.js");

test("oledLines: off it only says it is off; on, the big line is the state", () => {
  assert.strictEqual(oledLines({ power: false }, {})[1], "OFF");
  const disarmed = oledLines({ power: true, key: false, lock: true, page: 0, kpps: 30000, show: [[1, 2, 3]], frame: 0 }, {});
  assert.strictEqual(disarmed[1], "DISARMED");
  assert.match(disarmed[0], /^STATUS +1\/7$/);
  assert.strictEqual(oledLines({ power: true, key: true, lock: true, page: 0, kpps: 30000, show: [[]], frame: 0 }, {})[1], "LIVE");
  assert.strictEqual(oledLines({ power: true, key: true, lock: false, page: 0, kpps: 30000, show: [[]], frame: 0 }, {})[1], "SCAN FAIL");
});

test("oledLines: a field being edited carries '>' and the ERROR page shows the last error", () => {
  const dmx = oledLines({ power: true, page: PAGES.indexOf("DMX"), edit: true, field: 1, dmx: 7, univ: 3 }, {});
  assert.strictEqual(dmx[1], "ADDR 007");
  assert.strictEqual(dmx[2][0], " ", "field 0 is not selected");
  assert.strictEqual(dmx[3][0], ">", "field 1 selected");
  const err = oledLines({ power: true, page: PAGES.indexOf("ERROR"), err: { msg: "laser_open: no DAC", when: "20:34:00" } }, {});
  assert.strictEqual(err[1], "ERROR");
  assert.strictEqual(err[2], " LASER_OPEN: NO DAC");
  assert.ok(oledLines({ power: true, page: 0 }, {}).every(l => /^[\x20-\x7e]*$/.test(l)), "the display is ASCII only");
});

test("CONTROLS: one control, one family; power in a single spot", () => {
  const fam = new Set(["toggle", "momentary", "value", "nav", "map", "part"]);
  for (const k in CONTROLS) assert.ok(fam.has(CONTROLS[k][0]), k + " has a known family");
  assert.strictEqual(kindOf("power"), "toggle", "the rocker turns it on and off");
  const toggles = Object.keys(CONTROLS).filter(k => CONTROLS[k][0] === "toggle");
  assert.deepStrictEqual(toggles.sort(), ["keyswitch", "power"], "each toggle is a unique function");
  assert.strictEqual(kindOf("fuse"), "", "there is no fuse");
});

// ---- hover only on a control, and the interlock as an input (hud-4 front) ----
const { inert, labelOf } = require("../laser3d/engine.js");

test("a part with no function: no label, inert, and the USB does not exist any more", () => {
  // the owner: "I do not want hover menus on the front, lid and fins", "I do not want a hover
  // menu on the ac", "I do not want a hover menu on the fan", "I want you to delete the USB input".
  for (const k of ["lid", "side", "front", "aperture", "acin", "fan", "bench", "dichro", "fold", "psu", "pcb", "dac"])
    assert.strictEqual(CONTROLS[k][1], "", k + " has no tooltip");
  for (const k of ["side", "front", "aperture", "acin", "fan", "bench", "dichro", "fold", "psu", "pcb", "dac"])
    assert.ok(inert(k), k + " is a part: it does not light up and the click does nothing");
  assert.strictEqual(CONTROLS.usb, undefined, "the USB port left the device");
  assert.ok(inert("usb"), "a key outside the table is an inert part");
  assert.strictEqual(labelOf("usb"), "", "an unknown key does not become a tooltip with its own name");
  // the lid stays clickable (opening = preferences), only with no tooltip
  assert.strictEqual(kindOf("lid"), "nav");
  assert.ok(!inert("lid"), "the lid opens on click");
  assert.ok(!inert("pino.ilda"), "the pins of the Pino have their own owner and label");
  assert.ok(!inert("enc") && !inert("power") && !inert("rj45"), "a control is still a control");
});

test("the interlock is an input: family 'map', not toggle", () => {
  assert.strictEqual(kindOf("interlock"), "map", "clicking opens where you map what drives it");
  assert.ok(!inert("interlock"));
  assert.ok(CONTROLS.interlock[1], "the interlock still says what it is in the tooltip");
});

// The keyboard belongs to the device: a focused <input type=range> (the panel fader) can only keep the
// keys it uses. Before, any focused input returned early and killed the whole keyboard.
test("keyboard: a focused fader does not kill the device keys; Escape cancels the MIDI learn", () => {
  let n = 0;
  Bind.def("test.key", "test", () => { n++; }, { key: "Q" });
  assert.ok(press("q"), "a device key answers and consumes the event");
  assert.strictEqual(n, 1);
  press("q", { tagName: "INPUT", type: "range" });
  assert.strictEqual(n, 2, "a focused fader does not kill the device keyboard");
  assert.strictEqual(press("ArrowUp", { tagName: "INPUT", type: "range" }), false, "the arrow belongs to the fader");
  press("q", { tagName: "INPUT", type: "text" });
  assert.strictEqual(n, 2, "a text field swallows the key");
  press("q", { tagName: "DIV", isContentEditable: true });
  assert.strictEqual(n, 2, "an editable field too");

  Bind.click({ target: { closest: () => ({ dataset: { learn: "midi", id: "test.key" } }) } });
  assert.ok(Bind.learnState(), "MIDI learn armed");
  press("Q");
  assert.ok(Bind.learnState(), "during MIDI learn the key does not become a binding");
  assert.strictEqual(n, 2, "it does not fire the action either");
  press("Escape");
  assert.strictEqual(Bind.learnState(), null, "Escape cancels the MIDI learn");
});
