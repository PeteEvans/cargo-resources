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
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::io::Read;

use cargo_metadata::{CargoOpt, Metadata, Node, Package, PackageId, Resolve};
use cargo_metadata::camino::{Utf8Path, Utf8PathBuf};
use ring::digest::{Context, Digest, SHA256};
use serde_json::Value;

pub use declarations::ResourceDataDeclaration;
pub use resource_encoding::ResourceEncoding;
pub use specifications::ResourceSpecification;

use crate::declarations::ResourceConsumerDeclaration;
use crate::specifications::{PackageDetails, ResourceConsumerSpecification, ResourceRequirement};

mod resource_encoding;

mod declarations;

mod specifications;

/// The Resource Name
pub type ResourceName = String;

/// The Resource's SHA 256 Value
pub type ResourceSha = String;

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

    // Check the root package (may not be set for a workspace)
    let root_package = metadata.root_package()
        .expect("Unexpected error finding the consuming crate - please run in a crate not a workspace.");

    // Create a lookup of packages including whether they are in the root package's dependency tree.
    let packages_by_id = get_package_details(&metadata)?;

    // Filter out packages that aren't in the dependency tree.
    let child_packages = packages_by_id.iter()
        .filter(|(_id, details)| details.is_dependency())
        .map(|(_id, details)| &details.package)
        .collect::<Vec<_>>();

    // Find the declared resources in the dependency tree
    let mut declared_resources: HashMap<String, ResourceSpecification> = HashMap::new();
    for package in child_packages {
        get_package_resource_data(package, &mut declared_resources)?
    }

    // Find the resource requirement (for the consuming crate)
    let required_resources_spec = get_resource_requirement(&root_package, &declared_resources)?;

    // Where do we put the resources?
    let resource_root = required_resources_spec.resource_root;
    create_output_directory(&resource_root)?;

    if required_resources_spec.required_resources.len() <= 0 {
        println!("No resources were found - finishing early.");
        return Ok(());
    }

    let mut resolved_resources = vec!();
    for res_req in required_resources_spec.required_resources {
        let res_dec = declared_resources.get(&res_req.resource_name).ok_or(
            format!("No resource found matching requirement {}", res_req.resource_name)
        )?;
        copy_resource(&res_req, &res_dec, &resource_root)?;
        resolved_resources.push(res_dec);
    }

    // Write a record of the resources
    let res = serde_json::to_string(&resolved_resources)
        .expect("Unable to serialize the set of resolved resources");

    let record_file_path = resource_root.join("resolved_resources.json");
    fs::write(record_file_path, res).map_err(|e| format!("Failed writing record file:{:?}", e))?;

    Ok(())
}

/// Create the map of package details
fn get_package_details(metadata: &Metadata) -> Result<HashMap<PackageId, PackageDetails>, String> {
    let mut packages_by_id: HashMap<PackageId, PackageDetails> = HashMap::new();
    // Initialise the lookups without the dependency information (i.e. not in root deps)
    for ref package in metadata.packages.iter() {
        packages_by_id.insert(
            package.id.clone(),
            PackageDetails::new(&package)
        );
    }
    // Use the dependency tree from root to fix the dependency information
    let root_package = metadata.root_package()
        .ok_or("Unable to get root package")?;
    // Convert the dependency nodes from a list to a map!
    let dep_graph_root: &Resolve = metadata.resolve.as_ref().ok_or("Missing dependency graph.")?;
    let node_list = &dep_graph_root.nodes;
    let node_map: HashMap<PackageId, &Node> = node_list.iter().map(|n| (n.id.clone(), n)).collect();
    // All packages from the root node are dependencies so we could recursively visit all the dependencies
    // and then add them. However, using a stack and a set allows us to cut repetition.
    let mut processed_packages = HashSet::new();
    let mut pending_nodes = vec!(node_map.get(&root_package.id).ok_or("Missing dependency node")?);
    while let Some(node) = pending_nodes.pop() {
        // Set as a dependency
        let details = packages_by_id.get_mut(&node.id).ok_or("Missing details.")?;
        details.set_is_dependency();

        // Add to done
        processed_packages.insert(&node.id);

        // Add any unprocessed nodes to the pending queue.
        for pkg in &node.dependencies {
            if !processed_packages.contains(pkg) {
                pending_nodes.push(node_map.get(&pkg).ok_or("Missing details.")?);
            }
        }
    }
    Ok(packages_by_id)
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
                        let resolved_output_path = declaration
                            .output_path.
                            unwrap_or(declaration.crate_path.to_owned());
                        let resolved_name = declaration.resource_name.unwrap_or(
                            declaration.crate_path.file_name()
                                .expect("Illegal resource name").to_string().into()
                        );

                        // Paths should be relative
                        if declaration.crate_path.is_absolute() {
                            Err(
                                format!(
                                    "Crate {} declares an absolute resource path {}",
                                    &package.name,
                                    &declaration.crate_path
                                )
                            )?
                        }
                        if resolved_output_path.is_absolute() {
                            Err(
                                format!(
                                    "Crate {} declares an absolute output path {}",
                                    &package.name,
                                    &resolved_output_path
                                )
                            )?
                        }

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
                        if resources.contains_key(&resolved_name) {
                            println!(
                                "WARNING: Duplicate resource {}\nReplacing:\t{:?}\nWith:\t\t{:?}\n",
                                &resolved_name,
                                resources.get(&resolved_name).unwrap().full_crate_path,
                                &data.full_crate_path
                            );
                        }
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
                .map_err(|e| format!("Unable to read consuming crates [package.metadata.cargo_resources]: {}", e.to_string()))?
        }
        _ => panic!("Misconfigured [package.metadata.cargo_resources] in consuming package.")
    };

    let resource_root = consumer_declaration.resource_root.unwrap_or(Utf8PathBuf::from("target/resources"));

    let required_resources: Vec<ResourceRequirement> = match consumer_declaration.requires {
        None => { // Default is to use all available resources with default options
            available_resources.values().map(|res_spec| ResourceRequirement {
                resource_name: res_spec.resource_name.to_owned(),
                required_sha: None,
            }).collect()
        }
        Some(declarations) => { // Just convert each declaration to a spec
            declarations.into_iter().map(|dec| ResourceRequirement {
                resource_name: dec.resource_name.to_owned(),
                required_sha: dec.required_sha.to_owned(),
            }).collect()
        }
    };

    Ok(ResourceConsumerSpecification { resource_root, required_resources })
}

/// Copy the resource to the resources folder (if it doesn't already exist)
fn copy_resource(
    res_req: &ResourceRequirement,
    res_dec: &ResourceSpecification,
    resource_root: &Utf8PathBuf,
) -> Result<(), String> {
    let output_resources_path = resource_root
        .join(&res_dec.output_path);
    // Before copying, we should check the path isn't outside the resources root.
    verify_resource_is_in_root(&output_resources_path, &resource_root)?;

    // Create the output directory if it doesn't exist!
    let output_directory = output_resources_path.parent().unwrap();
    create_output_directory(output_directory)?;

    // Use sha256 to check if the file has changed, and verify against a required_sha
    let new_sha = hex::encode(get_file_sha(&res_dec.full_crate_path)?.as_ref());

    // Return error if the required sha is set and doesn't match.
    match res_req.required_sha {
        Some(ref req) => {
            if *req != new_sha {
                Err(
                    format!("Resource {} with sha {} does not match required sha {}.",
                            res_req.resource_name,
                            new_sha,
                            req
                    )
                )?
            }
        }
        _ => {}
    }

    // Only copy when the sha doesn't match (to avoid timestamp updates on the file)
    let mut already_exists = false;
    if output_resources_path.exists() {
        let existing_sha = hex::encode(get_file_sha(&output_resources_path)?.as_ref());
        if existing_sha == new_sha {
            already_exists = true;
        }
    }

    if !already_exists {
        fs::copy(&res_dec.full_crate_path, &output_resources_path)
            .map_err(|e|
                format!("Unable to copy resource {} to {}: {}",
                        &res_dec.full_crate_path,
                        &output_resources_path,
                        e
                )
            )?;
    }

    println!(
        "Resource {} {} {}",
        match already_exists {
            true => "existed:",
            false => " copied:"
        }.to_string(),
        &new_sha,
        &output_resources_path
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

// Check whether the resource is in the root!
fn verify_resource_is_in_root(
    resource_path: &Utf8PathBuf,
    root_path: &Utf8PathBuf,
) -> Result<(), String> {
    let can_root_path = root_path.canonicalize_utf8()
        .map_err(
            |e| format!(
                "Unable to canonicalize root path: {}: {}",
                root_path,
                e
            )
        )?;

    // Create interim folders to allow parentage check
    if !resource_path.parent().is_some() {
        return Ok(());
    }
    let mut walked_directory = Utf8PathBuf::new();
    let target_components = resource_path.parent().unwrap().components();
    for component in target_components {

        walked_directory = walked_directory.join(component);
        create_output_directory(&mut walked_directory)?;
    }
    let can_resource_path = resource_path.parent().unwrap().canonicalize_utf8()
        .map_err(
            |e| format!(
                "Unable to canonicalize resource path: {}: {}",
                resource_path,
                e
            )
        )?;

    if !can_resource_path.starts_with(&can_root_path) {
        Err(
            format!(
                "Can't copy to {:?} as not in resource root {:?}",
                can_resource_path,
                can_root_path
            )
        )?
    }
    Ok(())
}

/// Create the output directory if it doesn't exist.
fn create_output_directory(output_dir: &Utf8Path) -> Result<(), String> {
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir)
            .map_err(|e|
                format!("Unable to create output directory {}: {}", &output_dir, e)
            )?
    }
    Ok(())
}