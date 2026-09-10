# Command registry: every registered function becomes a CLI verb, an OSC-API call, a GUI action and an MCP tool.
import inspect

REGISTRY = {}
_TYPES = {int: "int", float: "float", str: "str", bool: "bool"}


def command(name=None, mcp=True):
    """@command / @command() / @command("name", mcp=False)."""
    if callable(name):
        return command()(name)

    def deco(fn):
        REGISTRY[name or fn.__name__] = {"fn": fn, "mcp": mcp}
        return fn
    return deco


def _coerce(typ, v):
    if typ is bool and isinstance(v, str):
        # "sim" stays: accepted input value, not display text
        return v.strip().lower() in ("1", "true", "yes", "on", "sim")
    if typ in _TYPES and not isinstance(v, typ):
        return typ(v)
    return v


def call(name, **kwargs):
    fn = REGISTRY[name]["fn"]
    params = inspect.signature(fn).parameters
    args = {k: _coerce(params[k].annotation, v) if k in params else v for k, v in kwargs.items()}
    return fn(**args)


def schema():
    out = []
    for name, c in REGISTRY.items():
        params = []
        for p in inspect.signature(c["fn"]).parameters.values():
            params.append({"name": p.name,
                           "type": _TYPES.get(p.annotation, "str"),
                           "default": None if p.default is inspect.Parameter.empty else p.default})
        out.append({"name": name, "doc": inspect.getdoc(c["fn"]) or "", "params": params, "mcp": c["mcp"]})
    return out
