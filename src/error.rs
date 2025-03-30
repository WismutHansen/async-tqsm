use thiserror::Error;

#[derive(Error, Debug)]
pub enum SegmenterError {
    #[error("Language '{0}' not supported by underlying libtqsm")]
    UnsupportedLanguage(String),

    #[error("Failed to load language data for '{0}': {1}")]
    LanguageLoadError(String, anyhow::Error), // Or more specific error type

    #[error("Buffer overflow: Maximum buffer size of {0} characters exceeded")]
    BufferOverflow(usize),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("UTF-8 decoding error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error("Stream processing error: {0}")]
    StreamError(String), // Generic stream error

    #[error("Underlying segmentation error: {0}")]
    SegmentationError(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, SegmenterError>;
