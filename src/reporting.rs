//! Provide better and configurable reporting of resource collection.
//! This allows finer control when using in a build script (i.e. build.rs file).

use build_print::{error, info, warn};
use cargo_metadata::camino::Utf8PathBuf;
use serde_json::Error;

/// Trait to allow configuration of progress reporting.
pub trait ReportingTrait {
    /// Report the collection of the resource. already_exists is true if the output already existed.
    fn report_resource_collection(&self, already_existed: bool, new_sha: &str, output_path: &Utf8PathBuf);
    
    /// Report that no resources were found to be collected.
    fn report_no_resources_found(&self);
    
    /// Report a missing resource.
    fn report_missing_resource(&self, resource_name: &str);
    
    /// Report a duplicate resource was found (what was replaced with what).
    fn report_duplicate_resource(&self, resolved_name: &str, replaced: &Utf8PathBuf, with: &Utf8PathBuf);
    
    /// Report a misformed resource declaration and the declaring crate.
    fn report_malformed_resource_declaration(&self, package_name: &str, err: &Error);
    
    /// Report a misformed section [package.metadata.cargo_resources] found while processing metadata.
    fn report_malformed_resources_section(&self);
}

/// The default reporting using the console - sensible for command line usage!
pub struct DefaultReporter {}
impl ReportingTrait for DefaultReporter {
    
    fn report_resource_collection(&self, already_existed: bool, new_sha: &str, output_path: &Utf8PathBuf) {
        println!(
            "Resource {} {} {}",
            match already_existed {
                true => "existed:",
                false => " copied:"
            }.to_string(),
            &new_sha,
            &output_path
        );
    }
    
    fn report_no_resources_found(&self) {
        println!("No resources were found - finishing early.");
    }

    fn report_missing_resource(&self, resource_name: &str) {
        println!("No resource found matching requirement {}", resource_name);
    }

    fn report_duplicate_resource(
        &self, 
        resolved_name: &str,
        replaced: &Utf8PathBuf,
        with: &Utf8PathBuf
    ) {
        println!(
            "WARNING: Duplicate resource {}\nReplacing:\t{:?}\nWith:\t\t{:?}\n",
            &resolved_name,
            replaced,
            with
        );
    }
    
    fn report_malformed_resource_declaration(&self, package_name: &str, err: &Error) {
        println!("Malformed resource declaration in {}: {}",
                package_name,
                err
        );   
    }
    
    fn report_malformed_resources_section(&self) {
        println!("unexpected type for [package.metadata.cargo_resources].provides in the json-metadata");
    }
}

#[allow(dead_code)]
/// A reporting implementation suitable for build.rs invocation.
pub struct BuildRsReporter {}
impl ReportingTrait for BuildRsReporter {

    fn report_resource_collection(&self, already_existed: bool, new_sha: &str, output_path: &Utf8PathBuf) {
        info!(
            "Resource {} {} {}",
            match already_existed {
                true => "existed:",
                false => " copied:"
            }.to_string(),
            &new_sha,
            &output_path
        );
    }

    fn report_no_resources_found(&self) {
        warn!("No resources were found - finishing early.");
    }

    fn report_missing_resource(&self, resource_name: &str) {
        error!("No resource found matching requirement {}", resource_name);
    }

    fn report_duplicate_resource(
        &self,
        resolved_name: &str,
        replaced: &Utf8PathBuf,
        with: &Utf8PathBuf
    ) {
        warn!(
            "WARNING: Duplicate resource {}\nReplacing:\t{:?}\nWith:\t\t{:?}\n",
            &resolved_name,
            replaced,
            with
        );
    }

    fn report_malformed_resource_declaration(&self, package_name: &str, err: &Error) {
        warn!("Malformed resource declaration in {}: {}",
                 package_name,
                 err
        );
    }

    fn report_malformed_resources_section(&self) {
        error!("Unexpected type for [package.metadata.cargo_resources].provides in the json-metadata");
    }
}