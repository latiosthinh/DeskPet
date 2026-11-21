// Checks the built-in pet modules are there to embed (windows\build.ps1 and macos/build.sh make them first).

fn main() {
    for pet in ["mochi", "ninja", "moona"] {
        let file = format!("../../build/pets/{pet}.deskpet");
        println!("cargo:rerun-if-changed={file}");
        if !std::path::Path::new(&file).exists() {
            panic!("build/pets/{pet}.deskpet is missing. Build with windows\\build.ps1 or macos/build.sh, which make the built-in pets first.");
        }
    }
}
