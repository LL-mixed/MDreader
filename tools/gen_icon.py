#!/usr/bin/env python3
"""Generate MDreader launcher icons (all densities) with Pillow.

Design: blue gradient background, white document card with a markdown '#'
heading and a few text lines — reads clearly as a markdown reader.
Full-bleed so the launcher can apply its own mask; same PNG used for round.

Run:  python3 tools/gen_icon.py   (requires Pillow: pip3 install --user Pillow)
"""
import os
from PIL import Image, ImageDraw, ImageFont

FONT_FALLBACKS = [
    "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
    "/Library/Fonts/Arial Bold.ttf",
    "/System/Library/Fonts/Helvetica.ttc",
]

# density -> px (legacy launcher icon sizes)
DENSITIES = {
    "mdpi": 48,
    "hdpi": 72,
    "xhdpi": 96,
    "xxhdpi": 144,
    "xxxhdpi": 192,
}

RES_DIR = os.path.join(os.path.dirname(__file__), "..", "app", "src", "main", "res")

TOP = (30, 58, 138)      # #1E3A8A deep blue
BOTTOM = (59, 130, 246)  # #3B82F6 bright blue
CARD = (255, 255, 255)
INK = (30, 58, 138)      # dark navy for '#'
LINE = (203, 213, 225)   # slate-300 text lines


def lerp(a, b, t):
    return tuple(int(a[i] + (b[i] - a[i]) * t) for i in range(3))


def load_font(size):
    for path in FONT_FALLBACKS:
        if os.path.exists(path):
            try:
                return ImageFont.truetype(path, size)
            except Exception:
                continue
    return ImageFont.load_default()


def make_icon(s):
    img = Image.new("RGB", (s, s), TOP)
    px = img.load()
    for y in range(s):
        c = lerp(TOP, BOTTOM, y / max(s - 1, 1))
        for x in range(s):
            px[x, y] = c
    d = ImageDraw.Draw(img, "RGBA")

    cw, ch = s * 0.56, s * 0.68
    left = s / 2 - cw / 2
    top = (s - ch) / 2
    radius = s * 0.10
    d.rounded_rectangle([left, top, left + cw, top + ch], radius=radius, fill=CARD)

    pad = s * 0.10
    inner_left = left + pad
    inner_w = cw - 2 * pad

    font = load_font(int(s * 0.30))
    d.text((inner_left, top + pad * 0.55), "#", font=font, fill=INK)

    line_top = top + pad * 0.55 + s * 0.30 + s * 0.05
    lh = max(int(s * 0.045), 2)
    gap = s * 0.045
    widths = [0.62, 0.82, 0.45]
    for i, wf in enumerate(widths):
        y = line_top + i * (lh + gap)
        if y + lh > top + ch - pad:
            break
        d.rounded_rectangle(
            [inner_left, y, inner_left + inner_w * wf, y + lh],
            radius=lh / 2,
            fill=LINE,
        )
    return img


def main():
    for density, s in DENSITIES.items():
        out_dir = os.path.join(RES_DIR, "mipmap-" + density)
        os.makedirs(out_dir, exist_ok=True)
        img = make_icon(s)
        img.save(os.path.join(out_dir, "ic_launcher.png"))
        img.save(os.path.join(out_dir, "ic_launcher_round.png"))
        print(f"{density}: {s}x{s} -> {out_dir}")


if __name__ == "__main__":
    main()
