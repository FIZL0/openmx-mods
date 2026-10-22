//! Getting members off a disc image: the ISO 9660 root, the PSR in it, and the `.res` archives in that.
//!
//! ```text
//! image.iso ──┬── SYSTEM.CNF       BOOT2 = cdrom0:\SLUS_211.04;1   which game this is
//!             └── MXVSATV.PSR ──┬── Env\Nat01\nat01.res ──┬── Nat01.ps2.trn
//!                               │                         └── Col\*.col
//!                               └── Env\Freeride.aty
//! ```
//!
//! The PSR and `.res` layouts are read by openmx's `game_sim::archive::{Psr, Res}` and nothing else
//! (AGENTS.md). What this crate adds is the part openmx's `game-render/src/disc.rs` does only for
//! `MX.PSR`: finding the container on any of the three discs, by the boot ELF rather than the image's
//! file name, and reading members by glob.
//!
//! Names are matched as the game does: `\` and `/` are the same separator and case does not matter.

use game_sim::archive::{Entry, Psr, Res};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// The ISO 9660 sector size, which is also the PSR's.
pub const SECTOR: u64 = 2048;

/// A game this crate knows, by its boot ELF (AGENTS.md's table).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Game {
    /// The boot ELF as SYSTEM.CNF names it, without the `;1`: `SLUS_211.04`.
    pub boot: &'static str,
    /// The folder under `extracted/` this game's files go in.
    pub key: &'static str,
    pub title: &'static str,
    /// The PSR that holds the game's data, then any others on the disc (Untamed's audio).
    pub psrs: &'static [&'static str],
}

pub const GAMES: &[Game] = &[
    Game {
        boot: "SLUS_207.67",
        key: "mxu",
        title: "MX Unleashed",
        psrs: &["MX.PSR"],
    },
    Game {
        boot: "SLUS_211.04",
        key: "mxvatv",
        title: "MX vs. ATV Unleashed",
        psrs: &["MXVSATV.PSR"],
    },
    Game {
        boot: "SLUS_217.01",
        key: "untamed",
        title: "MX vs. ATV Untamed",
        psrs: &["MX08PS2.PSR", "MX08AUD.PSR"],
    },
];

impl Game {
    pub fn by_boot(boot: &str) -> Option<&'static Game> {
        GAMES.iter().find(|g| g.boot.eq_ignore_ascii_case(boot))
    }

    /// By boot ELF or by `key`: what a user may type instead of a path.
    pub fn by_name(name: &str) -> Option<&'static Game> {
        GAMES
            .iter()
            .find(|g| g.boot.eq_ignore_ascii_case(name) || g.key.eq_ignore_ascii_case(name))
    }
}

/// One file in the image's root directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IsoFile {
    /// As stored, without the `;1` version.
    pub name: String,
    /// Byte offset in the image.
    pub at: u64,
    pub size: u32,
}

/// An opened image: its root directory, which game it is, and the directory of one PSR.
pub struct Disc {
    file: File,
    pub path: PathBuf,
    pub root: Vec<IsoFile>,
    /// The boot ELF from SYSTEM.CNF, if the disc has one.
    pub boot: Option<String>,
    pub game: Option<&'static Game>,
    /// The PSR that is open, as named in the root.
    pub psr_name: String,
    /// Where it starts in the image.
    pub psr_at: u64,
    psr_size: u64,
    pub psr: Psr,
}

impl Disc {
    /// Open an image and the game's main PSR (the first in [`Game::psrs`]). A disc this crate does not
    /// know is opened if its root holds exactly one `.PSR`; otherwise the error names its boot ELF.
    pub fn open(path: &Path) -> Result<Disc, String> {
        Disc::open_psr(path, None)
    }

    /// The same, choosing the PSR by name (`MX08AUD.PSR`).
    pub fn open_psr(path: &Path, psr: Option<&str>) -> Result<Disc, String> {
        let at = |e: String| format!("{}: {e}", path.display());
        let mut file = File::open(path).map_err(|e| at(e.to_string()))?;
        let root = iso_root(&mut file).map_err(at)?;
        let boot = match root.iter().find(|f| f.name.eq_ignore_ascii_case("SYSTEM.CNF")) {
            Some(cnf) => {
                let text = read_exact_at(&mut file, cnf.at, cnf.size as usize).map_err(at)?;
                boot_elf(&String::from_utf8_lossy(&text))
            }
            None => None,
        };
        let game = boot.as_deref().and_then(Game::by_boot);
        let want = match (psr, game) {
            (Some(name), _) => name.to_string(),
            (None, Some(g)) => g.psrs[0].to_string(),
            (None, None) => {
                let psrs: Vec<&IsoFile> = root
                    .iter()
                    .filter(|f| f.name.to_ascii_uppercase().ends_with(".PSR"))
                    .collect();
                match psrs.as_slice() {
                    [one] => one.name.clone(),
                    _ => {
                        return Err(at(format!(
                            "boot ELF {} is not a known game, and the root has {} .PSR files",
                            boot.as_deref().unwrap_or("(no SYSTEM.CNF)"),
                            psrs.len()
                        )));
                    }
                }
            }
        };
        let entry = root
            .iter()
            .find(|f| f.name.eq_ignore_ascii_case(&want))
            .ok_or_else(|| at(format!("no {want} in the root directory")))?;
        let (psr_name, psr_at, psr_size) = (entry.name.clone(), entry.at, entry.size as u64);
        // The header says how long the directory is; read exactly that. MX vs. ATV's 20k-member
        // directories are far past the 64 KiB that is enough for MX.PSR.
        let head = read_exact_at(&mut file, psr_at, 0x18).map_err(at)?;
        let dir_bytes = u32::from_le_bytes([head[0x10], head[0x11], head[0x12], head[0x13]]) as u64;
        if 0x18 + dir_bytes > psr_size {
            return Err(at(format!(
                "{psr_name}: a {dir_bytes}-byte directory in a {psr_size}-byte file"
            )));
        }
        let front = read_exact_at(&mut file, psr_at, 0x18 + dir_bytes as usize).map_err(at)?;
        let psr = Psr::parse(&front).map_err(|e| at(format!("{psr_name}: {e}")))?;
        for e in &psr.entries {
            if e.at + e.size as u64 > psr_size {
                return Err(at(format!("{psr_name}: {} runs past the end", e.name)));
            }
        }
        Ok(Disc {
            file,
            path: path.to_path_buf(),
            root,
            boot,
            game,
            psr_name,
            psr_at,
            psr_size,
            psr,
        })
    }

    /// The open PSR's size in bytes.
    pub fn psr_size(&self) -> u64 {
        self.psr_size
    }

    /// Every PSR member, in directory order.
    pub fn entries(&self) -> &[Entry] {
        &self.psr.entries
    }

    /// The members a glob matches, in directory order.
    pub fn matching(&self, glob: &str) -> Vec<&Entry> {
        self.psr
            .entries
            .iter()
            .filter(|e| glob_match(glob, &e.name))
            .collect()
    }

    /// One member by name, matched without regard to case or slash.
    pub fn find(&self, name: &str) -> Option<&Entry> {
        let want = normalise(name);
        self.psr.entries.iter().find(|e| normalise(&e.name) == want)
    }

    /// One member's bytes.
    pub fn read(&mut self, e: &Entry) -> Result<Vec<u8>, String> {
        read_exact_at(&mut self.file, self.psr_at + e.at, e.size as usize)
            .map_err(|why| format!("{}: {why}", e.name))
    }

    /// One member, copied to `out` without holding it in memory (MX Unleashed's `Aud\audio.res` is
    /// 164 MB).
    pub fn copy(&mut self, e: &Entry, out: &mut impl Write) -> Result<(), String> {
        let why = |x: std::io::Error| format!("{}: {x}", e.name);
        self.file
            .seek(SeekFrom::Start(self.psr_at + e.at))
            .map_err(why)?;
        let got = std::io::copy(&mut (&mut self.file).take(e.size as u64), out).map_err(why)?;
        if got != e.size as u64 {
            return Err(format!("{}: {got} of {} bytes", e.name, e.size));
        }
        Ok(())
    }

    /// A `.res` member, read whole and its directory parsed. One whose directory `Res::parse`
    /// refuses only for the zero bytes padding it (Untamed's map archives) is read through
    /// [`ResFile::unpadded`], and its `padding` says so.
    pub fn res(&mut self, e: &Entry) -> Result<ResFile, String> {
        let bytes = self.read(e)?;
        ResFile::unpadded(bytes).map_err(|why| format!("{}: {why}", e.name))
    }
}

/// A `.res` archive in memory, with its directory parsed by openmx's reader.
pub struct ResFile {
    /// The file, or with `padding` its view without the directory's zero bytes.
    pub bytes: Vec<u8>,
    pub dir: Res,
    /// Zero bytes taken off the end of the directory for `Res::parse` ([`ResFile::unpadded`]); 0
    /// when it read the file as it is.
    pub padding: usize,
}

impl ResFile {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<ResFile, String> {
        let dir = Res::parse(&bytes)?;
        Ok(ResFile { bytes, dir, padding: 0 })
    }

    /// [`ResFile::from_bytes`], or when `Res::parse` refuses the file, a view of it with the zero
    /// bytes that end its directory taken out and the directory length cut to match. MX vs. ATV
    /// Untamed pads every map archive's directory so (56 to 2,003 bytes, which the header's length
    /// counts: `notes/mxvatv-later.md`, `notes/openmx-asks.md`). Only a cut the reader then takes
    /// whole, directory and every block, is used, so every member is still read by openmx's reader;
    /// the cut is tools/survey's `res_view`.
    pub fn unpadded(bytes: Vec<u8>) -> Result<ResFile, String> {
        let why = match Res::parse(&bytes) {
            Ok(dir) => return Ok(ResFile { bytes, dir, padding: 0 }),
            Err(why) => why,
        };
        let refused = || format!("Res::parse: {why}");
        let word = |at: usize| bytes.get(at..at + 4).map(|w| u32::from_le_bytes(w.try_into().expect("4 bytes")) as usize);
        let count_at = 0x14 + word(0x10).ok_or_else(refused)?;
        let dir_bytes = word(count_at + 4).ok_or_else(refused)?;
        let dir_at = count_at + 8;
        let dir = bytes.get(dir_at..dir_at + dir_bytes).ok_or_else(refused)?;
        let zeros = dir.iter().rev().take_while(|&&b| b == 0).count();
        // The last entry ends in its size, whose high bytes may be zero too: the longest cut first.
        for cut in (1..=zeros).rev().take(4) {
            let mut v = Vec::with_capacity(bytes.len() - cut);
            v.extend_from_slice(&bytes[..count_at + 4]);
            v.extend_from_slice(&((dir_bytes - cut) as u32).to_le_bytes());
            v.extend_from_slice(&bytes[dir_at..dir_at + dir_bytes - cut]);
            v.extend_from_slice(&bytes[dir_at + dir_bytes..]);
            if let Ok(dir) = Res::parse(&v) {
                return Ok(ResFile { bytes: v, dir, padding: cut });
            }
        }
        Err(refused())
    }

    pub fn open(path: &Path) -> Result<ResFile, String> {
        let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        ResFile::from_bytes(bytes).map_err(|e| format!("{}: {e}", path.display()))
    }

    pub fn entries(&self) -> &[Entry] {
        &self.dir.entries
    }

    pub fn matching(&self, glob: &str) -> Vec<&Entry> {
        self.dir
            .entries
            .iter()
            .filter(|e| glob_match(glob, &e.name))
            .collect()
    }

    /// One member's bytes, inflating only the blocks it spans.
    pub fn read(&self, e: &Entry) -> Result<Vec<u8>, String> {
        self.dir.read(&self.bytes, e)
    }
}

/// A pattern that may reach into `.res` members: `Env\*\*.res` matches PSR members, and
/// `Env\Nat01\*.res:Col\*.col` matches members of the `.res` files the part before the `:` matches.
/// PSR names never hold a `:`, so it cannot be mistaken for part of one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pattern {
    pub outer: String,
    pub inner: Option<String>,
}

impl Pattern {
    pub fn parse(s: &str) -> Pattern {
        match s.split_once(':') {
            Some((outer, inner)) => Pattern {
                outer: outer.to_string(),
                inner: Some(if inner.is_empty() { "**" } else { inner }.to_string()),
            },
            None => Pattern {
                outer: s.to_string(),
                inner: None,
            },
        }
    }
}

/// Where a `.res` member is written when extracting: under a folder named for its archive with `.d`
/// added (`Env/Nat01/nat01.res.d/Col/a.col`), so the archive itself can sit beside it and no PSR
/// path can collide with it.
pub fn res_member_path(res: &Entry, inner: &Entry) -> PathBuf {
    PathBuf::from(format!("{}.d", res.path())).join(inner.path())
}

/// `*` is any run within one path segment, `**` any run across them, `?` one character that is not a
/// separator. `\` and `/` are both separators — so `\` is never an escape — and case is folded.
pub fn glob_match(glob: &str, name: &str) -> bool {
    let g: Vec<u8> = normalise(glob).into_bytes();
    let n: Vec<u8> = normalise(name).into_bytes();
    // Dynamic programming over (glob position, name position): no backtracking blow-up on `*a*a*a`.
    let mut can = vec![vec![false; n.len() + 1]; g.len() + 1];
    can[g.len()][n.len()] = true;
    for i in (0..g.len()).rev() {
        let star2 = g[i] == b'*' && g.get(i + 1) == Some(&b'*');
        for j in (0..=n.len()).rev() {
            can[i][j] = if star2 {
                // `**` eats nothing, or one more character of anything.
                can[i + 2][j] || (j < n.len() && can[i][j + 1])
            } else if g[i] == b'*' {
                can[i + 1][j] || (j < n.len() && n[j] != b'/' && can[i][j + 1])
            } else if j == n.len() {
                false
            } else if g[i] == b'?' {
                n[j] != b'/' && can[i + 1][j + 1]
            } else {
                g[i] == n[j] && can[i + 1][j + 1]
            };
        }
    }
    can[0][0]
}

/// Names compare with either slash and in any case.
pub fn normalise(name: &str) -> String {
    name.replace('\\', "/").to_ascii_lowercase()
}

/// The boot ELF a SYSTEM.CNF names: `BOOT2 = cdrom0:\SLUS_211.04;1` gives `SLUS_211.04`.
pub fn boot_elf(cnf: &str) -> Option<String> {
    for line in cnf.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if !key.trim().eq_ignore_ascii_case("BOOT2") {
            continue;
        }
        let path = value.trim();
        let path = path.split(';').next().unwrap_or(path);
        let name = path.rsplit(['\\', '/', ':']).next().unwrap_or(path).trim();
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    None
}

/// The files in an ISO 9660 image's root directory. The primary volume descriptor is at sector 16 and
/// its root directory record at +156; directory records never straddle a sector, and a zero length
/// byte means the rest of the sector is padding.
pub fn iso_root(file: &mut (impl Read + Seek)) -> Result<Vec<IsoFile>, String> {
    let pvd = read_exact_at(file, 16 * SECTOR, SECTOR as usize)?;
    if pvd[0] != 1 || &pvd[1..6] != b"CD001" {
        return Err("no ISO 9660 primary volume descriptor at sector 16".to_string());
    }
    let extent = le32(&pvd[156 + 2..]) as u64;
    let size = le32(&pvd[156 + 10..]) as usize;
    let dir = read_exact_at(file, extent * SECTOR, size)?;
    let mut out = Vec::new();
    let mut at = 0usize;
    while at < dir.len() {
        let len = dir[at] as usize;
        if len == 0 {
            at = (at / SECTOR as usize + 1) * SECTOR as usize;
            continue;
        }
        if len < 34 || at + len > dir.len() {
            return Err(format!("a {len}-byte directory record at {at:#x} of the root"));
        }
        let rec = &dir[at..at + len];
        let name_len = rec[32] as usize;
        if 33 + name_len > len {
            return Err(format!("a {name_len}-byte name in a {len}-byte record"));
        }
        let flags = rec[25];
        let raw = &rec[33..33 + name_len];
        // `.` and `..` are spelled as one byte, 0 and 1; directories are not files to read.
        if flags & 2 == 0 && raw != [0] && raw != [1] {
            let name = String::from_utf8_lossy(raw);
            let name = name.split(';').next().unwrap_or("").to_string();
            out.push(IsoFile {
                name,
                at: le32(&rec[2..]) as u64 * SECTOR,
                size: le32(&rec[10..]),
            });
        }
        at += len;
    }
    Ok(out)
}

/// The images under `dir` (`iso/`), each with what it was identified as, sorted by path. An image
/// that is not ISO 9660 is skipped.
pub fn images(dir: &Path) -> Result<Vec<(PathBuf, Option<String>)>, String> {
    let mut out = Vec::new();
    let rd = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for item in rd {
        let path = item.map_err(|e| e.to_string())?.path();
        if !path.is_file() {
            continue;
        }
        let Ok(mut f) = File::open(&path) else {
            continue;
        };
        let Ok(root) = iso_root(&mut f) else {
            continue;
        };
        let boot = root
            .iter()
            .find(|r| r.name.eq_ignore_ascii_case("SYSTEM.CNF"))
            .and_then(|cnf| read_exact_at(&mut f, cnf.at, cnf.size as usize).ok())
            .and_then(|t| boot_elf(&String::from_utf8_lossy(&t)));
        out.push((path, boot));
    }
    out.sort();
    Ok(out)
}

/// What a user typed for the disc: a path, or a game's boot ELF or key looked up in `iso_dir`.
pub fn resolve_image(arg: &str, iso_dir: &Path) -> Result<PathBuf, String> {
    let path = Path::new(arg);
    if path.is_file() {
        return Ok(path.to_path_buf());
    }
    let Some(game) = Game::by_name(arg) else {
        return Err(format!(
            "{arg}: not a file, and not a game ({})",
            GAMES
                .iter()
                .map(|g| format!("{} or {}", g.key, g.boot))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    };
    images(iso_dir)?
        .into_iter()
        .find(|(_, boot)| boot.as_deref().is_some_and(|b| b.eq_ignore_ascii_case(game.boot)))
        .map(|(p, _)| p)
        .ok_or_else(|| format!("no disc with boot ELF {} in {}", game.boot, iso_dir.display()))
}

fn le32(b: &[u8]) -> u32 {
    u32::from_le_bytes([b[0], b[1], b[2], b[3]])
}

/// Exactly `len` bytes at `at`, or an error saying how many were there.
fn read_exact_at(file: &mut (impl Read + Seek), at: u64, len: usize) -> Result<Vec<u8>, String> {
    file.seek(SeekFrom::Start(at)).map_err(|e| e.to_string())?;
    let mut buf = Vec::with_capacity(len);
    let got = file
        .take(len as u64)
        .read_to_end(&mut buf)
        .map_err(|e| e.to_string())?;
    if got != len {
        return Err(format!("{got} of {len} bytes at {at:#x}"));
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_glob_keeps_to_its_segment_unless_doubled() {
        assert!(glob_match(r"Env\*\*.res", r"Env\Nat01\nat01.res"));
        assert!(glob_match("env/*/*.RES", r"Env\Nat01\nat01.res"));
        assert!(!glob_match(r"Env\*.res", r"Env\Nat01\nat01.res"));
        assert!(glob_match(r"Env\**.res", r"Env\Nat01\nat01.res"));
        assert!(glob_match(r"**\*.col", r"a\b\c\x.col"));
        assert!(glob_match("*", "a.res"));
        assert!(!glob_match("*", r"a\b"));
        assert!(glob_match("**", r"a\b"));
        assert!(glob_match("nat0?.res", "NAT07.res"));
        assert!(!glob_match("nat0?.res", "nat0/.res"));
        assert!(!glob_match("a", "ab"));
    }

    #[test]
    fn the_boot_elf_comes_out_of_system_cnf() {
        let cnf = "BOOT2 = cdrom0:\\SLUS_211.04;1\r\nVER = 1.02\r\nVMODE = NTSC\r\n";
        assert_eq!(boot_elf(cnf).as_deref(), Some("SLUS_211.04"));
        assert_eq!(boot_elf("VER = 1.00"), None);
        assert_eq!(Game::by_boot("slus_217.01").unwrap().key, "untamed");
    }

    #[test]
    fn a_pattern_splits_at_the_colon() {
        assert_eq!(Pattern::parse(r"Env\*.res").inner, None);
        let p = Pattern::parse(r"Env\*.res:Col\*.col");
        assert_eq!((p.outer.as_str(), p.inner.as_deref()), (r"Env\*.res", Some(r"Col\*.col")));
        assert_eq!(Pattern::parse("a.res:").inner.as_deref(), Some("**"));
    }
}
