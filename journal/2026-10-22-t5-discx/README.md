# T5: tools/discx

The repo's first Rust crate, and the root workspace (`Cargo.toml`) it lives in.

## What it does

```
discx discs [DIR]                    images in iso/ and their boot ELFs
discx info  DISC                     game, PSR, member count
discx ls    DISC [PATTERN]
discx x     DISC PATTERN OUTDIR
discx res   FILE.res [GLOB [OUTDIR]]
```

- **DISC** is a path, or a game (`mxu`, `mxvatv`, `untamed`, or the boot ELF), looked up in `iso/`
  (`$DISCX_ISO` overrides) by reading each image's SYSTEM.CNF. Image file names are never used.
- The PSR is the one named by AGENTS.md's table for that boot ELF; `--psr MX08AUD.PSR` opens another.
  An unknown disc opens if its root has exactly one `.PSR`, otherwise the error names its boot ELF.
- **Globs:** `*` within a segment, `**` across them, `?` one char. `\` and `/` are both separators (so `\`
  is never an escape, which is what lets `'Env\*\*.res'` mean what it says), and case is folded, as the
  game's own lookups do. `OUTER:INNER` reaches into `.res` members (`'Env\Nat01\nat01.res:Col\*.col'`);
  PSR names never contain `:`.
- **Extraction** keeps paths with `\` → `/`. A `.res` member goes to `OUTDIR/<res path>.d/<member>`, so
  the `.res` itself can sit beside its unpacked folder and no PSR path can collide with it. PSR members
  are streamed (MXU's `Aud\audio.res` is 164 MB); `.res` members go through openmx's `Res::read`.
- PSR and `.res` are parsed only by `game_sim::archive::{Psr, Res}`, a path dependency on
  `openmx/crates/game-sim`. The crate's own reading code covers the ISO 9660 root directory, SYSTEM.CNF
  and the glob. openmx's `game-render::disc::Disc` already does the ISO part, but it is hard-wired
  to `MX.PSR` and needs game-render (wgpu); its directory read is also a fixed 64 KiB, which MX vs. ATV's
  20k-member directories overrun. So discx reads the length the PSR header gives at 0x10. That is not
  a port bug, since the port only opens MX.PSR, so nothing was added to `notes/openmx-asks.md`.

## Decisions

- **`exclude = ["openmx"]` in the workspace.** The first build compiled game-render and wgpu: cargo adds
  path dependencies that sit under the workspace root as implicit members, and the `openmx` link is
  under it. game-sim then inherited *this* workspace's `[workspace.package]` (version 0.1.0), and
  `cargo test --workspace` (the gate) would have run openmx's own suites. With the exclude, `cargo tree`
  shows game-sim 1.0.0 → ps2-float only, and `cargo metadata` lists `discx` as the one member.
- **`members = ["tools/*"]`**, so the next tool (T2, T6, …) does not have to edit `Cargo.toml` and
  conflict with other branches.
- **`Cargo.lock` is git-ignored.** The only dependencies are openmx paths. A committed lock changes
  whenever openmx bumps a version or a tool is added, and would conflict between parallel task branches.
  If a tool ever takes a crates.io dependency, revisit this.
- `notes/mxvatv.md`: one sentence pointing at discx where the note describes finding the PSR. None of
  the notes actually contains dd/bsdtar commands to replace (grep finds none).

## How it was proved

- `tools/discx/tests/image.rs` builds an ISO 9660 image by hand: PVD, terminator, a root holding `.`,
  `..`, a boot ELF, SYSTEM.CNF (`BOOT2 = cdrom0:\SLUS_211.04;1`) and `MXVSATV.PSR`. The PSR has four
  members plus a `<PAD>` entry. One member is a `.res` made of stored-deflate zlib blocks, holding a
  member that spans two blocks, a small one and an empty one. Tests: library listing and reading (and
  streamed copy) byte-exact; `discx ls` with and without `:`; `discx x` of every PSR member and every
  `.res` member compared byte for byte; `discx res` on the extracted `.res` (list, then extract,
  byte-exact); disc found by game name in an `iso/` with a non-ISO file in it; an absent game's error
  names its boot ELF. Unit tests cover the glob, SYSTEM.CNF and pattern parsing.
- Mutation check: shifting `Disc::read` by one byte fails all three byte-exact tests. Restored.
- Real discs, read-only from the main checkout's `iso/` (this worktree has no `iso` link), with
  `cargo test -p discx --release -- --ignored`: MXU MX.PSR 633 members; MX vs. ATV MXVSATV.PSR 20,391;
  Untamed MX08PS2.PSR 20,552 and MX08AUD.PSR 554. The counts match notes/mxvatv.md and
  mxvatv-later.md. Hot Shots Tennis is identified as SCUS_976.10 and skipped.
- `discx ls mxvatv 'Env\*\*.res'` lists 168 `.res`. `discx x mxvatv 'Env\Nat01\*'` gave 55 files. Then
  `Env\Nat01\nat01.res` (4,004,619 bytes) and `Env\Nat01\Nat01.scn` were each compared with `cmp`
  against bsdtar's copy of MXVSATV.PSR at the offset a scratch Python parser read from the directory:
  both byte-exact. That checks the ISO side independently of discx. `nat01.res:**` unpacked 102 members;
  `Env\nat01\Nat01.ps2.trn` starts `PLN\0` and the scene starts `[SceneInfo]`.
- `discx x mxu 'Aud\audio.res'` streamed the 163,823,616-byte member in 0.07 s.
- `./check.sh` passes.

## What failed on the way

- The first build without `exclude` (above).
- `discx ls … | head` panicked on a broken pipe. Output now goes through an `out!` macro that exits 0 on
  EPIPE.
- `discx res nat01.res '*.trn'` matched nothing. That was correct: the terrain is `Env\nat01\Nat01.ps2.trn`
  inside the `.res`, and `*` does not cross segments. `'**.trn'` finds it.

## Not verified

- `.res` members on real discs are not compared against an independent reader. They come from openmx's
  `Res::read` (the source of truth by AGENTS.md); only their leading bytes were looked at.
- The `--ignored` disc test and the real-disc runs used `DISCX_ISO=/home/rycs/repos/openmx-mods/iso`.
  This worktree has no `iso` link, so the default `iso/` path was exercised only by the synthetic test.
- Root directories longer than one sector, and ISO images over 4 GB: the code handles both (records
  skip to the next sector on a zero length; offsets are u64), but no test image had either. The real
  discs' roots each fit in one sector.
- Extraction was not run into `extracted/` itself (not linked here); it went to the session scratchpad,
  which was then deleted.
