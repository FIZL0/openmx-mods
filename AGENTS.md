# Agent notes

Rules for whoever works here, human or agent. Read this, then your task's brief, then the files it names.

This repo turns terrain from other games, and from real elevation data, into freeride track mods for
[OpenMX](https://github.com/FIZL0/openmx), the MX Unleashed (PS2) port. The port's half (loading a mod)
is openmx's Phase 14 (MD1–MD6); this repo's half is getting the maps out and into the standard.

## The non-negotiable ones

- **No game data in git.** Discs go in `iso/`, anything taken off them in `extracted/`, finished mods in
  `out/`. All three are ignored, and `./check.sh` fails if one is tracked or if any tracked file is over
  1 MB. Notes may name files, sizes and offsets; they may not hold dumps, decompiled code or images.
- **openmx's readers are the source of truth.** `openmx/crates/game-sim` (`archive.rs`: PSR and `.res`;
  `track.rs`: `.trn`, `.scn`, `.col`) and `openmx/crates/game-render` read MX Unleashed's formats, and the
  MX vs. ATV discs use the same ones. Depend on them by path (`openmx/crates/...`) instead of writing a
  second reader. Output is proved by reading it back with them.
- **Don't edit openmx from here.** `openmx` is a symlink to the user's checkout, which runs its own
  agents. When the port needs a change (a reader refuses a file, a size it cannot take), write it in
  `notes/openmx-asks.md` with the file and the failure, and say so in your journal. A human files it there.
- **Say what you did not verify.** Every journal entry ends with `## Not verified`, listing what was not
  checked and why, or the single word `none`.

## Discs

`iso/` holds the user's images; find a disc by its boot file, not its file name:

| boot ELF (SYSTEM.CNF) | game | container |
|---|---|---|
| SLUS_207.67 | MX Unleashed | `MX.PSR` |
| SLUS_211.04 | MX vs. ATV Unleashed | `MXVSATV.PSR` |
| SLUS_217.01 | MX vs. ATV Untamed | `MX08PS2.PSR` (+ `MX08AUD.PSR`) |

Every one is plain ISO 9660 with the PSR in the root (openmx's `game-render/src/disc.rs` finds it).
`7z` is not installed here; `bsdtar` is, and the tools should not need either.

## Decompiling

MX Unleashed's ELF is already in Ghidra in openmx: run `./kit fn <addr>`, `./kit xref`, `./kit dis` from
`openmx/` (read-only use is fine) and see its `research/oracle.txt` for names. The other games' ELFs
have no project yet; `../ps2-decomp-kit` is the same kit, and a task that needs one sets it up under
`extracted/<game>/ghidra`. Never read a decompilation dump whole.

## Layout

```
plan/       TODO.md (the task list), DONE.md, <ID>.md long forms
journal/    one folder per task: what was done, how it was proved, ## Not verified
notes/      format notes per source game
standard/   the track mod standard (T1)
tools/      Rust crates, one per tool, in the root workspace
```

## Finishing a task

`agent-runner done <ID> "<message>"` runs `./check.sh`, ticks the task, commits and pushes its branch.
Stuck: `agent-runner done <ID> "<what is solid>" --blocked "<why, and what unblocks it>"`. Never merge,
rebase or push `main` yourself.
