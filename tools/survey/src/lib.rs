//! Every map on a disc, read through openmx's readers, one row per map.
//!
//! ```text
//! <map>.res ──┬── <MAP>.ps2.trn   Terrain::parse   levels, leaves, spacing, side, blocks, texture pages
//!             ├── <MAP>.scn       Scene::parse     terrain keys, models, collision placements
//!             │   (<MAP>.scn.bpb  Untamed's binary scene: tools/bpb to INI, then Scene::parse)
//!             └── Col\*.col       Collision::parse class and size, or why it is refused
//! ```
//!
//! Two files on these discs are refused by openmx as they stand, and both are read here through a
//! *view*: a copy with one header word changed and the bytes the reader does not expect taken out of
//! its way. Nothing else in the copy moves, and the reader still checks every byte it accounts for.
//!
//! - **Terrains with texture pages** (openmx MD1). The header's texture table is non-empty and the pages
//!   sit between the last leaf block and the tree, where `Terrain::parse` wants the tree to start. This
//!   crate reads the table and walks the pages itself ([`Textures`]), then hands the reader a view in
//!   which the blocks end where the tree starts ([`terrain_view`]). A terrain the reader takes unchanged
//!   is read unchanged, so once MD1 lands the view stops being used and the row says so.
//! - **Untamed's map `.res`**, whose directory is padded with zero bytes past its last entry, so
//!   `Res::parse` finds fewer directory bytes than the header claims ([`res_view`]).

use game_sim::archive::Res;
use game_sim::track::{Collision, Scene, Terrain};
use std::borrow::Cow;

/// One leaf block of a PLN v7 terrain, as `game_sim::track` lays it out.
pub const BLOCK_BYTES: usize = 2129;
/// Samples along one side of a leaf: 17 stored, 16 of them its own.
const LEAF_SAMPLES: u32 = 16;

// ---------------------------------------------------------------------------------------------------
// the terrain's texture table
// ---------------------------------------------------------------------------------------------------

/// Where the parts of a `.trn` header are, up to the blocks.
#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    /// Offset of the per-block word count (`declared`), and that count.
    pub declared_at: usize,
    pub declared: u32,
    pub blocks: u32,
    /// The header's tree offset.
    pub tree_at: usize,
    /// The texture count, the count again, and the table of page offsets.
    pub textures: u32,
    pub textures_again: u32,
    pub table: Vec<u32>,
    pub blocks_at: usize,
    pub blocks_end: usize,
}

/// One texture page between the blocks and the tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Page {
    pub at: u32,
    pub format: u32,
    pub mips: u32,
    /// The first mip's width and height.
    pub width: u32,
    pub height: u32,
    /// The whole record: format, mip count, and per mip a 16-byte header and its pixels.
    pub bytes: u32,
}

/// The texture table and the pages it points at.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Textures {
    pub pages: Vec<Page>,
    /// The bytes between the last block and the tree.
    pub gap: u32,
}

fn word(b: &[u8], at: usize) -> Result<u32, String> {
    b.get(at..at + 4)
        .map(|w| u32::from_le_bytes(w.try_into().expect("4 bytes")))
        .ok_or_else(|| {
            format!(
                "terrain header: wanted a word at {at} of a {}-byte file",
                b.len()
            )
        })
}

/// The `.trn` header up to its blocks, read here because `Terrain::parse` keeps the texture table to
/// itself (and refuses a non-empty one in the place the pages are; openmx MD1).
///
/// ```text
/// "PLN\0" u8 version  char[4] platform  char[32] name  u32 palette?  [1024 palette]
/// f32 scale  i32 levels  i32 tree_at  i32 declared  i32 blocks  u32[declared]
/// i32 textures  i32 textures_again  u32[textures] page offsets      then the blocks
/// ```
pub fn layout(b: &[u8]) -> Result<Layout, String> {
    if b.get(..4) != Some(b"PLN\0") {
        return Err("not a terrain: no PLN tag".to_owned());
    }
    let mut at = 4 + 1 + 4 + 32;
    let palette = word(b, at)?;
    at += 4 + if palette == 0 { 0 } else { 1024 };
    at += 4; // the scale
    let _levels = word(b, at)?;
    let tree_at = word(b, at + 4)? as usize;
    let declared_at = at + 8;
    let declared = word(b, declared_at)?;
    let blocks = word(b, declared_at + 4)?;
    if declared > 1 << 20 || blocks > 1 << 20 {
        return Err(format!("terrain counts {declared}, {blocks}"));
    }
    at = declared_at + 8 + declared as usize * 4;
    let textures = word(b, at)?;
    let textures_again = word(b, at + 4)?;
    if textures > 1 << 12 {
        return Err(format!("terrain texture count {textures}"));
    }
    at += 8;
    let table = (0..textures as usize)
        .map(|i| word(b, at + i * 4))
        .collect::<Result<Vec<_>, _>>()?;
    let blocks_at = at + textures as usize * 4;
    Ok(Layout {
        declared_at,
        declared,
        blocks,
        tree_at,
        textures,
        textures_again,
        table,
        blocks_at,
        blocks_end: blocks_at + blocks as usize * BLOCK_BYTES,
    })
}

/// Walk the pages from the end of the blocks, as the game's loader does (`notes/mxu-terrain-limits.md`):
/// a format word, a mip count, then per mip a 16-byte header (width, height, pixel bytes, one more word)
/// and the pixels. Every table entry must name the page the walk is at, and the last page must end
/// where the tree starts.
pub fn textures(b: &[u8], l: &Layout) -> Result<Textures, String> {
    if l.blocks_end > l.tree_at {
        return Err(format!(
            "{} blocks end at {}, past the tree at {}",
            l.blocks, l.blocks_end, l.tree_at
        ));
    }
    let mut pages = Vec::new();
    let mut at = l.blocks_end;
    for (i, &want) in l.table.iter().enumerate() {
        if want as usize != at {
            return Err(format!(
                "texture {i} is listed at {want}, but the pages reach it at {at}"
            ));
        }
        let format = word(b, at)?;
        let mips = word(b, at + 4)?;
        if mips == 0 || mips > 16 {
            return Err(format!("texture {i} at {at}: {mips} mips"));
        }
        let (width, height) = (word(b, at + 8)?, word(b, at + 12)?);
        let mut end = at + 8;
        for _ in 0..mips {
            let pixels = word(b, end + 8)? as usize;
            end += 16 + pixels;
        }
        pages.push(Page {
            at: want,
            format,
            mips,
            width,
            height,
            bytes: (end - at) as u32,
        });
        at = end;
    }
    if at != l.tree_at {
        return Err(format!(
            "{} texture pages end at {at}, but the tree starts at {}",
            l.table.len(),
            l.tree_at
        ));
    }
    Ok(Textures {
        pages,
        gap: (l.tree_at - l.blocks_end) as u32,
    })
}

/// The terrain as `Terrain::parse` will take it: the file itself, or a view of it in which the blocks
/// end where the tree starts.
///
/// The view keeps every byte from the tree on at its own offset (every node and child offset in the
/// tree is absolute, so the tree cannot move). It moves the blocks up against the tree, and grows the
/// per-block word array (`declared`, which the reader skips) over the space they leave, with the
/// texture count and table zeroed. The blocks the reader then sees are the file's own, byte for byte.
pub fn terrain_view(b: &[u8], l: &Layout) -> Result<Vec<u8>, String> {
    let blocks_bytes = l.blocks as usize * BLOCK_BYTES;
    let new_blocks_at = l
        .tree_at
        .checked_sub(blocks_bytes)
        .ok_or("the blocks do not fit in front of the tree")?;
    let array_at = l.declared_at + 8;
    let grown = new_blocks_at
        .checked_sub(array_at + 8)
        .filter(|n| n % 4 == 0)
        .ok_or_else(|| format!("no whole-word view puts the blocks at {new_blocks_at}"))?;
    let mut v = b.to_vec();
    v[l.declared_at..l.declared_at + 4].copy_from_slice(&((grown / 4) as u32).to_le_bytes());
    let old_array_end = array_at + l.declared as usize * 4;
    v[old_array_end..new_blocks_at].fill(0);
    v[new_blocks_at..l.tree_at].copy_from_slice(&b[l.blocks_at..l.blocks_end]);
    Ok(v)
}

// ---------------------------------------------------------------------------------------------------
// one terrain
// ---------------------------------------------------------------------------------------------------

/// What a terrain row says.
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainRow {
    pub file: String,
    pub levels: u32,
    /// Leaves present, and the extent they cover in leaves.
    pub leaves: u32,
    pub across: u32,
    pub along: u32,
    /// The file's scale: feet per sample.
    pub spacing: f32,
    pub blocks: u32,
    pub textures: Textures,
    /// The texture count's second copy, where it differs from the first.
    pub textures_again: Option<u32>,
    /// Read through [`terrain_view`] because `Terrain::parse` refused the file itself; its error.
    pub viewed: Option<String>,
}

impl TerrainRow {
    /// The side in feet: the leaves across, times their samples, times the spacing.
    pub fn side(&self) -> f32 {
        (self.across * LEAF_SAMPLES) as f32 * self.spacing
    }
}

/// Read one terrain: through `Terrain::parse` if it takes the file, otherwise through the view.
pub fn terrain(file: &str, b: &[u8]) -> Result<TerrainRow, String> {
    let l = layout(b)?;
    let tex = textures(b, &l)?;
    let (view, viewed) = match Terrain::parse(b) {
        Ok(_) => (Cow::Borrowed(b), None),
        Err(why) => (Cow::Owned(terrain_view(b, &l)?), Some(why)),
    };
    let t = Terrain::parse(&view).map_err(|e| format!("even through the view: {e}"))?;
    // One sample per leaf over the tree's whole range: 16 root cells, then 16 per level, of 16 samples.
    let cells = 16u32.pow(t.levels.min(3));
    let (mut n, mut lo, mut hi) = (0u32, (u32::MAX, u32::MAX), (0u32, 0u32));
    for z in 0..cells {
        for x in 0..cells {
            if t.leaf(x * LEAF_SAMPLES, z * LEAF_SAMPLES).is_some() {
                n += 1;
                lo = (lo.0.min(x), lo.1.min(z));
                hi = (hi.0.max(x), hi.1.max(z));
            }
        }
    }
    let (across, along) = if n == 0 {
        (0, 0)
    } else {
        (hi.0 - lo.0 + 1, hi.1 - lo.1 + 1)
    };
    Ok(TerrainRow {
        file: file.to_owned(),
        levels: t.levels,
        leaves: n,
        across,
        along,
        spacing: t.scale.0,
        blocks: t.blocks,
        textures: tex,
        textures_again: (l.textures_again != l.textures).then_some(l.textures_again),
        viewed,
    })
}

// ---------------------------------------------------------------------------------------------------
// a .res
// ---------------------------------------------------------------------------------------------------

/// A `.res` as `Res::parse` will take it, and what had to be done for that.
pub struct ResView {
    pub bytes: Vec<u8>,
    pub res: Res,
    /// Zero bytes taken off the end of the directory, where the file pads it.
    pub padding: usize,
}

/// Open a `.res` through `Res::parse`; if it refuses the file, try it with the directory's trailing
/// zero bytes taken out and the directory length cut to match (Untamed's map archives). Only the cut
/// the reader then accepts in full (directory and every block accounted for) is used.
pub fn res_view(bytes: Vec<u8>) -> Result<ResView, String> {
    let why = match Res::parse(&bytes) {
        Ok(res) => {
            return Ok(ResView {
                bytes,
                res,
                padding: 0,
            });
        }
        Err(why) => why,
    };
    let refused = || format!("Res::parse: {why}");
    let table = word(&bytes, 0x10).map_err(|_| refused())? as usize;
    let count_at = 0x14 + table;
    let dir_bytes = word(&bytes, count_at + 4).map_err(|_| refused())? as usize;
    let dir_at = count_at + 8;
    let dir = bytes.get(dir_at..dir_at + dir_bytes).ok_or_else(refused)?;
    let zeros = dir.iter().rev().take_while(|&&b| b == 0).count();
    // The last entry ends in its size, whose high bytes may be zero too: try the longest cut first.
    for cut in (1..=zeros).rev().take(4) {
        let mut v = Vec::with_capacity(bytes.len() - cut);
        v.extend_from_slice(&bytes[..count_at + 4]);
        v.extend_from_slice(&((dir_bytes - cut) as u32).to_le_bytes());
        v.extend_from_slice(&bytes[dir_at..dir_at + dir_bytes - cut]);
        v.extend_from_slice(&bytes[dir_at + dir_bytes..]);
        if let Ok(res) = Res::parse(&v) {
            return Ok(ResView {
                bytes: v,
                res,
                padding: cut,
            });
        }
    }
    Err(refused())
}

impl ResView {
    /// The members a survey reads: terrains, scenes and collision, by name.
    pub fn files(&self) -> Result<Vec<(String, Vec<u8>)>, String> {
        let mut out = Vec::new();
        for e in &self.res.entries {
            if wanted(&e.name) {
                out.push((e.name.clone(), self.res.read(&self.bytes, e)?));
            }
        }
        Ok(out)
    }
}

/// A terrain, a scene (text or binary) or a `.col`.
pub fn wanted(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n.ends_with(".trn") || n.ends_with(".scn") || n.ends_with(".scn.bpb") || n.ends_with(".col")
}

// ---------------------------------------------------------------------------------------------------
// one map
// ---------------------------------------------------------------------------------------------------

/// What the map's scene says.
#[derive(Clone, Debug, PartialEq)]
pub enum SceneRow {
    Ini {
        file: String,
        name: String,
        terrain_file: String,
        width: u32,
        breadth: u32,
        width_scale: f32,
        breadth_scale: f32,
        models: usize,
        placements: usize,
        /// The scene was Untamed's binary `<MAP>.scn.bpb`, this many bytes, decoded by `tools/bpb`.
        bpb: Option<usize>,
        /// Distinct `.col` files the placements name, and how many of them were found in the map's own
        /// files, found loose on the disc, and not found.
        placed_files: usize,
        in_map: usize,
        loose: usize,
        missing: Vec<String>,
    },
    Refused {
        file: String,
        why: String,
    },
    None,
}

/// One `.col` and what `Collision::parse` made of it.
#[derive(Clone, Debug, PartialEq)]
pub struct ColRow {
    pub file: String,
    pub read: Result<String, String>,
}

/// One `.col`, read: its class, version and size, or the reader's refusal.
pub fn col(file: &str, b: &[u8]) -> ColRow {
    let read = Collision::parse(b).map(|c| match c {
        Collision::Mesh(m) => format!(
            "mesh v{} {} vertices {} triangles",
            m.version,
            m.vertices.len(),
            m.triangles.len()
        ),
        Collision::Rays(r) => format!("rays v{} {} segments", r.version, r.segments.len()),
    });
    ColRow {
        file: file.to_owned(),
        read,
    }
}

/// One map: where its files came from and what each reader said.
#[derive(Clone, Debug, PartialEq)]
pub struct MapRow {
    pub map: String,
    pub source: String,
    /// How the source had to be opened, where it was not plain (a `.res` view).
    pub source_note: Option<String>,
    /// Zero bytes the `.res` view took off the end of the directory ([`res_view`]).
    pub res_padding: usize,
    pub terrain: Result<TerrainRow, String>,
    pub scene: SceneRow,
    /// Every `.col` in the map's own files.
    pub cols: Vec<ColRow>,
    /// More than one terrain or scene in the map's files: the ones not surveyed.
    pub extra: Vec<String>,
}

fn base(name: &str) -> &str {
    name.rsplit(['\\', '/']).next().unwrap_or(name)
}

fn stem(name: &str) -> String {
    let b = base(name).to_ascii_lowercase();
    b.split('.').next().unwrap_or("").to_owned()
}

/// Survey one map from its files (out of its `.res`, or loose). `loose_col` finds a `.col` the scene
/// places that is not among them, the way the disc keeps shared shapes in its own `Col\` folder; it
/// says only whether one is there.
pub fn map(
    map: &str,
    source: &str,
    files: &[(String, Vec<u8>)],
    loose_col: &mut dyn FnMut(&str) -> bool,
) -> MapRow {
    let lower = |n: &String| n.to_ascii_lowercase();
    let want = map.to_ascii_lowercase();
    // The map's own terrain and scene by stem, where it holds several; otherwise the only one.
    let pick = |ends: &str| -> (Option<&(String, Vec<u8>)>, Vec<String>) {
        let all: Vec<&(String, Vec<u8>)> = files
            .iter()
            .filter(|(n, _)| lower(n).ends_with(ends))
            .collect();
        let main = all
            .iter()
            .find(|(n, _)| stem(n) == want)
            .or(all.first())
            .copied();
        let rest = all
            .iter()
            .filter(|f| main.is_none_or(|m| !std::ptr::eq(**f, m)))
            .map(|(n, _)| n.clone())
            .collect();
        (main, rest)
    };
    let (trn, mut extra) = pick(".trn");
    let (scn, more) = pick(".scn");
    extra.extend(more);
    let (bpb, _) = pick(".scn.bpb");

    let terrain = match trn {
        Some((n, b)) => terrain(n, b),
        None => Err("no terrain".to_owned()),
    };
    let cols: Vec<ColRow> = files
        .iter()
        .filter(|(n, _)| lower(n).ends_with(".col"))
        .map(|(n, b)| col(n, b))
        .collect();

    let mut read = |file: &String, ini: &[u8], bpb: Option<usize>| match Scene::parse(ini) {
        Ok(s) => {
            let mut placed: Vec<String> = s
                .collision
                .iter()
                .map(|p| base(&p.file).to_ascii_lowercase())
                .collect();
            placed.sort();
            placed.dedup();
            let (mut in_map, mut loose, mut missing) = (0, 0, Vec::new());
            for p in &placed {
                if cols.iter().any(|c| base(&c.file).eq_ignore_ascii_case(p)) {
                    in_map += 1;
                } else if loose_col(p) {
                    loose += 1;
                } else {
                    missing.push(p.clone());
                }
            }
            SceneRow::Ini {
                file: file.clone(),
                name: s.name,
                terrain_file: s.terrain_file,
                width: s.terrain.width,
                breadth: s.terrain.breadth,
                width_scale: s.terrain.width_scale.0,
                breadth_scale: s.terrain.breadth_scale.0,
                models: s.models.len(),
                placements: s.collision.len(),
                bpb,
                placed_files: placed.len(),
                in_map,
                loose,
                missing,
            }
        }
        Err(why) => SceneRow::Refused {
            file: file.clone(),
            why,
        },
    };
    let scene = match (scn, bpb) {
        (Some((file, b)), _) => read(file, b, None),
        // Untamed's: the same INI, compiled; `tools/bpb` gives the text back.
        (None, Some((file, b))) => match bpb::parse(b) {
            Ok(t) => read(file, &t.to_ini(), Some(b.len())),
            Err(why) => SceneRow::Refused {
                file: file.clone(),
                why: format!("bpb: {why}"),
            },
        },
        (None, None) => SceneRow::None,
    };
    MapRow {
        map: map.to_owned(),
        source: source.to_owned(),
        source_note: None,
        res_padding: 0,
        terrain,
        scene,
        cols,
        extra,
    }
}

/// Survey one map `.res` from its bytes.
pub fn map_res(
    map_name: &str,
    source: &str,
    bytes: Vec<u8>,
    loose_col: &mut dyn FnMut(&str) -> bool,
) -> Result<MapRow, String> {
    let v = res_view(bytes)?;
    let mut row = map(map_name, source, &v.files()?, loose_col);
    if v.padding > 0 {
        row.res_padding = v.padding;
        row.source_note = Some(
            "`Res::parse` refuses the `.res`: its directory is padded with zero bytes past the last \
             entry, so fewer bytes are read than the header claims; read through a view without them"
                .to_owned(),
        );
    }
    Ok(row)
}

// ---------------------------------------------------------------------------------------------------
// the tables
// ---------------------------------------------------------------------------------------------------

/// The header of [`MapRow::cells`].
pub const COLUMNS: &[&str] = &[
    "map",
    "levels",
    "leaves",
    "spacing (ft)",
    "side (ft)",
    "blocks",
    "texture pages",
    "scene",
    "scene terrain keys",
    "models",
    "collision placements",
    "`.col` read",
    "`.col` refused",
];

/// A single as the file holds it: the shortest decimal that reads back to the same bits.
fn num(x: f32) -> String {
    format!("{x}")
}

impl MapRow {
    /// The `.col` in the map's files that the reader refused.
    pub fn refused(&self) -> impl Iterator<Item = (&str, &str)> {
        self.cols.iter().filter_map(|c| {
            c.read
                .as_ref()
                .err()
                .map(|why| (c.file.as_str(), why.as_str()))
        })
    }

    /// One markdown table row, in [`COLUMNS`] order.
    pub fn cells(&self) -> Vec<String> {
        let mut c = vec![self.map.clone()];
        match &self.terrain {
            Ok(t) => {
                let leaves = if t.leaves == t.across * t.along {
                    format!("{}×{}", t.across, t.along)
                } else {
                    format!("{} in {}×{}", t.leaves, t.across, t.along)
                };
                let mut pages = match t.textures.pages.first() {
                    None => "0".to_owned(),
                    Some(p) => {
                        let same = t.textures.pages.iter().all(|q| {
                            (q.format, q.mips, q.width, q.height, q.bytes)
                                == (p.format, p.mips, p.width, p.height, p.bytes)
                        });
                        if same {
                            format!(
                                "{} × {}×{} fmt {:#x}, {} mip, {} B",
                                t.textures.pages.len(),
                                p.width,
                                p.height,
                                p.format,
                                p.mips,
                                p.bytes
                            )
                        } else {
                            format!("{} (mixed), {} B", t.textures.pages.len(), t.textures.gap)
                        }
                    }
                };
                if t.viewed.is_some() {
                    pages.push_str(" †");
                }
                if let Some(again) = t.textures_again {
                    pages.push_str(&format!(" (count again: {again})"));
                }
                c.extend([
                    t.levels.to_string(),
                    leaves,
                    num(t.spacing),
                    num(t.side()),
                    t.blocks.to_string(),
                    pages,
                ]);
            }
            Err(why) => {
                c.push(format!("terrain refused: {why}"));
                c.extend(std::iter::repeat_n("–".to_owned(), 5));
            }
        }
        match &self.scene {
            SceneRow::Ini {
                name,
                terrain_file,
                width,
                breadth,
                width_scale,
                breadth_scale,
                models,
                placements,
                bpb,
                placed_files,
                in_map,
                loose,
                missing,
                ..
            } => {
                let mut keys = format!("{width}×{breadth} @ {}", num(*width_scale));
                if breadth_scale != width_scale {
                    keys.push_str(&format!(" / {}", num(*breadth_scale)));
                }
                if let Ok(t) = &self.terrain
                    && !base(&t.file).eq_ignore_ascii_case(base(terrain_file))
                {
                    keys.push_str(&format!(", TerrainFile={terrain_file}"));
                }
                let mut placed = format!("{placements} ({placed_files} files: {in_map} here");
                if *loose > 0 {
                    placed.push_str(&format!(", {loose} loose"));
                }
                if !missing.is_empty() {
                    placed.push_str(&format!(", missing {}", missing.join(" ")));
                }
                placed.push(')');
                let mut scene = if name.is_empty() {
                    "(no SceneName)".to_owned()
                } else {
                    name.clone()
                };
                if let Some(bytes) = bpb {
                    scene.push_str(&format!(" (BPB, {bytes} B)"));
                }
                c.extend([scene, keys, models.to_string(), placed]);
            }
            SceneRow::Refused { why, .. } => {
                c.push(format!("refused: {why}"));
                c.extend(std::iter::repeat_n("–".to_owned(), 3));
            }
            SceneRow::None => c.extend(std::iter::repeat_n("–".to_owned(), 4)),
        }
        let refused: Vec<String> = self.refused().map(|(f, _)| base(f).to_owned()).collect();
        c.push(format!(
            "{} of {}",
            self.cols.len() - refused.len(),
            self.cols.len()
        ));
        c.push(if refused.is_empty() {
            "0".to_owned()
        } else {
            refused.join(" ")
        });
        c
    }
}

/// A markdown row, with `|` in a cell escaped.
pub fn md_row(cells: &[String]) -> String {
    let cells: Vec<String> = cells.iter().map(|c| c.replace('|', "\\|")).collect();
    format!("| {} |", cells.join(" | "))
}

/// The per-map table.
pub fn table(rows: &[MapRow]) -> String {
    let head: Vec<String> = COLUMNS.iter().map(|s| s.to_string()).collect();
    let mut out = vec![md_row(&head), format!("|{}", "---|".repeat(COLUMNS.len()))];
    out.extend(rows.iter().map(|r| md_row(&r.cells())));
    out.join("\n") + "\n"
}
