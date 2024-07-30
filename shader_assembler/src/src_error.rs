#[derive(Clone, Debug)]
pub struct SourceError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}