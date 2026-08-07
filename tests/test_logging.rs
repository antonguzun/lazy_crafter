//! End-to-end check of the file logger. Lives in its own test binary because
//! `logging::init()` installs the process-wide logger and can only run once.

use std::fs;
use std::path::PathBuf;

#[test]
fn records_land_in_a_log_file() {
    let dir: PathBuf = std::env::temp_dir().join(format!(
        "lazy_crafter_log_e2e_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);

    std::env::set_var("LAZY_CRAFTER_LOG_DIR", &dir);
    std::env::set_var("LAZY_CRAFTER_LOG_STDERR", "0");
    std::env::set_var("RUST_LOG", "info");
    lazy_crafter::logging::init();

    log::info!(target: "test", "hello from the test");
    log::debug!(target: "test", "filtered out by RUST_LOG=info");
    log::logger().flush();

    let files: Vec<PathBuf> = fs::read_dir(&dir)
        .expect("log directory was created")
        .flatten()
        .map(|e| e.path())
        .collect();
    assert_eq!(files.len(), 1, "expected one log file, got {:?}", files);

    let body = fs::read_to_string(&files[0]).unwrap();
    assert!(body.contains("hello from the test"), "body was {:?}", body);
    assert!(!body.contains("filtered out"), "body was {:?}", body);
    // the writer announces where it logs, and the file must be plain text
    assert!(body.contains("logging to"), "body was {:?}", body);
    assert!(!body.contains('\u{1b}'), "ansi escapes leaked into the file");

    fs::remove_dir_all(&dir).unwrap();
}
