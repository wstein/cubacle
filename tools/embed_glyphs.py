#!/usr/bin/env python3
"""Convert the label glyphs of an open font to polygons for src/render/glyphs/.

    python3 tools/embed_glyphs.py FONT.ttf OUT.rs --weight 700 \
        --name "Nunito Bold" --upstream URL --source "google/fonts@SHA ofl/..." \
        --license "OFL-1.1" --copyright "Copyright ..."

Needs `fonttools` (pip install fonttools). The font file is read from a
temporary directory and never enters the repository; only the flattened
outlines are written. Glyph units: the cap height is 1.4, y points down with
0 at the cap line, so the baseline is at y = 1.4 and `advance` is in the same
unit. Holes are joined to their outer contour by a zero-width bridge so each
polygon fills correctly with the even-odd and the non-zero rule alike.
"""
import argparse
import math

from fontTools.pens.basePen import BasePen
from fontTools.ttLib import TTFont

CHARS = "0123456789URFDLBwx'"
CAP = 1.4
TOLERANCE = 0.004  # cap units; the icons never show a curve this far off


class Flatten(BasePen):
    def __init__(self, glyph_set):
        super().__init__(glyph_set)
        self.contours = []
        self.cur = None

    def _moveTo(self, p):
        self.cur = [p]

    def _lineTo(self, p):
        self.cur.append(p)

    def _curveToOne(self, p1, p2, p3):
        p0 = self.cur[-1]
        dd = max(
            math.hypot(p0[0] - 2 * p1[0] + p2[0], p0[1] - 2 * p1[1] + p2[1]),
            math.hypot(p1[0] - 2 * p2[0] + p3[0], p1[1] - 2 * p2[1] + p3[1]),
        )
        n = max(2, min(16, math.ceil(math.sqrt(0.75 * dd * self.k / TOLERANCE))))
        for i in range(1, n + 1):
            t = i / n
            u = 1 - t
            self.cur.append(
                (
                    u**3 * p0[0] + 3 * u * u * t * p1[0] + 3 * u * t * t * p2[0] + t**3 * p3[0],
                    u**3 * p0[1] + 3 * u * u * t * p1[1] + 3 * u * t * t * p2[1] + t**3 * p3[1],
                )
            )

    def _qCurveToOne(self, p1, p2):
        p0 = self.cur[-1]
        dd = math.hypot(p0[0] - 2 * p1[0] + p2[0], p0[1] - 2 * p1[1] + p2[1])
        n = max(2, min(16, math.ceil(math.sqrt(0.25 * dd * self.k / TOLERANCE))))
        for i in range(1, n + 1):
            t = i / n
            u = 1 - t
            self.cur.append(
                (
                    u * u * p0[0] + 2 * u * t * p1[0] + t * t * p2[0],
                    u * u * p0[1] + 2 * u * t * p1[1] + t * t * p2[1],
                )
            )

    def _closePath(self):
        if self.cur and len(self.cur) > 2:
            self.contours.append(self.cur)
        self.cur = None

    _endPath = _closePath


def area(poly):
    return (
        sum(
            poly[i][0] * poly[(i + 1) % len(poly)][1] - poly[(i + 1) % len(poly)][0] * poly[i][1]
            for i in range(len(poly))
        )
        / 2
    )


def inside(pt, poly):
    x, y = pt
    c = False
    for i in range(len(poly)):
        a, b = poly[i], poly[(i + 1) % len(poly)]
        if (a[1] > y) != (b[1] > y) and x < (b[0] - a[0]) * (y - a[1]) / (b[1] - a[1]) + a[0]:
            c = not c
    return c


def cross(o, a, b):
    return (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])


def properly_cross(a, b, c, d):
    return cross(a, b, c) * cross(a, b, d) < 0 and cross(c, d, a) * cross(c, d, b) < 0


def bridge(outer, hole, others):
    """Join `hole` into `outer` with a zero-width slit; `others` are other polygons to avoid."""
    edges = []
    for poly in [outer, hole] + others:
        edges += [(poly[i], poly[(i + 1) % len(poly)]) for i in range(len(poly))]
    best = None
    for j, p in enumerate(outer):
        for i, q in enumerate(hole):
            d = math.hypot(p[0] - q[0], p[1] - q[1])
            if best is not None and d >= best[0]:
                continue
            if any(properly_cross(p, q, e0, e1) for e0, e1 in edges):
                continue
            best = (d, j, i)
    if best is None:
        raise SystemExit("no bridge found")
    _, j, i = best
    return outer[: j + 1] + hole[i:] + hole[: i + 1] + outer[j:]


def glyph_polygons(contours):
    areas = [area(c) for c in contours]
    sign = 1 if areas[max(range(len(areas)), key=lambda k: abs(areas[k]))] > 0 else -1
    outers = [c for c, a in zip(contours, areas) if a * sign > 0]
    holes = [c for c, a in zip(contours, areas) if a * sign < 0]
    groups = {id(o): [o, []] for o in outers}
    for h in holes:
        owners = [o for o in outers if inside(h[0], o)]
        owner = min(owners, key=lambda o: abs(area(o)))
        groups[id(owner)][1].append(h)
    result = []
    for outer, hs in groups.values():
        poly = outer
        for k, h in enumerate(hs):
            poly = bridge(poly, h, hs[k + 1 :])
        result.append(poly)
    return result


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("font")
    ap.add_argument("out")
    ap.add_argument("--weight", type=float)
    ap.add_argument("--name", required=True)
    ap.add_argument("--upstream", required=True)
    ap.add_argument("--source", required=True)
    ap.add_argument("--license", required=True)
    ap.add_argument("--copyright", required=True)
    args = ap.parse_args()

    font = TTFont(args.font)
    if "fvar" in font and args.weight is not None:
        from fontTools.varLib import instancer

        font = instancer.instantiateVariableFont(font, {"wght": args.weight})
    cap = font["OS/2"].sCapHeight
    k = CAP / cap
    cmap = font.getBestCmap()
    glyph_set = font.getGlyphSet()
    lines = []
    for ch in CHARS:
        name = cmap[ord(ch)]
        pen = Flatten(glyph_set)
        pen.k = k
        glyph_set[name].draw(pen)
        polys = glyph_polygons(pen.contours)
        advance = font["hmtx"][name][0] * k
        rendered = []
        for poly in polys:
            pts, last = [], None
            for x, y in poly:
                p = (round(x * k, 3), round(CAP - y * k, 3))
                if p != last:
                    pts.append(p)
                last = p
            if pts[0] == pts[-1]:
                pts.pop()
            rendered.append("        &[" + ", ".join(f"[{x}, {y}]" for x, y in pts) + "],")
        lit = "'\\''" if ch == "'" else repr(ch)
        lines.append(f"    ({lit},{round(advance, 3)}, &[\n" + "\n".join(rendered) + "\n    ]),")
    with open(args.out, "w") as f:
        f.write(
            f"""//! Glyph outlines derived from {args.name}, flattened to polygons.
//!
//! Origin: {args.upstream}
//! Taken from: {args.source}
//! License: {args.license}. {args.copyright}
//! The license text is in `licenses/`; see `NOTICE.md`. Only the characters
//! `{CHARS}` are embedded. Generated by `tools/embed_glyphs.py`; do not edit.
//! Units: cap height 1.4, y down from the cap line, `advance` in the same unit.

#![allow(clippy::approx_constant)]

use super::Glyph;

#[rustfmt::skip]
pub(crate) const GLYPHS: &[Glyph] = &[
"""
        )
        f.write("\n".join(lines))
        f.write("\n];\n")


if __name__ == "__main__":
    main()
