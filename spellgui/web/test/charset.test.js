"use strict";
// node --test spellgui/web/test/charset.test.js
// Not every server sends a charset in the Content-Type (python -m http.server, the Tauri asset
// protocol, file://). With no <meta charset> Chrome falls back to windows-1252 and the whole page
// turns into mojibake ("2 REAR ." really did become "2 REAR A." in app.html). So: every page
// declares the charset in the first 1024 bytes, and no source has a BOM or an invalid UTF-8 byte.

const { test } = require("node:test");
const assert = require("node:assert");
const fs = require("fs");
const path = require("path");

const RAIZ = path.join(__dirname, "..");

function varre(dir, exts) {
  const achados = [];
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    if (e.name === "vendor") continue;
    const p = path.join(dir, e.name);
    if (e.isDirectory()) achados.push(...varre(p, exts));
    else if (exts.includes(path.extname(e.name))) achados.push(p);
  }
  return achados;
}

const rel = (p) => path.relative(RAIZ, p).replace(/\\/g, "/");

test("every page declares <meta charset=utf-8> in the first 1024 bytes", () => {
  const paginas = varre(RAIZ, [".html"]);
  assert.ok(paginas.length > 0, "no .html found");
  for (const p of paginas) {
    const cabeca = fs.readFileSync(p).subarray(0, 1024).toString("latin1");
    assert.match(
      cabeca,
      /<meta\s+charset\s*=\s*["']?utf-?8["']?\s*\/?>/i,
      rel(p) + ": no <meta charset=utf-8> in the first 1024 bytes"
    );
  }
});

test("no source has a BOM or an invalid UTF-8 byte", () => {
  const dec = new TextDecoder("utf-8", { fatal: true });
  for (const p of varre(RAIZ, [".html", ".js", ".css"])) {
    const b = fs.readFileSync(p);
    assert.ok(
      !(b[0] === 0xef && b[1] === 0xbb && b[2] === 0xbf),
      rel(p) + ": starts with a BOM"
    );
    assert.doesNotThrow(() => dec.decode(b), rel(p) + ": does not decode as UTF-8");
  }
});
