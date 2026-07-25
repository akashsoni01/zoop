# ESP32-S3-WROOM-1-N16R8 case — print files

## Problem (original A1 plate)

[`a1_printer_ESP32-S3-WROOM-1-N16R8__Case.3mf`](./a1_printer_ESP32-S3-WROOM-1-N16R8__Case.3mf) placed the **main case body standing / tilted ~49°** on the bed. On a **Bambu P2S**, that tall orientation slips and struggles with first-layer / overhang stability.

| Part | Old orientation | Height |
| --- | --- | --- |
| Case body | Tilted (~49°) | ~tall |
| Lid | On side / 90° | medium |

## Fix — lie flat on the plate

Use this file on P2S:

**[`p2s_flat_ESP32-S3-WROOM-1-N16R8__Case_with_buttons.3mf`](./p2s_flat_ESP32-S3-WROOM-1-N16R8__Case_with_buttons.3mf)**  
(case + lid + BOOT + POWER buttons)

Case/lid only (no buttons): [`p2s_flat_ESP32-S3-WROOM-1-N16R8__Case.3mf`](./p2s_flat_ESP32-S3-WROOM-1-N16R8__Case.3mf)

Both parts are rotated so the **largest face is on the bed** (local Y → world Z):

| Part | Footprint (X×Y) | Height (Z) |
| --- | --- | --- |
| Case body | ~32.5 × 55.5 mm | **14.0 mm** |
| Lid | ~32.5 × 55.5 mm | **7.35 mm** |

Also:

- Supports **off** (not needed when flat)
- Outer **brim 5 mm** for PEI grip
- Printer profile already **Bambu Lab P2S**

### Plain STLs (any slicer)

| File | Use |
| --- | --- |
| [`case_body_flat.stl`](./case_body_flat.stl) | Main shell, already flat (Z min = 0) |
| [`case_lid_flat.stl`](./case_lid_flat.stl) | Lid / cover, already flat |
| [`button_boot.stl`](./button_boot.stl) | BOOT plunger (IO0) — same design as `prototype_v2` button |
| [`button_power.stl`](./button_power.stl) | POWER / RESET plunger (EN) |

### Buttons (BOOT + POWER)

Geometry matches [`prototype_v2_camers_case/button.3mf`](../prototype_v2_camers_case/button.3mf) (~**4.8 × 7.0 × 5.6 mm** each). Two copies of that plunger: one for **BOOT**, one for **POWER/RESET** on the ESP32-S3 board.

| File | Contents |
| --- | --- |
| [`button_boot.3mf`](./button_boot.3mf) / `.stl` | BOOT only |
| [`button_power.3mf`](./button_power.3mf) / `.stl` | POWER only |
| [`buttons_boot_power.3mf`](./buttons_boot_power.3mf) | Both on one plate |
| [`p2s_flat_ESP32-S3-WROOM-1-N16R8__Case_with_buttons.3mf`](./p2s_flat_ESP32-S3-WROOM-1-N16R8__Case_with_buttons.3mf) | Flat case + lid + both buttons |

Print buttons **flat on the bed** (already oriented). Drop into the case button wells over the board tactile switches.

## How to print (Bambu Studio)

1. Prefer `p2s_flat_ESP32-S3-WROOM-1-N16R8__Case_with_buttons.3mf` (case + lid + BOOT + POWER)
2. Confirm printer = **P2S**, plate = Textured PEI
3. Slice — case/lid **lying down**; buttons as small blocks on the plate
4. Print (PLA/PETG as preferred)

If anything still looks upright after open: select parts → **Lay on face** / auto-orient, then re-slice.
