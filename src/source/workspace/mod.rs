use crate::{
    manifest::target::{Target, TargetKind},
    names::PackageNames,
    source::{
        fs::PackageFS, specifiers::DependencySpecifiers, traits::PackageSource,
        version_id::VersionId, workspace::pkg_ref::WorkspacePackageRef, ResolveResult,
    },
    Project, DEFAULT_INDEX_NAME,
};
use relative_path::RelativePathBuf;
use reqwest::blocking::Client;
use std::collections::BTreeMap;

/// The workspace package reference
pub mod pkg_ref;
/// The workspace dependency specifier
pub mod specifier;

/// The workspace package source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorkspacePackageSource;

impl PackageSource for WorkspacePackageSource {
    type Specifier = specifier::WorkspaceDependencySpecifier;
    type Ref = WorkspacePackageRef;
    type RefreshError = errors::RefreshError;
    type ResolveError = errors::ResolveError;
    type DownloadError = errors::DownloadError;

    fn refresh(&self, _project: &Project) -> Result<(), Self::RefreshError> {
        // no-op
        Ok(())
    }

    fn resolve(
        &self,
        specifier: &Self::Specifier,
        project: &Project,
        _package_target: TargetKind,
    ) -> Result<ResolveResult<Self::Ref>, Self::ResolveError> {
        let (path, manifest) = 'finder: {
            let workspace_dir = project
                .workspace_dir
                .as_ref()
                .unwrap_or(&project.package_dir);

            for (path, manifest) in project.workspace_members(workspace_dir)? {
                if manifest.name == specifier.name {
                    break 'finder (path, manifest);
                }
            }

            return Err(errors::ResolveError::NoWorkspaceMember(
                specifier.name.to_string(),
            ));
        };

        Ok((
            PackageNames::Pesde(manifest.name.clone()),
            BTreeMap::from([(
                VersionId::new(manifest.version.clone(), manifest.target.kind()),
                WorkspacePackageRef {
                    // workspace_dir is guaranteed to be Some by the workspace_members method
                    // strip_prefix is guaranteed to be Some by same method
                    // from_path is guaranteed to be Ok because we just stripped the absolute path
                    path: RelativePathBuf::from_path(
                        path.strip_prefix(project.workspace_dir.clone().unwrap())
                            .unwrap(),
                    )
                    .unwrap(),
                    dependencies: manifest
                        .all_dependencies()?
                        .into_iter()
                        .map(|(alias, (mut spec, ty))| {
                            match &mut spec {
                                DependencySpecifiers::Pesde(spec) => {
                                    let index_name =
                                        spec.index.as_deref().unwrap_or(DEFAULT_INDEX_NAME);

                                    spec.index = Some(
                                        manifest
                                            .indices
                                            .get(index_name)
                                            .ok_or(errors::ResolveError::IndexNotFound(
                                                index_name.to_string(),
                                                manifest.name.to_string(),
                                            ))?
                                            .to_string(),
                                    )
                                }
                                #[cfg(feature = "wally-compat")]
                                DependencySpecifiers::Wally(spec) => {
                                    let index_name =
                                        spec.index.as_deref().unwrap_or(DEFAULT_INDEX_NAME);

                                    spec.index = Some(
                                        manifest
                                            .wally_indices
                                            .get(index_name)
                                            .ok_or(errors::ResolveError::IndexNotFound(
                                                index_name.to_string(),
                                                manifest.name.to_string(),
                                            ))?
                                            .to_string(),
                                    )
                                }
                                DependencySpecifiers::Git(_) => {}
                                DependencySpecifiers::Workspace(_) => {}
                            }

                            Ok((alias, (spec, ty)))
                        })
                        .collect::<Result<_, errors::ResolveError>>()?,
                    target: manifest.target,
                },
            )]),
        ))
    }

    fn download(
        &self,
        pkg_ref: &Self::Ref,
        project: &Project,
        _reqwest: &Client,
    ) -> Result<(PackageFS, Target), Self::DownloadError> {
        let path = pkg_ref.path.to_path(project.workspace_dir.clone().unwrap());

        Ok((
            PackageFS::Copy(path, pkg_ref.target.kind()),
            pkg_ref.target.clone(),
        ))
    }
}

/// Errors that can occur when using a workspace package source
pub mod errors {
    use thiserror::Error;

    /// Errors that can occur when refreshing the workspace package source
    #[derive(Debug, Error)]
    #[non_exhaustive]
    pub enum RefreshError {}

    /// Errors that can occur when resolving a workspace package
    #[derive(Debug, Error)]
    #[non_exhaustive]
    pub enum ResolveError {
        /// An error occurred reading the workspace members
        #[error("failed to read workspace members")]
        ReadWorkspaceMembers(#[from] crate::errors::WorkspaceMembersError),

        /// No workspace member was found with the given name
        #[error("no workspace member found with name {0}")]
        NoWorkspaceMember(String),

        /// An error occurred getting all dependencies
        #[error("failed to get all dependencies")]
        AllDependencies(#[from] crate::manifest::errors::AllDependenciesError),

        /// An index of a member package was not found
        #[error("index {0} not found in member {1}")]
        IndexNotFound(String, String),
    }

    /// Errors that can occur when downloading a workspace package
    #[derive(Debug, Error)]
    #[non_exhaustive]
    pub enum DownloadError {
        /// An error occurred reading the workspace members
        #[error("failed to read workspace members")]
        ReadWorkspaceMembers(#[from] std::io::Error),
    }
}