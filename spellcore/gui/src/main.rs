//! `spellcaster.exe` — o Spellcaster como programa, nao como aba do navegador.
//!
//!   spellcaster [show.spell] [--dir RAIZ]
//!
//! Sobe o barramento (`serve::serve`, o MESMO do `spellcore serve`) numa thread em
//! `127.0.0.1:0`, com o registry completo da CLI, e abre UMA janela no `index.html`. Fechar a
//! janela mata o processo, e com ele o barramento.
//!
//! ponytail: `tao` + `wry` (a janela e o WebView2 que o Windows 11 ja' tem), nao o Tauri
//! inteiro ; virar Tauri quando fizer falta menu nativo, updater, tray ou icone assinado — nada
//! disso existe aqui.

use std::path::{Path, PathBuf};

/// `spellcaster [show.spell] [--dir RAIZ]`. Devolve `(show, dir)`.
// ponytail: dois argumentos, parser a mao ; clap entra quando houver o terceiro.
fn args(it: impl Iterator<Item = String>) -> (Option<String>, Option<String>) {
    let (mut show, mut dir, mut espera_dir) = (None, None, false);
    for a in it {
        if espera_dir {
            dir = Some(a);
            espera_dir = false;
        } else if let Some(v) = a.strip_prefix("--dir=") {
            dir = Some(v.to_string());
        } else if a == "--dir" {
            espera_dir = true;
        } else if !a.starts_with('-') && show.is_none() {
            show = Some(a);
        }
    }
    (show, dir)
}

/// O primeiro diretorio de `d` para cima que contem `spellgui/web`.
fn acima(d: &Path) -> Option<PathBuf> {
    d.ancestors()
        .find(|p| p.join("spellgui").join("web").is_dir())
        .map(Path::to_path_buf)
}

/// Raiz do estatico: o argumento, senao a arvore do exe, senao a do diretorio corrente, senao o
/// proprio diretorio corrente (que dara' 404, e a janela mostra o que falta).
fn raiz(dir: Option<String>, exe: Option<&Path>, cwd: &Path) -> PathBuf {
    if let Some(d) = dir {
        return PathBuf::from(d);
    }
    exe.and_then(acima)
        .or_else(|| acima(cwd))
        .unwrap_or_else(|| cwd.to_path_buf())
}

/// O `.spell` da linha de comando visto da raiz: o argumento e' relativo ao diretorio de onde o
/// usuario chamou, e o processo passa a rodar na raiz do repo (e' de la' que o engine resolve
/// `shows/`, `profiles/` e `faces/`).
fn show_abs(show: Option<String>, cwd: &Path) -> Option<String> {
    let s = show?;
    let p = cwd.join(&s);
    if !Path::new(&s).is_absolute() && p.is_file() {
        return Some(p.to_string_lossy().into_owned());
    }
    Some(s)
}

#[cfg(windows)]
fn janela() -> Result<(), String> {
    use std::sync::mpsc;
    use std::time::Duration;
    use tao::dpi::LogicalSize;
    use tao::event::{Event, WindowEvent};
    use tao::event_loop::{ControlFlow, EventLoop};
    use tao::window::WindowBuilder;
    use wry::WebViewBuilder;

    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let (show, dir) = args(std::env::args().skip(1));
    let exe = std::env::current_exe().ok();
    let raiz = raiz(dir, exe.as_deref().and_then(Path::parent), &cwd);
    let show = show_abs(show, &cwd);
    std::env::set_current_dir(&raiz).map_err(|e| format!("{}: {}", raiz.display(), e))?;

    let (tx, rx) = mpsc::channel();
    let r = raiz.clone();
    std::thread::spawn(move || {
        if let Err(e) = serve::serve(cli::registry(), 0, r, show, move |a| {
            let _ = tx.send(a);
        }) {
            eprintln!("barramento: {}", e);
        }
    });
    let addr = rx
        .recv_timeout(Duration::from_secs(10))
        .map_err(|_| "o barramento nao ligou em 10 s".to_string())?;
    let url = format!("http://127.0.0.1:{}/spellgui/web/index.html", addr.port());

    let ev = EventLoop::new();
    let win = WindowBuilder::new()
        .with_title("Spellcaster")
        .with_inner_size(LogicalSize::new(1280.0, 800.0))
        .build(&ev)
        .map_err(|e| e.to_string())?;
    let _wv = WebViewBuilder::new()
        .with_url(&url)
        .build(&win)
        .map_err(|e| e.to_string())?;

    ev.run(move |e, _, fluxo| {
        *fluxo = ControlFlow::Wait;
        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = e
        {
            // fechar a janela encerra o processo, e com ele o barramento e as saidas
            *fluxo = ControlFlow::Exit;
        }
    });
}

#[cfg(windows)]
fn main() {
    if let Err(e) = janela() {
        eprintln!("spellcaster: {}", e);
        std::process::exit(1);
    }
}

// ponytail: a janela e' so' Windows (WebView2 ja' vem no 11) ; o Pi roda `spellcore serve` e abre
// a GUI pelo navegador, que e' o que a Lite promete. `tao`/`wry` tambem servem macOS e Linux —
// trocar o cfg (e as dependencias, hoje so' de `cfg(windows)`) quando alguem pedir.
#[cfg(not(windows))]
fn main() {
    eprintln!("spellcaster: a janela e' Windows; aqui use `spellcore serve` e abra no navegador.");
    std::process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn argumentos_show_e_dir() {
        let a = |v: &[&str]| args(v.iter().map(|s| s.to_string()));
        assert_eq!(a(&[]), (None, None));
        assert_eq!(
            a(&["shows/medgrupo.spell"]),
            (Some("shows/medgrupo.spell".into()), None)
        );
        assert_eq!(
            a(&["--dir", "D:/repo", "x.spell"]),
            (Some("x.spell".into()), Some("D:/repo".into()))
        );
        assert_eq!(
            a(&["--dir=D:/repo", "x.spell", "y.spell"]),
            (Some("x.spell".into()), Some("D:/repo".into())),
            "o segundo positional e' ignorado, nao vira --dir"
        );
    }

    /// A regra que faz o exe achar as paginas: argumento, arvore do exe, arvore do cwd, cwd.
    #[test]
    fn raiz_acha_spellgui_web() {
        let base = std::env::temp_dir().join("spellcaster_gui_raiz");
        let fundo = base.join("spellcore").join("gui");
        std::fs::create_dir_all(base.join("spellgui").join("web")).unwrap();
        std::fs::create_dir_all(&fundo).unwrap();
        let outro = std::env::temp_dir().join("spellcaster_gui_outro");
        std::fs::create_dir_all(&outro).unwrap();

        assert_eq!(raiz(None, Some(&fundo), &outro), base, "sobe do exe");
        assert_eq!(raiz(None, None, &fundo), base, "sobe do cwd");
        assert_eq!(raiz(None, None, &outro), outro, "sem raiz, fica no cwd");
        assert_eq!(
            raiz(Some("D:/repo".into()), Some(&fundo), &outro),
            PathBuf::from("D:/repo"),
            "o argumento manda"
        );
        std::fs::remove_dir_all(&base).ok();
        std::fs::remove_dir_all(&outro).ok();
    }

    #[test]
    fn show_vira_absoluto_quando_existe_no_cwd() {
        let d = std::env::temp_dir().join("spellcaster_gui_show");
        std::fs::create_dir_all(&d).unwrap();
        let f = d.join("x.spell");
        std::fs::write(&f, "{}").unwrap();
        assert_eq!(
            show_abs(Some("x.spell".into()), &d),
            Some(f.to_string_lossy().into_owned())
        );
        // nao existe no cwd: fica como veio, para o engine resolver a partir da raiz
        assert_eq!(
            show_abs(Some("shows/medgrupo.spell".into()), &d),
            Some("shows/medgrupo.spell".into())
        );
        assert_eq!(show_abs(None, &d), None);
        std::fs::remove_dir_all(&d).ok();
    }
}
