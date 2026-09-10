"use strict";
// widgets.js — a typed parameter generates the widget (design/FUNCOES/README.md, rule 1); nobody
// draws a widget per command. The source is the JSON schema that `schemars` generates for the
// `Args` of each `Registry::add`, served at `GET /commands`.
//
// Rule 3 (trigger, toggle and value are distinct types) falls out of here: a command with no
// property = button (trigger); boolean = toggle; number with min and max = slider + field; loose
// number and integer = numeric field; string with enum = select; string = field; object/array =
// JSON textarea.
//
// ponytail: object/array as a JSON textarea ; make it a sub-form when some registry command has
// a nested object the operator needs to edit field by field.

const WG = {};

/// Effective JSON type: schemars writes `Option<T>` as ["T","null"].
function jtype(s) {
  let t = s && s.type;
  if (Array.isArray(t)) t = t.filter(x => x !== "null")[0];
  if (!t && s && s.enum) return "string";
  return t || "";
}

/// schema of a property -> widget type.
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

/// An entry of `GET /commands` -> control type of the whole command.
/// With no property at all it is a trigger (one button, nothing to fill in).
WG.kindOfCommand = function (c) {
  const p = (c && c.params && c.params.properties) || {};
  return Object.keys(p).length ? "form" : "button";
};

/// Raw field text -> typed value. Throws on invalid JSON (the form shows the error).
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
      // Text that is JSON becomes JSON; the rest goes as text. Same rule as the engine `key_set`,
      // which is the one that validates: the page does not invent a second validation (CLAUDE.md
      // rule).
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

/// Command + raw field values -> `args` of the request. An empty optional field is not sent
/// (Rust `#[serde(default)]` decides), an empty required field goes as the type default.
WG.args = function (c, vals) {
  const props = (c && c.params && c.params.properties) || {};
  const req = (c && c.params && c.params.required) || [];
  const out = {};
  for (const name of Object.keys(props)) {
    const raw = vals ? vals[name] : undefined;
    const kind = WG.kindOf(props[name]);
    const empty = raw === undefined || raw === null || raw === "";
    if (empty && req.indexOf(name) < 0) continue;
    out[name] = WG.coerce(kind, empty ? "" : raw);
  }
  return out;
};

// ---- DOM ----------------------------------------------------------------
// From here down nothing has a rule: it builds the element of the type `kindOf` decided.

function el(tag, cls, txt) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (txt !== undefined) e.textContent = txt;
  return e;
}

/// widget(prop, schema, onChange) -> <label> with the control inside.
/// The returned element carries `get()` and `set(v)`; `onChange(typedValue)` on every change.
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

/// form(command, bus) -> <form> with one widget per property and the button that runs it.
/// A command with no property becomes just the button (trigger). No logic: the args come from
/// WG.args.
WG.form = function (c, bus, onResult) {
  const f = el("form", "wg-form");
  f.appendChild(el("div", "wg-doc", c.doc || ""));
  const props = (c.params && c.params.properties) || {};
  const fields = {};
  const out = el("div", "wg-out", "");
  for (const name of Object.keys(props)) {
    // touching a field clears the result of the previous run, which no longer matches what is on
    // screen — it is the only consumer of `onChange`.
    const w = WG.widget(name, props[name], () => (out.textContent = ""));
    fields[name] = w;
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
      for (const k of Object.keys(fields)) vals[k] = fields[k].get();
      args = WG.args(c, vals);
    } catch (err) {
      out.textContent = "invalid argument: " + err.message;
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
