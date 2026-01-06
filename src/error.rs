use thiserror::*;

#[derive(Debug, Clone, Copy)]
pub enum LoggingErrorKind {
    SetLoggerError,
    ThreadError,
}

#[derive(Debug, Error, Clone)]
pub enum Error {
    #[error("{0}")]
    Error(&'static str),

    #[error("{0}")]
    OwnedError(String),

    #[error("Terminal error: {0}")]
    TerminalError(String),

    #[error("IO error: {0}")]
    IOError(String),

    #[error("Invalid response from command: {0}")]
    JSONError(String),

    #[error("Logging Error ({0:?}): {1}")]
    LoggingError(LoggingErrorKind, String),
}

impl Default for Error {
    fn default() -> Self {
        Self::Error("an error occurred")
    }
}

impl Into<std::io::Error> for Error {
    fn into(self) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::Other, self.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(jserr: serde_json::Error) -> Self {
        Self::JSONError(format!("{jserr}"))
    }
}

impl From<tui_logger::TuiLoggerError> for Error {
    fn from(lerr: tui_logger::TuiLoggerError) -> Self {
        match lerr {
            tui_logger::TuiLoggerError::SetLoggerError(err) =>
                Error::LoggingError(LoggingErrorKind::SetLoggerError, format!("{err}")),
            tui_logger::TuiLoggerError::ThreadError(err) =>
                Error::LoggingError(LoggingErrorKind::ThreadError, format!("{err}")),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(ioerr: std::io::Error) -> Self {
        Self::IOError(format!("{ioerr}"))
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Self::OwnedError(s)
    }
}

impl From<&'static str> for Error {
    fn from(s: &'static str) -> Self {
        Self::Error(s)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
