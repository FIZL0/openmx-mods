# openmx MD60a: C12's wrapped field re-landed on main, after MD57's relief

C12 (`origin/task/C12`, b36970a: `Plan::period`/`images`, `UltraPlan::period`, tests/wrap.rs,
`stitch holes`) merged onto main 65db95d. It conflicted with MD57's relief in `tools/stitch/src/build.rs`
(the `Plan` fields, `plan()`'s `Plan` literal, the blend's two early returns, which now carry
`relief: 0.0`) and in `notes/stitch.md` (both sections kept, MD53's first). Both sides were kept whole.

The relief now reads across the wrap too. `Plan::relief_at` hears each map from each of its images
(`images(self.period)`), a term apiece: its nearest distance, its gates' clearance and its 1/d² weights are
the torus's. `Plan::relief_cands` takes the nearest image (`wrapped_gap`, and the farthest corner
distance from the nearest image). It stays 0 on the world's own edge, where openmx's strip closes the
wrap, as MD57 had it.

## How it was proved

- `cargo test --release -p stitch`: every non-ignored test passes, wrap.rs and rock.rs and relief.rs
  included. wrap.rs gained a check: in bands 600 ft along both far edges of the wrapped 2×2 world,
  `relief_at` from `relief_cands(band)` is `relief_at` from every source to the bit (122 of 402 points
  carry relief). Control: with `relief_cands` back on the plain `rect_gap`, that test fails.
- `./check.sh`: `check: ok, 148 passed, 0 failed`.
- world_mxu built alone from this tree (`stitch --mods out/mods --out STAGE`, the other specs moved aside
  for the run): 721 pieces, 37792 × 31960 ft, join slope 0.498 away from the seams. `stitch check`: ok,
  worst seam step 0.00299 ft, near edges against far ones 0.00403 ft. `stitch holes`: 0 holes, 0 of
  629584 points across the seams with no ground, steepest 0.229 ft in half a foot.
  openmx `--seams` (release, openmx 6c649058): no gaps, steepest step 0.700 ft at j023_011 | mxu_nat20,
  0.100 ft across the wrap (it read 7.977 ft on the installed, C12-less build).

## Not verified

- Only world_mxu was rebuilt and walked; world_mxu_free, world_mxvatv, world_untamed and world_ultra
  are MD60's to rebuild and install. Nothing was installed.
- `Room` (the relief's room lattice) does not wrap: its sweeps stop at the world's edge. The relief is
  0 there anyway.
- openmx's own `crates/stitch` has no wrapped field either; not changed here (filed in openmx).
