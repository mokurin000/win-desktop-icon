use std::string::FromUtf16Error;

use thiserror::Error;
use windows::core::Error as WinError;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Windows(#[from] WinError),
    #[error("desktop folder view is not available")]
    DesktopViewUnavailable,
    #[error("shell window did not expose a dispatch object")]
    MissingDispatch,
    #[error("utf16 error: {0}")]
    UTF16Error(#[from] FromUtf16Error),
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Bitcode Error: {0}")]
    BitError(#[from] bitcode::Error),
}
