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
Collate all resources declared by a crates (e.g. an executable) referenced crates them to a using crate using the cargo.

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
Resources are declared using Cargo metadata, as this is the cargo resources crate, these are declared in:

```toml
[package.metadata.cargo_resources]
```
Within this 'table' each resource is declared:

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

Normal usage is therefore setting crate_path and output_path, but just crate_path when output_path is the same.

## Declaring Resource Usage
By convention a crate does not need to specify resource usage and defaults to collating all resources from the dependencies, to a default resource path.

When this is not the required behaviour the using crate specifies its requirement in the Cargo.toml:

```toml
[package.metadata.cargo_resources]
```
Within this 'table' the following information can be provided:

### The list of required resources
These are specified as a list in 'requires'

```toml
requires = [
    { resource_name="hello_world.txt" }
]
```

The supported information for each resource is:

| Item          | Required? | Notes                                                                            |
|---------------|-----------|----------------------------------------------------------------------------------|
| resource_name | required  | The Unique Resource Name (as declared or derived in the providing crate).        |

NB. This is likely to be extended with extra options in future releases.

### Collation Options 

Collation options are provided as key value pairs, For instance:
```toml
resource_root = "target/resources"
```

The supported Options are:

| Optional      | Notes                                                                                                 |
|---------------|-------------------------------------------------------------------------------------------------------|
| resource_root | The directory to use for the resource root, relative to the crate root. Defaults to target/resources. |


## Features
This crate declares the following features:
None as yet!

## Troubleshooting

1. It works locally but not from meuse.
   * Check the resources are included in the published crate (add to include in the cargo.toml if required).
2. 
