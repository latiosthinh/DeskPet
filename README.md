# DeskPet

A tiny pixel pet that lives in the bottom-right corner of your screen: next to the Dock on a Mac, on top of the taskbar on Windows 10/11.

## Pets

Every pet is free. **Mochi** the cream cat, **Kage** the ninja and **Moona** the moon knight come with the app, one from each shelf (Critters, Heroes and Legends). The rest are on the DeskPet website, one page per pet with a GIF of each trick: open **Pet → Get More Pets…**, download a pet, and open the `.deskpet` file (double-click it, or drop it on your pet). **Pet → Remove** takes an added pet away again.

A pet is a small sandboxed module (`<id>.deskpet`, WebAssembly) that can't touch your files, the network or the system. The same file works in the Mac and the Windows app. Added pets are kept up to date from the website: the app checks shortly after it starts, then once a day.

Anyone can make a pet: see [CONTRIBUTING-PETS.md](CONTRIBUTING-PETS.md).

## What the pet does

- Wanders back and forth inside a small, limited area, sits, blinks, and turns to look at your mouse pointer.
- Does one of its own tricks every 4–8 minutes (the first comes 1–3 minutes after launch). You can also start one from the menu under **Tricks**.
- When you type, it gets busy too: Mochi types on a little laptop, and every pet has its own activity. Now and then it has a pastime, and Mochi also sweeps the floor.
- Reminds you to drink water, eat lunch and dinner, and take a stretch break, and acts each one out.
- Falls asleep after 90 seconds away, and wakes up when you come back.
- Click it to pet it (this also dismisses a reminder). Drag it to move its home. Some pets, like Moona, do a trick when clicked or right-clicked; Shift+right-click then opens the menu.

## Menu

On a Mac, click the pet's face in the menu bar, or right-click the pet. On Windows, click its tray icon, or right-click the pet.

| Item | What it does |
| --- | --- |
| Show Pet | Show or hide the pet |
| Pet | Switch pets (grouped by shelf), **Get More Pets…** from the website, or **Remove** an added pet. The menu-bar or tray icon follows your choice |
| Tricks | Start one of the current pet's tricks, or **Surprise Me** for a random one |
| Roam Range | How far it may walk from home: Stay Put, Tiny ±80 (default), Small ±160, Medium ±300, Large ±500 |
| Show Roam Area | Briefly outlines the area it walks in |
| Move Home to Corner | Puts it back in the bottom-right corner |
| Reminders | Turn each reminder on or off (all on by default): **Drink Water** every 20–30 minutes at the desk, **Main Meals** only at lunch (12:00–14:00) and dinner (18:30–20:30), once each a day, **Stretch Break** every 40–60 minutes of use (a 5-minute pause counts as a break). |
| Open at Login | Start DeskPet automatically when you log in |

## Mac

- Menu-bar only (a tiny picture of your pet), no Dock icon, and no Accessibility or Input Monitoring permission needed.
- Ranges larger than Tiny may overlap the right end of the Dock, depending on how wide your Dock is.
- Added pets live in `~/Library/Application Support/DeskPet/Pets`.

### Install

1. Download `DeskPet-<version>.dmg` from the latest release, open it, and drag **DeskPet** to **Applications**.
2. The app isn't notarized, so clear the download flag before the first launch:
   ```bash
   xattr -dr com.apple.quarantine /Applications/DeskPet.app
   ```
3. Open DeskPet.

### Build from source

Requires the Xcode Command Line Tools and [Rust](https://rustup.rs). macOS 13 or later.

```bash
macos/build.sh          # build and install to ~/Applications
macos/build.sh --dmg    # build a release disk image in build/
open ~/Applications/DeskPet.app --args --demo     # cycle through every activity, reminder and trick
open ~/Applications/DeskPet.app --args --tricks   # only the current pet's tricks
open ~/Applications/DeskPet.app --args --typing   # what the current pet does while you type
macos/build-pets.sh                                  # every pet, into build/pets
build/DeskPet.app/Contents/MacOS/DeskPet --pets build/pets --pet witcher --tricks   # try one without adding it
```

`--pastime` shows the current pet's pastime, and `--pet <id>` starts with that pet.

## Windows

The pet stands on top of the taskbar, above the clock.

- **Menu:** Windows 11 puts new tray icons in the overflow (**^**) at first; drag it onto the taskbar, or turn it on in **Settings → Personalization → Taskbar → Other system tray icons**.
- **Open at Login** adds DeskPet to your startup apps. If you turn it off in **Task Manager → Startup apps**, the menu shows a dot instead of a tick; choosing it turns it back on.
- The pet hides while a fullscreen app, game or presentation is in front.
- Added pets live in `%LOCALAPPDATA%\DeskPet\pets`. Settings are kept in the registry under `HKCU\Software\DeskPet`.

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
powershell -ExecutionPolicy Bypass -File windows\build-pets.ps1              # every pet, into build\pets
build\DeskPet.exe --pets build\pets --pet witcher --tricks                    # try one without adding it
```

`--demo`, `--tricks`, `--typing` and `--pastime` work as on the Mac, and `--pet <id>` starts with that pet. `windows\make-icon.ps1` redraws `DeskPet.ico` from Mochi's art.

## How it fits together

The apps and the pets are one Cargo workspace at the top of the repo. Each pet is its own crate in `pets/<id>` (one file per trick), written against the pet API in `shared/petkit`, and built as a WebAssembly module. `shared/host/` runs those modules in the wasmi interpreter and draws what they render into a pixel buffer. Both apps use it: the Windows app directly, the Mac app through a small C interface (`macos/core`, declared in `macos/core/deskpet.h`). Each app only adds the native parts: window, menus, mouse, reminders, settings, text and HTTP.

To make a pet of your own, see [CONTRIBUTING-PETS.md](CONTRIBUTING-PETS.md): `windows\new-pet.ps1` starts one from a template, `windows\try-pet.ps1` shows it on the taskbar and `windows\check-pet.ps1` checks it and makes its GIFs.

`DESKPET_SITE` points either app at another copy of the website for **Get More Pets…** and the update check, for example `http://localhost:8788` while trying `website/` locally.

## Repository layout

| Folder | What's in it |
| --- | --- |
| `macos/` | The Mac app (`main.swift`), `build.sh` and `build-pets.sh`. `core/` is the C interface to the shared host |
| `windows/` | The Windows app (Rust), `build.ps1`, and the pet scripts (`new-pet.ps1`, `try-pet.ps1`, `check-pet.ps1`, `build-pets.ps1`). `src/bin/petshot` is the pet checker and GIF maker |
| `shared/host/` | What both apps share: running pet modules, drawing them, and the list of pets (built in, added, updated) |
| `shared/petkit/` | The pet API every pet is written against, and the shared behaviour (walking, sleeping, reminders, picking tricks) |
| `pets/` | One Rust crate per pet: its art, one file per trick, and what it does while you type. `pets/_template` is the starting point for a new pet ([CONTRIBUTING-PETS.md](CONTRIBUTING-PETS.md)) |
| `website/` | The pet website: gallery, pet pages, downloads and counters |
| `prototypes/` | HTML mock-ups of the pets and their tricks |
| `.github/workflows/` | The build check and the release build |

Both builds write to `build/` at the top of the repo.

## Releasing

Push a version tag and GitHub Actions builds both apps and attaches them to a draft release:

```bash
git tag v1.2.0
git push origin v1.2.0
```

The release has `DeskPet-<version>.dmg` (Mac, universal) and `DeskPet-<version>-windows.zip`, each with a `.sha256`. Check the draft, then publish it.
