# Janela nativa via pywebview (WebView2). Sem pywebview, abre a GUI no navegador padrao.
from .server import GuiServer, serve


def window(port=8000):
    try:
        import webview
    except ImportError:
        print("pywebview ausente; abrindo no navegador padrao", flush=True)
        return serve(port=port, browser=True)
    srv = GuiServer(port=port).start()
    webview.create_window("Spellcaster", f"http://127.0.0.1:{srv.port}")
    webview.start()
    srv.stop()


if __name__ == "__main__":
    window()
