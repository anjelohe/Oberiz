#[cfg(windows)]
fn main() {
    // Without this, cargo has no way to know the embedded icon depends on this
    // file, so replacing it silently keeps the old icon in incremental builds.
    println!("cargo:rerun-if-changed=../packaging/windows/oberiz.ico");
    let mut resource = winresource::WindowsResource::new();
    resource.set_icon("../packaging/windows/oberiz.ico");
    resource
        .compile()
        .expect("compile Windows application icon");
}

#[cfg(not(windows))]
fn main() {}
