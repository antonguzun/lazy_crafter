<#
.SYNOPSIS
    Builds the Windows release binary and packs it with the data files the app
    reads at runtime into dist/lazy_crafter-<version>-windows-x86_64.zip.

.DESCRIPTION
    Used by .github/workflows/release.yml, and runnable by hand to reproduce a
    release artifact locally.

    The font is embedded in the binary (include_bytes! in src/ui/ui_app.rs), so
    assets/ is not shipped. The datasets are read from disk next to the exe and
    are shipped.

.EXAMPLE
    ./tools/package_release.ps1

.EXAMPLE
    # local build where the default toolchain is the (broken) GNU one
    ./tools/package_release.ps1 -Toolchain stable-x86_64-pc-windows-msvc
#>
[CmdletBinding()]
param(
    # Defaults to the version in Cargo.toml.
    [ValidatePattern('^\d+\.\d+\.\d+$')]
    [string]$Version,
    [string]$OutDir = 'dist',
    # Passed to cargo as `+<toolchain>`; empty means the default toolchain.
    [string]$Toolchain,
    # Pack whatever is already in target/release.
    [switch]$SkipBuild
)

$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path -Parent $PSScriptRoot
Push-Location $repoRoot
try {
    $match = Select-String -Path 'Cargo.toml' -Pattern '^version\s*=\s*"([^"]+)"' |
        Select-Object -First 1
    if (-not $match) { throw 'Could not read the version from Cargo.toml' }
    $packageVersion = $match.Matches[0].Groups[1].Value
    if ($Version -and $Version -ne $packageVersion) {
        throw "Version $Version does not match Cargo.toml version $packageVersion"
    }
    $Version = $packageVersion
    if ($Version -notmatch '^\d+\.\d+\.\d+$') { throw "Invalid release version: $Version" }

    if (-not $SkipBuild) {
        Write-Host "Building lazy_crafter $Version (release)"
        if ($Toolchain) {
            & cargo "+$Toolchain" build --locked --release --bin lazy_crafter
        } else {
            & cargo build --locked --release --bin lazy_crafter
        }
        if ($LASTEXITCODE -ne 0) { throw "cargo build failed with exit code $LASTEXITCODE" }
    }

    # Only the files FileRepo actually opens (see src/storage/files/local_db.rs).
    # The rest of the RePoE dump in data/ is ~250 MB of JSON the app never reads.
    $payload = @{
        'data'      = @(
            'mods.min.json',
            'base_items.min.json',
            'stat_translations.min.json',
            'mods_representation_pob.json'
        )
        'data_poe2' = @(
            'mods.min.json',
            'base_items.min.json',
            'manifest.json'
        )
    }

    $stageName = "lazy_crafter-$Version-windows-x86_64"
    $outputRoot = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($OutDir)
    $stage = [IO.Path]::GetFullPath((Join-Path $outputRoot $stageName))
    if ([IO.Path]::GetDirectoryName($stage) -ne $outputRoot.TrimEnd('\', '/')) {
        throw "Staging path must be inside the output directory: $stage"
    }
    if (Test-Path -LiteralPath $stage) {
        if ((Get-Item -LiteralPath $stage).Attributes -band [IO.FileAttributes]::ReparsePoint) {
            throw "Refusing to replace a staging directory that is a link: $stage"
        }
        Remove-Item -LiteralPath $stage -Recurse -Force
    }
    New-Item -ItemType Directory -Path $stage -Force | Out-Null

    $exe = 'target/release/lazy_crafter.exe'
    if (-not (Test-Path $exe)) { throw "$exe not found - build first, or drop -SkipBuild" }
    Copy-Item $exe $stage

    foreach ($dir in $payload.Keys) {
        $target = Join-Path $stage $dir
        New-Item -ItemType Directory -Path $target -Force | Out-Null
        foreach ($file in $payload[$dir]) {
            $source = Join-Path $dir $file
            if (-not (Test-Path $source)) { throw "Missing dataset file: $source" }
            Copy-Item $source $target
        }
    }

    Copy-Item 'README.md', 'CHANGELOG.md', 'LICENSE' $stage

    $zip = Join-Path $OutDir "$stageName.zip"
    if (Test-Path $zip) { Remove-Item $zip -Force }
    Compress-Archive -Path $stage -DestinationPath $zip

    $sizeMb = [math]::Round((Get-Item $zip).Length / 1MB, 1)
    Write-Host "Packed $zip ($sizeMb MB)"
    (Get-Item $zip).FullName
} finally {
    Pop-Location
}
