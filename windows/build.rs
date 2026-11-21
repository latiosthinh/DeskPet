// Embeds the icon, manifest (DPI awareness, common controls v6) and version info.

fn main() {
    for f in ["deskpet.rc", "app.manifest", "DeskPet.ico"] {
        println!("cargo:rerun-if-changed={f}");
    }
    embed_resource::compile("deskpet.rc", embed_resource::NONE)
        .manifest_required()
        .unwrap();
}
