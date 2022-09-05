#[derive(Debug, PartialEq, Eq)]
pub enum FieldAttribute {
    EnvVar(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Model {
    pub struct_name: String,
    pub image: String,
    pub health_check_command: Option<String>,
    pub ports: Vec<Port>,
    pub fields: Vec<ModelField>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ModelField {
    pub name: String,
    pub attributes: Vec<FieldAttribute>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Port {
    pub source: u16,
    pub target: u16,
}
