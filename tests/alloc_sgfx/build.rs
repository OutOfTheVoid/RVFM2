use std::process::Command;

fn main() {
    let mut shader_assembler = Command::new("../../target/debug/shader_assembler")
        .args(&["src/shader.shasm", "-v", "bin/vshader.bin"])
        .spawn()
        .unwrap();
    shader_assembler.wait().unwrap();
    println!("cargo::rerun-if-changed=bin/vshader.bin");
    println!("cargo::rerun-if-changed=bin/fshader.bin");
    let linker_script = std::env::var("DEP_RVFM_PLATFORM_LINKER_SCRIPT").unwrap();
    println!("cargo::rustc-link-arg=-T{}", linker_script);
}