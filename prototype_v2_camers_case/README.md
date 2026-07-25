# Prototype v2 — camera case (wider for ESP32-S3-WROOM-1-N16R8)

Widened remake of [`prototype_v1_camera_case`](../prototype_v1_camera_case) so the shell can enclose an **ESP32-S3-WROOM-1-N16R8 Camera** board (Freenove-class ~**57 × 28 mm** PCB).

| | |
| --- | --- |
| Reference board / case | [MakerWorld — Case ESP32-S3-WROOM-1-N16R8 Camera](https://makerworld.com/en/models/2753270-case-esp32-s3-wroom-1-n16r8-camera#profileId-3055152) |
| Design intent | Same curved shell, snap fit, and button as v1 — **width only** increased |
| Kit context | OceanLabz DIY AI Voice Kit — [`physical-components/hardware_spec.md`](../physical-components/hardware_spec.md) |

## What stayed the same

- Curved outer design (topology preserved; stretched only in X)
- Snap / clip features (same mesh, moved outward with width)
- Length (**Y = 58.4 mm**) and thickness (**Z** unchanged)
- **`button.3mf`** — copied from v1 with **no** scaling

## What changed

| | v1 | v2 |
| --- | --- | --- |
| Outer width (X) | **44.9 mm** | **63.0 mm** |
| Outer length (Y) | 58.4 mm | 58.4 mm |
| Front height (Z) | ~11.3 mm | ~11.3 mm |
| Back height (Z) | ~10.4 mm | ~10.4 mm |
| Scale | — | **X × 1.403** about bbox center |

Sizing basis:

- PCB target ≈ **57 mm** (long axis across case width)
- Clearance ≈ **1 mm** per side
- Wall allowance ≈ **2 mm** per side  
- → outer width **63 mm**

Parameters: [`scale_params.json`](./scale_params.json).

## Files

| File | Notes |
| --- | --- |
| `front_housing_v2.stl` / `.step` | Widened front |
| `back_housing_v2.stl` / `.step` | Widened back |
| `button.3mf` | Original dual-button module from v1 (two plungers) |
| `button_boot.stl` / `.3mf` | BOOT plunger (centered, print-ready) |
| `button_power.stl` / `.3mf` | POWER / RESET plunger (centered, print-ready) |
| `buttons_boot_power.3mf` | BOOT + POWER on one plate |
| `scale_params.json` | Reproducible scale inputs |

Button size ≈ **4.8 × 7.0 × 5.6 mm**. Same curved plunger as v1; use one over the board **BOOT** switch and one over **EN/RESET**.

## Print / fit notes

1. Print front + back + button as in v1 (same orientation / supports strategy).
2. Dry-fit the **ESP32-S3-WROOM-1-N16R8** camera board before final clips.
3. If your PCB is slightly wider/narrower than 57 mm, re-run a width-only scale (edit `TARGET_WIDTH` / `SCALE_X` in the generator script or rescale in CAD).
4. STEP files were X-scaled by rewriting `CARTESIAN_POINT` coordinates (best-effort). Prefer **STL** for slicing; re-export STEP from CAD if you need a clean B-rep.

## Regenerate from v1

From repo root (Python 3, no extra CAD app required for STL):

```bash
# See scale_params.json for current factors; regenerator lives in git history / re-run
# the widen script used to produce this folder if you change TARGET_WIDTH.
```

## Related

- v1 source: [`../prototype_v1_camera_case`](../prototype_v1_camera_case)
- Hardware kit: [`../physical-components/hardware_spec.md`](../physical-components/hardware_spec.md)
