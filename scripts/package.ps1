param(
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot

Push-Location $root
try {
    if (-not $SkipBuild) {
        cargo build --release
        if ($LASTEXITCODE -ne 0) {
            throw "cargo build falhou"
        }
    }

    $exe = Join-Path $root "target\release\neochat.exe"
    if (-not (Test-Path -LiteralPath $exe)) {
        throw "Executável não encontrado: $exe"
    }

    $dist = Join-Path $root "dist"
    New-Item -ItemType Directory -Path $dist -Force | Out-Null

    Get-Process -Name "NeoChat", "neochat-desktop-mistral", "NeoChat Mistral" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 300

    Copy-Item -LiteralPath $exe -Destination (Join-Path $dist "NeoChat.exe") -Force
    Copy-Item -LiteralPath $exe -Destination (Join-Path $dist "NeoChat Mistral.exe") -Force
    Copy-Item -LiteralPath (Join-Path $root "README.md") -Destination (Join-Path $dist "README.md") -Force
    Copy-Item -LiteralPath (Join-Path $root "README.pt-BR.md") -Destination (Join-Path $dist "README.pt-BR.md") -Force
    Copy-Item -LiteralPath (Join-Path $root "README.fr.md") -Destination (Join-Path $dist "README.fr.md") -Force
    Copy-Item -LiteralPath (Join-Path $root "LICENSE") -Destination (Join-Path $dist "LICENSE") -Force

    $size = [math]::Round((Get-Item -LiteralPath (Join-Path $dist "NeoChat.exe")).Length / 1MB, 1)
    Write-Host "Pacote gerado em $dist"
    Write-Host "NeoChat.exe - $size MB"
}
finally {
    Pop-Location
}
