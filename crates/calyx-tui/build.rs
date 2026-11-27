//! Windows: embeds the name ("Calyx") and the icon in calyx.exe. The media
//! overlay and Task Manager read them from the exe; without them Windows
//! shows "Unknown application" next to what calyx is playing.

fn main() {
    println!("cargo:rerun-if-changed=calyx.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("calyx.ico");
        res.set("FileDescription", "Calyx");
        res.set("ProductName", "Calyx");
        if let Err(e) = res.compile() {
            // A dev machine without the Windows SDK still builds, unbranded.
            println!("cargo:warning=calyx.exe built without its name and icon: {e}");
        }
    }
}
