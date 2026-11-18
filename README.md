# DeskPet

A tiny pixel critter that lives in the bottom-right corner of your Mac, next to the Dock.

- Wanders back and forth inside a small, limited area, sits, blinks, and turns to look at your mouse pointer.
- Bobs along while you type, falls asleep after 90 seconds away, and wakes up when you come back.
- Click it to pet it. Drag it to move its home.
- Optional reminder to stretch after 50 minutes of continuous use.
- Menu-bar only (a tiny pink-eared pet face), no Dock icon, and no Accessibility or Input Monitoring permission needed.

## Menu-bar menu

| Item | What it does |
| --- | --- |
| Show Pet | Show or hide the pet |
| Roam Range | How far it may walk from home: Stay Put, Tiny ±80 pt (default), Small ±160, Medium ±300, Large ±500 |
| Show Roam Area | Briefly outlines the area it walks in |
| Move Home to Corner | Puts it back in the bottom-right corner |
| Break Reminders | Stretch reminder every 50 minutes of use |

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
./build.sh          # build and install to ~/Applications
./build.sh --dmg    # build a release disk image in build/
```
