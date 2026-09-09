// Rede: chama net_report (o mesmo scan do `spell net`, mas devolvendo dict) e mostra o relatorio
// em texto e as sugestoes destacadas. O scan e sincrono no servidor: o botao fica travado enquanto roda.
"use strict";

(function () {
  let pre, sug, btn, tmo;

  function scan() {
    btn.disabled = true;
    btn.textContent = "escaneando...";
    App.rpc("net_report", { timeout: tmo.value }).then(d => {
      pre.textContent = d.report;
      sug.textContent = "";
      for (const s of d.suggestions || []) {
        const li = document.createElement("li");
        li.textContent = s;
        sug.appendChild(li);
      }
      App.log("net: " + (d.artnet.length || 0) + " nos Art-Net, " + (d.sacn.length || 0) + " fontes sACN, " +
              (d.etherdream.length || 0) + " DACs");
    }).catch(e => pre.textContent = "erro: " + e.message)
      .then(() => { btn.disabled = false; btn.textContent = "Rescan"; });
  }

  // O registro espera o DOMContentLoaded: App.route() usa elementos que app.js so pega la.
addEventListener("DOMContentLoaded", () => App.panel("network", {
    mount(el) {
      el.innerHTML =
        '<div class="tl-bar"><button id="nt-scan">Rescan</button>' +
        '<label>timeout <input id="nt-tmo" type="number" value="2" min="1" max="10" style="width:4em"></label></div>' +
        '<h4>Sugestoes</h4><ul class="nt-sug" id="nt-sug"></ul>' +
        '<h4>Relatorio</h4><pre class="nt-pre" id="nt-pre">-</pre>';
      pre = el.querySelector("#nt-pre");
      sug = el.querySelector("#nt-sug");
      btn = el.querySelector("#nt-scan");
      tmo = el.querySelector("#nt-tmo");
      btn.onclick = scan;
      scan();
    },
  }));
})();
