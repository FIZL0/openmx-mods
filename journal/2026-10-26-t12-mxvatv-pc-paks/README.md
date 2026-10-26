# T12: MX vs. ATV Unleashed's PC install as a source beside its PS2 ISO

openmx MD38 (the user, 2026-10-26): discx, import-tricks and import-mxvatv take the Steam install's
`data/*.pak` wherever they take SLUS-21104's ISO. Done for all three. The trick pack built off the paks
is the disc's but for two differences, both named. No map can be built off the paks: they have no
`.res` and are not the disc's map data, so import-mxvatv names every map and refuses it with the reason.
Details in notes/mxvatv.md, *The PC install*.

## What was done

- **One pak reader**: `tools/discx/src/pak.rs`. It reads the layout of openmx's `tools/mxvatv_pak.py`
  (u32 count, 108-byte entries, offsets from the table's end) and makes the same check (the last
  member ends at the end of the file). It also finds the paks for a path: a `.pak`, a directory of
  them, an install whose `data/` has them, or several joined with `:`.
- **`discx::Disc` takes the paks as a second backing.** `Disc::open_paks` / `Disc::open_any` (chosen
  by what the path is; anything that is not paks opens as an image, as before). Every pak member
  becomes an entry of `psr`, named as the PSR names it (`Data\` taken off), with `at` counted across
  the paks back to back. So `find`, `matching`, `read`, `copy` and `res` work unchanged for every
  caller. `disc.source` says which backing it is. `psr_name` is `data\*.pak`, `boot` is `None`, and
  `game` is MX vs. ATV Unleashed. A name found in two paks is refused, though none is.
  `discx::resolve_source` takes `pc` (`$DISCX_PC`, else `openmx/context/mxvatv-pc/data`) or a paks
  path. `discx::compare` and `discx cmp [--res] A B [PATTERN]` compare two sources byte for byte by
  name, and `--res` also compares each `.res` member against the other source's loose file at that path.
- **import-tricks**: `Disc::open_any` everywhere it opened the disc; `GAME=path` goes through
  `resolve_source` (`mxvatv=pc`).
- **import-mxvatv**: `map_folders` lists the paks' maps by `Env\<MAP>\<MAP>.scn`. `load` on paks
  returns `pc_refusal`, and `main` with a paks path prints FAIL with the reason for each map and
  exits 1, writing nothing.
- Tests:
  - `discx/tests/pc.rs`: the paks open as one directory (gate; skips, saying so, without the install),
    and the disc against the paks with counts (ignored).
  - `import-tricks/tests/pc.rs` (ignored): builds both packs with every read recorded. Every member
    read off the disc is identical off the paks, and the packs differ in exactly the two named places.
  - `import-mxvatv/tests/pc.rs`: every one of the 64 pak maps is refused with the reason (gate; skips
    without the install). Ignored: every file the disc's 56 imports read is looked up in the paks, and
    the totals must match the notes.
- notes/mxvatv.md *The PC install*; notes/openmx-asks.md: research/mxvatv-pc.md's two errors (70 paks,
  not 63; the maps are not byte-identical).

## What was found

- 70 paks, 20,415 members, no `.res`. The disc's map archives' members are loose in the paks at their
  own paths.
- Of what the disc's 56 map imports read: 15,799 identical, 342 different, 13,676 absent. Absent are
  the PS2 render files (the PC has `.dx.trn`/`.dxg`/`.dxm`/`.dxt` instead) and 4 banner `.col`s.
  Different are 84 distinct files: 13 scenes (placements moved, turned, added, dropped), 10 `.col`s,
  `.aud`, `.atyList`, `.mst`, `.esb`, `Nat12.toy` and 10 `.pst`.
- The paks have 8 maps the disc lacks: FS09–FS13, Nat19, SX19, SX20.
- Tricks: everything identical. The disc's `Rider\Anim\ATV_bike_NinjaNac.mot` is in the paks as
  `Veh\Anim\ATV_bike_NinjaNac.mot`, so the PC-built pack has the ATV bike's NinjaNac clip and the
  disc-built one does not.

## Tried and dropped

- The brief's test, *an import-mxvatv map built from the paks equals the ISO-built one*, cannot hold.
  The paks lack the archives a mod is made of, and their scenes are a later edit. I refused instead of
  rebuilding archives from loose files, since that would be a guess at the disc (AGENTS.md: accurate
  to the original). The test proves the refusal and records the differences instead.
- The trick pack's `source.psr` stays honest (`data\*.pak`) rather than claiming `MXVSATV.PSR`, so the
  two `mod.json`s are not byte-equal. The test pins exactly that difference and the NinjaNac line.
- My first `discx cmp` used a thread-local for `--res`; I replaced it with a parameter.

## Not verified

- No pak-built output was run in openmx or modcheck'd against a source. modcheck proves mods against
  the disc only, and I did not give it a paks source (not asked for, since no map comes off the paks).
- Whether the PC's `.dx.trn` holds the same heights as the `.ps2.trn` was not compared. That needs a
  reader for the PC terrain, which neither repo has.
- What the PC's differing scenes, `.col`s, `.esb`s and `.aud`s mean in the PC game was not looked at.
  Only byte equality was checked.
- The ignored tests were run on this machine with both sources present; the gate runs only the
  pak-table and refusal tests. Their skip was checked with `DISCX_PC=/nonexistent` (both say so and
  pass); the ignored ones were not run without a source.
- research/mxvatv-pc.md and tools/mxvatv_pak.py are openmx's (the brief lists them as on the line).
  They were read, not edited (AGENTS.md); the corrections are in notes/openmx-asks.md.
