# T7: the accuracy harness (tools/accuracy)

A track mod against its source game running in PCSX2, proved on MX vs. ATV Unleashed's Lago (`Nat05`).
`notes/accuracy.md` is the reference (how to run it, the addresses, the findings). This is how it went.

## What was built

- `extracted/mxvatv/kit.toml`: the ps2-decomp-kit config for MX vs. ATV Unleashed (SLUS_211.04, disc
  version 1.02), with PINE, `GAME_REL.BIN` as overlay `game` at `0x45c580`, Ghidra projects for boot and
  game. It is ignored like all of `extracted/`, so `accuracy setup --game mxvatv` writes it again.
- PCSX2 kit copy 9 (`~/repos/ps2-decomp-kit-slots/pcsx2/s9`), states 3 (main menu) and 4 (in Lago's
  Free Ride).
- `tools/accuracy`: `setup`, `capture`, `heights`, `pose`, `poses`, `shoot`, `pair` (`--help`).
  - `heights`: the original's two terrain queries through openmx's `ps2-oracle` on a RAM image, against
    `contact_search::ground`/`nearest_surface` on the mod's `.trn`, 4 modes × N points, every field bit
    for bit; then again with MX vs. ATV's normal step (`heights::Over127`).
  - `shoot`: holds the ride camera (vtable slot to a returning stub), writes each pose into the
    viewport over PINE, `./kit shot`, reads the viewport back, lets the camera go.
  - `pair`: openmx's shots from the same poses when openmx can take them (MD2 + MD10), `magick` side by
    side, `index.md`.
- Tests: `tests/generated.rs` (10, generated terrains and RAM layouts: a copied terrain is equal
  everywhere, normals over 127 leave the heights and are matched by the step, a changed sample is found
  where it is, the object and viewport finders, pose round trips, pose list and index, the side-by-side,
  the kit config) and `tests/lago.rs` (Lago's 6000 points against the original, skipped without the
  image and the mod).

## Results

- **Heights, Lago:** equal at all 8000 points in 4 modes, in both queries, with the box, scale and step
  of the original's object equal to the port's. Surfaces and weights are equal too. The normals differ
  at 7114 points by 1–3 ulp, all from MX vs. ATV's cell reader dividing the normal bytes by 127. With
  that step added on the port side, every field is equal. A second capture, taken later in the ride,
  gives the same.
- **Look, Lago:** the original's shots from 8 poses in `out/accuracy/nat05/` (spawn, the scene's start,
  centre ×4, overview, ridge). The same pose in two sessions comes out with an RMSE of 0.06 %.
  openmx's side is not taken: openmx has neither `--mod` (MD2) nor `--shot-pose` (MD10). `pair` says so
  in `index.md`.
- **Free Ride start:** a fresh Free Ride on Lago puts the rider on the scene's start line, rolling (110 ft
  along `DefaultStartDirection` ~10 s in). That answers M2's `todo` item as far as it goes (see Not
  verified).

## Follow-ups

- **T9** (filed): the openmx half of the pairs once MD2 and MD10 land, and the fov's axis.
- For M12: `notes/accuracy.md`, *For M12 and M13*. For openmx: two entries at the end of
  `notes/openmx-asks.md` (the MX vs. ATV query's normal step and clamp, and what the harness passes to
  MD10).

## What was tried and did not work

- **Finding the query by matching MX Unleashed's code.** Masked-word and opcode n-gram matches of
  MXU's `terrain_point_query` and `terrain_cell_read` against SLUS_211.04 found nothing (best 13 %).
  Neither did searching Ghidra's decompilation of the boot ELF for the cell reader's shape (`* 0x14`,
  `0x15c`). The code is not in the boot ELF at all: it is in `GAME_REL.BIN`, loaded into the ELF's BSS.
  It was found from the data instead: the root node in RAM, the object pointing at it, the object's
  field offsets, and then a scan of RAM code past the ELF for loads at those offsets.
- **The camera by its eye near the rider.** In the first image no vector sat behind the rider: the
  rider had rolled 140 ft from the scene's start, and the copies of the start position fooled the
  search. Diffing two images taken before and after riding found the global camera matrix (`0x3ea930`),
  and the viewport holds the same eye. The camera class came from the string `IsLookBackCam` → the preset
  loader → the vtable.
- **Reading the pose and shooting the live camera.** The camera swings round an idle rider, so the pose
  read before and after a 3–4 s grab disagreed by 2–5 ft and a few degrees, however long the wait (an
  early PINE read of the viewport also seemed not to follow the ride; the later images showed it does).
  Holding the camera through its vtable fixed it, and also made poses writable.
- **An overview from 400 ft up** shows only fog. It is now 150 ft.
- Menu presses are dropped while a track preview loads; one `pad send` per press with 0.5–0.9 s
  between them works.

## Not verified

- **openmx's shots and the pairs.** No pair image exists: openmx has no mod track flag (MD2) and no
  `--shot-pose` (MD10). `look::compose` is tested only on generated images. `pair`'s openmx command
  line is a template whose flag names are guesses until MD2/MD10 land.
- **The fov's axis.** Whether the viewport's 85° spans the width or the height. 349.74 at `+0x178` points
  to the width, but this was not established. MD10 must agree with it for the pairs to frame alike.
- **Roll, and what the viewport does with a written up vector.** The poses have no roll. Only up = square
  to forward with +y up was written.
- **Points past the edge.** MX vs. ATV clamps them to the border and openmx misses them. That comes from
  reading the decompilation and is not compared: the points are drawn inside the border.
- **The packed normal table** (`0x541518` non-zero): not modelled, and off on Lago. Which maps or modes
  turn it on is unknown.
- **MX Unleashed's `source` row** (`source::MXU`) is copied from openmx's research and was never run by
  this tool. Untamed has no row (M13).
- **Every other map.** Only Lago was run; M12 runs the rest.
- **The Free Ride's first frame.** The rider was seen 110 ft along the start line ~10 s after loading,
  not at frame 0. Whether the game gives the bike a starting speed, or it rolls downhill, was not looked
  at.
- **Heap addresses.** The viewport moves between sessions (`shoot` finds it each run). The terrain
  object was at `0xbbd720` in both of this session's images, and is found by its tree every run.
  Whether the camera vtable slot and the stub hold on other discs or versions than US 1.02 was not checked.
- **The HUD** is in the original's shots, and how to switch it off was not looked for.
- **`check.sh`** runs the generated tests and the Lago test, which skips in a checkout without the image
  and the mod (the merge's). Its real run was here only.
