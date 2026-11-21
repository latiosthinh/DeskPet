// Embeds the icon, manifest (DPI awareness, common controls v6) and version info.
// (The shared host crate checks the built-in pet modules are there to embed.)

fn main() {
    for f in ["deskpet.rc", "app.manifest", "DeskPet.ico"] {
        println!("cargo:rerun-if-changed={f}");
    }
    embed_resource::compile("deskpet.rc", embed_resource::NONE)
        .manifest_required()
        .unwrap();
}
