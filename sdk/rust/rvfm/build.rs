fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let linker_script = format!("{}/link.ld", manifest_dir);
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=link.ld");
    println!("cargo::metadata=LINKER_SCRIPT={}", linker_script);
}
