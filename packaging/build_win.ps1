# Windows build of Spellcaster: PyInstaller onedir -> USB-stick folder with Spellcaster.exe and spell.exe.
# pyinstaller is a DEVELOPMENT dependency: it goes neither into the runtime pyproject nor onto the Pi.
# Usage:  pwsh -File packaging\build_win.ps1 [-Out <folder>]
# The output must NOT sit in the repo (a folder synced by Google Drive holds a lock on binaries).
param([string]$Out = (Join-Path $env:TEMP "spellcaster_build"))

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $false
$py = if (Test-Path "C:\Python313\python.exe") { "C:\Python313\python.exe" } else { "python" }
$root = Split-Path -Parent $PSScriptRoot
$dist = Join-Path $Out "dist"
$app = Join-Path $dist "Spellcaster"

& $py -m PyInstaller --version *> $null
if ($LASTEXITCODE -ne 0) { & $py -m pip install pyinstaller; if ($LASTEXITCODE -ne 0) { throw "pip install pyinstaller failed" } }

& $py -m PyInstaller --noconfirm --clean --log-level WARN `
    --distpath $dist --workpath (Join-Path $Out "build") `
    (Join-Path $PSScriptRoot "spellcaster.spec")
if ($LASTEXITCODE -ne 0) { throw "PyInstaller failed" }

# shows/ and profiles/ next to the exe as well: that is what the operator edits (paths.py prefers these).
Copy-Item (Join-Path $root "profiles") $app -Recurse -Force
New-Item -ItemType Directory -Force (Join-Path $app "shows") | Out-Null
Get-ChildItem (Join-Path $root "shows\*") -File -Include *.spell, *.py, *.ild |
    Copy-Item -Destination (Join-Path $app "shows") -Force

@"
SPELLCASTER - portable show control (Feiticaria Industrial)
===========================================================

FIRST BOOT
1. Copy this whole folder to the USB stick. Do not separate Spellcaster.exe from _internal.
2. Run spell.exe net  (a double click will not do: open the Command Prompt in this folder).
   It lists interfaces, Art-Net nodes, sACN sources and says whether the IP works for Art-Net (2.x/10.x).
3. Double-click Spellcaster.exe: the GUI opens. Without WebView2 it opens in the default browser,
   at http://127.0.0.1:8000 . The port is in config.json, next to the exe.
4. Example show: spell.exe play_show medgrupo.spell   (it looks in shows\ automatically).

WHAT THE PROGRAM WRITES
Nothing outside this folder. No registry, no %APPDATA%, no installer.
config.json, shows\ and profiles\ stay here and can be edited by hand.

ANTIVIRUS
The executable is NOT signed (a code certificate is in the backlog). Windows Defender and
SmartScreen may block it on the first run:
  - SmartScreen: "More info" -> "Run anyway".
  - Defender: Windows Security -> Virus and threat protection -> Manage settings ->
    Exclusions -> Add an exclusion -> Folder -> pick this folder.
  - On a client PC, ask IT for the exclusion BEFORE the event: some corporate antivirus
    products delete the exe silently in the middle of the show.

COMMANDS
  spell.exe --help          lists the verbs (the same ones as the GUI, OSC and MCP)
  spell.exe net             network scan
  spell.exe tui             text monitor (transport, universes, network)
  spell.exe play_show X     plays a .spell
  spell.exe serve --browser starts the GUI only
"@ | Set-Content (Join-Path $app "LEIA-ME.txt") -Encoding utf8NoBOM

$mb = [math]::Round(((Get-ChildItem $app -Recurse -File | Measure-Object Length -Sum).Sum / 1MB), 1)
"dist: $app  ($mb MB)"
