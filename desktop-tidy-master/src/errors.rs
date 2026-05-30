use crate::model::DesktopCommand;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error from the UI backend.
    #[error("UI error: {0}")]
    Ui(#[from] winio::Error),
    /// An layouting error
    #[error("Layout error: {0}")]
    Layout(#[from] winio::layout::LayoutError<winio::Error>),

    /// desktop icon error
    #[error("Backend error: {0}")]
    Backend(#[from] desktop_icon::error::AppError),

    /// Channel error
    #[error("Channel error: {0}")]
    Channel(#[from] crossfire::SendError<DesktopCommand>),
}
