# Servidor MCP: tools geradas do registry, resources do show/patch/rede/log, prompts em portugues.
# Importar o pacote registra os comandos `mcp`, `mcp_install`, `show_summary` e `monitor`.
from . import tools, server, install  # noqa: F401
from .server import serve_stdio, handle, tool_list  # noqa: F401
