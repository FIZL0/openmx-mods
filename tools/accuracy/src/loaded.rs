//! What the original opened for a map (M12): the resource names it kept in a ride, against the files
//! the mod holds.
//!
//! MX vs. ATV Unleashed keeps the name of every resource it loads by name in one string pool on the
//! heap, in the order it asked for them, from the map's `env\<map>\<map>.scn` on: its props
//! (`geo\*.psg`), collision (`col\*.col`), materials (`tex\*.psm`), species (`eco\*.psg`), the sky
//! cube, the terrain, then the rider's and the machine's. Textures (`.pst`) are not named there: a
//! material opens its own. The pool is found by the scene's name and read to its ends.
//!
//! Each name is the mod's ([`Have::Path`]) when the mod has a file at that path in any case, or
//! [`Have::Base`] when only its file name is in the mod (the archive keeps it in another folder, where
//! the game's lookup by whole path would still find it in the loaded archive). Names from folders that
//! belong to the rider, the machine, the UI and the language ([`NOT_THE_MAP`]) are the game's, not the
//! map's, and are counted apart.

use std::collections::BTreeSet;
use std::path::Path;

/// Names that are the game's own, loaded on every map: the rider, the machines, the UI, text, fonts,
/// the machines' sounds, and the machines' particles (`Part\mx*.dat` and the emitters of wrecks, feet,
/// exhaust and tyres; a map's own particles are its `Part\<map>particles.prt`).
pub const NOT_THE_MAP: [&str; 18] = [
    "loc\\",
    "veh\\",
    "rider\\",
    "ui\\",
    "font\\",
    "aud\\vehicle",
    "aud\\event",
    "aud\\carengine",
    "aud\\machine",
    "part\\mxparticles.dat",
    "part\\mxaiparticles.dat",
    "part\\mxparticlezones.dat",
    "part\\wreck",
    "part\\foot",
    "part\\exhaust",
    "part\\tire",
    "part\\mxfront",
    "part\\mxtire",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Have {
    Path,
    Base,
    No,
}

/// The pool's names, in order, or none when the image has no pool for this map.
pub fn pool(ram: &[u8], map: &str) -> Vec<String> {
    let want = format!("env\\{0}\\{0}.scn", map.to_ascii_lowercase());
    let lower: Vec<u8> = ram.iter().map(u8::to_ascii_lowercase).collect();
    let Some(at) = lower.windows(want.len()).position(|w| w == want.as_bytes()) else {
        return Vec::new();
    };
    let text = |c: u8| c == 0 || (32..127).contains(&c);
    let gap = |i: usize| i >= 8 && ram[i - 8..i].iter().all(|&c| c == 0);
    let mut a = at;
    while a > 0 && text(ram[a - 1]) && !gap(a) {
        a -= 1;
    }
    let mut b = at;
    while b < ram.len() && text(ram[b]) && !(b + 8 <= ram.len() && ram[b..b + 8].iter().all(|&c| c == 0)) {
        b += 1;
    }
    ram[a..b]
        .split(|&c| c == 0)
        .filter(|w| w.len() >= 3)
        .map(|w| String::from_utf8_lossy(w).into_owned())
        .filter(|n| n.contains('\\') && n.rsplit('\\').next().is_some_and(|f| f.contains('.')))
        .collect()
}

/// Every file in the mod: (whole paths, file names), lower case, `\`-separated.
pub fn mod_files(dir: &Path) -> (BTreeSet<String>, BTreeSet<String>) {
    let (mut paths, mut names) = (BTreeSet::new(), BTreeSet::new());
    let mut stack = vec![dir.to_owned()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if let Ok(rel) = p.strip_prefix(dir) {
                let rel = rel.to_string_lossy().replace('/', "\\").to_ascii_lowercase();
                names.insert(rel.rsplit('\\').next().unwrap_or("").to_owned());
                paths.insert(rel);
            }
        }
    }
    (paths, names)
}

pub fn is_the_maps(name: &str) -> bool {
    let l = name.to_ascii_lowercase();
    !NOT_THE_MAP.iter().any(|p| l.starts_with(p)) && !l.ends_with(".res")
}

/// Each of the map's names once, with what the mod has of it.
pub fn audit(names: &[String], paths: &BTreeSet<String>, files: &BTreeSet<String>) -> Vec<(String, Have)> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for n in names {
        let l = n.to_ascii_lowercase();
        if !is_the_maps(&l) || !seen.insert(l.clone()) {
            continue;
        }
        let have = if paths.contains(&l) {
            Have::Path
        } else if files.contains(l.rsplit('\\').next().unwrap_or("")) {
            Have::Base
        } else {
            Have::No
        };
        out.push((n.clone(), have));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_pool_around_the_scene_and_sorts_it() {
        let mut ram = vec![0u8; 64];
        ram.extend_from_slice(b"WantsX = FALSE\0loc\\a.loc\0env\\nat05\\nat05.scn\0\0geo\\barn.psg\0Col\\Barn.col\0veh\\bike.psg\0Env\\nat05\\nat05.res\0tex\\x.psm\0");
        ram.extend(vec![0u8; 64]);
        let names = pool(&ram, "Nat05");
        assert_eq!(names, ["loc\\a.loc", "env\\nat05\\nat05.scn", "geo\\barn.psg", "Col\\Barn.col", "veh\\bike.psg", "Env\\nat05\\nat05.res", "tex\\x.psm"]);
        let paths: BTreeSet<String> = ["env\\nat05\\nat05.scn", "col\\barn.col"].map(String::from).into();
        let files: BTreeSet<String> = ["nat05.scn", "barn.col", "barn.psg"].map(String::from).into();
        let a = audit(&names, &paths, &files);
        assert_eq!(
            a,
            [
                ("env\\nat05\\nat05.scn".to_owned(), Have::Path),
                ("geo\\barn.psg".to_owned(), Have::Base),
                ("Col\\Barn.col".to_owned(), Have::Path),
                ("tex\\x.psm".to_owned(), Have::No),
            ]
        );
    }
}
