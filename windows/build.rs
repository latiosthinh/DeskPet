// Embeds the icon, manifest (DPI awareness, common controls v6) and version info,
// and makes sure the unlock-code secret has been generated (by build.ps1).

fn main() {
    let dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let secret = dir.join("../build/code_secret.rs");
    println!("cargo:rerun-if-changed={}", secret.display());
    for f in ["deskpet.rc", "app.manifest", "DeskPet.ico"] {
        println!("cargo:rerun-if-changed={f}");
    }
    if !secret.exists() {
        panic!("build/code_secret.rs is missing. Build with windows\\build.ps1, which makes it.");
    }
    embed_resource::compile("deskpet.rc", embed_resource::NONE)
        .manifest_required()
        .unwrap();
}
