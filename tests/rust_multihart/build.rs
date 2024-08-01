fn main() {
    let linker_script = std::env::var("DEP_RVFM_LINKER_SCRIPT").unwrap();
    println!("cargo::rustc-link-arg=-T{}", linker_script);
}
