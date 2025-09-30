use std::collections::HashSet;

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

#[derive(Debug, Clone)]
pub struct ModuleMappings(HashSet<ModuleMapping>);

#[derive(Debug)]
pub enum ModuleMappingsFromParamStringsError {
    InvalidParam(ModuleMappingFromParamStringError),
}

impl From<ModuleMappingFromParamStringError> for ModuleMappingsFromParamStringsError {
    fn from(value: ModuleMappingFromParamStringError) -> Self {
        ModuleMappingsFromParamStringsError::InvalidParam(value)
    }
}

impl ModuleMappings {
    pub fn from_param_strings(
        param_strings: Vec<String>,
    ) -> Result<Self, ModuleMappingsFromParamStringsError> {
        Ok(ModuleMappings(
            param_strings
                .iter()
                .map(|param_string| Ok(ModuleMapping::from_param_string(param_string)?))
                .collect::<Result<HashSet<ModuleMapping>, ModuleMappingsFromParamStringsError>>()?,
        ))
    }

    pub fn replace_import_aliases(&self, line: &str) -> String {
        self.0.iter().fold(line.to_string(), |acc, mapping| {
            mapping.replace_import_alias(&acc)
        })
    }
}

impl From<HashSet<ModuleMapping>> for ModuleMappings {
    fn from(value: HashSet<ModuleMapping>) -> Self {
        ModuleMappings(value)
    }
}
