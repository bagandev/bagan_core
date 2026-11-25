//! Colored console logging for Node.js.
//!
//! Use [`log`] with a [`LogLevel`] to write a message to the console.

use napi_derive::napi;

const RESET: &str = "\x1b[0m";
const BLUE: &str = "\x1b[34m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const BOLD: &str = "\x1b[1m";

#[napi(string_enum = "lowercase", namespace = "logger")]
/// Severity level for a console log message.
pub enum LogLevel {
    /// Informational message, written to standard output.
    Info,
    /// Warning message, written to standard output.
    Warning,
    /// Error message, written to standard error.
    Error,
}

#[napi(namespace = "logger")]
/// Writes a colored message to the console at the specified level.
///
/// Informational and warning messages go to standard output; error messages
/// go to standard error.
pub fn log(level: LogLevel, message: String) {
    match level {
        LogLevel::Info => {
            println!("{}{}[INFO]{} {}", BOLD, BLUE, RESET, message);
        }
        LogLevel::Warning => {
            println!("{}{}[WARNING]{} {}", BOLD, YELLOW, RESET, message);
        }
        LogLevel::Error => {
            eprintln!("{}{}[ERROR]{} {}", BOLD, RED, RESET, message);
        }
    }
}
