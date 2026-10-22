# What the port needs from openmx

Changes openmx has to make before a mod works there. Agents here write them; a human files them in openmx's
`plan/TODO.md` (Phase 14). Each entry: the file that fails, the reader and its error, and what would fix it.

## Filed

- **MD1** `Terrain::parse` rejects every MX vs. ATV terrain: `257 blocks of 2129 from 2189 end at 549342,
  but the tree starts at 1598302` (Untamed FS_A). The texture table is non-empty and the pages sit
  between the blocks and the tree.
- **MD2** a mods folder the shell loads tracks from (the standard, T1).
- **MD3** 80×80-leaf terrains at 4.5 ft (TerrainWidth 5): edge, radar, draw distances, height band.
- **MD4** the ground drawn from the terrain's own texture pages.
- **MD5** `Collision::parse`: `collision class 1 is not understood` (MX vs. ATV Unleashed
  `Col\Nat07_WM_Blade.col`).
- **MD6** online rooms on a mod track: same mod id and hash, or not seated.
- **MD7** music packs from mods/music/, with a setting for which games' music plays.
- **MD8** worlds of several terrains (the combined and ultra maps), each tile bit-exact to its source.
- **MD9** each map's own sky, fog, lights and ecosystem inside a combined world.
- **MD10** `--shot-pose`, for the accuracy harness (T7).
- **MD11** the track pick filtered by game: All games, MX Unleashed, MX vs. ATV Unleashed, Untamed, …

## Open
