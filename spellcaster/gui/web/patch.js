// Patch: grade de fixtures (nome, perfil, universo, endereco, canais). Quem valida e o registry:
// cada edicao grava o show (show_set) e pede patch_check; a sobreposicao vem de la e aparece em vermelho.
"use strict";

(function () {
  let rows = [], profs = [], tb, err, cnt;

  function refresh() {
    App.rpc("show_get").then(sh => {
      rows = sh.patch || [];
      render();
      check();
    }).catch(e => err.textContent = e.message);
  }

  function commit() {
    // pega o show inteiro de novo para nao passar por cima do que a timeline editou
    return App.rpc("show_get").then(sh => {
      sh.patch = rows;
      return App.rpc("show_set", { data: JSON.stringify(sh) });
    }).then(check);
  }

  function check() {
    return App.rpc("patch_check").then(r => {
      err.textContent = r.error || "";
      cnt.textContent = r.rows.length + " fixtures ok";
      const by = {};
      for (const x of r.rows) by[x.name] = x.channels;
      for (const tr of tb.rows) {
        const n = tr.dataset.name;
        tr.cells[4].textContent = by[n] === undefined ? "-" : by[n];
        tr.classList.toggle("bad", by[n] === undefined);
      }
    }).catch(e => err.textContent = e.message);
  }

  function cell(tr, value, on, tag) {
    const td = tr.insertCell();
    if (tag === "select") {
      const s = document.createElement("select");
      for (const p of profs) s.add(new Option(p, p));
      s.value = value;
      s.onchange = () => on(s.value);
      td.appendChild(s);
    } else if (tag === "text") {
      td.textContent = value;
    } else {
      const i = document.createElement("input");
      i.value = value;
      i.type = tag || "text";
      i.onchange = () => on(i.value);
      td.appendChild(i);
    }
    return td;
  }

  function render() {
    tb.textContent = "";
    rows.forEach((f, i) => {
      const tr = tb.insertRow();
      tr.dataset.name = f.name;
      cell(tr, f.name || "", v => { f.name = v; tr.dataset.name = v; commit(); });
      cell(tr, f.profile || profs[0] || "", v => { f.profile = v; commit(); }, "select");
      cell(tr, f.universe === undefined ? 1 : f.universe, v => { f.universe = +v || 1; commit(); }, "number");
      cell(tr, f.address === undefined ? 1 : f.address, v => { f.address = +v || 1; commit(); }, "number");
      cell(tr, "-", null, "text");
      const td = tr.insertCell();
      const b = document.createElement("button");
      b.textContent = "x";
      b.onclick = () => { rows.splice(i, 1); commit().then(refresh); };
      td.appendChild(b);
    });
  }

  // O registro espera o DOMContentLoaded: App.route() usa elementos que app.js so pega la.
addEventListener("DOMContentLoaded", () => App.panel("patch", {
    mount(el) {
      el.innerHTML =
        '<div class="tl-bar"><button id="pt-add">+ Fixture</button><button id="pt-reload">Recarregar</button>' +
        '<span class="tl-msg" id="pt-cnt"></span><span class="pt-err" id="pt-err"></span></div>' +
        '<table class="grid"><thead><tr><th>nome<th>perfil<th>universo<th>endereco<th>canais<th></tr></thead>' +
        '<tbody id="pt-tb"></tbody></table>';
      tb = el.querySelector("#pt-tb");
      err = el.querySelector("#pt-err");
      cnt = el.querySelector("#pt-cnt");
      el.querySelector("#pt-reload").onclick = refresh;
      el.querySelector("#pt-add").onclick = () => {
        rows.push({ name: "fix" + (rows.length + 1), profile: profs[0] || "dimmer_1", universe: 1,
                    address: rows.length ? 0 : 1 });
        const last = rows[rows.length - 1];
        if (!last.address) last.address = 1;
        render();
        commit();
      };
      App.rpc("profiles").then(p => { profs = p; refresh(); }).catch(() => refresh());
      addEventListener("hashchange", () => { if (location.hash === "#patch") refresh(); });
    },
  }));
})();
