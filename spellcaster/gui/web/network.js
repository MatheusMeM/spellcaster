// Network: calls `net` (the same command as `spell net`, which returns the dict with `report`) and shows the
// report as text with the suggestions highlighted. The scan is synchronous on the server: the button stays
// disabled while it runs.
"use strict";

(function () {
  let pre, sug, btn, tmo;

  function scan() {
    btn.disabled = true;
    btn.textContent = "scanning...";
    App.rpc("net", { timeout: tmo.value }).then(d => {
      pre.textContent = d.report;
      sug.textContent = "";
      for (const s of d.suggestions || []) {
        const li = document.createElement("li");
        li.textContent = s;
        sug.appendChild(li);
      }
      App.log("net: " + (d.artnet.length || 0) + " Art-Net nodes, " + (d.sacn.length || 0) + " sACN sources, " +
              (d.etherdream.length || 0) + " DACs");
    }).catch(e => pre.textContent = "error: " + e.message)
      .then(() => { btn.disabled = false; btn.textContent = "Rescan"; });
  }

  // Registration waits for DOMContentLoaded: App.route() uses elements that app.js only grabs there.
addEventListener("DOMContentLoaded", () => App.panel("network", {
    mount(el) {
      el.innerHTML =
        '<div class="tl-bar"><button id="nt-scan">Rescan</button>' +
        '<label>timeout <input id="nt-tmo" type="number" value="2" min="1" max="10" style="width:4em"></label></div>' +
        '<h4>Suggestions</h4><ul class="nt-sug" id="nt-sug"></ul>' +
        '<h4>Report</h4><pre class="nt-pre" id="nt-pre">-</pre>';
      pre = el.querySelector("#nt-pre");
      sug = el.querySelector("#nt-sug");
      btn = el.querySelector("#nt-scan");
      tmo = el.querySelector("#nt-tmo");
      btn.onclick = scan;
      scan();
    },
  }));
})();
