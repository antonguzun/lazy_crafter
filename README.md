# lazy_crafter

## Features

- Mods filtering by item class, item base, item level and text search. (Not tested enought, there are some represenation mods mistakes)
- Crafting chance calculation is not ready
- Auto crafting is not stable, but you can try it. (Ctrl+Shift+E on item with "currency in hand")

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

https://github.com/antonguzun/lazy_crafter/releases/tag/0.4.0

## Demo

[![demo](https://img.youtube.com/vi/tH3UOBZh0-w/0.jpg)](https://www.youtube.com/watch?v=tH3UOBZh0-w "Demo")

## Build
it requires rustc 1.65 or newer

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

### Dependences
used prepaired data by RePoe https://github.com/brather1ng/RePoE

used font https://www.exljbris.com/fontin.html
