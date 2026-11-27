//! Windows: embeds the name ("Calyx") and the icon in calyx.exe. Task
//! Manager and the Start Menu shortcut calyx writes take them from the exe.

fn main() {
    println!("cargo:rerun-if-changed=calyx.ico");
    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resources();
    }
}

#[cfg(windows)]
fn embed_resources() {
    let mut res = winresource::WindowsResource::new();
    res.set_icon("calyx.ico");
    res.set("FileDescription", "Calyx");
    res.set("ProductName", "Calyx");
    if let Err(e) = res.compile() {
        // A dev machine without the Windows SDK still builds, unbranded.
        println!("cargo:warning=calyx.exe built without its name and icon: {e}");
    }
}
