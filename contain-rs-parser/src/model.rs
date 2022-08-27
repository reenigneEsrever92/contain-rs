#[derive(Debug, PartialEq, Eq)]
pub enum FieldAttribute {
    EnvVar(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Model {
    pub struct_name: String,
    pub image: String,
    pub health_check_command: Option<String>,
    pub fields: Vec<ModelField>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ModelField {
    pub name: String,
    pub attributes: Vec<FieldAttribute>,
}
