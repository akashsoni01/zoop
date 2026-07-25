#!/usr/bin/env python3
"""Regenerate v2 housings: width-only scale from prototype_v1_camera_case."""
from pathlib import Path
import struct, math, re, shutil, json

ROOT = Path(__file__).resolve().parent
SRC = ROOT.parent / "prototype_v1_camera_case"
DST = ROOT

V1_WIDTH = 44.9
BOARD_WIDTH = 57.0
CLEARANCE_EACH = 1.0
WALL_EACH = 2.0
TARGET_WIDTH = BOARD_WIDTH + 2 * CLEARANCE_EACH + 2 * WALL_EACH
SCALE_X = TARGET_WIDTH / V1_WIDTH


def stl_bounds(data: bytes):
    n = struct.unpack_from("<I", data, 80)[0]
    mn = [1e9] * 3
    mx = [-1e9] * 3
    off = 84
    for _ in range(n):
        for v in range(3):
            x, y, z = struct.unpack_from("<fff", data, off + 12 + v * 12)
            for j, c in enumerate((x, y, z)):
                mn[j] = min(mn[j], c)
                mx[j] = max(mx[j], c)
        off += 50
    return mn, mx


def scale_stl_x(src: Path, dst: Path, sx: float, cx: float):
    data = bytearray(src.read_bytes())
    n = struct.unpack_from("<I", data, 80)[0]
    off = 84
    for _ in range(n):
        nx, ny, nz = struct.unpack_from("<fff", data, off)
        nx *= sx
        L = math.sqrt(nx * nx + ny * ny + nz * nz) or 1.0
        struct.pack_into("<fff", data, off, nx / L, ny / L, nz / L)
        for v in range(3):
            x, y, z = struct.unpack_from("<fff", data, off + 12 + v * 12)
            struct.pack_into("<fff", data, off + 12 + v * 12, cx + (x - cx) * sx, y, z)
        off += 50
    header = f"v2 widen X*{sx:.4f} S3-WROOM-N16R8".encode("ascii", "replace")[:80]
    data[0:80] = header.ljust(80, b"\0")
    dst.write_bytes(data)


def scale_step_x(src: Path, dst: Path, sx: float, cx: float):
    text = src.read_text(errors="ignore")

    def repl(m):
        name, coords = m.group(1), m.group(2)
        parts = [p.strip() for p in coords.split(",")]
        if len(parts) != 3:
            return m.group(0)
        try:
            x, y, z = float(parts[0]), float(parts[1]), float(parts[2])
        except ValueError:
            return m.group(0)
        return f"CARTESIAN_POINT({name},({cx + (x - cx) * sx:.6f},{y},{z}))"

    dst.write_text(re.sub(r"CARTESIAN_POINT\(([^,]*),\(([^)]+)\)\)", repl, text))


def main():
    front = SRC / "front_housing_v1.stl"
    mn, mx = stl_bounds(front.read_bytes())
    cx = 0.5 * (mn[0] + mx[0])
    scale_stl_x(front, DST / "front_housing_v2.stl", SCALE_X, cx)
    scale_stl_x(SRC / "back_housing_v3.stl", DST / "back_housing_v2.stl", SCALE_X, cx)
    scale_step_x(SRC / "front_housing_v1.step", DST / "front_housing_v2.step", SCALE_X, cx)
    scale_step_x(SRC / "back_housing_v3.step", DST / "back_housing_v2.step", SCALE_X, cx)
    shutil.copy2(SRC / "button.3mf", DST / "button.3mf")
    (DST / "scale_params.json").write_text(
        json.dumps(
            {
                "target_outer_width_mm": TARGET_WIDTH,
                "scale_x": SCALE_X,
                "board_pcb_mm": [BOARD_WIDTH, 28.0],
                "makerworld_ref": "https://makerworld.com/en/models/2753270-case-esp32-s3-wroom-1-n16r8-camera#profileId-3055152",
            },
            indent=2,
        )
        + "\n"
    )
    print(f"OK width={TARGET_WIDTH}mm scale_x={SCALE_X:.6f}")


if __name__ == "__main__":
    main()
