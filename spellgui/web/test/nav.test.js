"use strict";
// node --test spellgui/web/test/nav.test.js
// The bar has three rules: which tab is the current one, which key leads to which page, and when
// the show name goes to the engine (Enter or blur, NEVER on a key). The drawing is DOM and has no
// rule, so the document here is a fake one: only enough for `NAV.mount` to run with no browser.

const { test } = require("node:test");
const assert = require("node:assert");
const NAV = require("../nav.js");

function element(tag) {
  return {
    tagName: String(tag).toUpperCase(),
    children: [],
    listeners: {},
    className: "",
    textContent: "",
    value: "",
    href: "",
    type: "",
    id: "",
    title: "",
    appendChild(c) {
      this.children.push(c);
      return c;
    },
    addEventListener(n, f) {
      (this.listeners[n] = this.listeners[n] || []).push(f);
    },
    fire(n, e) {
      const base = { target: this, stopPropagation() {}, preventDefault() {} };
      for (const f of this.listeners[n] || []) f(Object.assign(base, e));
    },
  };
}

function fakeDoc(pathname) {
  const d = element("document");
  d.head = element("head");
  d.location = { pathname };
  d.createElement = element;
  return d;
}

// Fake bus with the same surface the bar uses: on/call/rev.
function fakeBus(name) {
  return {
    rev: 3,
    calls: [],
    ev: {},
    on(t, f) {
      (this.ev[t] = this.ev[t] || []).push(f);
    },
    emit(t, d) {
      for (const f of this.ev[t] || []) f(d);
    },
    call(cmd, args) {
      this.calls.push({ cmd, args });
      return Promise.resolve(cmd === "show_get" ? { name: name } : { rev: 4 });
    },
  };
}

const mount = (path, name) => {
  const doc = fakeDoc(path);
  const el = element("div");
  const bus = fakeBus(name || "MED GRUPO");
  NAV.mount(doc, el, bus);
  const field = el.children.find(f => f.id === "sc-nav-nome");
  return { doc, el, bus, field };
};
const patches = bus => bus.calls.filter(c => c.cmd === "show_patch");

test("tabs: the current one is the page one, and the root is the timeline", () => {
  const marked = p => NAV.tabs(p).find(a => a.current);
  assert.strictEqual(marked("/spellgui/web/patchbay.html").lab, "PATCHBAY");
  assert.strictEqual(marked("/spellgui/web/face.html").lab, "FACE");
  assert.strictEqual(marked("/spellgui/web/index.html").lab, "TIMELINE");
  assert.strictEqual(marked("/").lab, "TIMELINE");
  assert.strictEqual(marked("/spellgui/web/page_that_does_not_exist.html"), undefined);
  // only one marked, and FACE leads to the default view
  assert.strictEqual(NAV.tabs("/spellgui/web/teatro.html").filter(a => a.current).length, 1);
  assert.strictEqual(NAV.tabs("/").find(a => a.lab === "FACE").href, "face.html?face=quatro");
});

test("Shift+1..6 changes page; with no Shift, with Ctrl or inside a field, it does not", () => {
  const key = e => NAV.target(Object.assign({ target: { tagName: "BODY" } }, e));
  assert.strictEqual(key({ code: "Digit1", shiftKey: true }), "index.html");
  assert.strictEqual(key({ code: "Digit3", shiftKey: true }), "teatro.html");
  assert.strictEqual(key({ code: "Digit5", shiftKey: true }), "laser.html");
  assert.strictEqual(key({ code: "Digit6", shiftKey: true }), "help.html");
  assert.strictEqual(key({ code: "Digit7", shiftKey: true }), null);
  assert.strictEqual(key({ code: "Digit1" }), null);
  // ABNT2: Shift+2 writes `"`, and what identifies the key is the `code`
  assert.strictEqual(key({ code: "Digit2", key: '"', shiftKey: true }), "patchbay.html");
  assert.strictEqual(key({ key: "4", shiftKey: true }), "face.html?face=quatro", "no code");
  assert.strictEqual(key({ code: "Digit1", shiftKey: true, ctrlKey: true }), null);
  assert.strictEqual(
    NAV.target({ code: "Digit2", shiftKey: true, target: { tagName: "INPUT" } }),
    null,
    "the show name has digits: inside a field no key navigates"
  );
});

test("mounted: the bar navigates on the shortcut and carries the open show name", async () => {
  const { doc, el, bus, field } = mount("/spellgui/web/index.html");
  assert.strictEqual(el.children.filter(f => f.tagName === "A" && f.className === "atual").length, 1);
  assert.strictEqual(el.children.find(f => f.className === "atual").textContent, "TIMELINE");

  bus.emit("open");
  await new Promise(r => setTimeout(r, 0));
  assert.deepStrictEqual(bus.calls[0], { cmd: "show_get", args: {} });
  assert.strictEqual(field.value, "MED GRUPO");
  assert.strictEqual(el.children.find(f => f.className.indexOf("sc-nav-estado") === 0).textContent,
                     "ENGINE");

  doc.fire("keydown", { code: "Digit2", shiftKey: true, target: element("body") });
  assert.strictEqual(doc.location, "patchbay.html");
});

test("show name: only Enter and blur send show_patch", async () => {
  const { bus, field } = mount("/spellgui/web/index.html");
  bus.emit("open");
  await new Promise(r => setTimeout(r, 0));

  // typing sends nothing: one show_patch per character would rename the show seven times
  field.value = "MED GRUPO 2";
  field.fire("keydown", { key: "M" });
  field.fire("keydown", { key: "2" });
  assert.deepStrictEqual(patches(bus), []);

  field.fire("keydown", { key: "Enter" });
  assert.deepStrictEqual(patches(bus), [
    { cmd: "show_patch", args: { ops: [{ op: "replace", path: "/name", value: "MED GRUPO 2" }] } },
  ]);

  // leaving the field with no further change does not repeat the patch
  field.fire("blur", {});
  assert.strictEqual(patches(bus).length, 1);

  field.value = "OTHER";
  field.fire("blur", {});
  assert.strictEqual(patches(bus).length, 2);
  assert.strictEqual(patches(bus)[1].args.ops[0].value, "OTHER");

  // an empty field does not erase the show name: it goes back to what the engine has
  field.value = "   ";
  field.fire("blur", {});
  assert.strictEqual(patches(bus).length, 2);
  assert.strictEqual(field.value, "OTHER");
});

test("rename: the rule on its own, with no DOM", () => {
  const bus = fakeBus("A");
  assert.strictEqual(NAV.rename(bus, "A", "A"), null, "equal to the engine one does not go");
  assert.strictEqual(NAV.rename(bus, "A", ""), null);
  assert.strictEqual(NAV.rename(bus, "A", null), null);
  assert.deepStrictEqual(patches(bus), []);
  assert.ok(NAV.rename(bus, "A", "  B  ") instanceof Promise);
  assert.strictEqual(patches(bus)[0].args.ops[0].value, "B", "writes without the spaces");
});
