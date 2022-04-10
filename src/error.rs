use std::{collections::HashMap, fmt::Display};

use tracing_error::SpanTrace;

pub type Result<T> = std::result::Result<T, ContainersError>;

#[derive(Debug)]
pub enum ErrorType {
    Unrecoverable,
    ContainerStateError,
    CommandError,
    LogError,
    WaitError,
    PsError,
}

#[derive(Debug)]
pub struct Context {
    source: Option<Box<dyn std::error::Error>>,
    span_trace: SpanTrace,
    info: HashMap<String, String>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            source: None,
            span_trace: SpanTrace::capture(),
            info: HashMap::new(),
        }
    }

    pub fn source<T: std::error::Error + 'static>(mut self, error: T) -> Self {
        self.source = Some(Box::new(error));
        self
    }

    pub fn span_trace(mut self, span_trace: SpanTrace) -> Self {
        self.span_trace = span_trace;
        self
    }

    pub fn info<T: Into<String>, T2: std::fmt::Debug + ?Sized>(
        mut self,
        key: T,
        value: &T2,
    ) -> Self {
        self.info.insert(key.into(), format!("{:?}", value));
        self
    }

    pub fn into_error(self, typ: ErrorType) -> ContainersError {
        ContainersError::from_type_and_context(typ, self)
    }
}

#[derive(Debug)]
pub struct ContainersError {
    pub typ: ErrorType,
    pub context: Context,
}

impl ContainersError {
    fn from_type(typ: ErrorType) -> Self {
        Self::from_type_and_context(typ, Context::new())
    }

    fn from_type_and_context(typ: ErrorType, context: Context) -> Self {
        Self { typ, context }
    }
}

impl<T: std::error::Error + 'static> From<T> for Context {
    fn from(e: T) -> Self {
        Self::new().source(e).span_trace(SpanTrace::capture())
    }
}

impl Display for ContainersError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.typ, self.context,)
    }
}

impl Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorType::Unrecoverable => write!(f, "Unrecoverable Error"),
            ErrorType::ContainerStateError=> write!(f, "Container Is Not Running"),
            ErrorType::CommandError => write!(f, "Command Error"),
            ErrorType::LogError => write!(f, "Log Error"),
            ErrorType::WaitError => write!(f, "Wait Error"),
            ErrorType::PsError => write!(f, "Ps Error"),
            _ => todo!(),
        }
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.source {
            Some(source) => write!(
                f,
                "Span trace: \n{}\nSource error: \n{}\n",
                self.span_trace, source
            ),
            None => write!(f, "Span trace: \n{}\n", self.span_trace),
        }
    }
}

impl std::error::Error for ContainersError {}
