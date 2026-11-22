#!/usr/bin/env py
"""Build the committed README image assets under docs/readme/.

Sources are real petshot output in website/public/pets/<id>/ (gitignored):
  hero.gif      -> hero.gif
  <trick>.gif   -> trick-<id>-<trick>.gif
  gallery.png   -> pet-<id>.png (96x64 poster thumbnails)
Run `py docs/readme/build-assets.py` from the repo root. It also prints the
pet-grid HTML fragment to stdout (one-line summaries go to stderr).

desktop.png is NOT generated here: it is a real Windows taskbar capture
(run build\\DeskPet.exe --pet mochi, screenshot at ~1 s, crop to 730x205).

On a fresh checkout website/public/pets is gitignored: run petshot first
(windows\\build-pets.ps1 then target\\release\\petshot.exe site build\\pets website).
"""
import html
import json
import sys
from collections import Counter
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "docs" / "readme"
PETS = ROOT / "website" / "public" / "pets"
CREAM = (241, 227, 200)
CREAM_A = (241, 227, 200, 255)


def report(name, path, size=None):
    size = size or Image.open(path).size
    kb = path.stat().st_size / 1024
    print(f"{name}: {size[0]}x{size[1]} {kb:.1f} KB", file=sys.stderr)


def read_frames(path):
    im = Image.open(path)
    frames, durations = [], []
    for i in range(im.n_frames):
        im.seek(i)
        frames.append(im.convert("RGBA").copy())
        durations.append(im.info.get("duration", 50))
    return frames, durations


def drop_cream(frame):
    px = frame.load()
    w, h = frame.size
    for y in range(h):
        for x in range(w):
            if px[x, y] == CREAM_A:
                px[x, y] = (0, 0, 0, 0)
    return frame


def union_bbox(frames):
    boxes = [f.getbbox() for f in frames if f.getbbox()]
    left = min(b[0] for b in boxes)
    top = min(b[1] for b in boxes)
    right = max(b[2] for b in boxes)
    bottom = max(b[3] for b in boxes)
    return left, top, right, bottom


def build_palette(frames):
    counts = Counter()
    for frame in frames:
        for r, g, b, a in frame.getdata():
            if a:
                counts[(r, g, b)] += 1
    colors = [c for c, _ in counts.most_common(255)]
    if len(counts) > 255:
        kept = colors
        lookup = {}

        def nearest(rgb):
            if rgb not in lookup:
                lookup[rgb] = min(
                    kept, key=lambda k: (k[0] - rgb[0]) ** 2 + (k[1] - rgb[1]) ** 2 + (k[2] - rgb[2]) ** 2
                )
            return lookup[rgb]

        for frame in frames:
            px = frame.load()
            for y in range(frame.size[1]):
                for x in range(frame.size[0]):
                    r, g, b, a = px[x, y]
                    if a:
                        px[x, y] = (*nearest((r, g, b)), a)
    flat = [v for c in colors for v in c]
    flat += [0, 0, 0] * (256 - len(colors))
    pal = Image.new("P", (1, 1))
    pal.putpalette(flat)
    return pal, len(colors)


def quantize_frame(frame, pal, tidx):
    q = frame.convert("RGB").quantize(palette=pal, dither=Image.Dither.NONE)
    mask = frame.getchannel("A").point(lambda a: 255 if a == 0 else 0)
    q.paste(tidx, mask=mask)
    return q


def merge_frames(frames, durations):
    out_f, out_d = [], []
    for frame, duration in zip(frames, durations):
        if out_f and frame.tobytes() == out_f[-1].tobytes():
            out_d[-1] += duration
        else:
            out_f.append(frame)
            out_d.append(duration)
    return out_f, out_d


def build_gif(src, dst, margin, fill_first=False):
    frames, src_durations = read_frames(src)
    frames = [drop_cream(f) for f in frames]
    total_src = sum(src_durations)

    box = union_bbox(frames)
    if fill_first:
        union_top = box[1]
        for i, frame in enumerate(frames):
            b = frame.getbbox()
            if b and b[1] <= union_top + 2:
                frames = frames[i:] + frames[:i]
                src_durations = src_durations[i:] + src_durations[:i]
                break
    # Never start on a blank frame: GitHub shows frame 0 as a still image.
    while frames[0].getbbox() is None:
        if len(frames) == 1:
            raise AssertionError(f"{dst.name}: every frame is blank")
        src_durations[1] += src_durations[0]
        frames.pop(0)
        src_durations.pop(0)

    w, h = frames[0].size
    box = (max(0, box[0] - margin), max(0, box[1] - margin), min(w, box[2] + margin), min(h, box[3] + margin))
    cropped_size = (box[2] - box[0], box[3] - box[1])
    frames = [f.crop(box).resize((cropped_size[0] * 2, cropped_size[1] * 2), Image.Resampling.NEAREST) for f in frames]

    pal, ncolors = build_palette(frames)
    tidx = ncolors
    qframes = [quantize_frame(f, pal, tidx) for f in frames]
    # A fully transparent mid-stream frame equals Pillow's disposal background,
    # so the writer emits it as a second global header (corrupts/drops frames).
    # Give each blank frame a unique, still-transparent palette slot to differ.
    for i, q in enumerate(qframes):
        if q.getextrema() == (tidx, tidx):
            rgba = q.getpalette()
            rgba[3 * tidx:3 * tidx + 3] = [1 + i % 200, 2, 3]
            q.putpalette(rgba)
    qframes, durations = merge_frames(qframes, src_durations)

    if tidx == 256:
        raise AssertionError("palette overflow: no transparent slot")

    qframes[0].save(
        dst,
        save_all=True,
        append_images=qframes[1:],
        duration=durations,
        loop=0,
        disposal=2,
        transparency=tidx,
        optimize=False,
    )

    check = Image.open(dst)
    total_out = 0
    for i in range(check.n_frames):
        check.seek(i)
        total_out += check.info.get("duration", 50)
        rgb = check.convert("RGB")
        if CREAM in set(rgb.getdata()):
            raise AssertionError(f"{dst.name}: cream pixel in frame {i}")
    if total_out != total_src:
        raise AssertionError(f"{dst.name}: duration {total_out} != source {total_src}")
    if check.size != (cropped_size[0] * 2, cropped_size[1] * 2):
        raise AssertionError(f"{dst.name}: size {check.size} != 2x {cropped_size}")
    if fill_first:
        check.seek(0)
        b = check.convert("RGBA").getbbox()
        if b is None or b[1] > 2 * margin + 4:
            raise AssertionError(f"{dst.name}: frame 0 does not fill the top: {b}")
    return check.size


def build_pet(pet_id):
    src = PETS / pet_id / "gallery.png"
    im = Image.open(src).convert("RGBA")
    art = im.crop(im.getbbox())
    w, h = art.size
    art = art.resize((w // 2, h // 2), Image.Resampling.NEAREST)
    canvas = Image.new("RGBA", (96, 64), (0, 0, 0, 0))
    canvas.paste(art, ((96 - w // 2) // 2, (64 - h // 2) // 2))
    dst = OUT / f"pet-{pet_id}.png"
    canvas.save(dst, optimize=True)

    check = Image.open(dst).convert("RGBA")
    if check.size != (96, 64):
        raise AssertionError(f"{pet_id}: size {check.size}")
    alpha = check.getchannel("A")
    values = list(alpha.getdata())
    if min(values) != 0 or max(values) == 0:
        raise AssertionError(f"{pet_id}: expected transparent and opaque pixels")
    l, t, r, b = alpha.getbbox()
    if abs(l - (96 - r)) > 1 or abs(t - (64 - b)) > 1:
        raise AssertionError(f"{pet_id}: sprite not centered {alpha.getbbox()}")
    return check.size


def grid_html():
    pets = json.loads((ROOT / "website" / "pets.json").read_text(encoding="utf-8"))
    lines = []
    for shelf in ("critter", "hero", "legend"):
        lines.append(f"<!-- shelf: {shelf} -->")
        lines.append("<table>")
        row = [p for p in pets if p["category"] == shelf]
        for start in range(0, len(row), 3):
            lines.append("<tr>")
            for pet in row[start:start + 3]:
                name = html.escape(pet["name"], quote=False)
                desc = html.escape(pet["description"], quote=False)
                builtin = " · <em>included</em>" if pet.get("builtin") else ""
                cell = (
                    f'<td align="center" valign="top"><img src="docs/readme/pet-{pet["id"]}.png" '
                    f'width="96" height="64" alt="{name}"><br><strong>{name}</strong>{builtin}'
                    f"<br><sub>{desc}</sub></td>"
                )
                lines.append(cell)
            lines.append("</tr>")
        lines.append("</table>")
    return "\n".join(lines)


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    OUT.mkdir(parents=True, exist_ok=True)
    src = PETS / "mochi" / "hero.gif"
    dst = OUT / "hero.gif"
    report("hero.gif", dst, build_gif(src, dst, 0))

    tricks = [
        ("trick-mochi-yarn.gif", "mochi", "yarn"),
        ("trick-ninja-smoke.gif", "ninja", "smoke"),
        ("trick-moona-glaive.gif", "moona", "glaive"),
    ]
    for dst_name, pet_id, trick in tricks:
        dst = OUT / dst_name
        report(dst_name, dst, build_gif(PETS / pet_id / f"{trick}.gif", dst, 2, fill_first=True))

    pets = json.loads((ROOT / "website" / "pets.json").read_text(encoding="utf-8"))
    for pet in pets:
        dst = OUT / f"pet-{pet['id']}.png"
        report(f"pet-{pet['id']}.png", dst, build_pet(pet["id"]))

    # desktop.png comes from a live app capture; see header. Never regenerated here.
    print(grid_html())


if __name__ == "__main__":
    main()