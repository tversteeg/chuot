//! Minify WGSL shaders.

use std::path::Path;

use naga::{
    back::wgsl::WriterFlags,
    valid::{Capabilities, ValidationFlags, Validator},
};

/// Compile a WGSL shader into Spir-V bytes and write it to file.
fn minify_wgsl(source: impl AsRef<Path>, target: impl AsRef<Path>) {
    // Read the source WGSL
    let source = std::fs::read_to_string(source).expect("Error reading WGSL shader file");

    // Parse into NAGA module
    let mut module = naga::front::wgsl::parse_str(&source).expect("Error compiling WGSL shader");

    // Create the validator
    let info = Validator::new(ValidationFlags::all(), Capabilities::all())
        .validate(&module)
        .expect("Error while validating WGSL shader");

    // Optimize shader, removing unused stuff
    naga::compact::compact(&mut module);

    // Compile back into WGSL
    let output = naga::back::wgsl::write_string(&module, &info, WriterFlags::empty())
        .expect("Error converting WGSL module back to WGSL code");

    // Minify the WGSL
    let output = wgsl_minifier::minify_wgsl_source(&output);

    // Convert to bytes
    std::fs::write(target, output).expect("Error writing minified WGSL shader to file");
}

fn main() {
    // Rerun build script if shaders changed
    println!("cargo::rerun-if-changed=shaders/downscale.wgsl");
    println!("cargo::rerun-if-changed=shaders/rotation.wgsl");
    println!("cargo::rerun-if-changed=shaders/nearest_neighbor.wgsl");

    let out_dir_str = std::env::var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir_str);

    // Compile the shaders into binaries placed in the OUT_DIR
    minify_wgsl("shaders/downscale.wgsl", out_dir.join("downscale.wgsl"));
    minify_wgsl("shaders/rotation.wgsl", out_dir.join("rotation.wgsl"));
    minify_wgsl(
        "shaders/nearest_neighbor.wgsl",
        out_dir.join("nearest_neighbor.wgsl"),
    );
}
