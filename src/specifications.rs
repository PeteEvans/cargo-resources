use crate::resource_encoding::ResourceEncoding;
use crate::{ResourceName, ResourceSha};
use cargo_metadata::camino::Utf8PathBuf;
use cargo_metadata::semver::Version;
use cargo_metadata::Package;

/// The fully populated resource specification (derived from a crate's resource declaration).
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ResourceSpecification {
    /// The crate identifier
    pub declaring_crate_name: String,
    /// The crate version
    pub declaring_crate_version: Version,

    /// Whether resource's file encoding is text or binary
    pub encoding: ResourceEncoding,

    /// The full path of the resource within the crate.
    pub full_crate_path: Utf8PathBuf,

    /// The path of the resource as a resource
    pub output_path: Utf8PathBuf,

    /// The unique name for the resource
    pub resource_name: String,
}

/// The fully populated specification of the consuming package.
#[derive(serde::Deserialize, Debug)]
pub struct ResourceConsumerSpecification {
    /// The relative path of the resource root from the crate root
    pub resource_root: Utf8PathBuf,

    /// The required resources
    pub required_resources: Vec<ResourceRequirement>
}

/// The fully populated specification for a resource usage.
#[derive(serde::Deserialize, Debug)]
pub struct ResourceRequirement {
    /// The unique name of the required resource
    pub resource_name: ResourceName,

    /// The optional hex-encoded SHA256 value of the required resource
    pub required_sha: Option<ResourceSha>
}

/// Derived Package Details
#[derive(Debug)]
pub (crate) struct PackageDetails<'m> {
    /// True if is a dependency of the package (from the root package)
    is_dependency: bool,

    /// The package details from metadata
    pub (crate) package: &'m Package
}

impl<'m> PackageDetails<'m> {
    /// Create an instance initially assuming not a dependency.
    pub (crate) fn new(package: &'m Package) -> Self {
        Self {
            is_dependency: false,
            package,
        }
    }

    /// Mark package as a dependency (of root node).
    pub (crate) fn set_is_dependency(&mut self) {
        self.is_dependency = true;
    }

    /// True if the package is a dependency (of the root node, inclusively)
    pub (crate) fn is_dependency(&self) -> bool {
        self.is_dependency
    }

}
