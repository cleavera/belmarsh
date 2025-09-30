use belmarsh::{
    dependency::list::{DependencyList, DependencyListFromRepositoryError},
    repository::{Repository, child::RepositoryChildPath},
};

use super::ValidationFailure;

#[derive(Debug)]
pub enum ValidateBarrelImportsBarrelError {
    CouldNotGetDependencies(DependencyListFromRepositoryError),
}

impl From<DependencyListFromRepositoryError> for ValidateBarrelImportsBarrelError {
    fn from(value: DependencyListFromRepositoryError) -> Self {
        ValidateBarrelImportsBarrelError::CouldNotGetDependencies(value)
    }
}

pub fn validate_barrel_imports_barrel(
    repository: Repository,
) -> Result<Vec<ValidationFailure>, ValidateBarrelImportsBarrelError> {
    let dependencies: DependencyList<RepositoryChildPath, RepositoryChildPath> =
        repository.try_into()?;

    Ok(dependencies
        .as_ref()
        .iter()
        .filter_map(|dependency| {
            if dependency.from.is_barrel() && dependency.to.is_barrel() {
                Some(ValidationFailure::BarrelImportsBarrel(dependency.clone()))
            } else {
                None
            }
        })
        .collect())
}
