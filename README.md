# lazy_crafter

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
@set RUST_LOG=DEBUG
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
