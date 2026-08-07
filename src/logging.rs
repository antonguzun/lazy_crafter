//! Logging into rotating files, with retention by size and by age.
//!
//! Records go to `logs/lazy_crafter-<timestamp>.log` and, as before, to stderr —
//! so the documented `RUST_LOG=DEBUG cargo run` workflow keeps working. A fresh
//! file is started on every run, when the current one grows past
//! `LAZY_CRAFTER_LOG_MAX_FILE_MB` and when the date changes. On every rotation
//! old files are pruned: first everything older than
//! `LAZY_CRAFTER_LOG_MAX_AGE_DAYS`, then the oldest remaining ones until the
//! directory fits into `LAZY_CRAFTER_LOG_MAX_TOTAL_MB`.
//!
//! Logging must never take the app down: any filesystem error degrades to
//! stderr-only logging instead of propagating.

use chrono::{Duration, Local, NaiveDate, NaiveDateTime};
use log::info;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const LOG_TARGET: &str = "logging";

/// `lazy_crafter-2026-08-06_14-31-07.482.log`
const FILE_PREFIX: &str = "lazy_crafter-";
const FILE_EXT: &str = ".log";
const TS_FORMAT: &str = "%Y-%m-%d_%H-%M-%S%.3f";

const DEFAULT_DIR: &str = "logs";
const DEFAULT_MAX_FILE_MB: u64 = 5;
const DEFAULT_MAX_TOTAL_MB: u64 = 50;
const DEFAULT_MAX_AGE_DAYS: i64 = 14;
/// stderr-only logging defaulted to `error`; a log file nobody watches live is
/// only useful if it actually holds the run's history
const DEFAULT_FILTER: &str = "info";

const MB: u64 = 1024 * 1024;

/// Where log files live and how long they are kept. Every limit is overridable
/// from the environment; `0` disables the corresponding limit.
#[derive(Debug, Clone)]
pub struct LogConfig {
    pub dir: PathBuf,
    /// rotate to a new file once the current one reaches this size
    pub max_file_bytes: u64,
    /// retention by size: total budget for the whole log directory
    pub max_total_bytes: u64,
    /// retention by date: files older than this are removed
    pub max_age_days: i64,
    /// also mirror every record to stderr, as the app did before it logged to
    /// files; pointless for a windowed build with no console attached
    pub tee_stderr: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            dir: PathBuf::from(DEFAULT_DIR),
            max_file_bytes: DEFAULT_MAX_FILE_MB * MB,
            max_total_bytes: DEFAULT_MAX_TOTAL_MB * MB,
            max_age_days: DEFAULT_MAX_AGE_DAYS,
            tee_stderr: true,
        }
    }
}

impl LogConfig {
    pub fn from_env() -> Self {
        let d = Self::default();
        Self {
            dir: std::env::var("LAZY_CRAFTER_LOG_DIR")
                .map(PathBuf::from)
                .unwrap_or(d.dir),
            max_file_bytes: env_u64("LAZY_CRAFTER_LOG_MAX_FILE_MB", DEFAULT_MAX_FILE_MB) * MB,
            max_total_bytes: env_u64("LAZY_CRAFTER_LOG_MAX_TOTAL_MB", DEFAULT_MAX_TOTAL_MB) * MB,
            max_age_days: env_i64("LAZY_CRAFTER_LOG_MAX_AGE_DAYS", DEFAULT_MAX_AGE_DAYS),
            tee_stderr: !matches!(
                std::env::var("LAZY_CRAFTER_LOG_STDERR").as_deref(),
                Ok("0") | Ok("false")
            ),
        }
    }
}

fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(default)
}

fn env_i64(key: &str, default: i64) -> i64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(default)
}

/// Install the global logger. Filtering still comes from `RUST_LOG`.
pub fn init() {
    let config = LogConfig::from_env();
    let writer = RotatingWriter::new(config.clone());
    let file = writer.path.clone();

    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(DEFAULT_FILTER),
    )
    // the same records go to a file, and ANSI colors in a log file are noise
    .write_style(env_logger::WriteStyle::Never)
    .target(env_logger::Target::Pipe(Box::new(writer)))
    .init();

    match file {
        Some(path) => info!(
            target: LOG_TARGET,
            "logging to {} (rotate at {} MB, keep {} MB / {} days)",
            path.display(),
            config.max_file_bytes / MB,
            config.max_total_bytes / MB,
            config.max_age_days
        ),
        None => info!(target: LOG_TARGET, "file logging is off, stderr only"),
    }
}

/// A `Write` sink for env_logger that tees into stderr and into the current log
/// file, rotating and pruning as it goes. env_logger guards it with a mutex, so
/// no locking is needed here.
struct RotatingWriter {
    config: LogConfig,
    file: Option<File>,
    path: Option<PathBuf>,
    /// bytes in the current file, including what was there before this run
    written: u64,
    opened_on: NaiveDate,
    /// the filesystem refused us once; stop retrying on every record
    degraded: bool,
}

impl RotatingWriter {
    fn new(config: LogConfig) -> Self {
        let mut writer = Self {
            config,
            file: None,
            path: None,
            written: 0,
            opened_on: Local::now().date_naive(),
            degraded: false,
        };
        writer.open_new();
        writer
    }

    fn degrade(&mut self, err: &dyn std::fmt::Display) {
        self.file = None;
        self.path = None;
        self.degraded = true;
        // log! is unavailable here: this runs from inside the logger
        let _ = writeln!(
            io::stderr(),
            "lazy_crafter: file logging disabled, {} is not writable: {}",
            self.config.dir.display(),
            err
        );
    }

    fn open_new(&mut self) {
        if self.degraded {
            return;
        }
        if let Err(e) = fs::create_dir_all(&self.config.dir) {
            self.degrade(&e);
            return;
        }
        let now = Local::now();
        let path = free_path(&self.config.dir, now.naive_local());
        match OpenOptions::new().create(true).append(true).open(&path) {
            Ok(file) => {
                self.written = file.metadata().map(|m| m.len()).unwrap_or(0);
                self.file = Some(file);
                self.opened_on = now.date_naive();
                self.path = Some(path);
                prune(&self.config, self.path.as_deref());
            }
            Err(e) => self.degrade(&e),
        }
    }

    fn rotate_if_needed(&mut self, incoming: u64) {
        if self.degraded {
            return;
        }
        let rotate = match self.file {
            None => true,
            // an empty fresh file must take the record even if it is oversized,
            // otherwise a single huge record would rotate forever
            Some(_) => {
                (self.config.max_file_bytes > 0
                    && self.written > 0
                    && self.written + incoming > self.config.max_file_bytes)
                    || Local::now().date_naive() != self.opened_on
            }
        };
        if rotate {
            if let Some(mut file) = self.file.take() {
                let _ = file.flush();
            }
            self.open_new();
        }
    }
}

impl Write for RotatingWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.config.tee_stderr {
            let _ = io::stderr().write_all(buf);
        }
        self.rotate_if_needed(buf.len() as u64);
        match self.file.as_mut() {
            Some(file) => {
                let written = file.write(buf)?;
                self.written += written as u64;
                Ok(written)
            }
            // nothing to write to, but the record is not the caller's problem
            None => Ok(buf.len()),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.config.tee_stderr {
            let _ = io::stderr().flush();
        }
        match self.file.as_mut() {
            Some(file) => file.flush(),
            None => Ok(()),
        }
    }
}

fn path_for(dir: &Path, stamp: NaiveDateTime) -> PathBuf {
    dir.join(format!(
        "{}{}{}",
        FILE_PREFIX,
        stamp.format(TS_FORMAT),
        FILE_EXT
    ))
}

/// A log path that is not taken yet. File names are millisecond-precise, but two
/// rotations can still land in the same millisecond — appending to the file we
/// just rotated away from would let it grow past the size limit, so walk the
/// timestamp forward until a free name turns up.
fn free_path(dir: &Path, stamp: NaiveDateTime) -> PathBuf {
    let mut stamp = stamp;
    for _ in 0..1000 {
        let path = path_for(dir, stamp);
        if !path.exists() {
            return path;
        }
        stamp += Duration::milliseconds(1);
    }
    path_for(dir, stamp)
}

/// Timestamp encoded in a log file name, or None for anything that is not ours —
/// files we did not create are never deleted.
fn timestamp_of(file_name: &str) -> Option<NaiveDateTime> {
    let stamp = file_name
        .strip_prefix(FILE_PREFIX)?
        .strip_suffix(FILE_EXT)?;
    NaiveDateTime::parse_from_str(stamp, TS_FORMAT).ok()
}

/// Apply both retention limits to the log directory. `current` is the file being
/// written right now: it is never deleted, but it does count against the size
/// budget.
fn prune(config: &LogConfig, current: Option<&Path>) {
    let entries = match fs::read_dir(&config.dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    let mut files: Vec<(NaiveDateTime, PathBuf, u64)> = Vec::new();
    let mut current_size = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        if Some(path.as_path()) == current {
            current_size = size;
            continue;
        }
        let stamp = match path.file_name().and_then(|n| n.to_str()).and_then(timestamp_of) {
            Some(stamp) => stamp,
            None => continue,
        };
        files.push((stamp, path, size));
    }
    files.sort_by_key(|(stamp, _, _)| *stamp);

    // retention by date
    if config.max_age_days > 0 {
        let cutoff = Local::now().naive_local() - Duration::days(config.max_age_days);
        files.retain(|(stamp, path, _)| {
            if *stamp >= cutoff {
                return true;
            }
            // keep it in the list if it could not be removed, so the size pass
            // still accounts for it
            fs::remove_file(path).is_err()
        });
    }

    // retention by size: drop the oldest until the directory fits
    if config.max_total_bytes > 0 {
        let mut total: u64 = current_size + files.iter().map(|(_, _, size)| size).sum::<u64>();
        for (_, path, size) in files.iter() {
            if total <= config.max_total_bytes {
                break;
            }
            if fs::remove_file(path).is_ok() {
                total -= size;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "lazy_crafter_log_test_{}_{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_log_file(dir: &Path, age_days: i64, size: usize) -> PathBuf {
        let stamp = Local::now().naive_local() - Duration::days(age_days);
        let path = dir.join(format!(
            "{}{}{}",
            FILE_PREFIX,
            stamp.format(TS_FORMAT),
            FILE_EXT
        ));
        fs::write(&path, vec![b'x'; size]).unwrap();
        path
    }

    fn names(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(dir)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn timestamp_is_read_back_from_the_file_name() {
        let dir = temp_dir();
        let path = write_log_file(&dir, 0, 1);
        let name = path.file_name().unwrap().to_str().unwrap();
        assert!(timestamp_of(name).is_some());
        // foreign files are unrecognized, so pruning leaves them alone
        assert!(timestamp_of("notes.log").is_none());
        assert!(timestamp_of("lazy_crafter-whenever.log").is_none());
        assert!(timestamp_of("lazy_crafter-2026-08-06_14-31-07.482.txt").is_none());
    }

    #[test]
    fn prune_removes_files_older_than_max_age() {
        let dir = temp_dir();
        let old = write_log_file(&dir, 30, 10);
        let fresh = write_log_file(&dir, 1, 10);
        let config = LogConfig {
            dir: dir.clone(),
            max_age_days: 14,
            max_total_bytes: 0,
            ..LogConfig::default()
        };

        prune(&config, None);

        assert!(!old.exists());
        assert!(fresh.exists());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn prune_drops_oldest_until_total_size_fits() {
        let dir = temp_dir();
        let oldest = write_log_file(&dir, 3, 100);
        let older = write_log_file(&dir, 2, 100);
        let newest = write_log_file(&dir, 1, 100);
        let config = LogConfig {
            dir: dir.clone(),
            max_age_days: 0,
            max_total_bytes: 250,
            ..LogConfig::default()
        };

        prune(&config, None);

        assert!(!oldest.exists());
        assert!(older.exists());
        assert!(newest.exists());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn prune_counts_the_current_file_and_never_deletes_it() {
        let dir = temp_dir();
        let current = write_log_file(&dir, 0, 200);
        let older = write_log_file(&dir, 1, 100);
        let config = LogConfig {
            dir: dir.clone(),
            max_age_days: 0,
            max_total_bytes: 250,
            ..LogConfig::default()
        };

        prune(&config, Some(&current));

        assert!(current.exists());
        assert!(!older.exists());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn prune_ignores_files_it_did_not_create() {
        let dir = temp_dir();
        fs::write(dir.join("readme.txt"), "keep me").unwrap();
        fs::write(dir.join("lazy_crafter-broken-name.log"), "keep me too").unwrap();
        write_log_file(&dir, 90, 10);
        let config = LogConfig {
            dir: dir.clone(),
            max_age_days: 1,
            max_total_bytes: 1,
            ..LogConfig::default()
        };

        prune(&config, None);

        assert_eq!(
            names(&dir),
            vec!["lazy_crafter-broken-name.log", "readme.txt"]
        );
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn writer_rotates_when_the_file_gets_too_big() {
        let dir = temp_dir();
        let mut writer = RotatingWriter::new(LogConfig {
            dir: dir.clone(),
            max_file_bytes: 64,
            max_total_bytes: 0,
            max_age_days: 0,
            tee_stderr: false,
        });

        for _ in 0..10 {
            writer.write_all(&[b'x'; 32]).unwrap();
        }
        writer.flush().unwrap();

        let files = names(&dir);
        assert!(files.len() > 1, "expected a rotation, got {:?}", files);
        for name in &files {
            let size = fs::metadata(dir.join(name)).unwrap().len();
            assert!(size <= 64, "{} grew past the limit: {}", name, size);
        }
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn writer_applies_retention_while_rotating() {
        let dir = temp_dir();
        let mut writer = RotatingWriter::new(LogConfig {
            dir: dir.clone(),
            max_file_bytes: 64,
            max_total_bytes: 128,
            max_age_days: 0,
            tee_stderr: false,
        });

        for _ in 0..20 {
            writer.write_all(&[b'x'; 32]).unwrap();
        }
        writer.flush().unwrap();

        let total: u64 = fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .map(|e| e.metadata().unwrap().len())
            .sum();
        assert!(total <= 128 + 64, "log directory grew unbounded: {}", total);
        fs::remove_dir_all(&dir).unwrap();
    }
}
