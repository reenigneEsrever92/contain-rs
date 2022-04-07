pub mod postgres;

#[derive(Clone)]
pub struct Image {
    pub name: String,
    pub tag: String,
}

impl Image {
    pub fn from_name(name: &str) -> Self {
        Self::from_name_and_tag(name, "latest")
    }

    pub fn from_name_and_tag(name: &str, tag: &str) -> Self {
        Image {
            name: name.to_string(),
            tag: tag.to_string(),
        }
    }
}

impl From<Image> for String {
    fn from(i: Image) -> Self {
        format!("{}:{}", i.name, i.tag)
    }
}

impl From<&Image> for String {
    fn from(i: &Image) -> Self {
        format!("{}:{}", i.name, i.tag)
    }
}
