use std::collections::HashMap;

use belmarsh::{
    dependency::{
        cycle::CycleDetector,
        list::{DependencyList, DependencyListFromRepositoryError},
    },
    module::Module,
    repository::Repository,
};

use super::ValidationFailure;

#[derive(Debug)]
pub enum ValidateCircularModuleError {
    CouldNotGetDependencies(DependencyListFromRepositoryError),
}

impl From<DependencyListFromRepositoryError> for ValidateCircularModuleError {
    fn from(value: DependencyListFromRepositoryError) -> Self {
        ValidateCircularModuleError::CouldNotGetDependencies(value)
    }
}

pub fn validate_circular_modules(
    repository: Repository,
) -> Result<Vec<ValidationFailure>, ValidateCircularModuleError> {
    let dependencies: DependencyList<Module, Module> = repository.try_into()?;
    let grouped_by_from = dependencies.group_by_from();
    let string_grouped_dependencies: HashMap<String, Vec<String>> = grouped_by_from
        .into_iter()
        .map(|(from, to_list)| {
            (
                from.to_string(),
                to_list.into_iter().map(|to| to.to_string()).collect(),
            )
        })
        .collect();

    let detector = CycleDetector::new(string_grouped_dependencies);

    Ok(detector
        .find_cycles()
        .into_iter()
        .map(|chain| ValidationFailure::CircularDependency(chain))
        .collect())
}
