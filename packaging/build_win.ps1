# Build Windows do Spellcaster: PyInstaller onedir -> pasta de pendrive com Spellcaster.exe e spell.exe.
# pyinstaller e dependencia de DESENVOLVIMENTO: nao entra no pyproject de runtime nem no Pi.
# Uso:  pwsh -File packaging\build_win.ps1 [-Out <pasta>]
# A saida NAO pode ficar no repo (pasta sincronizada pelo Google Drive segura lock em binario).
param([string]$Out = (Join-Path $env:TEMP "spellcaster_build"))

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $false
$py = if (Test-Path "C:\Python313\python.exe") { "C:\Python313\python.exe" } else { "python" }
$root = Split-Path -Parent $PSScriptRoot
$dist = Join-Path $Out "dist"
$app = Join-Path $dist "Spellcaster"

& $py -m PyInstaller --version *> $null
if ($LASTEXITCODE -ne 0) { & $py -m pip install pyinstaller; if ($LASTEXITCODE -ne 0) { throw "pip install pyinstaller falhou" } }

& $py -m PyInstaller --noconfirm --clean --log-level WARN `
    --distpath $dist --workpath (Join-Path $Out "build") `
    (Join-Path $PSScriptRoot "spellcaster.spec")
if ($LASTEXITCODE -ne 0) { throw "PyInstaller falhou" }

# shows/ e profiles/ tambem ao lado do exe: e o que o operador edita (paths.py prefere estes).
Copy-Item (Join-Path $root "profiles") $app -Recurse -Force
New-Item -ItemType Directory -Force (Join-Path $app "shows") | Out-Null
Get-ChildItem (Join-Path $root "shows\*") -File -Include *.spell, *.py, *.ild |
    Copy-Item -Destination (Join-Path $app "shows") -Force

@"
SPELLCASTER - show control portatil (Feiticaria Industrial)
===========================================================

PRIMEIRO BOOT
1. Copie esta pasta inteira para o pendrive. Nao separe o Spellcaster.exe do _internal.
2. Rode spell.exe net  (duplo clique nao serve: abra o Prompt de Comando nesta pasta).
   Ele lista interfaces, nos Art-Net, fontes sACN e diz se o IP serve para Art-Net (2.x/10.x).
3. Duplo clique em Spellcaster.exe: abre a GUI. Sem WebView2, ela abre no navegador padrao,
   em http://127.0.0.1:8000 . A porta esta em config.json, ao lado do exe.
4. Show de exemplo: spell.exe play_show medgrupo.spell   (procura em shows\ automaticamente).

O QUE O PROGRAMA ESCREVE
Nada fora desta pasta. Sem registro, sem %APPDATA%, sem instalador.
config.json, shows\ e profiles\ ficam aqui e podem ser editados a mao.

ANTIVIRUS
O executavel NAO e assinado (certificado de codigo esta no backlog). Windows Defender e
SmartScreen podem bloquear na primeira execucao:
  - SmartScreen: "Mais informacoes" -> "Executar assim mesmo".
  - Defender: Seguranca do Windows -> Protecao contra virus e ameacas -> Gerenciar
    configuracoes -> Exclusoes -> Adicionar uma exclusao -> Pasta -> selecione esta pasta.
  - Em PC de cliente, peca a exclusao ao TI ANTES do evento: alguns antivirus corporativos
    apagam o exe em silencio no meio do show.

COMANDOS
  spell.exe --help          lista os verbos (os mesmos da GUI, do OSC e do MCP)
  spell.exe net             analise de rede
  spell.exe tui             monitor de texto (transporte, universos, rede)
  spell.exe play_show X     toca um .spell
  spell.exe serve --browser sobe so a GUI
"@ | Set-Content (Join-Path $app "LEIA-ME.txt") -Encoding utf8NoBOM

$mb = [math]::Round(((Get-ChildItem $app -Recurse -File | Measure-Object Length -Sum).Sum / 1MB), 1)
"dist: $app  ($mb MB)"
