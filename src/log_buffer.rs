//! In-memory ring buffer that captures `log::*` calls for on-device viewing.
//!
//! The buffer is exposed as a global static behind a `Mutex` so the custom
//! `log::Log` implementation can append entries from any call site.  The UI
//! reads the buffer via a Dioxus signal on `WorkoutState`.

use std::collections::VecDeque;
use std::sync::Mutex;

/// Maximum number of log entries retained in the ring buffer.
pub const DEFAULT_CAP: usize = 500;

/// A single captured log entry.
#[derive(Clone, Debug, PartialEq)]
pub struct LogEntry {
    /// Millisecond-precision timestamp (from `js_sys::Date::now()`).
    pub timestamp_ms: f64,
    /// Log level (Info / Warn / Error / Debug / Trace).
    pub level: log::Level,
    /// The `target` field from the log record (usually the module path).
    pub target: String,
    /// The formatted log message.
    pub message: String,
}

impl LogEntry {
    /// Format the entry as a single line suitable for clipboard export.
    pub fn format_line(&self) -> String {
        let secs = (self.timestamp_ms / 1000.0) as u64;
        let millis = (self.timestamp_ms % 1000.0) as u32;
        let h = (secs / 3600) % 24;
        let m = (secs / 60) % 60;
        let s = secs % 60;
        format!(
            "{:02}:{:02}:{:02}.{:03} [{:5}] {} — {}",
            h, m, s, millis, self.level, self.target, self.message
        )
    }
}

/// A bounded ring buffer of log entries.
#[derive(Clone, Debug)]
pub struct LogRingBuffer {
    entries: VecDeque<LogEntry>,
    cap: usize,
}

impl LogRingBuffer {
    pub fn new(cap: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(cap),
            cap,
        }
    }

    /// Append an entry, evicting the oldest if at capacity.
    pub fn push(&mut self, entry: LogEntry) {
        if self.entries.len() >= self.cap {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// Return entries newest-first.
    pub fn entries_newest_first(&self) -> Vec<LogEntry> {
        self.entries.iter().rev().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn cap(&self) -> usize {
        self.cap
    }
}

// ── Global singleton ────────────────────────────────────────────────────────

static GLOBAL_BUFFER: Mutex<Option<LogRingBuffer>> = Mutex::new(None);

/// Initialise the global buffer (idempotent — second call is a no-op).
pub fn init_global_buffer(cap: usize) {
    let mut guard = GLOBAL_BUFFER.lock().unwrap();
    if guard.is_none() {
        *guard = Some(LogRingBuffer::new(cap));
    }
}

/// Push an entry into the global buffer.  No-op if not initialised.
pub fn push_global(entry: LogEntry) {
    if let Ok(mut guard) = GLOBAL_BUFFER.lock()
        && let Some(buf) = guard.as_mut()
    {
        buf.push(entry);
    }
}

/// Snapshot the global buffer (newest-first).
pub fn snapshot_global() -> Vec<LogEntry> {
    GLOBAL_BUFFER
        .lock()
        .ok()
        .and_then(|guard| guard.as_ref().map(|b| b.entries_newest_first()))
        .unwrap_or_default()
}

/// Clear the global buffer.
pub fn clear_global() {
    if let Ok(mut guard) = GLOBAL_BUFFER.lock()
        && let Some(buf) = guard.as_mut()
    {
        buf.clear();
    }
}

/// Return the current entry count of the global buffer.
pub fn global_len() -> usize {
    GLOBAL_BUFFER
        .lock()
        .ok()
        .and_then(|guard| guard.as_ref().map(|b| b.len()))
        .unwrap_or(0)
}

// ── Custom log::Log sink (WASM only) ────────────────────────────────────────

/// A `log::Log` implementation that captures every record into the ring buffer
/// **and** forwards to the browser console via `web_sys` so nothing is lost.
#[cfg(target_arch = "wasm32")]
pub struct BufferLogger;

#[cfg(target_arch = "wasm32")]
impl log::Log for BufferLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let msg = format!("{}", record.args());

        // Always forward to browser console.
        let console_msg = format!("[{}] {} — {}", record.level(), record.target(), msg);
        match record.level() {
            log::Level::Error => {
                web_sys::console::error_1(&wasm_bindgen::JsValue::from_str(&console_msg))
            }
            log::Level::Warn => {
                web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(&console_msg))
            }
            _ => web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&console_msg)),
        }

        // Only capture Info, Warn, Error into the ring buffer (skip Debug/Trace).
        if record.level() > log::Level::Info {
            return;
        }

        let timestamp_ms = js_sys::Date::now();

        let entry = LogEntry {
            timestamp_ms,
            level: record.level(),
            target: record.target().to_string(),
            message: msg,
        };

        push_global(entry);
    }

    fn flush(&self) {}
}

/// Install the buffer logger as the primary `log` sink.
///
/// Must be called **before** `dioxus::logger::init()` so that we own the
/// `log` facade.  Dioxus uses `tracing` internally so its console output is
/// unaffected.  Our logger forwards every `log::*` call to both the ring
/// buffer and the browser console.
#[cfg(target_arch = "wasm32")]
pub fn install_buffer_logger() {
    init_global_buffer(DEFAULT_CAP);

    static LOGGER: BufferLogger = BufferLogger;
    // This will succeed because we call it before anything else sets a logger.
    let _ = log::set_logger(&LOGGER);
    log::set_max_level(log::LevelFilter::Info);
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_buffer_push_and_evict() {
        let mut buf = LogRingBuffer::new(3);
        for i in 0..5 {
            buf.push(LogEntry {
                timestamp_ms: i as f64,
                level: log::Level::Info,
                target: "test".into(),
                message: format!("msg-{}", i),
            });
        }
        assert_eq!(buf.len(), 3);
        let entries = buf.entries_newest_first();
        assert_eq!(entries[0].message, "msg-4");
        assert_eq!(entries[1].message, "msg-3");
        assert_eq!(entries[2].message, "msg-2");
    }

    #[test]
    fn ring_buffer_never_exceeds_cap() {
        let cap = 10;
        let mut buf = LogRingBuffer::new(cap);
        for i in 0..(cap + 50) {
            buf.push(LogEntry {
                timestamp_ms: i as f64,
                level: log::Level::Warn,
                target: "test".into(),
                message: format!("m{}", i),
            });
            assert!(buf.len() <= cap);
        }
        assert_eq!(buf.len(), cap);
    }

    #[test]
    fn ring_buffer_clear() {
        let mut buf = LogRingBuffer::new(10);
        buf.push(LogEntry {
            timestamp_ms: 0.0,
            level: log::Level::Info,
            target: "t".into(),
            message: "hello".into(),
        });
        assert!(!buf.is_empty());
        buf.clear();
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn log_entry_format_line() {
        let entry = LogEntry {
            timestamp_ms: 3_661_123.0, // 1h 1m 1s 123ms
            level: log::Level::Warn,
            target: "mymod".into(),
            message: "oops".into(),
        };
        let line = entry.format_line();
        assert!(line.contains("01:01:01.123"));
        assert!(line.contains("WARN"));
        assert!(line.contains("mymod"));
        assert!(line.contains("oops"));
    }

    #[test]
    fn entries_newest_first_order() {
        let mut buf = LogRingBuffer::new(5);
        for i in 0..3 {
            buf.push(LogEntry {
                timestamp_ms: i as f64,
                level: log::Level::Info,
                target: "t".into(),
                message: format!("m{}", i),
            });
        }
        let entries = buf.entries_newest_first();
        assert_eq!(entries[0].message, "m2");
        assert_eq!(entries[1].message, "m1");
        assert_eq!(entries[2].message, "m0");
    }
}
