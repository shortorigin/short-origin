fn main() {
    println!("cargo::rustc-check-cfg=cfg(tokio_unstable)");
    println!("cargo::rerun-if-changed=build.rs");
}
