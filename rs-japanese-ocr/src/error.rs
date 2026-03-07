use thiserror::Error;

#[derive(Error, Debug)]
pub enum JapaneseOCRError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Candle error: {0}")]
    Candle(#[from] candle_core::Error),

    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),

    #[error("Tokenizer error: {0}")]
    Tokenizer(String),

    #[error("Model error: {0}")]
    Model(String),

    #[error("Model download error: {0}")]
    Download(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Configuration error: {0}")]
    Config(String),
}
