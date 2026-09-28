#[cfg(windows)]
fn main() {
    let mut resource = winresource::WindowsResource::new();
    resource.set_icon("../packaging/windows/oberiz.ico");
    resource.compile().expect("compile Windows application icon");
}

#[cfg(not(windows))]
fn main() {}
