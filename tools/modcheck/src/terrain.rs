//! Reading a terrain whose texture pages sit between its blocks and its tree, before openmx MD1.
//!
//! `track::Terrain::parse` wants the blocks to end where the tree starts. MX vs. ATV's terrains, and
//! MX Unleashed's nationals and supercross tracks, have texture pages in between
//! (`notes/mxu-terrain-limits.md`), so it refuses them (`notes/openmx-asks.md`, MD1). Until MD1 lands,
//! [`md1_view`] walks the pages the way the game's loader does and gives `Terrain::parse` a copy in
//! which the gap is counted as texture-offset words instead: the header's word count grows by the
//! gap, the blocks move up against the tree, and every other byte (the tree and its absolute offsets
//! among them) stays where it was. What the parser then reads from the blocks and the tree is the
//! file's own bytes.

/// The bytes of one leaf block (`track.rs`'s `BLOCK_BYTES`).
const BLOCK_BYTES: usize = 2129;
/// The game keeps at most 128 page pointers and 6 mips a page (`notes/mxu-terrain-limits.md`).
const MAX_PAGES: usize = 128;
const MAX_MIPS: u32 = 6;

#[derive(Debug)]
pub struct View {
    pub bytes: Vec<u8>,
    /// How many pages the gap held.
    pub pages: usize,
    /// The gap's length in bytes.
    pub gap: usize,
}

fn u32_at(b: &[u8], at: usize) -> Result<u32, String> {
    b.get(at..at + 4)
        .map(|w| u32::from_le_bytes(w.try_into().expect("4 bytes")))
        .ok_or_else(|| format!("the terrain ends before its header word at {at}"))
}

/// The view, or why there is none: the gap is not exactly the pages the header lists.
pub fn md1_view(b: &[u8]) -> Result<View, String> {
    if !b.starts_with(b"PLN\0") {
        return Err("not a terrain".into());
    }
    let mut o = 4 + 1 + 4 + 32;
    if u32_at(b, o)? != 0 {
        o += 1024;
    }
    o += 4;
    let tree = u32_at(b, o + 8)? as usize;
    let declared = u32_at(b, o + 12)? as usize;
    let blocks = u32_at(b, o + 16)? as usize;
    o += 20 + declared * 4;
    let count_at = o;
    let offsets = u32_at(b, o)? as usize;
    let pages = u32_at(b, o + 4)? as usize;
    o += 8;
    let page_at: Vec<usize> = (0..offsets).map(|i| u32_at(b, o + i * 4).map(|w| w as usize)).collect::<Result<_, _>>()?;
    let blocks_at = o + offsets * 4;
    let blocks_end = blocks_at + blocks * BLOCK_BYTES;
    if blocks_end >= tree || tree > b.len() {
        return Err(format!("blocks end at {blocks_end} and the tree is at {tree}: no gap of pages"));
    }
    if pages > MAX_PAGES {
        return Err(format!("{pages} texture pages, more than the game's {MAX_PAGES}"));
    }
    // A page: a format word, a mip count, then per mip a 16-byte header (its third word is the
    // pixels' length) and the pixels. The root node follows the last page.
    let mut at = blocks_end;
    for p in 0..pages {
        if let Some(&want) = page_at.get(p)
            && want != at
        {
            return Err(format!("page {p} is at {at}, but the header's offset word says {want}"));
        }
        let mips = u32_at(b, at + 4)?;
        if mips == 0 || mips > MAX_MIPS {
            return Err(format!("page {p} at {at} has {mips} mips"));
        }
        at += 8;
        for _ in 0..mips {
            at += 16 + u32_at(b, at + 8)? as usize;
        }
        if at > tree {
            return Err(format!("page {p} runs to {at}, past the tree at {tree}"));
        }
    }
    if at != tree {
        return Err(format!("{pages} pages end at {at}, but the tree starts at {tree}"));
    }
    let gap = tree - blocks_end;
    if gap % 4 != 0 {
        return Err(format!("a gap of {gap} bytes is not whole words"));
    }
    let mut v = b.to_vec();
    v[count_at..count_at + 4].copy_from_slice(&((offsets + gap / 4) as u32).to_le_bytes());
    v.copy_within(blocks_at..blocks_end, blocks_at + gap);
    v[blocks_at..blocks_at + gap].fill(0);
    Ok(View { bytes: v, pages, gap })
}
