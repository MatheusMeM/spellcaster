"use strict";
// node --test spellgui/web/test/nav.test.js
// A barra tem tres regras: qual aba e' a atual, que tecla leva a que pagina, e quando o nome do
// show vai ao engine (Enter ou blur, NUNCA por tecla). O desenho e' DOM e nao tem regra, entao o
// document aqui e' de mentira: so' o bastante para `NAV.monta` rodar sem navegador.

const { test } = require("node:test");
const assert = require("node:assert");
const NAV = require("../nav.js");

function elemento(tag) {
  return {
    tagName: String(tag).toUpperCase(),
    filhos: [],
    ouvintes: {},
    className: "",
    textContent: "",
    value: "",
    href: "",
    type: "",
    id: "",
    title: "",
    appendChild(c) {
      this.filhos.push(c);
      return c;
    },
    addEventListener(n, f) {
      (this.ouvintes[n] = this.ouvintes[n] || []).push(f);
    },
    dispara(n, e) {
      const base = { target: this, stopPropagation() {}, preventDefault() {} };
      for (const f of this.ouvintes[n] || []) f(Object.assign(base, e));
    },
  };
}

function documento(pathname) {
  const d = elemento("document");
  d.head = elemento("head");
  d.location = { pathname };
  d.createElement = elemento;
  return d;
}

// Bus de mentira com a mesma superficie que a barra usa: on/call/rev.
function barramento(nome) {
  return {
    rev: 3,
    chamadas: [],
    ev: {},
    on(t, f) {
      (this.ev[t] = this.ev[t] || []).push(f);
    },
    emit(t, d) {
      for (const f of this.ev[t] || []) f(d);
    },
    call(cmd, args) {
      this.chamadas.push({ cmd, args });
      return Promise.resolve(cmd === "show_get" ? { name: nome } : { rev: 4 });
    },
  };
}

const monta = (path, nome) => {
  const doc = documento(path);
  const el = elemento("div");
  const bus = barramento(nome || "MED GRUPO");
  NAV.monta(doc, el, bus);
  const campo = el.filhos.find(f => f.id === "sc-nav-nome");
  return { doc, el, bus, campo };
};
const patches = bus => bus.chamadas.filter(c => c.cmd === "show_patch");

test("abas: a atual e' a da pagina, e a raiz e' a timeline", () => {
  const marcada = p => NAV.abas(p).find(a => a.atual);
  assert.strictEqual(marcada("/spellgui/web/patchbay.html").rot, "PATCHBAY");
  assert.strictEqual(marcada("/spellgui/web/face.html").rot, "FACE");
  assert.strictEqual(marcada("/spellgui/web/index.html").rot, "TIMELINE");
  assert.strictEqual(marcada("/").rot, "TIMELINE");
  assert.strictEqual(marcada("/spellgui/web/pagina_que_nao_existe.html"), undefined);
  // uma so' marcada, e a FACE leva a view padrao
  assert.strictEqual(NAV.abas("/spellgui/web/teatro.html").filter(a => a.atual).length, 1);
  assert.strictEqual(NAV.abas("/").find(a => a.rot === "FACE").href, "face.html?face=quatro");
});

test("Shift+1..5 troca de pagina; sem Shift, com Ctrl ou num campo, nao", () => {
  const tecla = e => NAV.destino(Object.assign({ target: { tagName: "BODY" } }, e));
  assert.strictEqual(tecla({ code: "Digit1", shiftKey: true }), "index.html");
  assert.strictEqual(tecla({ code: "Digit3", shiftKey: true }), "teatro.html");
  assert.strictEqual(tecla({ code: "Digit5", shiftKey: true }), "laser.html");
  assert.strictEqual(tecla({ code: "Digit6", shiftKey: true }), null);
  assert.strictEqual(tecla({ code: "Digit1" }), null);
  // ABNT2: Shift+2 escreve `"`, e quem identifica a tecla e' o `code`
  assert.strictEqual(tecla({ code: "Digit2", key: '"', shiftKey: true }), "patchbay.html");
  assert.strictEqual(tecla({ key: "4", shiftKey: true }), "face.html?face=quatro", "sem code");
  assert.strictEqual(tecla({ code: "Digit1", shiftKey: true, ctrlKey: true }), null);
  assert.strictEqual(
    NAV.destino({ code: "Digit2", shiftKey: true, target: { tagName: "INPUT" } }),
    null,
    "o nome do show tem digitos: dentro de um campo nenhuma tecla navega"
  );
});

test("montada: a barra navega no atalho e leva o nome do show aberto", async () => {
  const { doc, el, bus, campo } = monta("/spellgui/web/index.html");
  assert.strictEqual(el.filhos.filter(f => f.tagName === "A" && f.className === "atual").length, 1);
  assert.strictEqual(el.filhos.find(f => f.className === "atual").textContent, "TIMELINE");

  bus.emit("open");
  await new Promise(r => setTimeout(r, 0));
  assert.deepStrictEqual(bus.chamadas[0], { cmd: "show_get", args: {} });
  assert.strictEqual(campo.value, "MED GRUPO");
  assert.strictEqual(el.filhos.find(f => f.className.indexOf("sc-nav-estado") === 0).textContent,
                     "ENGINE");

  doc.dispara("keydown", { code: "Digit2", shiftKey: true, target: elemento("body") });
  assert.strictEqual(doc.location, "patchbay.html");
});

test("nome do show: so' o Enter e o blur mandam show_patch", async () => {
  const { bus, campo } = monta("/spellgui/web/index.html");
  bus.emit("open");
  await new Promise(r => setTimeout(r, 0));

  // digitar nao manda nada: um show_patch por caractere renomearia o show sete vezes
  campo.value = "MED GRUPO 2";
  campo.dispara("keydown", { key: "M" });
  campo.dispara("keydown", { key: "2" });
  assert.deepStrictEqual(patches(bus), []);

  campo.dispara("keydown", { key: "Enter" });
  assert.deepStrictEqual(patches(bus), [
    { cmd: "show_patch", args: { ops: [{ op: "replace", path: "/name", value: "MED GRUPO 2" }] } },
  ]);

  // sair do campo sem mudar de novo nao repete o patch
  campo.dispara("blur", {});
  assert.strictEqual(patches(bus).length, 1);

  campo.value = "OUTRO";
  campo.dispara("blur", {});
  assert.strictEqual(patches(bus).length, 2);
  assert.strictEqual(patches(bus)[1].args.ops[0].value, "OUTRO");

  // campo vazio nao apaga o nome do show: volta ao que o engine tem
  campo.value = "   ";
  campo.dispara("blur", {});
  assert.strictEqual(patches(bus).length, 2);
  assert.strictEqual(campo.value, "OUTRO");
});

test("renomeia: a regra sozinha, sem DOM", () => {
  const bus = barramento("A");
  assert.strictEqual(NAV.renomeia(bus, "A", "A"), null, "igual ao do engine nao vai");
  assert.strictEqual(NAV.renomeia(bus, "A", ""), null);
  assert.strictEqual(NAV.renomeia(bus, "A", null), null);
  assert.deepStrictEqual(patches(bus), []);
  assert.ok(NAV.renomeia(bus, "A", "  B  ") instanceof Promise);
  assert.strictEqual(patches(bus)[0].args.ops[0].value, "B", "grava sem os espacos");
});
