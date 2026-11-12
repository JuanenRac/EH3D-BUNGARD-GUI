//! Puts the icon and the version information of the program into the Windows executable.
//! On every other target nothing happens, and a missing icon or resource compiler only gives a warning.

/// The short commit of a repository folder (the one the program was built from and its siblings), or `unknown` when git cannot say.
fn commit_of(folder: &str) -> String {
    std::process::Command::new("git")
        .args(["-C", folder, "rev-parse", "--short=7", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty() && t.chars().all(|c| c.is_ascii_hexdigit()))
        .unwrap_or_else(|| "unknown".to_string())
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    // the commits the program is made of, for the About window: a bug report says exactly which code ran
    let parts = [
        ("BUNGARD-GUI", "."),
        ("COMMON", "../EH3D-COMMON"),
        ("DRILLPRO", "../EH3D-DRILLPRO"),
        ("ROUTEPRO", "../EH3D-ROUTEPRO"),
    ];
    let text: Vec<String> = parts
        .iter()
        .map(|(name, folder)| format!("{name} {}", commit_of(folder)))
        .collect();
    println!("cargo:rustc-env=EH3D_BUILD_COMMITS={}", text.join(" / "));
    println!("cargo:rerun-if-changed=images/brand/app.ico");
    #[cfg(windows)]
    embed();
}

#[cfg(windows)]
fn embed() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let icon = "images/brand/app.ico";
    if !std::path::Path::new(icon).is_file() {
        println!("cargo:warning=icon {icon} not found, the executable gets no icon");
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon(icon)
        .set("ProductName", "EH3D-BUNGARD-GUI")
        .set(
            "FileDescription",
            "Modern interface for the Bungard machines",
        )
        .set("CompanyName", "JuanenRac (Electro Hobby 3D)")
        .set(
            "LegalCopyright",
            "Copyright (C) JuanenRac (Electro Hobby 3D)",
        );
    if let Err(e) = res.compile() {
        println!("cargo:warning=could not embed the icon and version information: {e}");
    }
}
