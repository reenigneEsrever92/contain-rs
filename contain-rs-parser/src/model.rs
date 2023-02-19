use std::time::Duration;

#[derive(Debug, PartialEq, Eq)]
pub enum FieldAttribute {
    EnvVar(String),
    Arg(String),
    Port(u32),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Model {
    pub struct_name: String,
    pub image: String,
    pub command: Option<Command>,
    pub health_check: Option<HealthCheck>,
    pub wait_time: Option<WaitTime>,
    pub wait_log: Option<WaitLog>,
    pub fields: Vec<ModelField>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Command {
    pub args: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum HealthCheck {
    Command(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct WaitTime {
    pub time: Duration,
}

#[derive(Debug, PartialEq, Eq)]
pub struct WaitLog {
    pub message: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ModelField {
    pub name: String,
    pub r#type: FieldType,
    pub attributes: Vec<FieldAttribute>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FieldType {
    Simple,
    Option,
}
