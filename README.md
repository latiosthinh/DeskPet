# DeskPet

A tiny pixel pet that lives in the bottom-right corner of your Mac, next to the Dock.

Choose from five pets: **Mochi** the cream cat comes free, and **Ember** the fox, **Pip** the penguin, **Puddle** the slime and **Sprout** the seedling are premium pets you unlock with a code.

## Premium pets

Pick a locked pet (🔒) in the **Pet** menu, or choose **Unlock All Pets…**, and paste your code. It looks like `PET-040PCF-…`; spaces, dashes and upper or lower case don't matter. The code works offline and is remembered on this Mac.

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
./build.sh          # build and install to ~/Applications
./build.sh --dmg    # build a release disk image in build/
open ~/Applications/DeskPet.app --args --demo     # cycle through every activity, reminder and trick
open ~/Applications/DeskPet.app --args --tricks   # only the current pet's tricks
open ~/Applications/DeskPet.app --args --typing   # what the current pet does while you type
```

## Making unlock codes (developer)

Codes are signed with a private key that never goes into the repo, so the app can check a code but nobody can make one from the app or the source.

```bash
swift Scripts/make-codes.swift 10        # print 10 new codes
swift Scripts/make-codes.swift --public  # print the public key that main.swift checks against
```

- The first run creates the key at `~/.config/deskpet/signing-key`. **Back it up.** Anyone who has it can make codes, and if it's lost, new codes won't work in copies of the app already out there.
- Every code made is also logged in `~/.config/deskpet/codes.txt`.
- If you ever replace the key, put the new public key in `License.publicKey` in `main.swift` and release a new build; old codes will stop working.

