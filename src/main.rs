//! This is the cargo tool cargo-resources entry point. It is intended to be invoked via cargo rather
//! than directly.

use cargo_metadata::camino::Utf8PathBuf;
use clap::Parser;

use cargo_resources::collate_resources;
pub use resource_args::ResourceArgs;

mod resource_args;

fn main() -> Result<(), String> {
    let args = ResourceArgs::parse();

    match args.tool_name.as_deref() {
        None => println!("invoked without args - not from cargo"),
        Some("resources") => (),
        Some(&_) => panic!("incorrect invocation - call as a cargo tool - cargo resource ...")
    }

    let package_path = match args.package {
        None => {
            Utf8PathBuf::from_path_buf(
                std::env::current_dir().map_err(|_e|"Can't find current directory!".to_string())?
            ).map_err(|e| format!("Unable to convert provided package path to UTF8: {:?}", e))?
        }
        Some(p) => p
    };
    if !package_path.is_dir() {
        Err(format!("'package' parameter [{}] should be a directory.", package_path))?
    }
    let source_manifest = package_path.join("Cargo.toml");

    // Use the library to do the actual work
    collate_resources(&source_manifest)
}




