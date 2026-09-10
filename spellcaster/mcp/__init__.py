# MCP server: tools generated from the registry, resources for show/patch/network/log, prompts in English.
# Importing the package registers the `mcp`, `mcp_install`, `show_summary` and `monitor` commands.
from . import tools, server, install  # noqa: F401
from .server import serve_stdio, handle, tool_list  # noqa: F401
