//! Every terrain on the three discs: read with openmx → decoded → rewritten → read again gives the same
//! cells (heights, normals, surfaces) at every sample, the same decoded terrain, and the same leaf block
//! bytes.
//!
//! `cargo test -p trnwrite --release -- --ignored`. The images are found by boot ELF in `$DISCX_ISO`,
//! else `iso/` at the repo root (a worktree has none: set `DISCX_ISO`). A game whose disc is not there
//! is skipped and said so; no disc at all fails.

use std::path::PathBuf;

use discx::{Disc, ResFile, resolve_image};
use game_sim::track::{BLOCK_BYTES, Terrain};
use trnwrite::{header, openmx_view, read, write};

fn iso_dir() -> PathBuf {
    std::env::var_os("DISCX_ISO")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../iso"))
}

/// Every `.trn` on one disc, by name: loose in the PSR (Untamed) or inside a track `.res`.
fn terrains(game: &str) -> Option<Vec<(String, Vec<u8>)>> {
    let image = resolve_image(game, &iso_dir()).ok()?;
    let mut disc = Disc::open(&image).expect("the disc opens");
    let mut out = Vec::new();
    let loose: Vec<_> = disc.matching("**.trn").into_iter().cloned().collect();
    for e in loose {
        out.push((e.name.clone(), disc.read(&e).unwrap()));
    }
    let archives: Vec<_> = disc
        .matching(r"Tracks\*.res")
        .into_iter()
        .chain(disc.matching(r"Env\*\*.res"))
        .cloned()
        .collect();
    for e in archives {
        let res: ResFile = disc
            .res(&e)
            .unwrap_or_else(|err| panic!("{}: {err}", e.name));
        for m in res.matching("**.trn") {
            out.push((format!("{}:{}", e.name, m.name), res.read(m).unwrap()));
        }
    }
    Some(out)
}

/// Every sample's cell, at steps 1 and 2, compared between two readings, over the root cells that hold
/// anything (plus one sample past them, which has to be a miss in both).
fn same_cells(a: &Terrain, b: &Terrain, extent: (u32, u32)) -> Result<usize, String> {
    let mut n = 0;
    for step in [1, 2] {
        for sz in 0..=extent.1 {
            for sx in 0..=extent.0 {
                let (qa, qb) = (a.cell(sx, sz, step), b.cell(sx, sz, step));
                if qa != qb {
                    return Err(format!("cell {sx},{sz} step {step}: {qa:?} against {qb:?}"));
                }
                n += usize::from(qa.is_some());
            }
        }
    }
    Ok(n)
}

#[test]
#[ignore = "needs the disc images"]
fn every_disc_terrain_round_trips() {
    let mut games = 0;
    let mut total = 0;
    for game in ["mxu", "mxvatv", "untamed"] {
        let Some(list) = terrains(game) else {
            eprintln!("{game}: no disc in {}, skipped", iso_dir().display());
            continue;
        };
        games += 1;
        assert!(!list.is_empty(), "{game}: no terrains found");
        let (mut blocks_in, mut blocks_out) = (0, 0);
        for (name, src) in &list {
            let model = read(src).unwrap_or_else(|e| panic!("{name}: {e}"));
            let (out, w) = write(&model).unwrap_or_else(|e| panic!("{name}: {e}"));
            let again = read(&out).unwrap_or_else(|e| panic!("{name} rewritten: {e}"));
            assert!(
                again == model,
                "{name}: the rewritten terrain decodes differently"
            );

            let (va, vb) = (openmx_view(src).unwrap(), openmx_view(&out).unwrap());
            let (ta, tb) = (Terrain::parse(&va).unwrap(), Terrain::parse(&vb).unwrap());
            let (cw, cd) = model.root_cells();
            let cells = same_cells(&ta, &tb, (cw * 256, cd * 256))
                .unwrap_or_else(|e| panic!("{name}: {e}"));
            assert!(cells > 0, "{name}: no cells");

            // Each leaf's block, byte for byte (less the three unused height columns of a row, zero on
            // every disc block, which the rewrite writes as zero too).
            let (ha, hb) = (header(src).unwrap(), header(&out).unwrap());
            assert_eq!(ha.pages, hb.pages, "{name}: pages");
            assert_eq!(ha.palette, hb.palette, "{name}: palette");
            assert_eq!(hb.tree_at, w.tree_at);
            let blk = |b: &[u8], h: &trnwrite::Header, i: u16| {
                let at = h.blocks_at + usize::from(i) * BLOCK_BYTES;
                b[at..at + BLOCK_BYTES].to_vec()
            };
            for lz in 0..cd * 16 {
                for lx in 0..cw * 16 {
                    let (la, lb) = (ta.leaf(lx * 16, lz * 16), tb.leaf(lx * 16, lz * 16));
                    assert_eq!(la.is_some(), lb.is_some(), "{name}: leaf {lx},{lz}");
                    if let (Some((na, ..)), Some((nb, ..))) = (la, lb) {
                        assert_eq!(
                            blk(src, &ha, na.block),
                            blk(&out, &hb, nb.block),
                            "{name}: leaf {lx},{lz}"
                        );
                    }
                }
            }
            blocks_in += ta.blocks;
            blocks_out += w.blocks;
            total += 1;
            eprintln!(
                "{game} {name}: {} leaves, {} blocks → {}, {} pages, {cells} cells, {} → {} bytes",
                w.leaves,
                ta.blocks,
                w.blocks,
                w.pages,
                src.len(),
                out.len()
            );
        }
        eprintln!(
            "{game}: {} terrains, {blocks_in} blocks on the disc, {blocks_out} rewritten",
            list.len()
        );
    }
    assert!(games > 0, "no disc found in {}", iso_dir().display());
    eprintln!("{total} terrains round-tripped over {games} discs");
}
