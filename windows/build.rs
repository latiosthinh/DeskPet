// Embeds the icon, manifest (DPI awareness, common controls v6) and version info, and checks the
// built-in pet modules are there to embed (build.ps1 makes them).

fn main() {
    for f in ["deskpet.rc", "app.manifest", "DeskPet.ico"] {
        println!("cargo:rerun-if-changed={f}");
    }
    for pet in ["mochi", "ninja", "moona"] {
        let file = format!("../build/pets/{pet}.deskpet");
        println!("cargo:rerun-if-changed={file}");
        if !std::path::Path::new(&file).exists() {
            panic!("build/pets/{pet}.deskpet is missing. Build with windows\\build.ps1, which makes the built-in pets first.");
        }
    }
    embed_resource::compile("deskpet.rc", embed_resource::NONE)
        .manifest_required()
        .unwrap();
}
