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
- Settings are kept in the registry under `HKCU\Software\DeskPet`. Your code is remembered on this PC.

### Install

1. Download `DeskPet-<version>-windows.zip` from the latest release and unzip it anywhere, for example into `%LOCALAPPDATA%\Programs\DeskPet`.
2. Run `DeskPet.exe`. It isn't signed, so SmartScreen may warn you the first time: choose **More info → Run anyway**.

It needs nothing else: it's a single native exe of about 375 KB, written in Rust. It uses about 2 MB of private memory and around 1–2% of one CPU core while the pet moves, and almost nothing while it sits still.

### Build from source

Requires [Rust](https://rustup.rs) (stable, with the default `x86_64-pc-windows-msvc` toolchain) and the Visual Studio C++ build tools. The exe links the C runtime statically, so it runs without any redistributable.

```powershell
powershell -ExecutionPolicy Bypass -File windows\build.ps1                    # build build\DeskPet.exe
powershell -ExecutionPolicy Bypass -File windows\build.ps1 -Zip               # also make the release zip
powershell -ExecutionPolicy Bypass -File windows\build.ps1 -Run -RunArgs --demo
```

`--demo`, `--tricks` and `--typing` work the same as on the Mac. `windows\make-icon.ps1` redraws `DeskPet.ico` from Mochi's art.

To make codes on Windows, use `Scripts\make-codes.ps1` (same codes, same secret file):

```powershell
powershell -ExecutionPolicy Bypass -File Scripts\make-codes.ps1 10                # All Pets codes
powershell -ExecutionPolicy Bypass -File Scripts\make-codes.ps1 10 -Pet lunette   # codes for Lunette only
```

**Copy your secret first.** The Windows build reads the same `~/.config/deskpet/code-secret` as the Mac build (`C:\Users\<you>\.config\deskpet\code-secret` on Windows). Copy that file over from your Mac before the first Windows build, or the build makes a new secret and your existing codes won't work in it.

## Making unlock codes (developer)

A code is 20 characters: a random serial number plus a check value made with a secret key. The check value also says what the code unlocks: all pets, or one pet by its id. The secret is never in the repo; `mac/build.sh` and `windows\build.ps1` build it into the app.

```bash
swift Scripts/make-codes.swift 10                 # 10 All Pets codes
swift Scripts/make-codes.swift 10 --pet lunette   # 10 codes that unlock only Lunette
```

Pet ids: `pip`, `puddle`, `sprout`, `grumbo`, `wispaling`, `hexley`, `mossback`, `sirclank`, `lunette`, `cowy` (Mochi and Ember are free). A code for a pet that a copy of the app doesn't have yet is rejected there, so add a pet to the app before selling codes for it.

- The first run (or the first `mac/build.sh`) creates the secret at `~/.config/deskpet/code-secret`. **Back it up.** Anyone who has it can make codes, and if it's lost, copies of the app already out there won't accept new codes.
- Every code made is also logged in `~/.config/deskpet/codes.txt`, with the pet it unlocks.
- Someone who builds from source without your secret gets their own new secret, so your codes won't work in their build, and theirs won't work in yours.
- Because the app has to contain the secret to check codes, a determined person could dig it out of the app and make codes. Short codes trade that risk for being easy to type.

## Repository layout

| Folder | What's in it |
| --- | --- |
| `mac/` | The Mac app (`main.swift`) and `build.sh` |
| `windows/` | The Windows app (Rust) and `build.ps1` |
| `Scripts/` | Code makers for both platforms (`make-codes.swift`, `make-codes.ps1`) |
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
