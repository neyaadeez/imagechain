fn main() {
    // Rebuild when the build script changes
    println!("cargo:rerun-if-changed=build.rs");
    
    // Generate build information
    built::write_built_file()
        .expect("Failed to acquire build-time information");
}
