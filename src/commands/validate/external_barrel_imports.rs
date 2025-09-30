use belmarsh::{
    dependency::list::{DependencyList, DependencyListFromRepositoryError},
    repository::{Repository, child::RepositoryChildPath},
};

use super::ValidationFailure;

#[derive(Debug)]
pub enum ValidateExternalBarrelImportsError {
    CouldNotGetDependencies(DependencyListFromRepositoryError),
}

impl From<DependencyListFromRepositoryError> for ValidateExternalBarrelImportsError {
    fn from(value: DependencyListFromRepositoryError) -> Self {
        ValidateExternalBarrelImportsError::CouldNotGetDependencies(value)
    }
}

pub fn validate_external_barrel_imports(
    repository: Repository,
) -> Result<Vec<ValidationFailure>, ValidateExternalBarrelImportsError> {
    let dependencies: DependencyList<RepositoryChildPath, RepositoryChildPath> =
        repository.try_into()?;

    Ok(dependencies
        .as_ref()
        .iter()
        .filter(|dependency| !dependency.is_internal())
        .filter_map(|dependency| {
            if dependency.to.is_barrel() {
                None
            } else {
                Some(ValidationFailure::ExternalBarrelImport(dependency.clone()))
            }
        })
        .collect())
}
