fn main() {
    println!("cargo::rerun-if-changed=bin/vshader.bin");
    println!("cargo::rerun-if-changed=bin/fshader.bin");
    let linker_script = std::env::var("DEP_RVFM_LINKER_SCRIPT").unwrap();
    println!("cargo::rustc-link-arg=-T{}", linker_script);
}