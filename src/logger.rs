/// -----------------------------------------------------------------
/// This file contains the implementation of a simple logger in Rust.
/// It uses the `log` crate to provide logging macros and the `chrono` crate to timestamp logs.
///
/// The `log_info` and `log_debug` macros are defined to log information and debug messages respectively.
/// These macros include the file name, module path, and line number in the log message.
///
/// The `SimpleLogger` struct is defined and implements the `Log` trait from the `log` crate.
/// The `enabled` method checks if the log level is less than or equal to `Debug`.
/// The `log` method prints the log message to the console if logging is enabled.
/// The log message includes a timestamp, the log level, the file name, module path, line number, and the log message itself.
///
/// The `LOGGER` static variable is an instance of `SimpleLogger` that can be used throughout the application.
///
/// My goal was to enable logging that's as informative as the logging I typically use in python:
///     lglvl: str = os.environ.get( 'LOGLEVEL', 'DEBUG' )
///     lglvldct = {
///         'DEBUG': logging.DEBUG,
///         'INFO': logging.INFO }
///     logging.basicConfig(
///         level=lglvldct[lglvl],  # assigns the level-object to the level-key loaded from the envar
///         format='[%(asctime)s] %(levelname)s [%(module)s-%(funcName)s()::%(lineno)d] %(message)s',
///         datefmt='%d/%b/%Y %H:%M:%S' )
///     log = logging.getLogger( __name__ )
///     log.debug( 'logging working' )
///
/// I haven't yet been able to figure out how to log the function name.
/// -----------------------------------------------------------------
use chrono::Local;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
use std::env;

pub static LOGGER: SimpleLogger = SimpleLogger;

/*
The macros below allow me to use logging statements like:
`log_debug!("some message");` and `log_info!("some message");`

$: The dollar sign is used to denote the beginning of a macro variable or a repetition sequence.

(...): Parentheses group the pattern that matches the macro input. In the case of your logging macros, it groups the entire variable argument list.

$arg:tt: This part is defining a macro variable:

$arg is the name of the variable that will hold the input passed to the macro.
:tt specifies the fragment specifier. tt stands for "token tree," a flexible specifier that can capture a wide variety of syntactic constructs, including literals, identifiers, types, expressions, etc. It's used here to allow the macro to accept any Rust expression as input.
*: This is a repetition operator, indicating that the pattern preceding it can repeat zero or more times. This allows the macro to accept a variable number of arguments.


*/

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        log::debug!("{}", format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        log::info!("{}", format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        log::warn!("{}", format_args!($($arg)*))
    };
}

pub struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        /* if the log-level is set to `INFO`, then the `log_debug!()` macro's message will not be printed. */
        if self.enabled(record.metadata()) {
            println!(
                "[{}] {} [{}::{}] {}",
                Local::now().format("%d/%b/%Y %H:%M:%S"),
                record.level(),
                record.file().unwrap_or("<unknown>"),
                // record.module_path().unwrap_or("<unknown>"),  // can't determine the fuction-name
                record.line().unwrap_or(0), // the default value of 0 implies that the line-number is not available
                record.args()
            );
        }
    }

    fn flush(&self) {} // my output to the console doesn't require this, but it's required by the `Log` trait
}

pub fn init_logger() -> Result<(), SetLoggerError> {
    // let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "debug".to_string());
    let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "warn".to_string());
    let level_filter = match log_level.to_lowercase().as_str() {
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        _ => LevelFilter::Warn,
    };

    /*
       In the line below, the part between the vertical pipes is the cloaure `arcument`.
       The part `log::set_max_level(level_filter)` is the closure `body`.
    */
    log::set_logger(&LOGGER).map(|()| log::set_max_level(level_filter))
}
