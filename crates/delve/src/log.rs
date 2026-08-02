//! Logging
//! Somewhat naive implementation. Essentially we have a single global logger that gets written to.
//! Flushing happens based on the size of the buffer, not handled by a central thread but rather
//! whoever fills the buffer first. Synchronization done via a global mutex.
//!
//! The reasoning behind this is because thread local loggers would be hard to make flush properly.
//! Also, trying to make thread local senders to a global receiver runs into issues because
//! Receiver doesn't implement Send.
//!
//! set DLV_LOG_TARGETS="<comma delimited list>" as an env variable to filter.
//!
//! TODO: revisit this if log perf becomes a problem.

use log::Level;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::{Mutex, OnceLock, atomic::Ordering};

static THREAD_ID_INC: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    static THREAD_ID: usize = THREAD_ID_INC.fetch_add(1, Ordering::SeqCst);
}

pub struct DelveLogger {
    location: OnceLock<PathBuf>,
    level: Level,
    autoflush_enabled: bool,
    autoflush_size: usize,
    inner_buf: Mutex<String>,
    targets: OnceLock<Vec<String>>,
}

impl DelveLogger {
    pub const DEFAULT_LOG_NAME: &'static str = "delve.log";
    pub const TARGET_ENV_VAR_NAME: &'static str = "DLV_LOG_TARGETS";
    pub const DEFAULT_AUTOFLUSH_SIZE: usize = 1024;

    pub const fn new(level: Level, autoflush_enabled: bool, autoflush_size: usize) -> Self {
        Self {
            location: OnceLock::new(),
            level,
            autoflush_enabled,
            autoflush_size,
            inner_buf: Mutex::new(String::new()),
            targets: OnceLock::new(),
        }
    }

    fn init_targets(&self) -> Vec<String> {
        if let Ok(val) = std::env::var(Self::TARGET_ENV_VAR_NAME) {
            return val.split(",").map(String::from).collect();
        }

        Vec::new()
    }

    /// idempotent
    pub fn init(&self, location: Option<PathBuf>) {
        let path_buf = match location {
            None => std::env::current_dir()
                .expect("unable to get current working directory.")
                .join(Self::DEFAULT_LOG_NAME),
            Some(pb) => pb,
        };

        self.targets.get_or_init(|| self.init_targets());

        self.location.get_or_init(|| path_buf);
    }
}

impl log::Log for DelveLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        if metadata.level() < self.level {
            return false;
        }

        metadata.target().is_empty()
            || self
                .targets
                .get_or_init(|| self.init_targets())
                .iter()
                .find(|&item| item.eq(metadata.target()))
                .is_some()
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S.%6f");

        let log_line = format!(
            "[{}]-(thread {})-<{}> {}\n",
            time,
            THREAD_ID.with(|t| *t),
            record.metadata().level(),
            record.args()
        );

        let curr_log_size = {
            let mut inner_buf = self.inner_buf.lock().expect("log mutex poisoned on write");
            inner_buf.push_str(&log_line);
            inner_buf.len()
        };

        if self.autoflush_enabled && curr_log_size >= self.autoflush_size {
            self.flush();
        }
    }

    fn flush(&self) {
        let location = self.location.get().expect("flush called before init");
        let mut file_handle = File::options()
            .create(true)
            .append(true)
            .open(location)
            .unwrap_or_else(|_| {
                panic!(
                    "log file at location {:?} unable to be opened",
                    self.location
                )
            });

        // released when file handle dropped
        file_handle
            .lock()
            .unwrap_or_else(|_| panic!("unable to acquire lock on log file {:?}", self.location));

        let mut inner_buf = self.inner_buf.lock().expect("log mutex poisoned on flush");
        let _ = file_handle.write(inner_buf.as_bytes());
        inner_buf.clear();
    }
}
