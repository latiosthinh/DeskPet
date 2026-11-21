# DeskPet

A tiny pixel pet that lives in the bottom-right corner of your Mac, next to the Dock. There's a [Windows 10/11 version](#windows) too, which stands on top of the taskbar.

Choose from five pets: **Mochi** the cream cat comes free, and **Ember** the fox, **Pip** the penguin, **Puddle** the slime and **Sprout** the seedling are premium pets you unlock with a code.

## Premium pets

Mochi and Ember come free. Every other pet needs a code, sold one pet at a time or as **All Pets**, which also covers pets added in later versions.

Pick a locked pet (🔒) in the **Pet** menu, or choose **Enter Unlock Code…**, and paste your code. It looks like `FNPW2-A4V00-T2WYH-026P8`; spaces, dashes and upper or lower case don't matter. Codes work offline and are remembered on this Mac, so you can add pets one by one and they all stay unlocked. A code bought before single-pet codes existed is an All Pets code.

## Pet tricks

Besides the shared activities, each pet has its own tricks. It does one on its own every 4–8 minutes (the first comes 1–3 minutes after launch). You can also start one from the menu bar under **Tricks**.

| Pet | Tricks |
| --- | --- |
| Mochi | Plays with a ball of yarn, naps in a cardboard box, chases a laser dot, goes fishing |
| Ember the fox | Lights fireworks, explores with a flashlight, flips an egg in a pan, toasts a marshmallow over a campfire |
| Pip the penguin | Hides in a pool and peeks out, spits water, swims, blows bubbles |
| Puddle the slime | Eats ice cream, splits in two and plays catch with itself, cycles through colours, pops out a snack for you |
| Sprout the seedling | Gives you a flower, takes a happy shower under a rain cloud, plants itself in soil for a nap, grows big, chases a butterfly |

- Wanders back and forth inside a small, limited area, sits, blinks, and turns to look at your mouse pointer.
- When you type, it gets busy too: Mochi types on a little laptop, Ember cooks an egg over a campfire, Pip swims, Puddle reads with its glasses on, and Sprout digs soil with a shovel.
- Now and then it sits down to read a book. Mochi also sweeps the floor.
- Reminds you to drink water, eat lunch and dinner, and take a stretch break, and acts each one out.
- Falls asleep after 90 seconds away, and wakes up when you come back.
- Click it to pet it (this also dismisses a reminder). Drag it to move its home.
- Menu-bar only (a tiny picture of your pet), no Dock icon, and no Accessibility or Input Monitoring permission needed.

## Menu-bar menu

| Item | What it does |
| --- | --- |
| Show Pet | Show or hide the pet |
| Pet | Switch between Mochi, Ember, Pip, Puddle and Sprout, or unlock the premium pets. The menu-bar icon follows your choice |
| Tricks | Start one of the current pet's tricks, or **Surprise Me** for a random one |
| Roam Range | How far it may walk from home: Stay Put, Tiny ±80 pt (default), Small ±160, Medium ±300, Large ±500 |
| Show Roam Area | Briefly outlines the area it walks in |
| Move Home to Corner | Puts it back in the bottom-right corner |
| Reminders | Turn each reminder on or off (all on by default): **Drink Water** every 20–30 minutes at the desk, **Main Meals** only at lunch (12:00–14:00) and dinner (18:30–20:30), once each a day, **Stretch Break** every 40–60 minutes of use (a 5-minute pause counts as a break). |
| Open at Login | Start DeskPet automatically when you log in |

Ranges larger than Tiny may overlap the right end of the Dock, depending on how wide your Dock is.

## Install

1. Download `DeskPet-<version>.dmg` from the latest release, open it, and drag **DeskPet** to **Applications**.
2. The app isn't notarized, so clear the download flag before the first launch:
   ```bash
   xattr -dr com.apple.quarantine /Applications/DeskPet.app
   ```
3. Open DeskPet.

## Build from source

Requires the Xcode Command Line Tools. macOS 13 or later.

```bash
mac/build.sh          # build and install to ~/Applications
mac/build.sh --dmg    # build a release disk image in build/
open ~/Applications/DeskPet.app --args --demo     # cycle through every activity, reminder and trick
open ~/Applications/DeskPet.app --args --tricks   # only the current pet's tricks
open ~/Applications/DeskPet.app --args --typing   # what the current pet does while you type
```

## Windows

The Windows version (in `windows/`) has the same pets, tricks, typing activities, reminders and menu. The pet stands on top of the taskbar, above the clock.

- **Menu:** click the pet's icon in the system tray, or right-click the pet itself. Windows 11 puts new tray icons in the overflow (**^**) at first; drag it onto the taskbar, or turn it on in **Settings → Personalization → Taskbar → Other system tray icons**.
- **Open at Login** adds DeskPet to your startup apps. If you turn it off in **Task Manager → Startup apps**, the menu shows a dot instead of a tick; choosing it turns it back on.
- The pet hides while a fullscreen app, game or presentation is in front.
- Speech bubbles show little pixel pictures instead of emoji.
- Every pet is free on Windows; there are no codes. Mochi, Kage and Moona come with the app. Get the others one at a time from the DeskPet website (**Pet → Get More Pets…**): double-click a downloaded `.deskpet` file, or drop it on your pet. **Pet → Remove** takes an added pet away again.
- Added pets live in `%LOCALAPPDATA%\DeskPet\pets`, and the app keeps them up to date from the website (it checks shortly after it starts, then once a day).
- Settings are kept in the registry under `HKCU\Software\DeskPet`.

### Install

1. Download `DeskPet-<version>-windows.zip` from the latest release and unzip it anywhere, for example into `%LOCALAPPDATA%\Programs\DeskPet`.
2. Run `DeskPet.exe`. It isn't signed, so SmartScreen may warn you the first time: choose **More info → Run anyway**.

It needs nothing else: it's a single native exe of about 1.5 MB, written in Rust. It uses about 8 MB of private memory and around 1–2% of one CPU core while the pet moves, and almost nothing while it sits still.

### Build from source

Requires [Rust](https://rustup.rs) (stable, with the default `x86_64-pc-windows-msvc` toolchain) and the Visual Studio C++ build tools. The exe links the C runtime statically, so it runs without any redistributable.

```powershell
powershell -ExecutionPolicy Bypass -File windows\build.ps1                    # build build\DeskPet.exe
powershell -ExecutionPolicy Bypass -File windows\build.ps1 -Zip               # also make the release zip
powershell -ExecutionPolicy Bypass -File windows\build.ps1 -Run -RunArgs --demo
```

`--demo`, `--tricks`, `--typing` and `--pastime` work the same as on the Mac, and `--pet <id>` starts with that pet. `windows\make-icon.ps1` redraws `DeskPet.ico` from Mochi's art.

The app and the pets are one Cargo workspace at the top of the repo: `sdk/petkit` is the pet API and the behaviour every pet shares, and each pet is its own crate in `pets/<id>`, with one file per trick.

Each pet is built as a sandboxed WebAssembly module, `<id>.deskpet`. The app has Mochi, Kage and Moona built in (`build.ps1` makes them first); the others are separate files:

```powershell
powershell -ExecutionPolicy Bypass -File windows\build-pets.ps1              # every pet, into build\pets
build\DeskPet.exe --pets build\pets --pet witcher --tricks                    # try one without installing it
```

To make a pet of your own, see [CONTRIBUTING-PETS.md](CONTRIBUTING-PETS.md): `windows\new-pet.ps1` starts one from a template, `windows\try-pet.ps1` shows it on the taskbar and `windows\check-pet.ps1` checks it and makes its GIFs.

`DESKPET_SITE` points the app at another copy of the website for **Get More Pets…** and the update check, for example `http://localhost:8788` while trying `site/` locally.

## Making unlock codes (developer, Mac app only)

Only the Mac app uses codes; the Windows app has none.

A code is 20 characters: a random serial number plus a check value made with a secret key. The check value also says what the code unlocks: all pets, or one pet by its id. The secret is never in the repo; `mac/build.sh` builds it into the app.

```bash
swift Scripts/make-codes.swift 10                 # 10 All Pets codes
swift Scripts/make-codes.swift 10 --pet lunette   # 10 codes that unlock only Lunette
```

Pet ids: `pip`, `puddle`, `sprout`, `grumbo`, `wispaling`, `hexley`, `mossback`, `sirclank`, `lunette`, `moona`, `witcher`, `ninja`, `samurai` (Mochi and Ember are free; Moona, Orbis the witcher, Kage the ninja and Ronin the samurai are in the Windows app only so far). A code for a pet that a copy of the app doesn't have yet is rejected there, so add a pet to the app before selling codes for it.

- The first run (or the first `mac/build.sh`) creates the secret at `~/.config/deskpet/code-secret`. **Back it up.** Anyone who has it can make codes, and if it's lost, copies of the app already out there won't accept new codes.
- Every code made is also logged in `~/.config/deskpet/codes.txt`, with the pet it unlocks.
- Someone who builds from source without your secret gets their own new secret, so your codes won't work in their build, and theirs won't work in yours.
- Because the app has to contain the secret to check codes, a determined person could dig it out of the app and make codes. Short codes trade that risk for being easy to type.

## Repository layout

| Folder | What's in it |
| --- | --- |
| `mac/` | The Mac app (`main.swift`) and `build.sh` |
| `windows/` | The Windows app (Rust), `build.ps1`, and the pet scripts (`new-pet.ps1`, `try-pet.ps1`, `check-pet.ps1`, `build-pets.ps1`). `src/bin/petshot` is the pet checker and GIF maker |
| `sdk/petkit/` | The pet API every pet is written against, and the shared behaviour (walking, sleeping, reminders, picking tricks) |
| `pets/` | One Rust crate per pet: its art, one file per trick, and what it does while you type. `pets/_template` is the starting point for a new pet ([CONTRIBUTING-PETS.md](CONTRIBUTING-PETS.md)) |
| `site/` | The pet website: gallery, pet pages, downloads and counters |
| `Scripts/` | Code makers for the Mac app (`make-codes.swift`, `make-codes.ps1`) |
| `prototypes/` | HTML mock-ups of the pets and their tricks |
| `.github/workflows/` | The release build |

Both builds write to `build/` at the top of the repo.

## Releasing

Push a version tag and GitHub Actions builds both apps and attaches them to a draft release:

```bash
git tag v1.2.0
git push origin v1.2.0
```

The release has `DeskPet-<version>.dmg` (Mac, universal) and `DeskPet-<version>-windows.zip`, each with a `.sha256`. Check the draft, then publish it.

The workflow needs your code secret so that your codes work in the released apps. Add it once, from the Mac that has it:

```bash
base64 -i ~/.config/deskpet/code-secret | gh secret set DESKPET_CODE_SECRET
```

Without that secret the release build stops instead of making a new one.
