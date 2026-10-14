# Grape brand assets

The mark is a bunch of deep red grapes hanging on the diagonal from a woody
stem, with three dark vine leaves and two tendrils. It is a vector redraw of
the original 128 px raster logo, which had no source file: same composition,
same wine-red berries with a bright highlight, same olive leaf above the
bunch.

## Which file to use

| | |
|---|---|
| `svg/grape-logo.svg` | the full mark for dark backgrounds, with the leaves lifted a little so they do not sink into the background |
| `svg/grape-logo-light.svg` | the same mark for light backgrounds, with the darker leaves and tendrils of the original |
| `svg/grape-icon.svg` | the application icon: nine berries, one leaf and the stem, readable down to 16 px |

The full mark has two dozen berries in front of a darker second layer; below
about 64 px they blur into one red smear. The icon keeps the bunch's silhouette
and colours with fewer, larger berries, so it stays a bunch of grapes in a tray
or a taskbar.

## Where each one ends up

| File | From | Used by |
|---|---|---|
| `png/grape-logo-512.png` | `svg/grape-logo.svg` | the README header, dark theme |
| `png/grape-logo-light-1024.png` | `svg/grape-logo-light.svg` | the README header, light theme |
| `../logo.png` | `svg/grape-icon.svg`, 128 px | the identity button in the top bar (`src/ui/app/view.rs`) |
| `../icons/icon.png` | `svg/grape-icon.svg`, 256 px | the window icon (`src/ui/app/mod.rs`) and the Colony store (`colony.json`) |
| `../icons/icon.ico` | `svg/grape-icon.svg`, 16 to 256 px | the icon embedded in `grape.exe` (`build.rs`) |
| `../logo_16.rgba` | `svg/grape-icon.svg`, 16 px | the tray icon, raw straight RGBA (`src/system_integration/tray/`) |

## Regenerating the PNGs

With librsvg, ImageMagick and Pillow:

```sh
cd assets
rsvg-convert -w 512  brand/svg/grape-logo.svg       -o brand/png/grape-logo-512.png
rsvg-convert -w 1024 brand/svg/grape-logo-light.svg -o brand/png/grape-logo-light-1024.png
rsvg-convert -w 128  brand/svg/grape-icon.svg       -o logo.png
rsvg-convert -w 256  brand/svg/grape-icon.svg       -o icons/icon.png
rsvg-convert -w 16   brand/svg/grape-icon.svg | magick png:- -depth 8 rgba:logo_16.rgba
d=$(mktemp -d)
for s in 16 24 32 48 64 128 256; do rsvg-convert -w $s brand/svg/grape-icon.svg -o "$d/$s.png"; done
python3 - "$d" <<'PY'
import sys
from PIL import Image
sizes = [16, 24, 32, 48, 64, 128, 256]
ims = [Image.open(f"{sys.argv[1]}/{s}.png") for s in sizes]
ims[-1].save("icons/icon.ico", sizes=[(s, s) for s in sizes], append_images=ims[:-1])
PY
```

Each size of the `.ico` is rendered from the SVG rather than scaled down from
the largest one, so the small sizes stay sharp.

Original work, GPL-3.0-or-later with the rest of Grape.
