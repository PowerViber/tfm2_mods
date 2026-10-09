"""Round 108: the Coder's second effect sheet 'coder_vfx2': the new functions' effects and lasting marks, and the
data-center HUD (power and GPU meters). Same software-UI palette as coder_vfx (round 106).

  fx_cloud    a cloud node over the target, a packet dropping out of it (cloud_deploy, serverless)
  fx_docker   a container box closing around an ally (docker: untargetable)
  fx_push     a force-push shockwave of arrows (git_push_force, vacuum, quicksort)
  fx_git      a branch-and-merge glyph (cron, rebase, merge_conflict, git_stash, cherry_pick, transaction, ...)
  fx_lock     a padlock snapping shut (ransomware, chmod, sql_injection, cors, deadlock, captcha)
  fx_beam     a CUDA beam slamming down (cuda_kernel, quantum, overfit)
  fx_glitch   RGB-split glitch blocks (deepfake, dns_spoof, phishing, chaos_monkey, quantum's miss)
  fx_trap     a regex trap waiting on the ground: /.*/ in a dashed ring, about 3 s
  buffs       cd_cloud, cd_lock, cd_noheal, cd_honeypot, cd_tether, cd_gate, cd_lb, cd_bloom, cd_cdn, cd_stash,
              cd_pow0..8 (power draw against the breaker), cd_gpu0..8 (GPU load)

Run from the repo root: python3 "Claude outputs/coder/coder_vfx2.py" [--preview]
"""
import json
import math
import os
import random
import sys

from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from coder_vfx import (Cv, rgba, glow, star4, popup, glyph, pack, UI_B, UI_W, UI_D, UI_PANEL, UI_EDGE, CYAN, AMBER,  # noqa: E402
                       RED, ORANGE, GOLD, OUT)

GREEN_OK = rgba('#5fd6a0')
VIOLET = rgba('#b48cff')


def a_(c, a):
    return c[:3] + (int(max(0, min(255, a))),)


# ------------------------------------------------------------------ effects

def cloud(f):
    cv = Cv(40, 40)
    life = 1 - max(0, f - 3) / 3
    for cx, cy, r in ((14, 12, 5), (21, 9, 6), (27, 12, 5), (20, 14, 5)):
        cv.disc(cx, cy, r, a_(UI_W, 235 * life))
    for x in range(10, 32):
        cv.add(x, 17, a_(UI_B, 200 * life))
    y = 18 + f * 3
    if f >= 1:
        cv.disc(20, y, 1.8, a_(UI_B, 255 * life))
        cv.add(20, y - 2, a_(UI_W, 200 * life))
    if f >= 3:
        cv.ring(20, 34, (f - 2) * 2.5, a_(UI_B, 220 * life), 1.0)
    return cv.im


def docker(f):
    cv = Cv(48, 64)
    close = min(1.0, f / 3)
    a = 230 if f < 6 else 230 - (f - 6) * 60
    w, h = 26, 34
    x0, y0 = 24 - w // 2, 54 - h
    for x in range(x0, x0 + w):
        for y in (y0, y0 + h):
            cv.add(x, y, a_(UI_B, a))
    for y in range(y0, y0 + h + 1):
        for x in (x0, x0 + w - 1):
            cv.add(x, y, a_(UI_B, a))
    for k in range(1, 4):   # the container's ribs
        x = x0 + k * w // 4
        for y in range(y0 + 1, y0 + h):
            if y < y0 + int(h * close):
                cv.add(x, y, a_(UI_D, a * 0.8))
    for x in range(x0 + 1, x0 + w - 1):   # the lid sliding shut
        for y in range(y0 + 1, y0 + 1 + int(4 * close)):
            cv.add(x, y, a_(UI_EDGE, a * 0.6))
    return cv.im


def push(f):
    cv = Cv(64, 40)
    life = 1 - f / 6
    r = 6 + f * 4
    for k in range(8):
        ang = k / 8 * math.tau
        x, y = 32 + math.cos(ang) * r, 20 + math.sin(ang) * r * 0.5
        ex, ey = 32 + math.cos(ang) * (r + 4), 20 + math.sin(ang) * (r + 4) * 0.5
        cv.line(x, y, ex, ey, a_(UI_W, 240 * life))
        cv.add(ex, ey, a_(AMBER, 255 * life))
    cv.ring(32, 20, r * 0.8, a_(UI_B, 150 * life), 1.0)
    return cv.im


def git(f):
    cv = Cv(32, 40)
    life = 1 - max(0, f - 3) / 3
    a = 245 * life
    cv.line(10, 30, 10, 8, a_(UI_W, a))
    cv.line(10, 22, 22, 14, a_(UI_B, a))
    cv.line(22, 14, 22, 8, a_(UI_B, a))
    for x, y, c in ((10, 30, UI_W), (10, 8, UI_W), (22, 8, AMBER)):
        cv.disc(x, y, 2.4 if f % 2 == 0 else 2.0, a_(c, a))
    if f >= 2:
        star4(cv, 22, 8, a_(UI_W, a))
    return cv.im


def lock(f):
    cv = Cv(32, 40)
    life = 1 - max(0, f - 3) / 3
    a = 250 * life
    drop = max(0, 4 - f * 2)
    cv.ring(16, 14 - drop, 4.5, a_((200, 205, 215), a), 1.3)
    for y in range(17, 27):
        for x in range(10, 23):
            cv.add(x, y, a_(GOLD if (x + y) % 6 else (255, 240, 180), a))
    cv.add(16, 21, (60, 50, 20, int(a)))
    cv.add(16, 22, (60, 50, 20, int(a)))
    if f == 2:
        glow(cv, 16, 21, 8, UI_W, 160)
    return cv.im


def beam(f):
    cv = Cv(32, 80)
    life = 1 - max(0, f - 2) / 4
    w = [2, 5, 7, 6, 4, 2][f]
    for y in range(0, 70):
        for dx in range(-w, w + 1):
            k = 1 - abs(dx) / (w + 1)
            cv.add(16 + dx, y, a_(UI_W if abs(dx) < w / 2 else VIOLET, 255 * life * k))
    cv.ring(16, 70, 4 + f * 2, a_(VIOLET, 230 * life), 1.2)
    if f >= 1:
        glow(cv, 16, 70, 8, UI_W, 200 * life)
    return cv.im


def glitch(f):
    cv = Cv(40, 48)
    rnd = random.Random(808 + f)
    life = 1 - max(0, f - 3) / 3
    for _ in range(7):
        x, y, w = rnd.randint(6, 28), rnd.randint(8, 38), rnd.randint(3, 9)
        for xx in range(w):
            cv.add(x + xx, y, a_((255, 70, 90), 220 * life))
            cv.add(x + xx + 2, y + 1, a_((70, 220, 255), 220 * life))
            cv.add(x + xx + 1, y + 2, a_((120, 255, 140), 160 * life))
    return cv.im


def trap(f):
    cv = Cv(40, 24)
    a = 230 if f % 2 == 0 else 170
    for j in range(36):
        if j % 3 == 2:
            continue
        ang = j / 36 * math.tau
        cv.add(20 + math.cos(ang) * 15, 12 + math.sin(ang) * 6, a_(AMBER, a))
    text = '/.*/'
    x = 20 - (len(text) * 4) // 2
    for i, ch in enumerate(text):
        glyph(cv, ch, x + i * 4, 9, a_(UI_W, a), a)
    return cv.im


# ------------------------------------------------------------------ lasting marks (buffs: 48 x 96 over the unit)

def cloud_buff(f):
    cv = Cv(48, 96)
    dx = (f % 4) - 1.5
    for cx, cy, r in ((19, 7, 3), (24, 5, 4), (29, 7, 3), (24, 8, 3)):
        cv.disc(cx + dx, cy, r, a_(UI_W, 220))
    for x in range(16, 33):
        cv.add(x + dx, 10, a_(UI_B, 180))
    return cv.im


def lock_buff(f):
    cv = Cv(48, 96)
    cv.ring(24, 9, 3, a_((200, 205, 215), 255), 1.0)
    for y in range(11, 18):
        for x in range(19, 30):
            cv.put(x, y, a_(RED if (x + y + f) % 5 else (255, 200, 200), 255))
    cv.put(24, 14, (40, 10, 10, 255))
    return cv.im


def noheal_buff(f):
    cv = Cv(48, 96)
    a = 255 if f % 2 == 0 else 200
    for d in range(-3, 4):
        cv.put(24 + d, 12, a_(GREEN_OK, a))
        cv.put(24, 12 + d, a_(GREEN_OK, a))
    cv.ring(24, 12, 6, a_(RED, a), 1.0)
    for d in range(-4, 5):
        cv.put(24 + d, 12 + d, a_(RED, a))
    return cv.im


def honeypot_buff(f):
    cv = Cv(48, 96)
    for y in range(40, 50):
        w = 6 - abs(y - 45) // 2
        for x in range(24 - w, 24 + w + 1):
            cv.put(x + 12, y, a_(GOLD, 240))
    cv.put(36, 39 - (f % 3), a_((255, 230, 120), 255))
    for k in range(3):
        ang = f / 6 * math.tau + k * 2.1
        cv.add(24 + math.cos(ang) * 14, 46 + math.sin(ang) * 5, a_(AMBER, 200))
    return cv.im


def tether_buff(f):
    cv = Cv(48, 96)
    for k in range(4):
        y = 6 + ((f * 3 + k * 6) % 22)
        cv.add(24, y, a_(UI_B, 230))
        cv.add(24, y + 1, a_(UI_W, 180))
    cv.ring(24, 30, 3 + (f % 2), a_(UI_B, 220), 1.0)
    return cv.im


def gate_buff(f):
    cv = Cv(48, 56)
    for x in range(14, 35):
        cv.add(x, 50, a_(UI_B, 180))
    for x in (16, 32):
        for y in range(40, 51):
            cv.add(x, y, a_(UI_W, 200))
    for x in range(16, 33):
        cv.add(x, 40, a_(UI_W, 200))
    cv.add(24, 44 - (f % 4), a_(AMBER, 255))
    return cv.im


def lb_buff(f):
    cv = Cv(48, 96)
    tilt = [-1, 0, 1, 0][f % 4]
    cv.line(17, 10 + tilt, 31, 10 - tilt, a_(UI_W, 240))
    cv.line(24, 10, 24, 17, a_(UI_W, 240))
    for x in (17, 31):
        cv.ring(x, 13 + (tilt if x == 17 else -tilt), 2, a_(UI_B, 240), 1.0)
    return cv.im


def bloom_buff(f):
    cv = Cv(48, 96)
    for k in range(5):
        ang = k / 5 * math.tau + f * 0.2
        cv.add(24 + math.cos(ang) * 10, 46 + math.sin(ang) * 4, a_(CYAN, 150))
    return cv.im


def cdn_buff(f):
    cv = Cv(48, 56)
    for k in range(3):
        y = 40 + k * 4
        x0 = 8 + ((f * 4 + k * 5) % 10)
        cv.line(x0, y, x0 + 8, y, a_(UI_W, 200 - k * 40))
    return cv.im


def stash_buff(f):
    cv = Cv(48, 96)
    for y in range(40, 47):
        for x in range(6, 13):
            cv.put(x, y, a_(UI_PANEL, 230) if 6 < x < 12 and 40 < y < 46 else a_(UI_B, 255))
    cv.put(9, 43, a_(AMBER, 255 if f % 2 == 0 else 180))
    return cv.im


def pow_bar(n):
    """Power draw against the breaker, under the RAM and storage rows: amber, red at the top."""
    cv = Cv(48, 96)
    for k in range(8):
        c = (AMBER if n < 7 else RED) if k < n else rgba('#33291a', 220)
        cv.put(14 + k * 2.5, 21, c)
        cv.put(15 + k * 2.5, 21, c)
    return cv.im


def gpu_bar(n):
    cv = Cv(48, 96)
    for k in range(8):
        c = rgba('#7ef0b8') if k < n else rgba('#1d3328', 220)
        cv.put(14 + k * 2.5, 23, c)
    return cv.im


def all_anims():
    A = {}
    A['fx_cloud'] = ([cloud(f) for f in range(6)], 0.06)
    A['fx_docker'] = ([docker(f) for f in range(8)], 0.18)
    A['fx_push'] = ([push(f) for f in range(6)], 0.05)
    A['fx_git'] = ([git(f) for f in range(6)], 0.06)
    A['fx_lock'] = ([lock(f) for f in range(6)], 0.06)
    A['fx_beam'] = ([beam(f) for f in range(6)], 0.05)
    A['fx_glitch'] = ([glitch(f) for f in range(6)], 0.05)
    A['fx_trap'] = ([trap(f) for f in range(4)], 0.75)
    A['cloud'] = ([cloud_buff(f) for f in range(4)], 0.12)
    A['lock'] = ([lock_buff(f) for f in range(2)], 0.2)
    A['noheal'] = ([noheal_buff(f) for f in range(2)], 0.2)
    A['honeypot'] = ([honeypot_buff(f) for f in range(6)], 0.1)
    A['tether'] = ([tether_buff(f) for f in range(8)], 0.06)
    A['gate'] = ([gate_buff(f) for f in range(4)], 0.12)
    A['lb'] = ([lb_buff(f) for f in range(4)], 0.15)
    A['bloom'] = ([bloom_buff(f) for f in range(6)], 0.12)
    A['cdn'] = ([cdn_buff(f) for f in range(4)], 0.08)
    A['stash'] = ([stash_buff(f) for f in range(2)], 0.3)
    for n in range(9):
        A[f'pow{n}'] = ([pow_bar(n)], 0.1)
        A[f'gpu{n}'] = ([gpu_bar(n)], 0.1)
    return A


def preview(A, folder):
    bg = (54, 74, 60, 255)
    keys = [k for k in A if not k.startswith(('pow', 'gpu'))]
    rows = []
    for k in keys:
        ims = A[k][0]
        r = Image.new('RGBA', (sum(i.width + 4 for i in ims), max(i.height for i in ims)), bg)
        x = 0
        for i in ims:
            r.alpha_composite(i, (x, (r.height - i.height) // 2))
            x += i.width + 4
        rows.append(r)
    out = Image.new('RGBA', (max(r.width for r in rows), sum(r.height + 4 for r in rows)), bg)
    y = 0
    for r in rows:
        out.alpha_composite(r, (0, y))
        y += r.height + 4
    out.resize((out.width * 2, out.height * 2), Image.NEAREST).save(os.path.join(folder, 'vfx2.png'))


if __name__ == '__main__':
    A = all_anims()
    sheet, fan = pack(A)
    if '--preview' in sys.argv:
        preview(A, os.path.join(HERE, 'preview'))
    sheet.save(os.path.join(OUT, 'coder_vfx2#sheet.png'), optimize=True)
    with open(os.path.join(OUT, 'coder_vfx2#anim.fanim'), 'w', encoding='utf-8') as fh:
        json.dump(fan, fh, separators=(',', ':'))
    print('coder_vfx2', sheet.size, len(fan['anims']), 'anims')
