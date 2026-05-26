param(
    [string]$WebsiteRoot,
    [switch]$SkipWebsite,
    [switch]$SkipCargo
)

$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
if (-not $WebsiteRoot) {
    $WebsiteRoot = Join-Path (Split-Path $repoRoot -Parent) "MesmerPrism.github.io"
}

function Invoke-Step {
    param(
        [string]$Name,
        [scriptblock]$Command
    )

    Write-Host "==> $Name"
    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "$Name failed with exit code $LASTEXITCODE"
    }
}

function Assert-HashPair {
    param(
        [string]$Report,
        [string]$Asset
    )

    $reportPath = Join-Path $repoRoot $Report
    $assetPath = Join-Path $WebsiteRoot $Asset
    if (-not (Test-Path $reportPath)) {
        throw "missing report: $Report"
    }
    if (-not (Test-Path $assetPath)) {
        throw "missing website asset: $Asset"
    }

    $reportHash = (Get-FileHash $reportPath).Hash
    $assetHash = (Get-FileHash $assetPath).Hash
    if ($reportHash -ne $assetHash) {
        throw "hash mismatch: $Report -> $Asset ($($reportHash.Substring(0, 12)) != $($assetHash.Substring(0, 12)))"
    }
    Write-Host "hash ok: $Report"
}

Push-Location $repoRoot
try {
    if (-not $SkipCargo) {
        Invoke-Step "cargo fmt" {
            cargo fmt --manifest-path rust-v1-sim\Cargo.toml --check
        }
        Invoke-Step "cargo test" {
            cargo test --manifest-path rust-v1-sim\Cargo.toml
        }
        Invoke-Step "cargo clippy" {
            cargo clippy --manifest-path rust-v1-sim\Cargo.toml --all-targets -- -D warnings
        }
    }

    Invoke-Step "repo git diff --check" {
        git diff --check
    }

    if (-not $SkipWebsite) {
        if (-not (Test-Path $WebsiteRoot)) {
            throw "website root not found: $WebsiteRoot"
        }

        Push-Location $WebsiteRoot
        try {
            $jsChecks = @(
                "scripts\generate-agent-artifacts.js",
                "scripts\rule-dynamics-explorer.js",
                "scripts\driven-field-diagnostics.js",
                "scripts\bressloff-calibration-panel.js"
            )
            foreach ($script in $jsChecks) {
                if (Test-Path $script) {
                    Invoke-Step "node --check $script" {
                        node --check $script
                    }
                }
            }
            Invoke-Step "website git diff --check" {
                git diff --check
            }
        }
        finally {
            Pop-Location
        }

        $pairs = @(
            @("reports\figure-targets\bressloff-generated-stills.json", "assets\bressloff-v1\figure-targets\bressloff-generated-stills.json"),
            @("reports\rule-2011-sweep.json", "assets\bressloff-v1\deep-dive\rule-2011-sweep.json"),
            @("reports\rule-2011-sweep-dense.json", "assets\bressloff-v1\deep-dive\rule-2011-sweep-dense.json"),
            @("reports\rule-2011-floquet.json", "assets\bressloff-v1\deep-dive\rule-2011-floquet.json"),
            @("reports\source-curves\rule-2011-fig8-source-curves.json", "assets\bressloff-v1\deep-dive\rule-2011-fig8-source-curves.json"),
            @("reports\driven-neural-fields-registry.json", "assets\bressloff-v1\driven\driven-neural-fields-registry.json"),
            @("reports\mackay-localized-input.json", "assets\bressloff-v1\driven\mackay-localized-input.json"),
            @("reports\bolelli-time-periodic-input.json", "assets\bressloff-v1\driven\bolelli-time-periodic-input.json"),
            @("reports\nicks-orthogonal-response.json", "assets\bressloff-v1\driven\nicks-orthogonal-response.json")
        )

        foreach ($pair in $pairs) {
            Assert-HashPair -Report $pair[0] -Asset $pair[1]
        }
    }
}
finally {
    Pop-Location
}

Write-Host "verification passed"
