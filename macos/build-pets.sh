#!/bin/zsh
# Builds pet modules (<id>.deskpet, WebAssembly) into build/pets. The same files run on Windows.
#
#   macos/build-pets.sh                 # every pet in pets/
#   macos/build-pets.sh ninja moona     # just these
#
# Try them in the app with:  build/DeskPet.app/Contents/MacOS/DeskPet --pets build/pets --pet <id>
set -e
cd "$(dirname "$0")/.."   # the repo root

target=wasm32-unknown-unknown
pets=("$@")
if (( ${#pets} == 0 )); then
  for dir in pets/*(/); do
    name=${dir:t}
    [[ $name == _* ]] || pets+=$name
  done
fi

rustup target list --installed | grep -qx $target || rustup target add $target

args=()
for p in $pets; do args+=(-p "pet-$p"); done
cargo build --release --target $target $args

mkdir -p build/pets
for p in $pets; do
  cp "target/$target/release/pet_${p//-/_}.wasm" "build/pets/$p.deskpet"
  echo "Built build/pets/$p.deskpet ($(( $(stat -f%z build/pets/$p.deskpet) / 1024 )) KB)"
done
