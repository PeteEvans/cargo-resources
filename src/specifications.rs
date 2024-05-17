use cargo_metadata::semver::Version;
use cargo_metadata::camino::Utf8PathBuf;
use crate::resource_encoding::ResourceEncoding;
use crate::{ResourceName, ResourceSha};

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
