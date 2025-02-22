# cargo-resources
 A cargo executable crate for managing resources.

## TLDR
Declare files as resources from your Cargo.toml files.
```toml
[package.metadata.cargo_resources]
provides = [
    { crate_path = "resources/hello_world.txt", output_path="hello_world.txt" }
]
```
Collate all the resources declared by a crate's dependencies (i.e. referenced crates), using a cargo command.

`
cargo resources
`

## Overview

### What do we mean by Resources
Resources are all those non-code files/artifacts that you need to help you software build or run as intended.
These include:
* static resources (i.e. fixed at compile time)
* dynamic resources (i.e. determined at run-time)

Often Resources are used during the execution of the program itself (possibly including tests, benches etc.), but there are also valid use-cases during build or deployment.

### Why don't we just use Rust/Cargo?
Rust and Cargo don't provide much structured help for dealing with resources, especially for the dynamic cases.

By default, Cargo will include any git configured files in the published crate, and Rust provides the [include_str!](https://doc.rust-lang.org/stable/std/macro.include_str.html) and [include_bytes!](https://doc.rust-lang.org/stable/std/macro.include_bytes.html) macros.
Which support simple static resource usage for things like test data (e.g. unit-test data-files declared within the crate).

For simple cases an external tool/crate isn't required and a 'resources' folder next to 'src' and usages via include_str! (with a relative path) are sufficient.

For help with dynamic (run/build time) or resources from external crates this crate provides a more structured approach.

## Installation
Recommend installation is via cargo install with fixed dependencies:

`
cargo install cargo-resources --fixed
`

## Configuring Declared Resources
Resources are declared using Cargo metadata, as this is the cargo resources crate, these are declared in a 'section':

```toml
[package.metadata.cargo_resources]
```
Within this 'section' each resource is declared in the provides 'array' as a key/value table:

```toml
provides = [
    { crate_path = "resources/hello_world.txt", output_path="hello_world.txt" }
]
```

The supported information for each resource is:

| Item          | Required? | Notes                                                                                       |
|---------------|-----------|---------------------------------------------------------------------------------------------|
| resource_name | optional  | Unique resource name, derived from output_path when not set.                                |
| crate_path    | required  | The path of the resource file within the source crate.                                      |
| output_path   | optional  | The relative resource path used on output, derived from crate_path when not set.            |
| encoding      | optional  | File encoding (Txt or Bin), defaults to text. NB. Primarily for using crates.               |

Normal usage is therefore setting crate_path and output_path, or just crate_path when output_path is identical.

## Declaring Resource Usage
By convention a crate does not need to specify resource usage and defaults to collating all resources from the dependencies, to a default resource path.

When this is not the required behaviour, the using crate specifies its requirement in its Cargo.toml:

```toml
[package.metadata.cargo_resources]
```
Within this 'section' the following information can be provided:

### The list of required resources
These are specified the 'requires' array:

```toml
requires = [
    { resource_name="hello_world.txt" }
]
```

The supported information for each resource is:

| Item          | Required? | Notes                                                                     |
|---------------|-----------|---------------------------------------------------------------------------|
| resource_name | required  | The Unique Resource Name (as declared or derived in the providing crate). |
| required_sha  | optional  | An optional SHA256 hex value. If specified the resource's sha must match. |

NB. If the required sha is set any change of the upstream resource will require a deliberate update in the using crate.

### Collation Options 

Collation options are provided as key value pairs within the 'section', For instance:
```toml
resource_root = "target/resources"
```

The supported Options are:

| Collation Option | Notes                                                                                                 |
|------------------|-------------------------------------------------------------------------------------------------------|
| resource_root    | The directory to use for the resource root, relative to the crate root. Defaults to target/resources. |


## Features
This crate declares the following features:
None as yet!

## Version History

| Version | Notes                                                                                                                                                        |
|---------|--------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 1.0.0   | Initial Release.                                                                                                                                             |
| 1.0.1   | Fix error for missing folder when no resources are copied.                                                                                                   |
| 1.1.0   | Addition of required_sha in resource requirements.<br/> Terminate when resources would be copied outside of resource root.                                   |
| 1.1.5   | Updated Licence to MIT.                                                                                                                                      |
| 1.1.6   | Fixed bug where resources could be collated from the workspace instead of the dependency tree.<br/> Added warning when finding resources with the same name. |

## Troubleshooting

1. It works locally but not from a published crate.
   * Check the resources are included in the published crate (add to include in the cargo.toml if required).

2. Returns an error of : "Unable to canonicalize resource path: ...".
   * A directory/folder in the output path does not exist.