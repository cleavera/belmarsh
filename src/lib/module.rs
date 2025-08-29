use std::path::Component;

#[derive(Debug)]
pub enum ModuleFromComponentError {
    InvalidComponent(String),
}

fn component_to_string(component: Component) -> String {
    match component {
        Component::Normal(os_str) => os_str.to_string_lossy().into_owned(),
        Component::RootDir => "/".to_string(),
        Component::CurDir => ".".to_string(),
        Component::ParentDir => "..".to_string(),
        Component::Prefix(prefix) => prefix.as_os_str().to_string_lossy().into_owned(),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Module(String);

impl<'a> TryFrom<Component<'a>> for Module {
    type Error = ModuleFromComponentError;

    fn try_from(value: Component<'a>) -> Result<Self, Self::Error> {
        match value {
            Component::Normal(os_str) => Ok(Module(os_str.to_string_lossy().into_owned())),
            other => Err(ModuleFromComponentError::InvalidComponent(
                component_to_string(other),
            )),
        }
    }
}

impl From<String> for Module {
    fn from(value: String) -> Self {
        Module(value)
    }
}
