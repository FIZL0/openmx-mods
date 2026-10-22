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
pub const NOT_THE_MAP: [&str; 19] = [
    "loc\\",
    "veh\\",
    "rider\\",
    "ui\\",
    "font\\",
    "aud\\vehicle",
    "aud\\event",
    "aud\\carengine",
    "aud\\machine",
    // Untamed's rider sounds (`Aud\Rider.aud.bpb`, in MX08AUD.PSR), asked for on FS_C (M13).
    "aud\\rider",
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
    /// Not in the mod, and no loose member of the disc's PSR either: the game asked and fell back
    /// (MC10 asks for `env\mc10\mxparticlezones.dat`, then takes `part\`'s).
    Absent,
}

/// The [`Have::No`]s that no loose PSR member answers (`psr`: every member's name, [`discx::normalise`]d)
/// become [`Have::Absent`].
pub fn absent(rows: &mut [(String, Have)], psr: &BTreeSet<String>) {
    for r in rows.iter_mut() {
        if r.1 == Have::No && !psr.contains(&discx::normalise(&r.0)) {
            r.1 = Have::Absent;
        }
    }
}

/// The disc a mod came from, found by its boot ELF in `$DISCX_ISO` (else `iso/`), as its PSR's member
/// names; `None` when it is not there.
pub fn psr_names(boot: &str) -> Option<BTreeSet<String>> {
    let dir = std::env::var("DISCX_ISO").unwrap_or_else(|_| "iso".into());
    psr_names_in(Path::new(&dir), boot)
}

/// [`psr_names`] from the images in `dir`.
pub fn psr_names_in(dir: &Path, boot: &str) -> Option<BTreeSet<String>> {
    let (iso, _) = discx::images(dir).ok()?.into_iter().find(|(_, b)| b.as_deref().is_some_and(|b| b.eq_ignore_ascii_case(boot)))?;
    let disc = discx::Disc::open(&iso).ok()?;
    Some(disc.entries().iter().map(|e| discx::normalise(&e.name)).collect())
}

/// The pool's names, in order, or none when the image has no pool.
///
/// The pool is names one after another with two NULs between them (the PSR's directory has one, the
/// archive tables bytes between). The game keeps other lists like it (the machines' animations). And it
/// reuses the buffer: on most maps the text of a later file overwrites the pool's head, and the names
/// from the scene's on are lost up to where the overwriting stopped. Of every such run of names in the
/// image, the pool is the one holding `env\<map>\<map>.scn` (in any case), else the one with the most
/// of the map's own kinds of name (`geo\`, `col\`, `eco\`, `tex\`, `skies\`, `env\<map>\`);
/// [`whole`] says whether the scene's name is in it.
pub fn pool(ram: &[u8], map: &str) -> Vec<String> {
    let map = map.to_ascii_lowercase();
    let scene = format!("env\\{map}\\{map}.scn");
    let own = [format!("env\\{map}\\"), "geo\\".into(), "col\\".into(), "eco\\".into(), "tex\\".into(), "skies\\".into()];
    let name_byte = |c: u8| (33..127).contains(&c);
    let mut runs: Vec<Vec<String>> = Vec::new();
    let mut cur: Vec<String> = Vec::new();
    let mut i = 0;
    while i < ram.len() {
        if !name_byte(ram[i]) {
            i += 1;
            continue;
        }
        let start = i;
        while i < ram.len() && (name_byte(ram[i]) || ram[i] == b' ') {
            i += 1;
        }
        let token = String::from_utf8_lossy(&ram[start..i]).into_owned();
        let two = i + 2 < ram.len() && ram[i] == 0 && ram[i + 1] == 0 && ram[i + 2] != 0;
        if token.contains('\\') || !cur.is_empty() {
            cur.push(token);
        }
        if two {
            i += 2;
        } else {
            if cur.len() >= 2 {
                runs.push(std::mem::take(&mut cur));
            }
            cur.clear();
        }
    }
    if cur.len() >= 2 {
        runs.push(cur);
    }
    let keep = |run: Vec<String>| -> Vec<String> {
        run.into_iter().filter(|n| n.contains('\\') && n.rsplit('\\').next().is_some_and(|f| f.contains('.'))).collect()
    };
    let lower = |n: &String| n.to_ascii_lowercase();
    // Untamed's scene is `<map>.scn.bpb` (M13).
    let is_scene = |n: &String| lower(n) == scene || lower(n) == format!("{scene}.bpb");
    if let Some(run) = runs.iter().filter(|r| r.iter().any(is_scene)).max_by_key(|r| r.len()) {
        return keep(run.clone());
    }
    let score = |r: &Vec<String>| r.iter().filter(|n| own.iter().any(|o| lower(n).starts_with(o.as_str()))).count();
    // Its first name is where the overwriting stopped, and may be the tail of a name joined to the
    // text before it: it is dropped.
    runs.into_iter().max_by_key(score).filter(|r| score(r) > 0).map(|r| keep(r.into_iter().skip(1).collect())).unwrap_or_default()
}

/// Whether the pool still holds the map's scene, `env\<map>\<map>.scn` (Untamed's `.scn.bpb`, in
/// any case): when it does not, its head was overwritten and the names before the first left are not
/// known.
pub fn whole(names: &[String], map: &str) -> bool {
    let want = format!("env\\{0}\\{0}.scn", map.to_ascii_lowercase());
    names.iter().any(|n| n.eq_ignore_ascii_case(&want) || n.eq_ignore_ascii_case(&format!("{want}.bpb")))
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
        // The PSR's directory: one NUL between names. Not the pool.
        ram.extend_from_slice(b"Env\\Nat05\\Nat05.scn\0Env\\Nat05\\a.psg\0Env\\Nat05\\b.psg\0Env\\Nat05\\c.psg\0Env\\Nat05\\d.psg\0Geo\\e.psg\0Geo\\f.psg\0Geo\\g.psg\0Geo\\h.psg\0");
        ram.extend(vec![0u8; 64]);
        ram.extend_from_slice(b"WantsX = FALSE\0\0loc\\a.loc\0\0env\\nat05\\nat05.scn\0\0geo\\barn.psg\0\0Col\\Barn.col\0\0veh\\bike.psg\0\0Env\\nat05\\nat05.res\0\0tex\\x.psm\0");
        ram.extend(vec![0u8; 64]);
        let names = pool(&ram, "Nat05");
        assert!(whole(&names, "Nat05"));
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
