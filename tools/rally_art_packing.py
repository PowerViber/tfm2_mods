"""Deterministic lossless rectangle placement; never trims, rotates or edits pixels."""


def pack(images, width=2048):
    candidates = []
    for order in (
        sorted(range(len(images)), key=lambda i: (-images[i].height, -images[i].width, i)),
        sorted(range(len(images)), key=lambda i: (-images[i].width * images[i].height, -images[i].height, i)),
    ):
        for bottom_first in (False, True):
            free = [(0, 0, width, 2048)]
            positions = {}
            for i in order:
                w, h = images[i].size
                fits = [(y+h if bottom_first else min(fw-w, fh-h), y, x, fw*fh-w*h)
                        for x, y, fw, fh in free if w <= fw and h <= fh]
                if not fits:
                    break
                _, y, x, _ = min(fits)
                positions[i] = (x, y)
                split = []
                for fx, fy, fw, fh in free:
                    if x >= fx+fw or x+w <= fx or y >= fy+fh or y+h <= fy:
                        split.append((fx, fy, fw, fh))
                        continue
                    if x > fx: split.append((fx, fy, x-fx, fh))
                    if x+w < fx+fw: split.append((x+w, fy, fx+fw-x-w, fh))
                    if y > fy: split.append((fx, fy, fw, y-fy))
                    if y+h < fy+fh: split.append((fx, y+h, fw, fy+fh-y-h))
                unique = sorted(set(split))
                free = [a for a in unique if not any(a != b and a[0] >= b[0] and a[1] >= b[1]
                        and a[0]+a[2] <= b[0]+b[2] and a[1]+a[3] <= b[1]+b[3] for b in unique)]
            if len(positions) == len(images):
                height = max((positions[i][1]+im.height for i, im in enumerate(images)), default=1)
                candidates.append((height, positions))
    if not candidates:
        raise ValueError('Atlas exceeds 2048 pixels')
    height, positions = min(candidates, key=lambda c: c[0])
    return (width, height), positions
