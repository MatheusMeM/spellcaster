// ponytail: outside Windows the window does not exist and the path helpers go unused ; drop
// this when the window gets Linux/mac
#![cfg_attr(not(windows), allow(dead_code))]
// no console window behind the program; errors go to `spellcaster.log` beside the exe
#![cfg_attr(windows, windows_subsystem = "windows")]
//! `spellcaster.exe` - the Spellcaster as a program, not as a browser tab.
//!
//!   spellcaster [show.spell] [--dir ROOT]
//!
//! Starts the bus (`serve::serve`, the SAME one as `spellcore serve`) in a thread on
//! `127.0.0.1:0`, with the full CLI registry, and opens ONE window on `index.html`. Closing the
//! window kills the process, and with it the bus.
//!
//! ponytail: `tao` + `wry` (the window and the WebView2 that Windows 11 already has), not the
//! whole Tauri ; go Tauri when a native menu, an updater, a tray or a signed icon is missed -
//! none of that exists here.

use include_dir::{include_dir, Dir};
use std::path::{Path, PathBuf};

// The portable exe carries the pages and the seed data. Outside a checkout it unpacks them
// beside itself (a USB stick), else in %LOCALAPPDATA%\Spellcaster. Pages and design are
// overwritten on every start (a new exe brings new pages); shows, profiles, faces and modules are
// the user's and are only seeded when missing.
// ponytail: whole `design/` (1 MB) because the pages read tokens/ and SHORTCUTS.md from it ;
// trim to those two when the exe gets fat.
static WEB: Dir = include_dir!("$CARGO_MANIFEST_DIR/../../spellgui/web");
static DESIGN: Dir = include_dir!("$CARGO_MANIFEST_DIR/../../design");
static SHOWS: Dir = include_dir!("$CARGO_MANIFEST_DIR/../../shows");
static PROFILES: Dir = include_dir!("$CARGO_MANIFEST_DIR/../../profiles");
static FACES: Dir = include_dir!("$CARGO_MANIFEST_DIR/../../faces");
static MODULES: Dir = include_dir!("$CARGO_MANIFEST_DIR/../../modules");

/// Unpacks the embedded tree under `base`. Returns the error of the first write that fails.
fn desempacotar(base: &Path) -> std::io::Result<()> {
    for (d, rel, sempre) in [
        (&WEB, "spellgui/web", true),
        (&DESIGN, "design", true),
        (&SHOWS, "shows", false),
        (&PROFILES, "profiles", false),
        (&FACES, "faces", false),
        (&MODULES, "modules", false),
    ] {
        let alvo = base.join(rel);
        if sempre || !alvo.is_dir() {
            std::fs::create_dir_all(&alvo)?;
            d.extract(&alvo)?;
        }
    }
    Ok(())
}

/// The root when no checkout is found: beside the exe, else %LOCALAPPDATA%\Spellcaster.
fn raiz_portatil(exe_dir: Option<&Path>) -> Result<PathBuf, String> {
    let mut tentativas = Vec::new();
    if let Some(d) = exe_dir {
        tentativas.push(d.to_path_buf());
    }
    if let Some(l) = std::env::var_os("LOCALAPPDATA") {
        tentativas.push(PathBuf::from(l).join("Spellcaster"));
    }
    let mut erro = String::from("no directory to unpack into");
    for t in tentativas {
        match desempacotar(&t) {
            Ok(()) => return Ok(t),
            Err(e) => erro = format!("{}: {}", t.display(), e),
        }
    }
    Err(erro)
}

/// The page the window opens. It is the 3D laser view (`design/laser/` became a product in the
/// `ui-3d` front), by the owner's request: the program is the model of the device, not a tab
/// with a timeline. Until the `ui-3d` front lands, this path is a 404 and the window opens
/// blank - switch it to `/spellgui/web/index.html` to get the timeline back.
const PAGINA: &str = "/spellgui/web/laser3d/app.html";

/// `spellcaster [show.spell] [--dir ROOT]`. Returns `(show, dir)`.
// ponytail: two arguments, hand-written parser ; clap comes in when there is a third one.
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

/// The first directory from `d` upwards that contains `spellgui/web`.
fn acima(d: &Path) -> Option<PathBuf> {
    d.ancestors()
        .find(|p| p.join("spellgui").join("web").is_dir())
        .map(Path::to_path_buf)
}

/// Static root: the argument, else the exe tree, else the current directory tree, else the
/// current directory itself (which will 404, and the window shows what is missing).
fn raiz(dir: Option<String>, exe: Option<&Path>, cwd: &Path) -> PathBuf {
    if let Some(d) = dir {
        return PathBuf::from(d);
    }
    exe.and_then(acima)
        .or_else(|| acima(cwd))
        .unwrap_or_else(|| cwd.to_path_buf())
}

/// The command line `.spell` seen from the root: the argument is relative to the directory the
/// user called from, and the process then runs at the repo root (that is where the engine
/// resolves `shows/`, `profiles/` and `faces/`).
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
    let exe_dir = exe.as_deref().and_then(Path::parent);
    let mut raiz = raiz(dir, exe_dir, &cwd);
    if !raiz.join("spellgui").join("web").is_dir() {
        raiz = raiz_portatil(exe_dir)?;
    }
    let show = show_abs(show, &cwd);
    std::env::set_current_dir(&raiz).map_err(|e| format!("{}: {}", raiz.display(), e))?;

    // The MIDI map calls the FULL registry, the same one `spellcore` builds (cli/src/lib.rs).
    engine::midi::builder(cli::registry);

    let (tx, rx) = mpsc::channel();
    let r = raiz.clone();
    std::thread::spawn(move || {
        if let Err(e) = serve::serve(cli::registry(), 0, r, show, move |a| {
            let _ = tx.send(a);
        }) {
            eprintln!("bus: {}", e);
        }
    });
    let addr = rx
        .recv_timeout(Duration::from_secs(10))
        .map_err(|_| "the bus did not come up in 10 s".to_string())?;
    let url = format!("http://127.0.0.1:{}{}", addr.port(), PAGINA);

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
            // closing the window ends the process, and with it the bus and the outputs
            *fluxo = ControlFlow::Exit;
        }
    });
}

#[cfg(windows)]
fn main() {
    if let Err(e) = janela() {
        eprintln!("spellcaster: {}", e);
        // windows_subsystem = "windows": there is no console, the log is the only trace
        let log = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("spellcaster.log")))
            .unwrap_or_else(|| PathBuf::from("spellcaster.log"));
        let _ = std::fs::write(log, format!("spellcaster: {}
", e));
        std::process::exit(1);
    }
}

// ponytail: the window is Windows only (WebView2 already ships with 11) ; the Pi runs
// `spellcore serve` and opens the GUI in the browser, which is what the Lite promises.
// `tao`/`wry` also serve macOS and Linux - switch the cfg (and the dependencies, today only
// under `cfg(windows)`) when someone asks.
#[cfg(not(windows))]
fn main() {
    eprintln!("spellcaster: the window is Windows only; here use `spellcore serve` and open it in a browser.");
    std::process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn show_and_dir_arguments() {
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
            "the second positional is ignored, it does not become --dir"
        );
    }

    /// The rule that makes the exe find the pages: argument, exe tree, cwd tree, cwd.
    #[test]
    fn root_finds_spellgui_web() {
        let base = std::env::temp_dir().join("spellcaster_gui_raiz");
        let fundo = base.join("spellcore").join("gui");
        std::fs::create_dir_all(base.join("spellgui").join("web")).unwrap();
        std::fs::create_dir_all(&fundo).unwrap();
        let outro = std::env::temp_dir().join("spellcaster_gui_outro");
        std::fs::create_dir_all(&outro).unwrap();

        assert_eq!(
            raiz(None, Some(&fundo), &outro),
            base,
            "climbs from the exe"
        );
        assert_eq!(raiz(None, None, &fundo), base, "climbs from the cwd");
        assert_eq!(raiz(None, None, &outro), outro, "no root, stays in the cwd");
        assert_eq!(
            raiz(Some("D:/repo".into()), Some(&fundo), &outro),
            PathBuf::from("D:/repo"),
            "the argument wins"
        );
        std::fs::remove_dir_all(&base).ok();
        std::fs::remove_dir_all(&outro).ok();
    }

    /// The portable exe: an empty directory receives the pages and the seed, and a second
    /// unpack does not touch the user's shows.
    #[test]
    fn unpacks_pages_and_seeds_data_once() {
        let d = std::env::temp_dir().join("spellcaster_gui_portatil");
        std::fs::remove_dir_all(&d).ok();
        std::fs::create_dir_all(&d).unwrap();
        let r = raiz_portatil(Some(&d)).unwrap();
        assert_eq!(r, d);
        assert!(d.join("spellgui/web/laser3d/app.html").is_file());
        assert!(d.join("design/SHORTCUTS.md").is_file());
        assert!(d.join("profiles").is_dir());
        let meu = d.join("shows").join("meu.spell");
        std::fs::write(&meu, "{}").unwrap();
        raiz_portatil(Some(&d)).unwrap();
        assert!(meu.is_file(), "a second unpack keeps the user's show");
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn show_becomes_absolute_when_it_exists_in_the_cwd() {
        let d = std::env::temp_dir().join("spellcaster_gui_show");
        std::fs::create_dir_all(&d).unwrap();
        let f = d.join("x.spell");
        std::fs::write(&f, "{}").unwrap();
        assert_eq!(
            show_abs(Some("x.spell".into()), &d),
            Some(f.to_string_lossy().into_owned())
        );
        // it does not exist in the cwd: it stays as it came, for the engine to resolve from the
        // root
        assert_eq!(
            show_abs(Some("shows/medgrupo.spell".into()), &d),
            Some("shows/medgrupo.spell".into())
        );
        assert_eq!(show_abs(None, &d), None);
        std::fs::remove_dir_all(&d).ok();
    }
}
