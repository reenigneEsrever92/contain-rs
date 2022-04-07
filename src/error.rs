#[derive(Debug)]
pub enum Error {
    RunError { message: String },
    LogError { message: String },
    WaitError { message: String }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::RunError {
            message: e.to_string(),
        } // TODO how to convert io error to RunError!?
    }
}
