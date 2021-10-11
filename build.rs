use cc::Build;
use std::env::var;

fn main() {
    let asm_name = format!(
        "src/imp/linux_raw/arch/outline/{}.S",
        var("CARGO_CFG_TARGET_ARCH").unwrap()
    );
    let os_name = var("CARGO_CFG_TARGET_OS").unwrap();
    let is_x32 = var("CARGO_CFG_TARGET_ARCH").unwrap() == "x86_64"
        && var("CARGO_CFG_TARGET_POINTER_WIDTH").unwrap() == "32";

    // If rsix_use_libc is set, or if we're on an architecture/OS that doesn't
    // have raw syscall support, use libc.
    if var("CARGO_CFG_RSIX_USE_LIBC").is_ok()
        || os_name != "linux"
        || std::fs::metadata(&asm_name).is_err()
        || is_x32
    {
        println!("cargo:rustc-cfg=libc");
    } else {
        println!("cargo:rustc-cfg=linux_raw");

        if let rustc_version::Channel::Nightly = rustc_version::version_meta()
            .expect("query rustc release channel")
            .channel
        {
            println!("cargo:rustc-cfg=linux_raw_inline_asm");
            println!("cargo:rustc-cfg=rustc_attrs");
            println!("cargo:rustc-cfg=doc_cfg");
        } else {
            Build::new().file(&asm_name).compile("asm");
            println!("cargo:rerun-if-changed={}", asm_name);
            println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ARCH");
        }
        if rustc_version::version().unwrap() >= rustc_version::Version::parse("1.56.0").unwrap() {
            println!("cargo:rustc-cfg=const_fn_union");
        }
    }
    println!("cargo:rerun-if-env-changed=CARGO_CFG_RSIX_USE_LIBC");
}
