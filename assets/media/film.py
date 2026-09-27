"""Composites captured Mercury frames into a product film.

Everything is drawn here rather than in ffmpeg filters: easing, camera moves,
title cards, callouts and transitions all need to be choreographed against
each other, and one compositor with full control beats a filter graph.
"""
import math, pathlib, sys
from PIL import Image, ImageDraw, ImageFont, ImageFilter

ROOT = pathlib.Path(__file__).parent
FRAMES = ROOT / "frames"
OUT = ROOT / "out"
import os
REPO = pathlib.Path(os.environ.get("MERCURY_REPO") or (sys.argv[1] if len(sys.argv) > 1 else "."))

W, H, FPS = 1600, 1000, 30           # render size / rate
SHOT = (1600, 1000)                   # captured app frame size

# Mercury's own palette (src/ui/theme.rs) so the film matches the product
BG        = (11, 13, 16)
PANEL     = (18, 21, 25)
INK       = (228, 232, 238)
MUTED     = (152, 161, 174)
ACCENT    = (167, 139, 250)
ACCENT_HI = (196, 181, 253)
GREEN     = (74, 222, 128)
AMBER     = (251, 191, 36)
ROSE      = (251, 113, 133)

F = ROOT.parent.parent / "fonts"      # filled in by the caller
def font(name, size):
    return ImageFont.truetype(str(FONTS / name), size)

FONTS = REPO / "assets" / "fonts"
SEMI = "Inter-SemiBold.subset.ttf"
REG  = "Inter-Regular.subset.ttf"
MONO = "JetBrainsMono-Regular.subset.ttf"

# ---------------------------------------------------------------- easing
def clamp(x, a=0.0, b=1.0):
    return max(a, min(b, x))

def ease_out(t):      # decelerate — for things arriving
    return 1 - (1 - clamp(t)) ** 3

def ease_in_out(t):
    t = clamp(t)
    return 4 * t ** 3 if t < 0.5 else 1 - (-2 * t + 2) ** 3 / 2

def ease_out_back(t):
    t = clamp(t)
    c1, c3 = 1.70158, 2.70158
    return 1 + c3 * (t - 1) ** 3 + c1 * (t - 1) ** 2

# ---------------------------------------------------------------- helpers
_cache = {}
def shot(scene, i):
    """A captured frame, clamped to the scene's range."""
    files = _cache.get(scene)
    if files is None:
        files = sorted(FRAMES.glob(f"{scene}-*.png"))
        _cache[scene] = files
    if not files:
        raise SystemExit(f"no frames for scene {scene!r}")
    return Image.open(files[max(0, min(i, len(files) - 1))]).convert("RGB")

def rounded(img, r):
    mask = Image.new("L", img.size, 0)
    ImageDraw.Draw(mask).rounded_rectangle([0, 0, img.size[0] - 1, img.size[1] - 1], r, fill=255)
    out = img.convert("RGBA")
    out.putalpha(mask)
    return out

def shadow(size, r, blur, alpha):
    pad = blur * 3
    layer = Image.new("RGBA", (size[0] + pad * 2, size[1] + pad * 2), (0, 0, 0, 0))
    ImageDraw.Draw(layer).rounded_rectangle(
        [pad, pad, pad + size[0], pad + size[1]], r, fill=(0, 0, 0, alpha))
    return layer.filter(ImageFilter.GaussianBlur(blur)), pad

def backdrop(t):
    """Slow aurora wash so static beats still breathe."""
    bg = Image.new("RGB", (W, H), BG)
    glow = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    d = ImageDraw.Draw(glow)
    for i, (col, rx, ry, ph) in enumerate([
            (ACCENT, 620, 430, 0.0), ((80, 140, 255), 520, 380, 2.1)]):
        cx = W * (0.30 + 0.16 * math.sin(t * 0.35 + ph)) + i * 420
        cy = H * (0.28 + 0.13 * math.cos(t * 0.29 + ph))
        d.ellipse([cx - rx, cy - ry, cx + rx, cy + ry], fill=col + (34,))
    glow = glow.filter(ImageFilter.GaussianBlur(190))
    bg = Image.alpha_composite(bg.convert("RGBA"), glow)
    return bg.convert("RGB")

def camera(frame, zoom, cx, cy):
    """Crop a window around (cx, cy) in 0..1 space and fill the canvas."""
    zoom = max(1.0, zoom)
    cw, ch = SHOT[0] / zoom, SHOT[1] / zoom
    x = clamp(cx * SHOT[0] - cw / 2, 0, SHOT[0] - cw)
    y = clamp(cy * SHOT[1] - ch / 2, 0, SHOT[1] - ch)
    return frame.crop((int(x), int(y), int(x + cw), int(y + ch))).resize((W, H), Image.LANCZOS)

# ---------------------------------------------------------------- type
def text_w(d, s, f):
    return d.textbbox((0, 0), s, font=f)[2]

def draw_text(canvas, xy, s, f, fill, anchor="la", alpha=255, shift=0):
    if alpha <= 0:
        return
    layer = Image.new("RGBA", canvas.size, (0, 0, 0, 0))
    ImageDraw.Draw(layer).text((xy[0], xy[1] + shift), s, font=f, fill=fill + (alpha,), anchor=anchor)
    canvas.alpha_composite(layer)

def kinetic(canvas, xy, s, f, fill, t, anchor="la", stagger=0.022):
    """Words rise and fade in one after another."""
    d = ImageDraw.Draw(canvas)
    total = text_w(d, s, f)
    x = xy[0] - total / 2 if anchor.startswith("m") else xy[0]
    for i, word in enumerate(s.split(" ")):
        p = ease_out((t - i * stagger) / 0.34)
        draw_text(canvas, (x, xy[1]), word, f, fill,
                  alpha=int(255 * clamp(p)), shift=int((1 - p) * 26))
        x += text_w(d, word + " ", f)

def chip(canvas, xy, label, color, t):
    """A small pill that pops in — used to name what's on screen."""
    p = ease_out_back(t / 0.42)
    if p <= 0.01:
        return
    f = font(SEMI, 27)
    d = ImageDraw.Draw(canvas)
    tw = text_w(d, label, f)
    pw, ph = tw + 42, 52
    layer = Image.new("RGBA", canvas.size, (0, 0, 0, 0))
    ld = ImageDraw.Draw(layer)
    a = int(235 * clamp(p))
    x, y = xy[0], xy[1] + int((1 - p) * 18)
    ld.rounded_rectangle([x, y, x + pw * p, y + ph], ph // 2, fill=color + (int(38 * clamp(p)),),
                         outline=color + (a,), width=2)
    if p > 0.55:
        ld.text((x + pw * p / 2, y + ph / 2), label, font=f, fill=color + (a,), anchor="mm")
    canvas.alpha_composite(layer)

SCRIM_H = 300
_SCRIM = None

def _scrim():
    """Built once: a per-frame pixel loop is 1600x300 too many times."""
    global _SCRIM
    if _SCRIM is None:
        ramp = Image.linear_gradient("L").resize((1, SCRIM_H))       # 0 at top
        ramp = ramp.point(lambda v: int(215 * (v / 255) ** 1.5))
        band = Image.new("RGBA", (W, H), (0, 0, 0, 0))
        band.paste(Image.new("RGBA", (W, SCRIM_H), (0, 0, 0, 255)), (0, H - SCRIM_H),
                   ramp.resize((W, SCRIM_H)))
        _SCRIM = band
    return _SCRIM


def lower_third(canvas, text, color, t, kicker=None):
    """Caption band across the bottom: scrim, accent rule, words that rise.

    Placing captions over whatever happens to be empty in the footage works
    until the footage changes. A fixed band always reads.
    """
    p = ease_out(t / 0.45)
    if p <= 0.01:
        return
    scrim = _scrim().copy()
    scrim.putalpha(scrim.getchannel("A").point(lambda v: int(v * p)))
    canvas.alpha_composite(scrim)
    bar_y = canvas.size[1] - 150
    underline(canvas, 88, bar_y - 4, 64, color, t, 6)
    if kicker:
        draw_text(canvas, (88, bar_y + 22), kicker.upper(), font(SEMI, 22), color,
                  alpha=int(230 * p), shift=int((1 - p) * 10))
        kinetic(canvas, (88, bar_y + 56), text, font(SEMI, 44), INK, t - 0.1)
    else:
        kinetic(canvas, (88, bar_y + 26), text, font(SEMI, 46), INK, t - 0.08)

def underline(canvas, x, y, w, color, t, thick=5):
    p = ease_out(t / 0.5)
    if p <= 0:
        return
    layer = Image.new("RGBA", canvas.size, (0, 0, 0, 0))
    ImageDraw.Draw(layer).rounded_rectangle(
        [x, y, x + w * p, y + thick], thick // 2, fill=color + (255,))
    canvas.alpha_composite(layer)

def spotlight(canvas, box, t, color=ACCENT):
    """Dim everything but `box`, then ring it — points the eye."""
    p = ease_in_out(t / 0.45)
    if p <= 0.01:
        return
    x0, y0, x1, y1 = box
    dark = Image.new("RGBA", canvas.size, (0, 0, 0, int(105 * p)))
    hole = Image.new("L", canvas.size, 255)
    ImageDraw.Draw(hole).rounded_rectangle([x0, y0, x1, y1], 16, fill=0)
    dark.putalpha(Image.eval(hole, lambda v: int(v / 255 * 105 * p)))
    canvas.alpha_composite(dark)
    ring = Image.new("RGBA", canvas.size, (0, 0, 0, 0))
    ImageDraw.Draw(ring).rounded_rectangle([x0, y0, x1, y1], 16,
                                           outline=color + (int(230 * p),), width=3)
    canvas.alpha_composite(ring)

def keycap(canvas, xy, keys, t):
    """⌘ K style keycaps that press in."""
    p = ease_out_back(t / 0.4)
    if p <= 0.01:
        return
    f = font(SEMI, 30)
    d = ImageDraw.Draw(canvas)
    x, y = xy
    layer = Image.new("RGBA", canvas.size, (0, 0, 0, 0))
    ld = ImageDraw.Draw(layer)
    for k in keys:
        kw = max(54, text_w(d, k, f) + 30)
        yy = y + int((1 - p) * 14)
        ld.rounded_rectangle([x, yy, x + kw, yy + 54], 12,
                             fill=(30, 34, 41, int(240 * clamp(p))),
                             outline=(70, 78, 92, int(255 * clamp(p))), width=2)
        ld.text((x + kw / 2, yy + 27), k, font=f, fill=INK + (int(255 * clamp(p)),), anchor="mm")
        x += kw + 12
    canvas.alpha_composite(layer)

# ---------------------------------------------------------------- staging
LOGO = Image.open(REPO / "assets" / "icons" / "icon.png").convert("RGBA")

def framed(canvas, app_img, scale, t_in=1.0, lift=0):
    """The app as a floating window: rounded, shadowed, slightly lifted."""
    w = int(W * scale)
    h = int(w * SHOT[1] / SHOT[0])
    img = app_img.resize((w, h), Image.LANCZOS)
    img = rounded(img, 18)
    sh, pad = shadow((w, h), 18, 40, int(150 * t_in))
    x, y = (W - w) // 2, (H - h) // 2 + lift
    canvas.alpha_composite(sh, (x - pad, y - pad + 18))
    if t_in < 1.0:
        img.putalpha(img.getchannel("A").point(lambda v: int(v * t_in)))
    canvas.alpha_composite(img, (x, y))
    return (x, y, x + w, y + h)

def full(canvas, app_img):
    canvas.alpha_composite(app_img.convert("RGBA"), (0, 0))

# ---------------------------------------------------------------- scenes
def sc_open(t, d):
    c = backdrop(t).convert("RGBA")
    # logo lands, then the wordmark and tagline rise
    p = ease_out_back(t / 0.9)
    size = int(150 * clamp(p, 0.01, 1.2))
    lg = LOGO.resize((size, size), Image.LANCZOS)
    glow = lg.filter(ImageFilter.GaussianBlur(26))
    gy = int(H * 0.33) - size // 2
    if t < 2.6:
        c.alpha_composite(glow, (W // 2 - size // 2, gy))
    c.alpha_composite(lg, (W // 2 - size // 2, gy))
    kinetic(c, (W // 2, int(H * 0.50)), "Mercury", font(SEMI, 112), INK, t - 0.55, "ma", 0.0)
    kinetic(c, (W // 2, int(H * 0.635)), "A fast, minimal API client", font(REG, 40), MUTED, t - 0.95, "ma")
    underline(c, W // 2 - 90, int(H * 0.72), 180, ACCENT, t - 1.35, 4)
    kinetic(c, (W // 2, int(H * 0.775)), "Your requests are just files", font(REG, 32), ACCENT_HI, t - 1.6, "ma")
    if t > d - 0.5:                      # ease out of the card
        fade(c, (t - (d - 0.5)) / 0.5)
    return c

def fade(canvas, p, col=(0, 0, 0)):
    canvas.alpha_composite(Image.new("RGBA", canvas.size, col + (int(255 * clamp(p)),)))

def sc_tree(t, d):
    c = backdrop(6 + t).convert("RGBA")
    box = framed(c, shot("post", int(t * 5)), 0.80, ease_out(t / 0.6), lift=-60)
    lower_third(c, "Plain JSON files, in a folder you choose", ACCENT, t - 0.35, "Collections")
    if t > 1.1:
        x0 = box[0] + int((box[2] - box[0]) * 0.005)
        spotlight(c, (x0, box[1] + int((box[3] - box[1]) * 0.12),
                      x0 + int((box[2] - box[0]) * 0.165), box[3] - 12), t - 1.1)
    return c

def sc_type(t, d):
    f = int(t * 26)
    c = Image.new("RGBA", (W, H), BG + (255,))
    full(c, camera(shot("type", f), 1.9, 0.42, 0.085))
    lower_third(c, "{{BASE}} comes from your .env", ACCENT, t - 0.7, "Environments")
    return c

def sc_send(t, d):
    f = int(t * 26)
    c = Image.new("RGBA", (W, H), BG + (255,))
    z = 1.9 - ease_in_out(clamp((t - 0.45) / 1.15)) * 0.9      # pull back to reveal
    cx = 0.42 + ease_in_out(clamp((t - 0.45) / 1.15)) * 0.10
    cy = 0.085 + ease_in_out(clamp((t - 0.45) / 1.15)) * 0.39
    full(c, camera(shot("send", f), z, cx, cy))
    if t > 1.5:
        lower_third(c, "Send. Read. Nothing in between.", GREEN, t - 1.5, "200 OK · 354 ms")
    return c

def sc_find(t, d):
    f = int(t * 26)
    c = Image.new("RGBA", (W, H), BG + (255,))
    full(c, camera(shot("find", f), 1.58, 0.79, 0.30))
    keycap(c, (70, 70), ["⌘", "F"], t - 0.15)
    lower_third(c, "Find in response — every match, counted", ACCENT, t - 0.6, "⌘ F")
    return c

def sc_post(t, d):
    c = backdrop(20 + t).convert("RGBA")
    framed(c, shot("post", int(t * 12) + 14), 0.80, ease_out(t / 0.5), lift=-60)
    lower_third(c, "JSON, form or text — the header follows", ACCENT, t - 0.3, "Request body")
    return c

def sc_auth(t, d):
    f = int(t * 13)
    c = Image.new("RGBA", (W, H), BG + (255,))
    full(c, camera(shot("auth", f), 1.5, 0.36, 0.26))
    lower_third(c, "Auth is a header you can read", AMBER, t - 0.4, "No hidden state")
    return c

def sc_palette(t, d):
    f = int(t * 20)
    c = Image.new("RGBA", (W, H), BG + (255,))
    full(c, camera(shot("palette", f), 1.42, 0.5, 0.36))
    keycap(c, (70, 70), ["⌘", "K"], t - 0.1)
    lower_third(c, "Every request and every command", ACCENT, t - 0.55, "⌘ K")
    return c

def sc_env(t, d):
    f = int(t * 16)
    c = Image.new("RGBA", (W, H), BG + (255,))
    full(c, camera(shot("env", f), 1.75, 0.88, 0.055))
    lower_third(c, "Production is always red", ROSE, t - 0.9, "Environments")
    return c

def sc_light(t, d):
    f = int(t * 14)
    c = Image.new("RGBA", (W, H), BG + (255,))
    app = shot("light", f)
    full(c, camera(app, 1.14, 0.5, 0.42))
    lower_third(c, "Light or dark, follows your system", ACCENT, t - 0.3, "⌘ D")
    return c

def sc_end(t, d):
    c = backdrop(40 + t).convert("RGBA")
    size = 118
    lg = LOGO.resize((size, size), Image.LANCZOS)
    p = ease_out(t / 0.7)
    c.alpha_composite(lg.filter(ImageFilter.GaussianBlur(24)), (W // 2 - size // 2, int(H * 0.235)))
    c.alpha_composite(lg, (W // 2 - size // 2, int(H * 0.235)))
    kinetic(c, (W // 2, int(H * 0.44)), "Small, boring, fast", font(SEMI, 74), INK, t - 0.3, "ma")
    # the install line, in a terminal-ish slab
    f = font(MONO, 34)
    d2 = ImageDraw.Draw(c)
    cmd = "brew install Harry-kp/tap/mercury"
    tw = text_w(d2, cmd, f)
    bw, bh = tw + 76, 78
    q = ease_out((t - 0.85) / 0.55)
    if q > 0.01:
        x, y = (W - bw) // 2, int(H * 0.575) + int((1 - q) * 22)
        layer = Image.new("RGBA", c.size, (0, 0, 0, 0))
        ld = ImageDraw.Draw(layer)
        ld.rounded_rectangle([x, y, x + bw, y + bh], 16, fill=(14, 17, 20, int(240 * q)),
                             outline=(48, 55, 66, int(255 * q)), width=2)
        ld.text((x + 30, y + bh / 2), "$", font=f, fill=ACCENT + (int(255 * q),), anchor="lm")
        ld.text((x + 60, y + bh / 2), cmd, font=f, fill=INK + (int(255 * q),), anchor="lm")
        c.alpha_composite(layer)
    kinetic(c, (W // 2, int(H * 0.735)), "github.com/Harry-kp/mercury", font(REG, 33), MUTED, t - 1.3, "ma")
    if t > d - 0.6:
        fade(c, (t - (d - 0.6)) / 0.6)
    return c

# ---------------------------------------------------------------- timeline
XF = 0.42                                  # crossfade length, seconds
TIMELINE = [
    (sc_open,    3.6),
    (sc_tree,    3.5),
    (sc_type,    2.9),
    (sc_send,    3.4),
    (sc_find,    3.6),
    (sc_post,    3.0),
    (sc_auth,    2.7),
    (sc_palette, 3.4),
    (sc_env,     3.0),
    (sc_light,   3.2),
    (sc_end,     4.2),
]

def build():
    cuts, t = [], 0.0
    for fn, d in TIMELINE:
        cuts.append((t, d, fn))
        t += d - XF                        # scenes overlap by the crossfade
    return cuts, t + XF

CUTS, TOTAL = build()

def render(t):
    live = [(t0, d, fn) for (t0, d, fn) in CUTS if t0 <= t < t0 + d]
    if not live:
        live = [CUTS[-1]]
    t0, d, fn = live[0]
    out = fn(clamp(t - t0, 0, d), d)
    for t0, d, fn in live[1:]:             # blend the incoming scene over it
        nxt = fn(clamp(t - t0, 0, d), d)
        a = ease_in_out((t - t0) / XF)
        out = Image.blend(out, nxt, clamp(a))
    return out.convert("RGB")

if __name__ == "__main__":
    OUT.mkdir(exist_ok=True)
    for f in OUT.glob("*.png"):
        f.unlink()
    n = int(TOTAL * FPS)
    for i in range(n):
        render(i / FPS).save(OUT / f"{i:05}.png")
        if i % 30 == 0:
            print(f"  {i}/{n}  {i/FPS:5.1f}s", flush=True)
    print(f"rendered {n} frames, {TOTAL:.1f}s")

# ---------------------------------------------------------------------------
# How the footage was captured
#
# The app frames are the real binary, not a mock: a temporary `src/shots.rs`
# drives MercuryApp through a scripted beat list and writes one PNG per
# logical frame from the app's own framebuffer (ViewportCommand::Screenshot),
# so typing, sending and highlighting are all genuine. That harness is not
# committed — it exists only while a film is being cut. This file is kept so
# the edit itself is reproducible and reviewable.
