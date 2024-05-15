//! Cargo Resources provides a cargo command line tool and library (i.e. this module), to help
//! declare and collate resources within Cargo Crates.
//!
//! Usage:
//! ```
//! use std::env::current_dir;
//! use cargo_metadata::camino::Utf8PathBuf;
//! use cargo_resources::collate_resources;
//! use std::error::Error;
//!
//! let cwd = current_dir().unwrap();
//! let manifest_file = Utf8PathBuf::from_path_buf(cwd).unwrap().join("Cargo.toml");
//!
//! // Collate resources from the crate's dependencies.
//! let _r = collate_resources(&manifest_file);
//! ```
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use serde_json::Value;
use cargo_metadata::camino::Utf8PathBuf;
use cargo_metadata::{CargoOpt, Metadata, Package};
use ring::digest::{Context, Digest, SHA256};

mod resource_encoding;

pub use resource_encoding::ResourceEncoding;

mod declarations;

pub use declarations::ResourceDataDeclaration;

mod specifications;

pub use specifications::ResourceSpecification;
use crate::declarations::ResourceConsumerDeclaration;
use crate::specifications::{ResourceConsumerSpecification, ResourceRequirement};

/// Collate the resources for the given crate, into the crate.
///
/// # Arguments
/// * source_manifest: The path of the cargo manifest (Cargo.toml) of the crate.
///
/// # Returns
/// Nothing on success, or a string error describing the failure.
pub fn collate_resources(source_manifest: &Utf8PathBuf) -> Result<(), String> {
    if !source_manifest.exists() {
        Err(format!("Source manifest does not exist: {}", source_manifest))?
    }
    // Now lets get the metadata of a package
    let mut metadata_cmd = cargo_metadata::MetadataCommand::new();
    let metadata: Metadata = metadata_cmd
        .manifest_path(&source_manifest)
        .features(CargoOpt::AllFeatures)
        .exec()
        .unwrap();

    // Find all the declared resources!

    // Find the packages recursively
    let all_packages: &Vec<Package> = &metadata.packages;
    let mut declared_resources: HashMap<String, ResourceSpecification> = HashMap::new();
    for package in all_packages {
        get_package_resource_data(package, &mut declared_resources)?
    }

    // Find the resource requirement (for the consuming crate)
    let root_package = metadata.root_package().expect("Unexpected error finding the consuming crate");
    let required_resources_spec = get_resource_requirement(&root_package, &declared_resources)?;

    // Where do we put the resources?

    let resource_root = required_resources_spec.resource_root;

    if required_resources_spec.required_resources.len() <= 0 {
        println!("No resources were found - finishing early.");
        return Ok(())
    }

    let mut resolved_resources = vec!();
    for req in required_resources_spec.required_resources {
        let res = declared_resources.get(&req.resource_name).ok_or(
            format!("No resource found matching requirement {}", req.resource_name)
        )?;
        copy_resource(&res, &resource_root)?;
        resolved_resources.push(res);
    }

    // Write a record of the resources

    let res = serde_json::to_string(&resolved_resources)
        .expect("Unable to serialize the set of resolved resources");

    let record_file_path = resource_root.join("resolved_resources.json");
    fs::write(record_file_path, res).map_err(|e| format!("Failed writing record file:{:?}", e))?;

    Ok(())
}

/// Get all the resources information declared by a package
fn get_package_resource_data(
    package: &Package,
    resources: &mut HashMap<String, ResourceSpecification>,
) -> Result<(), String> {
    // We have the metadata, resources uses cargo_resources.provides as a collection within this!
    let cargo_resource_metadata: &Value = &package.metadata["cargo_resources"];
    if !cargo_resource_metadata.is_object() {
        return Ok(()); // No metadata for us
    }
    let provides_metadata = &cargo_resource_metadata["provides"];
    match provides_metadata {
        Value::Array(resource_entries) => {
            for resource_entry in resource_entries {
                let declaration_result = serde_json::from_value::<ResourceDataDeclaration>(resource_entry.clone());
                match declaration_result {
                    Ok(declaration) => {
                        // Do the conversions for optionals
                        let resolved_output_path = declaration.output_path.unwrap_or(declaration.crate_path.to_owned());
                        let resolved_name = declaration.resource_name.unwrap_or(
                            declaration.crate_path.file_name()
                                .expect("Illegal resource name").to_string().into()
                        );

                        let full_source_path = package
                            .manifest_path.parent().expect("No manifest directory!")
                            .join(declaration.crate_path);
                        let data = ResourceSpecification {
                            declaring_crate_name: package.name.to_owned(),
                            declaring_crate_version: package.version.to_owned(),
                            encoding: declaration.encoding.unwrap_or(ResourceEncoding::Txt),
                            full_crate_path: full_source_path,
                            output_path: resolved_output_path,
                            resource_name: resolved_name.to_owned(),
                        };

                        // Later resources will overwrite old ones!
                        resources.insert(resolved_name.to_owned(), data);
                    }

                    Err(err) => {
                        return Err(format!("Malformed resource declaration in {}: {}",
                                           package.name,
                                           err));
                    }
                }
            }
            Ok(())
        }
        Value::Null => Ok(()),
        _ => {
            Err(
                "unexpected type for [package.metadata.cargo_resources].provides in the json-metadata".to_owned()
            )
        }
    }
}

/// Get the resource requirement for a package
fn get_resource_requirement(
    package: &Package,
    available_resources: &HashMap<String, ResourceSpecification>,
) -> Result<ResourceConsumerSpecification, String> {
    // We have the metadata, requirements are declared in  cargo_resources.
    let cargo_resource_metadata: &Value = &package.metadata["cargo_resources"];

    // When nothing is specified use default options and packages
    let consumer_declaration = match &cargo_resource_metadata {
        Value::Null => ResourceConsumerDeclaration {
            resource_root: None,
            requires: None,
        },
        Value::Object(_) => {
            serde_json::from_value(cargo_resource_metadata.clone())
                .map_err(|e| format!("Unable to read comsuming crates [package.metadata.cargo_resources]: {}", e.to_string()))?
        }
        _ => panic!("Misconfigured [package.metadata.cargo_resources] in consuming package.")
    };

    let resource_root = consumer_declaration.resource_root.unwrap_or(Utf8PathBuf::from("target/resources"));

    let required_resources: Vec<ResourceRequirement> = match consumer_declaration.requires {
        None => { // Default is to use all available resources with default options
            available_resources.values().map(|res_spec| ResourceRequirement {
                resource_name: res_spec.resource_name.to_owned(),
            }).collect()
        }
        Some(declarations) => { // Just convert each declaration to a spec
            declarations.into_iter().map(|dec| ResourceRequirement {
                resource_name: dec.resource_name.to_owned(),
            }).collect()
        }
    };

    Ok(ResourceConsumerSpecification { resource_root, required_resources })
}

/// Copy the resource to the resources folder (if it doesn't already exist)
fn copy_resource(
    resource: &ResourceSpecification,
    resource_root: &Utf8PathBuf,
) -> Result<(), String> {
    let output_resources_path = resource_root.join(&resource.output_path);

    let output_directory = output_resources_path.parent().unwrap();
    if !output_directory.exists() {
        fs::create_dir_all(&output_directory)
            .map_err(|e|
                format!("Unable to create output directory {}: {}", &output_directory, e)
            )?
    }

    // Use sha256 (which is overkill) to check if the file has changed
    let new_sha = hex::encode(get_file_sha(&resource.full_crate_path)?.as_ref());
    let mut already_exists = false;
    if output_resources_path.exists() {
        let existing_sha = hex::encode(get_file_sha(&output_resources_path)?.as_ref());
        if existing_sha == new_sha {
            already_exists = true;
        }
    }

    if !already_exists {
        fs::copy(&resource.full_crate_path, &output_resources_path)
            .map_err(|e|
                format!("Unable to copy resource {} to {}: {}",
                        &resource.full_crate_path,
                        &output_resources_path,
                        e
                )
            )?;
    }

    println!(
        "Resource {} {:50} {}",
        match already_exists {
            true => "existed:",
            false => " copied:"
        }.to_string(),
        &output_resources_path,
        &new_sha,
    );
    Ok(())
}

/// Work out the SHA 256 value of a file from the path
fn get_file_sha(path: &Utf8PathBuf) -> Result<Digest, String> {
    let mut sha = Context::new(&SHA256);
    let mut file = File::open(path).map_err(|e| format!("Error opening {}, {}", path, e))?;
    let mut buffer = [0; 4096]; // Read sensible sized blocks from disk!

    loop {
        let bytes_read = file.read(&mut buffer)
            .map_err(|e| format!("Error calculating SHA256 {}", e))?;
        if bytes_read == 0 {
            break;
        }
        sha.update(&buffer[..bytes_read]);
    }

    Ok(sha.finish())
}