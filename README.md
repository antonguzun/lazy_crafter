# lazy_crafter

A desktop modifier browser and experimental auto crafter for Path of Exile 1
and 2. Version **0.4.3** ships with both datasets; **PoE 2 is selected on
startup**. See [release notes](CHANGELOG.md) for changes since 0.4.2.

## Features

- Filter modifiers by game, item class, base, item level and text.
- Select target modifiers and stop on all targets (AND) or any target (OR),
  accepting the selected tier or a better one.
- Require a minimum roll within the matched tier, with an optional exception
  for multiple matching targets in OR mode.
- Browse and craft PoE 2 jewels: Ruby, Emerald, Sapphire, Diamond and their
  four Time-Lost variants, each with its own modifier pool.
- Read advanced English item descriptions from the clipboard and write
  diagnostic logs with rotation and retention limits.

Auto crafting is Windows-only and experimental. Chance and cost estimation
are unfinished, and some modifier representations still have known mismatches.
Bundled datasets are snapshots and may not reflect subsequent game updates.

## Disclaimer

This application doesn't follow GGG's ToS. GGG would ban you if you use that application.

The app doesn't change game files. It works with your clipboard buffer and control your clicks only.

It may be quite hard and expensive to reveal the usage of that kind of app. However, I can't give you any guaranties.

## Download and run (Windows x86_64)

1. Open [GitHub Releases](https://github.com/antonguzun/lazy_crafter/releases)
   and download the Windows x86_64 zip for the desired release. Version 0.4.3
   will appear there after its draft is published.
2. Extract the entire archive. Keep `data/` and `data_poe2/` beside
   `lazy_crafter.exe`; do not run the executable from inside the zip.
3. Start the application from the extracted folder. If using a shortcut, set
   its **Start in** field to that folder: data and log paths are relative to
   the current working directory.

Rust and a separate data download are not needed to run the release archive.
The older [0.4.2 release](https://github.com/antonguzun/lazy_crafter/releases/tag/0.4.2)
does not include the features described for 0.4.3.

## Using auto crafting

1. Choose `PoE 1` or `PoE 2`. Switching games clears the selected targets.
2. Choose the item class, base and level, or paste an English item description
   into `or paste item`. For PoE 2 jewels, select the `Jewel` class.
3. Select target modifiers from the filtered list. Choose `AND` to require
   every target or `OR` to accept any target. An empty selection matches
   immediately and does not spend currency.
4. Set `Max autocraft tries` (default: 5) and the roll requirements below.
5. Focus the game, pick up the currency to apply, hover the target item and
   press **left Ctrl+N**. The app holds Shift+Alt, copies the advanced item
   description with Ctrl+C, and clicks to apply currency when it does not match.

Keep the game focused and the pointer on the item while crafting. Ctrl+N starts
a run; it is not a pause or cancellation shortcut. The maximum number of tries
counts currency clicks, not clipboard retries. The result of the final allowed
click is checked only when you start another run.

Crafting stops on a match, the click limit, a parsing error, persistent clipboard
access failure, or five unchanged clipboard retries. Temporary clipboard access
errors are retried without additional currency clicks. Held keys are released
when the run exits. After an error, correct the cause and press Ctrl+N again.

### Target rolls

`Minimum roll within tier` sets a shared threshold for selected mods: 0%
accepts the start of the range, and 100% requires its end. For example, 80% of a
10–20 range requires 18 or higher. Each value in a hybrid mod must meet the
threshold, using the rolled mod's own tier range, including when a better tier
is rolled. Fixed stats always pass. The ranges come from advanced item text.

In OR mode, `Ignore roll at N OR mods` accepts any roll when at least N distinct
mods on the item match selected targets and their tier requirements. One mod
matching several selected alternatives counts once. Set N to 0 to disable this
exception; it does not apply in AND mode. Defaults (0%, N = 0) preserve tier-only
matching. These settings are captured when each crafting run starts.

## Data

The GUI reads prepared directories and does not download data itself:

| Game | Default directory | Override |
| --- | --- | --- |
| PoE 1 | `data/` | `LAZY_CRAFTER_DATA_DIR` |
| PoE 2 | `data_poe2/` | `LAZY_CRAFTER_DATA_DIR_POE2` |

Switching games reloads the dataset without restarting. If the requested
dataset is missing or invalid, an error is shown and the previous game stays
active. If the initial PoE 2 dataset cannot load, fix its directory or override
and restart the app.

An optional `manifest.json` next to the data files selects the representation:

```json
{
  "game": "poe2",
  "representation": "inline_text",
  "source": "https://repoe-fork.github.io/poe2"
}
```

- `inline_text` uses the modifier's `text` field and strips GGG markup such as
  `[ElementalDamage|Elemental Damage]`. It reads `base_items.min.json` and
  `mods.min.json`.
- `pob_file` is the legacy default when no manifest is present. It additionally
  reads `stat_translations.min.json` and `mods_representation_pob.json`.

### Fetching a dataset (from source)

The separate `fetch_data` binary downloads data from
[RePoE-fork](https://repoe-fork.github.io) and writes an `inline_text` manifest.
It requires Rust and `curl` on PATH. Always pass `--out`: the fetcher's current
default is `data`, even for PoE 2. Existing files in that directory are replaced.
Use a new directory to try updated data without replacing the bundled snapshot.

```powershell
cargo +stable-x86_64-pc-windows-msvc run --locked --bin fetch_data -- --game poe2 --out data_poe2_new
$env:LAZY_CRAFTER_DATA_DIR_POE2 = 'data_poe2_new'
cargo +stable-x86_64-pc-windows-msvc run --locked
```

For PoE 1, use `--game poe1 --out data_poe1_new` and set
`LAZY_CRAFTER_DATA_DIR` instead. The fetcher's final hint currently names that
PoE 1 variable for custom paths; use the variable for the game you downloaded.

## Build and test

The release target is Windows x86_64 with the MSVC Rust toolchain. Install Rust
through rustup and the Visual Studio C++ build tools / Windows SDK. This release
is validated with Rust 1.98.1; older toolchains have not been revalidated.
Run commands from the repository root so the tests can find the bundled data.

```powershell
rustup toolchain install stable-x86_64-pc-windows-msvc
cargo +stable-x86_64-pc-windows-msvc build --locked
cargo +stable-x86_64-pc-windows-msvc run --locked
cargo +stable-x86_64-pc-windows-msvc fmt --all -- --check
cargo +stable-x86_64-pc-windows-msvc test --locked --all-targets
cargo +stable-x86_64-pc-windows-msvc test --locked --doc
```

`cargo run` launches the GUI. `cargo test` runs tests; `cargo run tests` does
not. To test just the PoE 2 jewel cases:

```powershell
cargo +stable-x86_64-pc-windows-msvc test --locked --test test_poe2_jewels
```

Close a running debug build before rebuilding it on Windows, or use
`cargo +stable-x86_64-pc-windows-msvc test --locked --release --all-targets`.
Cargo.lock is committed; keep it in sync with Cargo.toml and use `--locked`
for validation and packaging.

## Release

1. Update the version in Cargo.toml and Cargo.lock, the release notes in
   CHANGELOG.md, and the version references in this README.
2. Run the formatting and test commands above, then build the archive:

   ```powershell
   ./tools/package_release.ps1 -Toolchain stable-x86_64-pc-windows-msvc
   ```

   The output is `dist/lazy_crafter-0.4.3-windows-x86_64.zip`, containing the
   executable, both runtime datasets, README, release notes and license. The
   font is embedded in the executable. No dataset download is needed.
3. Extract the zip into a separate folder and check startup, both game modes
   and the target controls. Commit the prepared changes and push the branch.
4. Tag the tested commit with its bare package version and push the tag:

   ```sh
   git tag 0.4.3
   git push origin 0.4.3
   ```

The [release workflow](.github/workflows/release.yml) checks that the tag matches
Cargo.toml, checks formatting, runs all test targets and doc tests, builds the Windows archive,
and attaches it to a **draft** GitHub release with CHANGELOG.md as its notes.
Review the draft and archive before publishing. Running the workflow manually
builds an Actions artifact without creating a GitHub release.

## Logs and troubleshooting

Logs go to stderr and `logs/lazy_crafter-<timestamp>.log`. Files rotate at 5 MB
and on date changes; retention defaults to 14 days and 50 MB in total. Files
the app did not create are not removed. Override these defaults before starting:

```powershell
$env:RUST_LOG = 'debug'                     # default: info
$env:LAZY_CRAFTER_LOG_DIR = 'logs'
$env:LAZY_CRAFTER_LOG_MAX_FILE_MB = '5'
$env:LAZY_CRAFTER_LOG_MAX_TOTAL_MB = '50'
$env:LAZY_CRAFTER_LOG_MAX_AGE_DAYS = '14'
$env:LAZY_CRAFTER_LOG_STDERR = '0'           # disable stderr mirroring
.\lazy_crafter.exe
```

Setting a numeric log limit to `0` disables that limit. When reporting a parsing
or crafting failure, include the relevant log excerpt, selected game/base,
target settings and the English advanced item text. For missing data errors,
check the working directory and data overrides. For unchanged clipboard errors,
check game focus and pointer position.

## Demo and credits

The [original demo](https://www.youtube.com/watch?v=tH3UOBZh0-w) shows an older UI.
Data comes from [RePoE](https://github.com/brather1ng/RePoE) and
[RePoE-fork](https://repoe-fork.github.io). The embedded font is
[Fontin](https://www.exljbris.com/fontin.html). See [LICENSE](LICENSE).
