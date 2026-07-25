# ESP32-S3-WROOM-1-N16R8 case — print files

## Problem (original A1 plate)

[`a1_printer_ESP32-S3-WROOM-1-N16R8__Case.3mf`](./a1_printer_ESP32-S3-WROOM-1-N16R8__Case.3mf) placed the **main case body standing / tilted ~49°** on the bed. On a **Bambu P2S**, that tall orientation slips and struggles with first-layer / overhang stability.

| Part | Old orientation | Height |
| --- | --- | --- |
| Case body | Tilted (~49°) | ~tall |
| Lid | On side / 90° | medium |

## Fix — lie flat on the plate

Use this file on P2S:

**[`p2s_flat_ESP32-S3-WROOM-1-N16R8__Case.3mf`](./p2s_flat_ESP32-S3-WROOM-1-N16R8__Case.3mf)**

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

## How to print (Bambu Studio)

1. Open `p2s_flat_ESP32-S3-WROOM-1-N16R8__Case.3mf`
2. Confirm printer = **P2S**, plate = Textured PEI
3. Slice — parts should look **lying down**, not towers
4. Print (PLA/PETG as preferred)

If anything still looks upright after open: select both → **Lay on face** / auto-orient, then re-slice.
