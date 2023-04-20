#[test]
fn test_sync() {
    rustix::fs::sync();
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[test]
fn test_syncfs() {
    let f = std::fs::File::open("Cargo.toml").unwrap();
    rustix::fs::syncfs(&f).unwrap();
}
