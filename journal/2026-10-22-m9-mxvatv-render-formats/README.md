# M9: MX vs. ATV's render-side formats through openmx game-render's readers

## What was done

- New crate `tools/renderprobe` (lib + bin). It depends on game-render (with no default features, so no
  wgpu), game-sim, discx and survey, and has no reader of its own. Two pieces:
  - `render_res`: game-render makes a `Res` only through `Disc`, which wants `MX.PSR` and a directory
    under 64 KiB. So the probe builds a minimal ISO in memory (a PVD, a root directory naming `MX.PSR`,
    and a version-2 PSR whose one member is the `.res`, unchanged) and opens it with game-render's own
    `Disc::read_from` / `Disc::res`. After that the loaders run as the shell runs them:
    `Assets::load_placed`, `Sky::load`, `Grove::load`, `Splat::load`.
  - `page_pst`: puts the `.pst` magic and the terrain's palette in front of a texture page, so that
    `Texture::parse` decodes it.

  Two unit tests: the image hands game-render the payload byte for byte, and a tiny page decodes through
  its palette.
- Ran `renderprobe mxvatv <MAP>` on all 56 maps, and the loose members by glob (`psg '**.psg'`,
  `pst '**.pst'`, `pst '**.atm'`, `skies`). Ran MXU `free06` and `nat01` as the baseline.
- Wrote `--png` mosaics of the pages, and the sky faces, into the session scratchpad (not the repo).
  Looked at Nat01's and SX01's pages and Nat01's sky face 0.
- Findings are in `notes/mxvatv.md`, *Render-side formats (M9)*. Port asks are in `notes/openmx-asks.md`:
  - game-render opens only `MX.PSR` / `Res` has no constructor from bytes
  - material texture names with an extension or a space, and `TextureMap_2`
  - `EcosystemFile` without `.esb`
  - `.psg` V2-8 UVs, mask `0x1f`, and V3-16
  - more MD4 evidence
  - `.prt` in the shell crate

## How it was proved

Each claim in the notes names the map and member it was seen on, from the probe's output. Counts were
re-added from the per-map reports: 1,812 pages, 23 `.esb` read, 56 maps with `EcoAmbientScale`.

## What failed or was corrected on the way

- The first extent for `.esb` multiplied by 16 once too often (92,160 ft). The survey's `across` is
  already in leaves. Fixed to leaves × 16 × spacing: 5,760 ft, which the farthest plant on every map
  fits.
- I first wrote "1,968 pages" from mental arithmetic; summing the reports gives 1,812.

## Not verified

- Nothing was drawn and compared against the running game (T7). "Reads" means only that the reader
  accepts the file and the counts are plausible.
- Of the decoded pictures, I looked only at Nat01's and SX01's page mosaics and Nat01's sky face 0. The
  other 54 maps' pages and the other skies were decoded but not looked at.
- Which page a terrain block draws, and how the detail map is blended, were not worked out (MD4).
- The original's handling of texture names with an extension or a space, and of an `EcosystemFile`
  without `.esb`, was not looked up in the ELF. No MX vs. ATV Ghidra project exists. The asks say so.
- `northernlights.psg`'s stream mask `0x1f` was read off its bytes, and the meaning of the fifth bit was
  not found. The `1, 1, 4` header of `Tex\Box01.pst` and two others was not decoded.
- `.toy` animated toys, `.prt`, `.aud`, `.pco`, `.mcf`/`.mot`, `.vut`, `.emt` and `<map>_mst.res`
  (`.mst`) were not read: game-render has no reader for them.
- Untamed was not run (its scenes are binary, M10).
