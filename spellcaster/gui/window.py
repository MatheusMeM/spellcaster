# Native window through pywebview (WebView2). Without pywebview, it opens the GUI in the default browser.
from .server import GuiServer, serve


def window(port=8000):
    try:
        import webview
    except ImportError:
        print("pywebview missing; opening in the default browser", flush=True)
        return serve(port=port, browser=True)
    srv = GuiServer(port=port).start()
    webview.create_window("Spellcaster", f"http://127.0.0.1:{srv.port}")
    webview.start()
    srv.stop()


if __name__ == "__main__":
    window()
