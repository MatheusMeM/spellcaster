"use strict";
// node --test spellgui/web/test/charset.test.js
// Nem todo servidor manda charset no Content-Type (python -m http.server, protocolo asset do
// Tauri, file://). Sem <meta charset> o Chrome cai em windows-1252 e a pagina inteira vira
// mojibake ("2 TRAS ." virou "2 TRAS A." de verdade em app.html). Entao: toda pagina declara o
// charset nos primeiros 1024 bytes, e nenhum fonte tem BOM nem byte invalido em UTF-8.

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

test("toda pagina declara <meta charset=utf-8> nos primeiros 1024 bytes", () => {
  const paginas = varre(RAIZ, [".html"]);
  assert.ok(paginas.length > 0, "nenhum .html encontrado");
  for (const p of paginas) {
    const cabeca = fs.readFileSync(p).subarray(0, 1024).toString("latin1");
    assert.match(
      cabeca,
      /<meta\s+charset\s*=\s*["']?utf-?8["']?\s*\/?>/i,
      rel(p) + ": sem <meta charset=utf-8> nos primeiros 1024 bytes"
    );
  }
});

test("nenhum fonte tem BOM nem byte invalido em UTF-8", () => {
  const dec = new TextDecoder("utf-8", { fatal: true });
  for (const p of varre(RAIZ, [".html", ".js", ".css"])) {
    const b = fs.readFileSync(p);
    assert.ok(
      !(b[0] === 0xef && b[1] === 0xbb && b[2] === 0xbf),
      rel(p) + ": comeca com BOM"
    );
    assert.doesNotThrow(() => dec.decode(b), rel(p) + ": nao decodifica como UTF-8");
  }
});
