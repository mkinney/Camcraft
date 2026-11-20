#!/usr/bin/env python3
"""Provenance: original rectangular plate and square hole for CamCraft (#2873)."""
from pathlib import Path


def polyline(points):
    groups = ["0", "LWPOLYLINE", "90", str(len(points)), "70", "1"]
    for x, y in points:
        groups.extend(["10", str(x), "20", str(y)])
    return groups


groups = ["0", "SECTION", "2", "HEADER", "9", "$INSUNITS", "70", "4",
          "0", "ENDSEC", "0", "SECTION", "2", "ENTITIES"]
# Deliberately store the outline first. Generation must schedule the hole first.
groups += polyline([(-40, -30), (40, -30), (40, 30), (-40, 30)])
groups += polyline([(-8, -8), (8, -8), (8, 8), (-8, 8)])
groups += ["0", "ENDSEC", "0", "EOF"]
Path(__file__).with_name("profile.dxf").write_text("\n".join(groups) + "\n", encoding="ascii")
