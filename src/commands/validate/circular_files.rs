use std::collections::HashMap;

use belmarsh::{
    dependency::{
        cycle::CycleDetector,
        list::{DependencyList, DependencyListFromRepositoryError},
    },
    repository::{Repository, child::RepositoryChildPath},
};

use super::ValidationFailure;

#[derive(Debug)]
pub enum ValidateCircularFilesError {
    CouldNotGetDependencies(DependencyListFromRepositoryError),
}

impl From<DependencyListFromRepositoryError> for ValidateCircularFilesError {
    fn from(value: DependencyListFromRepositoryError) -> Self {
        ValidateCircularFilesError::CouldNotGetDependencies(value)
    }
}

pub fn validate_circular_files(
    repository: Repository,
) -> Result<Vec<ValidationFailure>, ValidateCircularFilesError> {
    let dependencies: DependencyList<RepositoryChildPath, RepositoryChildPath> =
        repository.try_into()?;
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
