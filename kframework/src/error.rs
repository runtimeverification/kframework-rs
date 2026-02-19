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

/// This impl exists so we can change function signatures
/// from returning Result<_, String> to Result<_, KError>
/// without having things break everywhere
impl From<KError> for String {
    fn from(value: KError) -> Self {
        match value {
            KError::KoreParseError(msg) => msg,
            KError::KoreLexerError(msg) => msg,
        }
    }
}
