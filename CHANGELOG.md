# Release notes

## 0.4.3

### Added

- Bundled Path of Exile 2 data and an in-app PoE 1 / PoE 2 switch. PoE 2 is
  selected on startup; switching games clears the selected modifiers.
- Support for Ruby, Emerald, Sapphire, Diamond and their Time-Lost variants,
  including separate modifier pools and advanced clipboard descriptions.
- AND / OR target matching, a minimum roll percentage within each tier, and an
  optional OR exception based on the number of distinct matching modifiers.
- A separate `fetch_data` command for preparing datasets from RePoE-fork.
- File logging with rotation and retention limits.
- A Windows x86_64 release archive containing both datasets and documentation.

### Fixed

- Clipboard access failures now retry and report an error instead of panicking.
  Crafting releases held keys on success, failure and unwinding; unchanged
  clipboard contents stop crafting after five retries without additional rolls.
- Parsing of hybrid modifiers, fractured items, blank modifier lines, and
  Time-Lost jewel effects and radius annotations.
- Modifier text formatting now applies numeric formats to the stat value.
- Public modifier text lookup uses the same bundled representations as the UI;
  the regression test checks all 10,180 reference entries without exclusions.

### Release checks

- Track Cargo.lock and use locked dependencies for tests and release builds.
- Check formatting and run all test targets before packaging in the release
  workflow. Tags must match the package version; GitHub releases start as drafts.

### Known limitations

- Auto crafting is experimental and supported only on Windows. The parser
  expects English item descriptions and the bundled data may lag game updates.
- Crafting chance and cost estimation are unfinished. Some modifier
  representations still have known mismatches.
- The roll limit counts currency clicks; the result of the final permitted
  click is not checked until crafting is started again.
