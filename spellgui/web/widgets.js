"use strict";
// widgets.js — parametro tipado gera o widget (design/FUNCOES/README.md, regra 1); ninguem
// desenha widget por comando. A fonte e' o schema JSON que o `schemars` gera para os `Args` de
// cada `Registry::add`, servido em `GET /commands`.
//
// Regra 3 (trigger, toggle e valor sao tipos distintos) cai daqui: comando sem propriedade =
// botao (trigger); boolean = toggle; number com min e max = slider + campo; number solto e
// integer = campo numerico; string com enum = select; string = campo; object/array = textarea
// JSON.
//
// ponytail: object/array como textarea JSON ; virar sub-formulario quando algum comando do
// registry tiver um objeto aninhado que o operador precise editar campo a campo.

const WG = {};

/// Tipo JSON efetivo: schemars escreve `Option<T>` como ["T","null"].
function jtype(s) {
  let t = s && s.type;
  if (Array.isArray(t)) t = t.filter(x => x !== "null")[0];
  if (!t && s && s.enum) return "string";
  return t || "";
}

/// schema de uma propriedade -> tipo de widget.
WG.kindOf = function (s) {
  s = s || {};
  if (Array.isArray(s.enum) && s.enum.length) return "select";
  const t = jtype(s);
  if (t === "boolean") return "toggle";
  if (t === "integer") return "spin";
  if (t === "number") {
    return s.minimum !== undefined && s.maximum !== undefined ? "slider" : "num";
  }
  if (t === "string") return "text";
  return "json";
};

/// Entrada de `GET /commands` -> tipo do controle do comando inteiro.
/// Sem propriedade nenhuma = trigger (um botao, nada para preencher).
WG.kindOfCommand = function (c) {
  const p = (c && c.params && c.params.properties) || {};
  return Object.keys(p).length ? "form" : "button";
};

/// Texto cru do campo -> valor tipado. Lanca em JSON invalido (o formulario mostra o erro).
WG.coerce = function (kind, raw) {
  switch (kind) {
    case "toggle":
      return raw === true || raw === "true" || raw === "on" || raw === 1;
    case "spin":
      return Math.round(Number(raw) || 0);
    case "slider":
    case "num":
      return Number(raw) || 0;
    case "json":
      // Texto que e' JSON vira JSON; o resto vai como texto. Mesma regra do `key_set` do engine,
      // que e' quem valida: a pagina nao inventa uma segunda validacao (regra do CLAUDE.md).
      if (typeof raw !== "string") return raw;
      try {
        return JSON.parse(raw);
      } catch (e) {
        return raw;
      }
    default:
      return String(raw == null ? "" : raw);
  }
};

/// Comando + valores crus dos campos -> `args` do request. Campo vazio nao obrigatorio nao vai
/// (o `#[serde(default)]` do Rust decide), campo vazio obrigatorio vai como default do tipo.
WG.args = function (c, vals) {
  const props = (c && c.params && c.params.properties) || {};
  const req = (c && c.params && c.params.required) || [];
  const out = {};
  for (const name of Object.keys(props)) {
    const raw = vals ? vals[name] : undefined;
    const kind = WG.kindOf(props[name]);
    const vazio = raw === undefined || raw === null || raw === "";
    if (vazio && req.indexOf(name) < 0) continue;
    out[name] = WG.coerce(kind, vazio ? "" : raw);
  }
  return out;
};

// ---- DOM ----------------------------------------------------------------
// Daqui para baixo nada tem regra: monta o elemento do tipo que `kindOf` decidiu.

function el(tag, cls, txt) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (txt !== undefined) e.textContent = txt;
  return e;
}

/// widget(prop, schema, onChange) -> <label> com o controle dentro.
/// O elemento devolvido leva `get()` e `set(v)`; `onChange(valorTipado)` a cada mudanca.
WG.widget = function (prop, schema, onChange) {
  schema = schema || {};
  const kind = WG.kindOf(schema);
  const wrap = el("label", "wg wg-" + kind);
  const cap = el("span", "wg-lab", schema.title || prop);
  if (schema.description) wrap.title = schema.description;
  let inp;
  let num = null;
  if (kind === "select") {
    inp = el("select");
    for (const o of schema.enum) inp.appendChild(el("option", null, String(o)));
  } else if (kind === "toggle") {
    inp = el("input");
    inp.type = "checkbox";
  } else if (kind === "json") {
    inp = el("textarea");
    inp.rows = 3;
  } else {
    inp = el("input");
    inp.type = kind === "text" ? "text" : kind === "slider" ? "range" : "number";
    if (schema.minimum !== undefined) inp.min = schema.minimum;
    if (schema.maximum !== undefined) inp.max = schema.maximum;
    if (kind === "slider") inp.step = (schema.maximum - schema.minimum) / 1000;
    else if (kind === "num") inp.step = "any";
  }
  if (schema.default !== undefined && schema.default !== null) {
    if (kind === "toggle") inp.checked = !!schema.default;
    else inp.value = String(schema.default);
  }
  wrap.appendChild(cap);
  wrap.appendChild(inp);
  if (kind === "slider") {
    num = el("input", "wg-num");
    num.type = "number";
    num.step = "any";
    num.value = inp.value;
    wrap.appendChild(num);
    num.oninput = () => {
      inp.value = num.value;
      if (onChange) onChange(WG.coerce(kind, inp.value));
    };
  }
  wrap.get = () => WG.coerce(kind, kind === "toggle" ? inp.checked : inp.value);
  inp.oninput = () => {
    if (num) num.value = inp.value;
    if (onChange) onChange(wrap.get());
  };
  inp.onchange = inp.oninput;
  return wrap;
};

/// form(command, bus) -> <form> com um widget por propriedade e o botao que executa.
/// Comando sem propriedade vira so' o botao (trigger). Nada de logica: os args saem de WG.args.
WG.form = function (c, bus, onResult) {
  const f = el("form", "wg-form");
  f.appendChild(el("div", "wg-doc", c.doc || ""));
  const props = (c.params && c.params.properties) || {};
  const campos = {};
  const out = el("div", "wg-out", "");
  for (const name of Object.keys(props)) {
    // mexer num campo apaga o resultado da execucao anterior, que ja' nao vale para o que
    // esta' na tela — e' o unico consumidor de `onChange`.
    const w = WG.widget(name, props[name], () => (out.textContent = ""));
    campos[name] = w;
    f.appendChild(w);
  }
  const bt = el("button", "wg-go", c.name);
  bt.type = "submit";
  f.appendChild(bt);
  f.appendChild(out);
  f.onsubmit = e => {
    e.preventDefault();
    let args;
    try {
      const vals = {};
      for (const k of Object.keys(campos)) vals[k] = campos[k].get();
      args = WG.args(c, vals);
    } catch (err) {
      out.textContent = "argumento invalido: " + err.message;
      return;
    }
    bus.call(c.name, args).then(
      r => {
        out.textContent = JSON.stringify(r);
        if (onResult) onResult(r);
      },
      err => {
        out.textContent = err.message;
      }
    );
  };
  return f;
};

if (typeof window !== "undefined") window.WG = WG;
if (typeof module !== "undefined") module.exports = WG;
