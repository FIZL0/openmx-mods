# DONE — finished tasks, under the heading they had in TODO.md

### Tooling
- [x] **T5** `tools/discx`: a Rust crate (root workspace, the first one) that opens a disc image from `iso/`, identifies it by the boot ELF in SYSTEM.CNF (AGENTS.md's table), finds its PSR in the ISO 9660 root, and lists or extracts members by glob, `.res` members included (`discx ls ISO 'Env\*\*.res'`, `discx x ISO GLOB extracted/<game>/`, `discx res FILE.res`). Use openmx's `game_sim::archive::{Psr, Res}` by path, not a second reader; it replaces the ad-hoc dd/bsdtar steps in the notes → Cargo.toml · tools/discx/ | t: tools/discx tests — an ISO 9660 image with a small PSR and a `.res` built in the test lists and extracts byte-exact | j: discx
