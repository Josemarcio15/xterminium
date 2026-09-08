# ============================================================
# xterminium — Instalador / Atualizador (Windows)
# Uso:  irm https://raw.githubusercontent.com/Josemarcio15/xterminium/main/install.ps1 | iex
# ============================================================

$ErrorActionPreference = 'Stop'

$Repo = "Josemarcio15/xterminium"
$Api = "https://api.github.com/repos/$Repo/releases"

function Write-Step {
    param([string]$Message)
    Write-Host "[xterminium] " -ForegroundColor Cyan -NoNewline
    Write-Host $Message
}

function Write-Success {
    param([string]$Message)
    Write-Host "[xterminium] " -ForegroundColor Green -NoNewline
    Write-Host $Message -ForegroundColor Green
}

function Write-Banner {
    Write-Host @"

  ██╗  ██╗████████╗███████╗██████╗ ███╗   ███╗██╗███╗   ██╗██╗██╗   ██╗███╗   ███╗
  ╚██╗██╔╝╚══██╔══╝██╔════╝██╔══██╗████╗ ████║██║████╗  ██║██║██║   ██║████╗ ████║
   ╚███╔╝    ██║   █████╗  ██████╔╝██╔████╔██║██║██╔██╗ ██║██║██║   ██║██╔████╔██║
   ██╔██╗    ██║   ██╔══╝  ██╔══██╗██║╚██╔╝██║██║██║╚██╗██║██║██║   ██║██║╚██╔╝██║
  ██╔╝ ██╗   ██║   ███████╗██║  ██║██║ ╚═╝ ██║██║██║ ╚████║██║╚██████╔╝██║ ╚═╝ ██║
  ╚═╝  ╚═╝   ╚═╝   ╚══════╝╚═╝  ╚═╝╚═╝     ╚═╝╚═╝╚═╝  ╚═══╝╚═╝ ╚═════╝ ╚═╝     ╚═╝

"@ -ForegroundColor Cyan
    Write-Host "Instalador / Atualizador Automatico - Windows`n" -ForegroundColor White
}

Write-Banner

# 1. Consulta o release mais recente
Write-Step "Consultando versao mais recente em github.com/$Repo..."
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

try {
    $releases = Invoke-RestMethod -Uri $Api -Headers @{ "User-Agent" = "xterminium-installer" }
} catch {
    Write-Host "Erro ao consultar API do GitHub: $_" -ForegroundColor Red
    exit 1
}

if (-not $releases -or $releases.Count -eq 0) {
    Write-Host "Nenhum release encontrado no repositorio." -ForegroundColor Red
    exit 1
}

$latestRelease = $releases[0]
$tag = $latestRelease.tag_name
Write-Step "Versao mais recente encontrada: $tag"

# 2. Localiza o asset de instalacao (.exe ou .msi)
$asset = $latestRelease.assets | Where-Object { 
    $_.name -match '\.(exe|msi)$' -and $_.name -notmatch 'sig$' 
} | Select-Object -First 1

if (-not $asset) {
    Write-Host "Nenhum instalador (.exe ou .msi) disponivel na versao $tag ainda." -ForegroundColor Yellow
    exit 1
}

$downloadUrl = $asset.browser_download_url
$fileName = $asset.name
$tempPath = Join-Path $env:TEMP $fileName

# 3. Download do instalador
Write-Step "Baixando $fileName..."
Invoke-WebRequest -Uri $downloadUrl -OutFile $tempPath -UseBasicParsing

Write-Step "Download concluido com sucesso."

# 4. Executa o instalador
Write-Step "Iniciando instalacao..."
if ($fileName -match '\.msi$') {
    Start-Process msiexec.exe -ArgumentList "/i `"$tempPath`" /qb" -Wait
} else {
    # Instalador NSIS (.exe) - executa silenciosamente se suportado ou normal
    Start-Process -FilePath $tempPath -Wait
}

# Remove instalador temporario
if (Test-Path $tempPath) {
    Remove-Item $tempPath -Force -ErrorAction SilentlyContinue
}

Write-Success "xterminium $tag instalado/atualizado com sucesso! 🚀"
Write-Step "Procure por 'xterminium' no Menu Iniciar para abrir."
