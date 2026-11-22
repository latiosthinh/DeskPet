# DeskPet development

Technical notes for working on DeskPet. For what the pet is and what it does, see the [DeskPet README](../README.md).

## How it fits together

The apps and the pets are one Cargo workspace at the top of the repo. Each pet is its own crate in `pets/<id>` (one file per trick), written against the pet API in `shared/petkit`, and built as a WebAssembly module. `shared/host/` runs those modules in the wasmi interpreter and draws what they render into a pixel buffer. Both apps use it: the Windows app directly, the Mac app through a small C interface (`macos/core`, declared in `macos/core/deskpet.h`). Each app only adds the native parts: window, menus, mouse, reminders, settings, text and HTTP.

## Repository layout

| Folder | What's in it |
| --- | --- |
| `macos/` | The Mac app (`main.swift`), `build.sh` and `build-pets.sh`. `core/` is the C interface to the shared host |
| `windows/` | The Windows app (Rust), `build.ps1`, and the pet scripts (`new-pet.ps1`, `try-pet.ps1`, `check-pet.ps1`, `build-pets.ps1`). `src/bin/petshot` is the pet checker and GIF maker |
| `shared/host/` | What both apps share: running pet modules, drawing them, and the list of pets (built in, added, updated) |
| `shared/petkit/` | The pet API every pet is written against, and the shared behaviour (walking, sleeping, reminders, picking tricks) |
| `pets/` | One Rust crate per pet: its art, one file per trick, and what it does while you type. `pets/_template` is the starting point for a new pet ([CONTRIBUTING-PETS.md](../CONTRIBUTING-PETS.md)) |
| `website/` | The pet website: gallery, pet pages, downloads and counters |
| `prototypes/` | HTML mock-ups of the pets and their tricks |
| `.github/workflows/` | The build check and the release build |

Both builds write to `build/` at the top of the repo.

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

## Menu reference

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

## Pets and modules

A pet is a small sandboxed module (`<id>.deskpet`, WebAssembly) that can't touch your files, the network or the system. The same file works in the Mac and the Windows app. Added pets are kept up to date from the website: the app checks shortly after it starts, then once a day.

To install one, open **Pet → Get More Pets…**, download a pet, and open the `.deskpet` file (double-click it, or drop it on your pet). **Pet → Remove** takes an added pet away again.

To make a pet of your own, see [CONTRIBUTING-PETS.md](../CONTRIBUTING-PETS.md): `windows\new-pet.ps1` starts one from a template, `windows\try-pet.ps1` shows it on the taskbar and `windows\check-pet.ps1` checks it and makes its GIFs.

The tool behind those scripts is petshot (`windows\src\bin\petshot`), which runs pets the same way DeskPet does, off screen:

- `petshot check <pet.deskpet | folder>...` — try every trick, activity and reminder; report problems.
- `petshot gifs <pet.deskpet> <folder>` — write `hero.gif` and `<trick-id>.gif`.
- `petshot site <pets folder> <site folder>` — for the website: check every pet, make its GIFs, copy its module and rewrite `pets.json`.

Build pets first with `windows\build-pets.ps1` (they go to `build\pets`).

## Website

The site lives in `website/`; its stack, build, local development, tests and deploy are documented in [website/README.md](../website/README.md). It is generated from `website/pets.json` and the GIFs and modules that `petshot site` writes. It is published at <https://deskpet-vn.pages.dev> from the Cloudflare Pages project `deskpet-vn`. `DESKPET_SITE` points either app at another copy of the website for **Get More Pets…** and the update check, for example `http://localhost:8788` while trying `website/` locally (for example with `wrangler pages dev`).

Setting the site up is a one-time owner task, run from `website/`. Log in to Cloudflare and create the D1 database named `deskpet`, copy the printed `database_id` into `wrangler.toml`, then apply the schema and create the Pages project and its `STATS_TOKEN` secret:

```powershell
npx wrangler login
npx wrangler d1 create deskpet
# copy the printed database_id into wrangler.toml, then:
npx wrangler d1 execute deskpet --remote --file=schema.sql
npx wrangler pages project create deskpet-vn --production-branch main
npx wrangler pages secret put STATS_TOKEN --project-name deskpet-vn
```

For each deploy, from the repository root run `powershell -ExecutionPolicy Bypass -File windows\check-pet.ps1` and `target\release\petshot.exe site build\pets website`, then from `website/` run `npm run build` and `npx wrangler pages deploy`. Full detail is in [website/README.md](../website/README.md), under **Deploying (owner, not CI)**.

## Releasing

Push a version tag and GitHub Actions builds both apps and attaches them to a draft release:

```bash
git tag v1.2.0
git push origin v1.2.0
```

The release has `DeskPet-<version>.dmg` (Mac, universal) and `DeskPet-<version>-windows.zip`, each with a `.sha256`. Check the draft, then publish it.