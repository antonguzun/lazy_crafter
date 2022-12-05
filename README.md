# lazy_crafter

## Features

- Mods filtering by item class, item base, item level and text search. (Not tested enought, there are some represenation mods mistakes)
- Crafting chance calculation is not ready
- Auto crafting is not ready, but you can try it in develop mod. (Ctrl+Alt+C on item with "currency in hand")

### Planned features

- Filtering stabilization
- Add currency choices for estimation
- Implement estimation for selected mods and average/median cost
- Add target mods for auto-crafting feature
- Add maximum count of tries for auto-crafting feature
- Auto-crafting stabilization
- Imrove test coverage
- CI

### lower priority plans 

- Telemetry for bugs
- Telemetry for statistic

## Download

earlier version x86

https://github.com/antonguzun/lazy_crafter/releases/tag/0.1.0

## Build

```sh
cargo build
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

## Run tests

### unit tests

```sh
cargo run tests
```

### integration tests

```sh
cargo test --test test_item_parser
```
