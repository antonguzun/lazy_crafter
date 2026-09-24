//! Standalone data-fetching command.
//!
//! The application itself never downloads anything; it only reads a prepared
//! data directory. This binary fetches a dataset from the RePoE-fork export and
//! writes a `manifest.json` next to the files so the app knows how to parse it.
//!
//! Usage:
//!   cargo run --bin fetch_data -- --game poe2 [--out data]
//!   cargo run --bin fetch_data -- --game poe1 --out data_poe1
//!
//! Requires `curl` on PATH (ships with Windows 10+, macOS and most Linux).

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

struct GameSpec {
    id: &'static str,
    base_url: &'static str,
}

fn spec(game: &str) -> Option<GameSpec> {
    match game {
        // RePoE-fork poe2 export (same JSON format as poe1, mods carry inline text).
        "poe2" => Some(GameSpec {
            id: "poe2",
            base_url: "https://repoe-fork.github.io/poe2",
        }),
        // RePoE-fork poe1 export (root of the gh-pages domain).
        "poe1" => Some(GameSpec {
            id: "poe1",
            base_url: "https://repoe-fork.github.io",
        }),
        _ => None,
    }
}

/// Files the application actually loads. Both come from the same export and the
/// representation is read inline from each mod's `text` field, so no separate
/// stat_translations / representation files are needed.
const FILES: &[&str] = &["base_items.min.json", "mods.min.json"];

fn download(url: &str, dest: &Path) -> Result<(), String> {
    let status = Command::new("curl")
        .args(["-fsSL", "--retry", "3", "--max-time", "180", "-o"])
        .arg(dest)
        .arg(url)
        .status()
        .map_err(|e| format!("failed to run curl (is it installed and on PATH?): {}", e))?;
    if !status.success() {
        return Err(format!(
            "curl failed for {} (exit {:?})",
            url,
            status.code()
        ));
    }
    Ok(())
}

fn print_help() {
    println!(
        "fetch_data - download a Path of Exile dataset for lazy_crafter\n\n\
         USAGE:\n    cargo run --bin fetch_data -- --game <poe1|poe2> [--out <dir>]\n\n\
         OPTIONS:\n\
         \x20   --game <id>   Which dataset to fetch: poe1 or poe2 (default: poe2)\n\
         \x20   --out <dir>   Output directory (default: data)\n\
         \x20   -h, --help    Show this help\n\n\
         The app reads ./data by default; point it elsewhere with the\n\
         LAZY_CRAFTER_DATA_DIR environment variable."
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut game = String::from("poe2");
    let mut out = String::from("data");
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--game" => {
                i += 1;
                match args.get(i) {
                    Some(v) => game = v.clone(),
                    None => {
                        eprintln!("--game requires a value");
                        std::process::exit(2);
                    }
                }
            }
            "--out" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out = v.clone(),
                    None => {
                        eprintln!("--out requires a value");
                        std::process::exit(2);
                    }
                }
            }
            "-h" | "--help" => {
                print_help();
                return;
            }
            other => {
                eprintln!("unknown argument: {}", other);
                print_help();
                std::process::exit(2);
            }
        }
        i += 1;
    }

    let spec = match spec(&game) {
        Some(s) => s,
        None => {
            eprintln!("unknown game {:?}; expected \"poe1\" or \"poe2\"", game);
            std::process::exit(2);
        }
    };

    let out_dir = Path::new(&out);
    if let Err(e) = fs::create_dir_all(out_dir) {
        eprintln!("cannot create output directory {}: {}", out, e);
        std::process::exit(1);
    }

    println!(
        "Fetching {} dataset from {} into {}/",
        spec.id, spec.base_url, out
    );
    for f in FILES {
        let url = format!("{}/{}", spec.base_url, f);
        let dest = out_dir.join(f);
        print!("  {} ... ", f);
        std::io::stdout().flush().ok();
        match download(&url, &dest) {
            Ok(()) => println!("ok"),
            Err(e) => {
                println!("FAILED");
                eprintln!("  {}", e);
                std::process::exit(1);
            }
        }
    }

    // Write the manifest the loader keys off of. `inline_text` tells the app to
    // resolve mod representations from each mod's `text` field.
    let manifest = serde_json::json!({
        "game": spec.id,
        "representation": "inline_text",
        "source": spec.base_url,
    });
    let manifest_path = out_dir.join("manifest.json");
    let manifest_str = serde_json::to_string_pretty(&manifest).unwrap();
    if let Err(e) = fs::write(&manifest_path, format!("{}\n", manifest_str)) {
        eprintln!("cannot write {}: {}", manifest_path.display(), e);
        std::process::exit(1);
    }
    println!("Wrote {}", manifest_path.display());

    if out == "data" {
        println!(
            "Done. Start the app normally to use the {} dataset.",
            spec.id
        );
    } else {
        println!(
            "Done. Run the app with LAZY_CRAFTER_DATA_DIR={} to use this dataset.",
            out
        );
    }
}
