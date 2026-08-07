# lazy_crafter

## Features

- Mods filtering by item class, item base, item level and text search. (Not tested enought, there are some represenation mods mistakes)
- Crafting chance calculation is not ready
- Auto crafting is not stable, but you can try it. (Ctrl+N on item with "currency in hand")

## Disclaimer

This application doesn't follow GGG's ToS. GGG would ban you if you use that application.

The app doesn't change game files. It works with your clipboard buffer and control your clicks only.

It may be quite hard and expensive to reveal the usage of that kind of app. However, I can't give you any guaranties.

### Planned features

- Filtering stabilization
- Auto-crafting stabilization
- Auto-colorization
- Auto-linking
- Improve test coverage
- Add currency choices for estimation
- Implement estimation for selected mods and average/median cost
- CI

### lower priority plans 

- Telemetry for bugs
- Telemetry for statistic

## Download

earlier version x86

https://github.com/antonguzun/lazy_crafter/releases/tag/0.4.2

## Demo

[![demo](https://img.youtube.com/vi/tH3UOBZh0-w/0.jpg)](https://www.youtube.com/watch?v=tH3UOBZh0-w "Demo")

## Data (Path of Exile 1 & 2)

The app does not download anything itself — it only reads prepared data
directories (`data/` for poe1, `data_poe2/` for poe2). Fetching a dataset is a
separate command. Switch the game at runtime with the `PoE 1` / `PoE 2` toggle
at the top of the input panel — the dataset is reloaded on the fly, no restart
needed. If the selected dataset is missing or broken, an error message is shown
and the app stays on the previous one.

The dataset format is auto-detected from an optional `manifest.json` placed next
to the data files:

```json
{
  "game": "poe2",
  "representation": "inline_text",
  "source": "https://repoe-fork.github.io/poe2"
}
```

- `representation: "inline_text"` — read each mod's representation from its
  inline `text` field (the [RePoE-fork](https://repoe-fork.github.io) poe1/poe2
  exports). GGG markup like `[ElementalDamage|Elemental Damage]` is stripped to
  plain text automatically.
- `representation: "pob_file"` — legacy: read representations from a separate
  `mods_representation_pob.json`. This is the default when no manifest is
  present, so existing poe1 data directories keep working unchanged.

### Fetching a dataset

Downloads `base_items.min.json` + `mods.min.json` and writes a matching
`manifest.json`. Requires `curl` (ships with Windows 10+, macOS and most Linux).

```sh
# Path of Exile 2 into ./data_poe2 (picked up by the in-app PoE 2 toggle)
cargo run --bin fetch_data -- --game poe2 --out data_poe2
```

The default directories can be overridden without rebuilding:

```PowerShell
$env:LAZY_CRAFTER_DATA_DIR = 'my_poe1_data'        # PoE 1 slot, default: data
$env:LAZY_CRAFTER_DATA_DIR_POE2 = 'my_poe2_data'   # PoE 2 slot, default: data_poe2
cargo run
```

Data is sourced from the RePoE-fork export, which datamines the same game files
as poe2db.tw and publishes them as JSON in the RePoE format.

## Build
it requires rustc 1.65 or newer

```sh
cargo build
```

## Release

Both datasets (`data/`, `data_poe2/`) live in the repository, so a release needs
no extra fetching. Bump the version in `Cargo.toml`, then push a tag named after
it:

```sh
git tag 0.4.3 && git push origin 0.4.3
```

`.github/workflows/release.yml` builds the Windows binary, packs it with the
data files the app reads at runtime and attaches the zip to a **draft** GitHub
release — review it there and press publish. The workflow fails early if the tag
and the `Cargo.toml` version disagree.

The same archive can be built locally (this is what CI runs):

```PowerShell
./tools/package_release.ps1
```

## Run as debug

### unix

```sh
RUST_LOG=DEBUG cargo run
```

### windows

```PowerShell
$env:RUST_LOG='DEBUG'
cargo run
```

## Logs

The app writes every log record both to stderr and to a file in `logs/`:

```
logs/lazy_crafter-2026-08-06_14-31-07.482.log
```

A new file is started on every run, when the current one reaches the size limit
and when the date changes. Old files are cleaned up on every rotation — first by
age, then by total size, so the directory never grows without bound.

Verbosity is still `RUST_LOG` (`info` by default now, so a file is worth
keeping). Everything else is tunable without rebuilding; `0` disables a limit:

```PowerShell
$env:LAZY_CRAFTER_LOG_DIR = 'logs'          # where log files go
$env:LAZY_CRAFTER_LOG_MAX_FILE_MB = '5'     # rotate to a new file at this size
$env:LAZY_CRAFTER_LOG_MAX_TOTAL_MB = '50'   # retention by size, whole directory
$env:LAZY_CRAFTER_LOG_MAX_AGE_DAYS = '14'   # retention by date
$env:LAZY_CRAFTER_LOG_STDERR = '0'          # stop mirroring to stderr
cargo run
```

Files in `logs/` that the app did not create are never touched.

## Run tests

### unit tests

```sh
cargo run tests
```

### integration tests

```sh
cargo test --test test_item_parser
```

### Dependences
used prepaired data by RePoe https://github.com/brather1ng/RePoE

used font https://www.exljbris.com/fontin.html
