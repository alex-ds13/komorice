#[derive(Debug, Clone)]
pub struct AppError {
    pub title: String,
    pub description: Option<String>,
    pub kind: AppErrorKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppErrorKind {
    Info,
    Warning,
    Error,
}
