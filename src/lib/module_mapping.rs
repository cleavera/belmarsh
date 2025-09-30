#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ModuleMapping {
    from: String,
    to: String,
}

#[derive(Debug)]
pub enum ModuleMappingFromParamStringError {
    InvalidFormat(String),
}

impl ModuleMapping {
    pub fn from_param_string(
        param_string: &str,
    ) -> Result<Self, ModuleMappingFromParamStringError> {
        let parts: Vec<&str> = param_string.splitn(2, ':').collect();
        if parts.len() == 2 {
            Ok(ModuleMapping {
                from: parts[0].to_string(),
                to: parts[1].to_string(),
            })
        } else {
            Err(ModuleMappingFromParamStringError::InvalidFormat(
                param_string.to_string(),
            ))
        }
    }

    pub fn replace_import_alias(&self, line: &str) -> String {
        if line.trim_start().starts_with("import") && line.contains(&self.from) {
            line.replace(&self.from, &self.to)
        } else {
            line.to_string()
        }
    }
}
