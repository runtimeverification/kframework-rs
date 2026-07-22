#[derive(Debug)]
pub enum KError {
    KoreLexerError(String),
    KoreParseError(String),
}

impl std::fmt::Display for KError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KError::KoreParseError(msg) => write!(f, "{msg}"),
            KError::KoreLexerError(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for KError {}
