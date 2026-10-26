//! MX vs. ATV Unleashed's PC (Steam) install: its `data/*.pak` files, read as a second source of the
//! same game's files beside SLUS-21104's disc (T12).
//!
//! ```text
//! data/common.pak ── u32 count
//!                 ── count × { char name[100]; u32 offset; u32 size }   offset from the end of the table
//!                 ── the members, back to back                          Data\Veh\mx_250_race_a.phy
//! ```
//!
//! The layout is openmx's `tools/mxvatv_pak.py` (`ls|x|check`, research/mxvatv-pc.md), and so is the
//! check: every pak's last member ends exactly at the end of the file. Names are `Data\`-rooted; the
//! `Data\` is taken off so a member is named as the PSR names the same file (`Veh\mx_250_race_a.phy`).
//! There is no compression and no `.res` level: what the disc keeps in a map's `.res` archives the
//! paks hold loose (notes/mxvatv.md, *The PC install*).

use std::path::{Path, PathBuf};

/// One table entry: a 100-byte name, then offset and size.
pub const ENTRY: usize = 108;
const NAME: usize = ENTRY - 8;

/// The root every pak member's name starts with, taken off.
pub const ROOT: &str = "Data\\";

/// One member of a pak.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PakEntry {
    /// Without the `Data\`: `Veh\mx_250_race_a.phy`.
    pub name: String,
    /// Byte offset in the pak file (the table's offset plus the table's end).
    pub at: u64,
    pub size: u32,
}

/// One pak's table.
#[derive(Clone, Debug)]
pub struct Pak {
    pub path: PathBuf,
    /// The file's size in bytes.
    pub size: u64,
    pub entries: Vec<PakEntry>,
}

impl Pak {
    /// Read a pak's table, and check that every member lies in the file and the last ends at its end.
    pub fn open(path: &Path) -> Result<Pak, String> {
        let at = |e: String| format!("{}: {e}", path.display());
        let mut f = std::fs::File::open(path).map_err(|e| at(e.to_string()))?;
        let size = f.metadata().map_err(|e| at(e.to_string()))?.len();
        let mut head = [0u8; 4];
        std::io::Read::read_exact(&mut f, &mut head).map_err(|e| at(format!("count: {e}")))?;
        let n = u32::from_le_bytes(head) as u64;
        let base = 4 + n * ENTRY as u64;
        if base > size {
            return Err(at(format!("a {n}-entry table in a {size}-byte file")));
        }
        let mut table = vec![0u8; (n as usize) * ENTRY];
        std::io::Read::read_exact(&mut f, &mut table).map_err(|e| at(format!("table: {e}")))?;
        let entries = Pak::parse_table(&table, base).map_err(at)?;
        for e in &entries {
            if e.at + e.size as u64 > size {
                return Err(at(format!("{} runs past the end", e.name)));
            }
        }
        if let Some(last) = entries.iter().max_by_key(|e| e.at) {
            if last.at + last.size as u64 != size {
                return Err(at(format!(
                    "its last member, {}, ends at {:#x}, not at the end of the file ({size:#x})",
                    last.name,
                    last.at + last.size as u64
                )));
            }
        }
        Ok(Pak { path: path.to_path_buf(), size, entries })
    }

    /// The entries of a table that ends at `base`, `Data\` taken off their names.
    pub fn parse_table(table: &[u8], base: u64) -> Result<Vec<PakEntry>, String> {
        let mut out = Vec::with_capacity(table.len() / ENTRY);
        for (k, e) in table.chunks_exact(ENTRY).enumerate() {
            let raw = &e[..NAME];
            let raw = &raw[..raw.iter().position(|&b| b == 0).unwrap_or(NAME)];
            // Latin-1, as tools/mxvatv_pak.py decodes it; every name is ASCII in practice.
            let full: String = raw.iter().map(|&b| b as char).collect();
            let name = match full.get(..ROOT.len()) {
                Some(p) if p.eq_ignore_ascii_case(ROOT) => full[ROOT.len()..].to_string(),
                _ => return Err(format!("entry {k}, {full:?}, is not under {ROOT}")),
            };
            let off = u32::from_le_bytes(e[NAME..NAME + 4].try_into().expect("4 bytes")) as u64;
            let size = u32::from_le_bytes(e[NAME + 4..].try_into().expect("4 bytes"));
            out.push(PakEntry { name, at: base + off, size });
        }
        Ok(out)
    }
}

/// True when `path` names PC paks rather than a disc image: a `.pak` file, or a directory holding
/// any (the install's `data/`), or an install whose `data/` does.
pub fn is_paks(path: &Path) -> bool {
    pak_files(path).is_ok_and(|v| !v.is_empty())
}

/// The paks a path stands for: itself if it is a `.pak`, every `*.pak` in it if it is a directory
/// (by name, so the order is fixed), or in its `data/` if it has none of its own. Several paths may
/// be given in one, joined as `$PATH` is (`a.pak:b.pak`).
pub fn pak_files(path: &Path) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    for p in std::env::split_paths(path.as_os_str()) {
        if p.is_file() {
            if !has_pak_extension(&p) {
                return Err(format!("{}: not a .pak", p.display()));
            }
            out.push(p);
        } else if p.is_dir() {
            let mut here = paks_in(&p)?;
            if here.is_empty() {
                if let Some(data) = std::fs::read_dir(&p)
                    .map_err(|e| format!("{}: {e}", p.display()))?
                    .filter_map(|e| e.ok().map(|e| e.path()))
                    .find(|d| d.is_dir() && d.file_name().is_some_and(|n| n.eq_ignore_ascii_case("data")))
                {
                    here = paks_in(&data)?;
                }
            }
            out.extend(here);
        } else {
            return Err(format!("{}: no such file or directory", p.display()));
        }
    }
    Ok(out)
}

fn has_pak_extension(p: &Path) -> bool {
    p.extension().is_some_and(|x| x.eq_ignore_ascii_case("pak"))
}

fn paks_in(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file() && has_pak_extension(p))
        .collect();
    v.sort();
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, off: u32, size: u32) -> Vec<u8> {
        let mut e = vec![0u8; ENTRY];
        e[..name.len()].copy_from_slice(name.as_bytes());
        e[NAME..NAME + 4].copy_from_slice(&off.to_le_bytes());
        e[NAME + 4..].copy_from_slice(&size.to_le_bytes());
        e
    }

    #[test]
    fn a_table_is_named_without_data_and_placed_after_itself() {
        let mut t = entry("Data\\Veh\\a.phy", 0, 3);
        t.extend(entry("data\\Rider\\Anim\\b.ini", 3, 5));
        let base = 4 + 2 * ENTRY as u64;
        let got = Pak::parse_table(&t, base).unwrap();
        assert_eq!(got[0], PakEntry { name: "Veh\\a.phy".into(), at: base, size: 3 });
        assert_eq!(got[1], PakEntry { name: "Rider\\Anim\\b.ini".into(), at: base + 3, size: 5 });
        assert!(Pak::parse_table(&entry("Veh\\a.phy", 0, 1), base).is_err());
    }

    #[test]
    fn a_pak_must_end_with_its_last_member() {
        let dir = std::env::temp_dir().join(format!("discx-pak-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut f = 2u32.to_le_bytes().to_vec();
        f.extend(entry("Data\\x.txt", 0, 2));
        f.extend(entry("Data\\y.txt", 2, 3));
        f.extend(b"hiabc");
        let good = dir.join("good.pak");
        std::fs::write(&good, &f).unwrap();
        let p = Pak::open(&good).unwrap();
        assert_eq!(p.entries.len(), 2);
        f.push(0);
        let bad = dir.join("bad.PAK");
        std::fs::write(&bad, &f).unwrap();
        assert!(Pak::open(&bad).unwrap_err().contains("not at the end"));
        assert_eq!(pak_files(&dir).unwrap(), vec![bad.clone(), good.clone()]);
        assert!(is_paks(&dir) && is_paks(&good));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
