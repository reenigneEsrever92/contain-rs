#[derive(Debug)]
pub struct Error {
    pub message: String
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self {
            message: e.to_string(),
        } // TODO how to convert io error to RunError!?
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        match e.classify() {
            serde_json::error::Category::Io => Self { message: "Io error while serializing".to_string() },
            serde_json::error::Category::Syntax => Self { message: "Syntax error in json".to_string() },
            serde_json::error::Category::Data => Self { message: "Json contains unexpected data".to_string() },
            serde_json::error::Category::Eof => Self { message: "Incomplete json data".to_string() },
        }
    }
}
