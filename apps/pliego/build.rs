//! Windows only: embed the app icon and version info (VERSIONINFO) into `pliego.exe`, so it
//! shows in Explorer, the taskbar, the Start menu and Alt-Tab.
//!
//! On every other target this does nothing. A missing resource compiler is a warning, so a
//! cross-compile from macOS or Linux still links, unless `PLIEGO_REQUIRE_WINRES=1` turns it
//! into an error (for release builds).

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/app-icon/pliego.ico");
    println!("cargo:rerun-if-env-changed=PLIEGO_REQUIRE_WINRES");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon("../../assets/app-icon/pliego.ico")
        .set("ProductName", "Pliego")
        .set("FileDescription", "Pliego page layout")
        .set("CompanyName", "Nodalia")
        .set("LegalCopyright", "Copyright (c) the Pliego authors. MIT OR Apache-2.0.")
        .set("OriginalFilename", "pliego.exe")
        .set("InternalName", "pliego");
    if let Err(e) = res.compile() {
        if std::env::var_os("PLIEGO_REQUIRE_WINRES").is_some() {
            eprintln!("embedding Windows resources failed: {e}");
            std::process::exit(1);
        }
        println!("cargo:warning=pliego.exe built without icon/version resources: {e}");
    }
}
