//! MX Unleashed's freeride and national maps as track mods, only ever a world's tiles (§1): M2's
//! importer with MXU's two archives in front (C4).
//!
//! ```text
//! MX.PSR ── Tracks\free06.res ───┐
//!        ── Tracks\free06_2.res ─┴─ unpacked, member paths kept ──> <out>/mxu_free06/
//! ```
//!
//! What differs from MX vs. ATV Unleashed is said where it is done:
//!
//! - a track is the two archives the shell loads, `Tracks\<track>.res` and `Tracks\<track>_2.res`
//!   (`track_res_load`, boot `0x00205c40`, openmx's oracle), with no folder of its own on the PSR;
//! - the `_ss` pair is the same track cut back for two views (`world_load`: fewer props, no ruts),
//!   so it is not a map of its own and is never imported; supercross (`sx*`) is a stadium and not a
//!   C4 map;
//! - there is no `.atyList`: the name is the track's own (`free06`), and the start is the scene's;
//! - no loose PSR member is pulled: the PSR's loose members are `UI\`, `AppStart\` and one
//!   `Col\template.cmt`, none of which a map's files name.

use crate::{FreeRide, Map, Origin, base};
use discx::Disc;
use std::collections::BTreeMap;

pub const BOOT: &str = "SLUS_207.67";
pub const GAME: &str = "MX Unleashed";
pub const MXU: Origin = Origin { game: GAME, boot: BOOT, prefix: "mxu_", archives: "Tracks" };

/// Whether a track is one C4 takes: a freeride (`freeNN`) or a national (`natNN`).
pub fn chosen(track: &str) -> bool {
    let t = track.to_ascii_lowercase();
    ["free", "nat"].iter().any(|p| t.strip_prefix(p).is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit())))
}

/// Every track on the disc: each `Tracks\<track>.res` that is not a `_2` or `_ss` half, in PSR order.
pub fn tracks(disc: &Disc) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for e in disc.matching(r"Tracks\*.res") {
        let stem = base(&e.name);
        let stem = &stem[..stem.len() - 4];
        let l = stem.to_ascii_lowercase();
        if l.ends_with("_2") || l.ends_with("_ss") || out.iter().any(|f| f.eq_ignore_ascii_case(stem)) {
            continue;
        }
        out.push(stem.to_string());
    }
    out
}

/// The tracks C4 imports: [`tracks`] that are [`chosen`].
pub fn map_folders(disc: &Disc) -> Vec<String> {
    tracks(disc).into_iter().filter(|t| chosen(t)).collect()
}

/// One track's two archives off the disc.
pub fn load_map(disc: &mut Disc, track: &str) -> Result<Map, String> {
    let mut archives = Vec::new();
    for suffix in ["", "_2"] {
        let want = format!(r"{}\{track}{suffix}.res", MXU.archives);
        let e = disc.find(&want).cloned().ok_or_else(|| format!("{want}: not on the disc"))?;
        archives.push((e.name.clone(), disc.res(&e)?));
    }
    let mut elsewhere: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for e in disc.entries() {
        if !e.name.to_ascii_lowercase().ends_with(".res") {
            elsewhere.entry(base(&e.name).to_ascii_lowercase()).or_default().push(e.name.clone());
        }
    }
    let ride = FreeRide {
        name: track.to_string(),
        spawn: None,
        todo: vec![format!(
            "{track}: the name is the track's PSR name: MX Unleashed has no .atyList, and its menu names (UI\\{track}.lsi is only the loading picture) were not read; \
             no spawn is given, so it is the scene's DefaultStartPosition (§2b)"
        )],
    };
    Ok(Map {
        origin: &MXU,
        folder: track.to_string(),
        archives,
        atylist: None,
        loose: Vec::new(),
        elsewhere,
        pulled: Vec::new(),
        ride: Some(ride),
    })
}
