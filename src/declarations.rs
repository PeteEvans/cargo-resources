use cargo_metadata::camino::Utf8PathBuf;
use crate::resource_encoding::ResourceEncoding;

/// The structure matching the resource declaration (provides) in the package metadata.
#[derive(serde::Deserialize, Debug)]
pub struct ResourceDataDeclaration {
    /// Whether resource's file encoding is text or binary
    pub encoding: Option<ResourceEncoding>,

    /// The path of the resource within the crate
    pub crate_path: Utf8PathBuf,

    /// The path of the resource as a resource
    pub output_path: Option<Utf8PathBuf>,

    /// The unique name for the resource
    pub resource_name: Option<String>
}

/// The structure matching the resource usage declaration in the consuming package metadata.
#[derive(serde::Deserialize, Debug)]
pub struct ResourceConsumerDeclaration {
    /// The relative path of the resource root from the crate root
    pub resource_root: Option<Utf8PathBuf>,

    /// The list of required resources
    pub requires: Option<Vec<ResourceRequirementDeclaration>>
}

/// The structure matching the resource requirement in the consuming package.
#[derive(serde::Deserialize, Debug)]
pub struct ResourceRequirementDeclaration {
    /// The unique name of the required resource
    pub resource_name: String
}
