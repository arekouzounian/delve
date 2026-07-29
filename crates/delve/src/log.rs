//! Logging

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
}

impl DelveLogger {
    const DEFAULT_LOG_NAME: &'static str = "delve.log";
    const DEFAULT_AUTOFLUSH_SIZE: usize = 1024;

    pub const fn new(level: Level, autoflush_enabled: bool, autoflush_size: usize) -> Self {
        Self {
            location: OnceLock::new(),
            level,
            autoflush_enabled,
            autoflush_size,
            inner_buf: Mutex::new(String::new()),
        }
    }

    /// idempotent
    pub fn init(&self, location: Option<PathBuf>) {
        let path_buf = match location {
            None => std::env::current_dir()
                .expect("unable to get current working directory.")
                .join(Self::DEFAULT_LOG_NAME),
            Some(pb) => pb,
        };

        self.location.get_or_init(|| path_buf);
    }
}

impl log::Log for DelveLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() >= self.level
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S.%6f");

        let log_line = format!(
            "[{}]-(thread {})-<{}> {}\n",
            time,
            THREAD_ID.with(|t| t.clone()),
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
        let location = self.location.get().expect("logger not initialized");
        let mut file_handle = File::options()
            .create(true)
            .write(true)
            .append(true)
            .open(location)
            .expect(&format!(
                "log file at location {:?} unable to be opened",
                &self.location
            ));

        let _lock_guard = file_handle.lock().expect(&format!(
            "unable to acquire lock on log file {:?}",
            &self.location
        ));

        let mut inner_buf = self.inner_buf.lock().expect("log mutex poisoned on flush");
        let _ = file_handle.write(inner_buf.as_bytes());
        inner_buf.clear();
    }
}
