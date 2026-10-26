//! MX vs. ATV Unleashed's and Untamed's tricks as trick packs (standard §6, `notes/tricks.md`).
//!
//! A trick pack is what openmx needs to give a rider the later games' tricks, as the disc has it:
//!
//! - **the stunt table**, the file the game's loader reads (SLUS_211.04 `0x2688e0`, SLUS_217.01
//!   `0x251e60`): `[Stunt%02d]` sections of `Name`, `DisplayName`, `HoldTime`, `SafeTime`, `Duration`,
//!   `LinkModTime`, `PointValue`, `HoldPointValue`, `UsedForBike`, `HasAnimation` and the default
//!   gesture (`DefaultKeyMapping`, Untamed's `DefaultKeyMappingClassic`), plus `[Depreciation]`.
//!   [`Table::parse`] reads it with the loader's count and defaults.
//! - **the stunt animations**: each rider set's stunt `.mcf` and `.MLB` (the two files the ELF names),
//!   every `.mot` the `.mcf` names, and the bike's clip of each stunt (`Veh\Anim\<P>bike_<name>.mot`,
//!   found by name as the bike's are on MX Unleashed).
//! - **the rotations' tuning**: the bikes' and quads' `.phy`, whose `[Backflip]` and `[Three Sixty]`
//!   the backflip and the 360 read (the 360 is new; `notes/tricks.md` has the step).
//!
//! Every file is the disc member's bytes. [`check`] reads a built pack back: every table with
//! [`Table::parse`] and with openmx's own `trick::Stunts::parse`, every clip with openmx's
//! `skin::Clip::parse`.

use game_sim::trick::Stunts;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub const AUTHOR: &str = "openmx-mods tools/import-tricks 0.1";

/// One rider set: the stunt animations the game loads for that rider (SLUS_211.04 names
/// `Rider\Anim\<P>_RiderStunts.mcf` and `.MLB` for `MX`, `MXF` and `ATV`) and the bike clips' prefix.
#[derive(Clone, Copy, Debug)]
pub struct Rider {
    /// `MX`, `MXF`, `ATV`: the `.mcf`'s motions are `<prefix>_rider_<stunt name>`.
    pub prefix: &'static str,
    /// The stunt table this rider's stunts come from.
    pub table: &'static str,
    pub mcf: &'static str,
    pub mlb: &'static str,
    /// The bike's stunt clips are `Veh\Anim\<bike>bike_<stunt name>.mot` (`Bike_ThreeSixtyL.mot`).
    pub bike: &'static str,
}

/// One game's pack.
#[derive(Clone, Copy, Debug)]
pub struct Pack {
    /// discx's game key.
    pub game: &'static str,
    pub id: &'static str,
    pub name: &'static str,
    /// The archive the loader opens the table out of; the tables below are its members, also loose
    /// in the PSR byte for byte (the disc test proves it).
    pub res: &'static str,
    /// The tables, as PSR members.
    pub tables: &'static [&'static str],
    /// How many `[Stunt%02d]` sections the loader reads.
    pub stunts: usize,
    /// The name buffers' size, terminator included.
    pub name_len: usize,
    /// The key the default gesture is read from.
    pub key_field: &'static str,
    pub riders: &'static [Rider],
    /// The `.phy` members (globs) whose `[Backflip]` and `[Three Sixty]` tune the rotations.
    pub phy: &'static [&'static str],
}

pub const PACKS: [Pack; 2] = [
    Pack {
        game: "mxvatv",
        id: "mxvatv_tricks",
        name: "MX vs. ATV Unleashed tricks",
        res: "StuntInfo.res",
        tables: &[
            "Rider\\Anim\\MX_Rider_Stunts.ini",
            "Rider\\Anim\\MXF_Rider_Stunts.ini",
            "Rider\\Anim\\ATV_Rider_Stunts.ini",
        ],
        stunts: 46,
        name_len: 0x20,
        key_field: "DefaultKeyMapping",
        riders: &[
            Rider {
                prefix: "MX",
                table: "Rider\\Anim\\MX_Rider_Stunts.ini",
                mcf: "Rider\\Anim\\MX_RiderStunts.mcf",
                mlb: "Rider\\Anim\\MX_RiderStunts.MLB",
                bike: "",
            },
            Rider {
                prefix: "MXF",
                table: "Rider\\Anim\\MXF_Rider_Stunts.ini",
                mcf: "Rider\\Anim\\MXF_RiderStunts.mcf",
                mlb: "Rider\\Anim\\MXF_RiderStunts.MLB",
                bike: "MXF_",
            },
            Rider {
                prefix: "ATV",
                table: "Rider\\Anim\\ATV_Rider_Stunts.ini",
                mcf: "Rider\\Anim\\ATV_RiderStunts.mcf",
                mlb: "Rider\\Anim\\ATV_RiderStunts.MLB",
                bike: "ATV_",
            },
        ],
        phy: &["Veh\\mx_*.phy", "Veh\\atv_*.phy"],
    },
    // SLUS_217.01 names one table, `MX_Rider_Stunts.ini`, opened out of `res\StuntInfo.res` (which
    // holds only its BPB); the MXF and ATV tables on the disc are never opened, so every rider set
    // takes its stunts from the MX table.
    Pack {
        game: "untamed",
        id: "untamed_tricks",
        name: "MX vs. ATV Untamed tricks",
        res: "res\\StuntInfo.res",
        tables: &["Rider\\Anim\\MX_Rider_Stunts.ini.bpb"],
        stunts: 54,
        name_len: 0x1b,
        key_field: "DefaultKeyMappingClassic",
        riders: &[
            Rider {
                prefix: "MX",
                table: "Rider\\Anim\\MX_Rider_Stunts.ini.bpb",
                mcf: "Rider\\Anim\\MX_RiderStunts.mcf",
                mlb: "Rider\\Anim\\MX_RiderStunts.MLB",
                bike: "",
            },
            Rider {
                prefix: "MXF",
                table: "Rider\\Anim\\MX_Rider_Stunts.ini.bpb",
                mcf: "Rider\\Anim\\MXF_RiderStunts.mcf",
                mlb: "Rider\\Anim\\MXF_RiderStunts.MLB",
                bike: "MXF_",
            },
            Rider {
                prefix: "ATV",
                table: "Rider\\Anim\\MX_Rider_Stunts.ini.bpb",
                mcf: "Rider\\Anim\\ATV_RiderStunts.mcf",
                mlb: "Rider\\Anim\\ATV_RiderStunts.MLB",
                bike: "ATV_",
            },
        ],
        phy: &["Veh\\mx_*.phy.bpb", "Veh\\atv_*.phy.bpb"],
    },
];

impl Pack {
    pub fn by_game(name: &str) -> Option<&'static Pack> {
        PACKS.iter().find(|p| p.game.eq_ignore_ascii_case(name) || p.id.eq_ignore_ascii_case(name))
    }
}

// ---------------------------------------------------------------------------------------------------
// the INI, text or BPB
// ---------------------------------------------------------------------------------------------------

/// An INI member as text: BPB (Untamed) decoded by tools/bpb, anything else as the disc writes it.
pub fn ini_text(name: &str, bytes: &[u8]) -> Result<String, String> {
    if name.to_ascii_lowercase().ends_with(".bpb") {
        let b = bpb::parse(bytes).map_err(|e| format!("{name}: {e}"))?;
        Ok(b.to_ini().iter().map(|&c| c as char).collect())
    } else {
        Ok(bytes.iter().map(|&c| c as char).collect())
    }
}

/// Sections in file order, each with its keys (lower-cased) and values (trimmed). `;` starts a
/// comment outside a quoted value.
pub fn sections(text: &str) -> Vec<(String, Vec<(String, String)>)> {
    let mut out: Vec<(String, Vec<(String, String)>)> = Vec::new();
    for raw in text.lines() {
        let line = strip_comment(raw).trim();
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            out.push((name.trim().to_owned(), Vec::new()));
            continue;
        }
        let Some((k, v)) = line.split_once('=') else { continue };
        if let Some((_, keys)) = out.last_mut() {
            keys.push((k.trim().to_ascii_lowercase(), v.trim().to_owned()));
        }
    }
    out
}

fn strip_comment(line: &str) -> &str {
    let mut quoted = false;
    for (i, c) in line.char_indices() {
        match c {
            '"' => quoted = !quoted,
            ';' if !quoted => return &line[..i],
            _ => {}
        }
    }
    line
}

/// A leading and a trailing quote, each dropped on its own (openmx's `trick::unquote`).
fn unquote(v: &str) -> &str {
    let v = v.strip_prefix('"').unwrap_or(v);
    v.strip_suffix('"').unwrap_or(v)
}

/// The loader's float: `strtod`, then the EE's truncating double-to-float (openmx's `phy::number`).
fn number(v: &str) -> Result<f32, String> {
    v.trim().parse::<f64>().map_err(|e| format!("{v:?} is not a number: {e}"))?;
    Ok(game_sim::phy::number(v.trim()).0)
}

fn switch(v: &str) -> Result<bool, String> {
    match unquote(v.trim()).to_ascii_uppercase().as_str() {
        "T" | "TRUE" | "Y" | "YES" | "1" | "ON" => Ok(true),
        "F" | "FALSE" | "N" | "NO" | "0" | "OFF" => Ok(false),
        other => Err(format!("{other:?} is not T or F")),
    }
}

// ---------------------------------------------------------------------------------------------------
// the stunt table
// ---------------------------------------------------------------------------------------------------

/// One stunt as the loader holds it.
#[derive(Clone, Debug, PartialEq)]
pub struct Stunt {
    /// The section, as the file titles it (`Stunt45`, BPB's `Stunt45` for `[Stunt%02d]` 45).
    pub section: String,
    /// The animation name (`threesixtyL`): the rider's clip is `<P>_rider_<name>`, the bike's
    /// `<P>bike_<name>`; `backflip`, `threesixtyL` and `threesixtyR` are found by it.
    pub name: String,
    pub display_name: String,
    pub hold_time: f32,
    pub safe_time: f32,
    pub duration: f32,
    /// Added to `HoldTime` when the stunt is linked from another (the file's own note).
    pub link_mod_time: f32,
    /// Points for landing it; the loader's default is 1000.
    pub point_value: f32,
    /// Points for holding it (the backflip, the two 360s, wheelie, stoppie); default 1000.
    pub hold_point_value: f32,
    /// Which of the two the file gives, rather than the loader's default.
    pub point_value_given: bool,
    pub hold_point_value_given: bool,
    pub used_for_bike: Option<bool>,
    pub has_animation: Option<bool>,
    /// The default gesture (`"1CTU"`): see [`key_bits`]. `None` when the stunt has none, which is
    /// every stunt past the gesture chart's 24 (they are only reached by remapping).
    pub key: Option<String>,
}

/// `[Depreciation]`: how fast a repeated stunt loses its points (the ATV table's only).
#[derive(Clone, Debug, PartialEq)]
pub struct Depreciation {
    pub pct: f32,
    pub restore_one: i32,
    pub max_depletion: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Table {
    pub stunts: Vec<Stunt>,
    pub depreciation: Option<Depreciation>,
    /// Sections the file has that the loader does not read.
    pub unread: Vec<String>,
}

impl Table {
    /// Read a table the way `pack`'s loader does: `pack.stunts` sections numbered from 1, each key
    /// with the loader's default when it is missing, names cut to the loader's buffer.
    pub fn parse(text: &str, pack: &Pack) -> Result<Table, String> {
        Table::parse_with(text, pack.stunts, pack.name_len, pack.key_field)
    }

    pub fn parse_with(text: &str, count: usize, name_len: usize, key_field: &str) -> Result<Table, String> {
        let secs = sections(text);
        let find = |i: usize| {
            let (a, b) = (format!("stunt{i:02}"), format!("stunt{i}"));
            secs.iter().find(|(t, _)| t.eq_ignore_ascii_case(&a) || t.eq_ignore_ascii_case(&b))
        };
        let key_field = key_field.to_ascii_lowercase();
        let mut stunts = Vec::with_capacity(count);
        let mut read = BTreeSet::new();
        for i in 1..=count {
            let (title, keys) = find(i).ok_or_else(|| format!("no [Stunt{i:02}]"))?;
            read.insert(title.to_ascii_lowercase());
            let get = |k: &str| keys.iter().find(|(key, _)| key == k).map(|(_, v)| v.as_str());
            let text = |k: &str| get(k).map(|v| cut(unquote(v), name_len)).unwrap_or_default();
            let clock = |k: &str| get(k).map(number).transpose().map(|v| v.unwrap_or(0.0));
            let points = |k: &str| get(k).map(number).transpose().map(|v| v.unwrap_or(1000.0));
            let flag = |k: &str| get(k).map(switch).transpose();
            let at = |e: String| format!("[{title}]: {e}");
            stunts.push(Stunt {
                section: title.clone(),
                name: text("name"),
                display_name: text("displayname"),
                hold_time: clock("holdtime").map_err(at)?,
                safe_time: clock("safetime").map_err(at)?,
                duration: clock("duration").map_err(at)?,
                link_mod_time: clock("linkmodtime").map_err(at)?,
                point_value: points("pointvalue").map_err(at)?,
                hold_point_value: points("holdpointvalue").map_err(at)?,
                point_value_given: get("pointvalue").is_some(),
                hold_point_value_given: get("holdpointvalue").is_some(),
                used_for_bike: flag("usedforbike").map_err(at)?,
                has_animation: flag("hasanimation").map_err(at)?,
                key: get(&key_field).map(|v| unquote(v).trim().to_owned()).filter(|v| !v.is_empty()),
            });
        }
        let depreciation = match secs.iter().find(|(t, _)| t.eq_ignore_ascii_case("depreciation")) {
            None => None,
            Some((_, keys)) => {
                read.insert("depreciation".into());
                let get = |k: &str| keys.iter().find(|(key, _)| key == k).map(|(_, v)| v.as_str());
                let int = |k: &str| get(k).map(|v| v.trim().parse::<i32>().map_err(|e| format!("[Depreciation] {k}: {e}"))).transpose();
                Some(Depreciation {
                    pct: get("depreciationpct").map(number).transpose()?.unwrap_or(game_sim::phy::number("0.8").0),
                    restore_one: int("numstuntstorestoreone")?.unwrap_or(0),
                    max_depletion: int("numstuntstomaxdepletion")?.unwrap_or(0),
                })
            }
        };
        let unread = secs.iter().map(|(t, _)| t.clone()).filter(|t| !read.contains(&t.to_ascii_lowercase())).collect();
        Ok(Table { stunts, depreciation, unread })
    }

    pub fn by_name(&self, name: &str) -> Option<&Stunt> {
        self.stunts.iter().find(|s| s.name.eq_ignore_ascii_case(name))
    }
}

fn cut(v: &str, len: usize) -> String {
    let mut end = len.saturating_sub(1).min(v.len());
    while !v.is_char_boundary(end) {
        end -= 1;
    }
    v[..end].to_owned()
}

// ---------------------------------------------------------------------------------------------------
// the gesture
// ---------------------------------------------------------------------------------------------------

/// The trick buttons' bits, as SLUS_211.04 `0x267da0` decodes a mapping and as the rider's live
/// button byte (`+0x14ec`) holds them: trick 1 (circle), trick 2 (triangle), trick 3 (R1) and a
/// fourth (R2, which no table uses) in the low nibble, the stick's direction in the high one.
pub const CIRCLE: u8 = 0x01;
pub const TRIANGLE: u8 = 0x02;
pub const R1: u8 = 0x04;
pub const R2: u8 = 0x08;
pub const LEFT: u8 = 0x10;
pub const RIGHT: u8 = 0x20;
pub const UP: u8 = 0x40;
pub const DOWN: u8 = 0x80;

/// A mapping string as the loader decodes it: `C T 1 2 L R U D` set a bit each; `3 4 S W B` and
/// anything else are skipped.
pub fn key_bits(code: &str) -> u8 {
    code.bytes().fold(0, |b, c| {
        b | match c {
            b'C' => CIRCLE,
            b'T' => TRIANGLE,
            b'1' => R1,
            b'2' => R2,
            b'L' => LEFT,
            b'R' => RIGHT,
            b'U' => UP,
            b'D' => DOWN,
            _ => 0,
        }
    })
}

/// `R1+Circle+Triangle, up` for a bit set.
pub fn describe(bits: u8) -> String {
    let buttons: Vec<&str> = [(R1, "R1"), (R2, "R2"), (CIRCLE, "Circle"), (TRIANGLE, "Triangle")]
        .iter()
        .filter(|(b, _)| bits & b != 0)
        .map(|(_, n)| *n)
        .collect();
    let dirs: Vec<&str> = [(UP, "up"), (DOWN, "down"), (LEFT, "left"), (RIGHT, "right")]
        .iter()
        .filter(|(b, _)| bits & b != 0)
        .map(|(_, n)| *n)
        .collect();
    match (buttons.is_empty(), dirs.is_empty()) {
        (true, true) => "none".into(),
        (false, true) => buttons.join("+"),
        (true, false) => dirs.join("+"),
        (false, false) => format!("{}, {}", buttons.join("+"), dirs.join("+")),
    }
}

// ---------------------------------------------------------------------------------------------------
// the animations
// ---------------------------------------------------------------------------------------------------

/// A `.mcf`'s `ContentDirectory` and its motions' `NAME` and `VUE`, in file order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mcf {
    pub dir: String,
    pub motions: Vec<(String, String)>,
}

impl Mcf {
    pub fn parse(text: &str) -> Mcf {
        let mut m = Mcf::default();
        for (title, keys) in sections(text) {
            let get = |k: &str| keys.iter().find(|(key, _)| key == k).map(|(_, v)| unquote(v).trim().to_owned());
            if title.eq_ignore_ascii_case("general info") {
                m.dir = get("contentdirectory").unwrap_or_default();
            } else if title.to_ascii_lowercase().starts_with("motion")
                && let (Some(n), Some(v)) = (get("name"), get("vue"))
            {
                m.motions.push((n, v));
            }
        }
        m
    }

    /// The member a motion's `VUE` names: the content directory plus the file, its written extension
    /// replaced by `.mot` (openmx's `anim::Set::member`, rule 1).
    pub fn member(&self, name: &str) -> Option<String> {
        let (_, vue) = self.motions.iter().find(|(n, _)| n.eq_ignore_ascii_case(name))?;
        let stem = vue.rsplit_once('.').map(|(s, _)| s).unwrap_or(vue);
        Some(format!("{}{stem}.mot", self.dir))
    }
}

/// One clip in a pack.
#[derive(Clone, Debug, PartialEq)]
pub struct Clip {
    /// The PSR member, as the PSR spells it.
    pub member: String,
    /// Seconds and joint channels, as openmx's `skin::Clip::parse` reads them; the error otherwise.
    pub read: Result<(f32, usize), String>,
}

/// A `.mot`'s version: its first word negated when negative (openmx's `skin::Clip::parse`), else 1.
pub fn mot_version(bytes: &[u8]) -> Option<i32> {
    let w = i32::from_le_bytes(bytes.get(..4)?.try_into().ok()?);
    Some(if w < 0 { -w } else { 1 })
}

/// Read a `.mot` with openmx's reader: its length and channel count.
pub fn read_clip(bytes: &[u8]) -> Result<(f32, usize), String> {
    let c = game_render::skin::Clip::parse(bytes)?;
    Ok((c.duration(), c.rotation.len() + c.position.len()))
}

// ---------------------------------------------------------------------------------------------------
// MX Unleashed's table, through openmx
// ---------------------------------------------------------------------------------------------------

/// MX Unleashed's `rider\anim\stunts.ini` (in `appstart.res`) through openmx's `Stunts::parse`.
pub const MXU_STUNTS: (&str, &str) = ("appstart.res", "rider\\anim\\stunts.ini");

/// The 57 names of MX Unleashed's table, read by openmx's loader.
pub fn mxu_names(stunts_ini: &[u8]) -> Result<Vec<String>, String> {
    Ok(Stunts::parse(stunts_ini)?.stunts.into_iter().map(|s| s.name).collect())
}

/// MX Unleashed's names off its disc in `iso` (by boot ELF), if it is there.
pub fn mxu_names_from(iso: &Path) -> Result<Vec<String>, String> {
    let image = discx::resolve_image("mxu", iso)?;
    let mut disc = discx::Disc::open(&image)?;
    let e = disc.find(MXU_STUNTS.0).cloned().ok_or("appstart.res is not on the MX Unleashed disc")?;
    let res = disc.res(&e)?;
    let inner = res.dir.find(MXU_STUNTS.1).cloned().ok_or("rider\\anim\\stunts.ini is not in appstart.res")?;
    mxu_names(&res.read(&inner)?)
}

/// What openmx's own loader makes of a later table: it reads 57 sections, so it refuses one with
/// fewer. `Ok(n)` is the stunts it read.
pub fn openmx_reads(text: &str) -> Result<usize, String> {
    Stunts::parse(text.as_bytes()).map(|s| s.stunts.len())
}

// ---------------------------------------------------------------------------------------------------
// the pack
// ---------------------------------------------------------------------------------------------------

/// How a trick MX Unleashed lacks moves the bike, which is what openmx needs for it.
pub fn moves(name: &str) -> &'static str {
    match name.to_ascii_lowercase().as_str() {
        "threesixtyl" | "threesixtyr" => "rotation: the 360's yaw and pitch torque, [Three Sixty] of the .phy",
        "wheelie" | "stoppie" => "ground: scored while held (HoldPointValue), no clip",
        _ => "pose: the rider's and the bike's clips, no torque",
    }
}

/// A table in a pack, read.
#[derive(Clone, Debug)]
pub struct Read {
    pub member: String,
    pub table: Table,
    /// `openmx_reads` on it.
    pub openmx: Result<usize, String>,
}

#[derive(Clone, Debug)]
pub struct Built {
    pub dir: PathBuf,
    pub tables: Vec<Read>,
    pub clips: Vec<Clip>,
    /// The stunts MX Unleashed's table has no row of, by name, across the pack's tables, in table order.
    pub new: Vec<String>,
    pub members: Vec<String>,
    pub todo: Vec<String>,
}

/// What the disc is, for `mod.json`'s `source`.
#[derive(Clone, Debug)]
pub struct Origin {
    pub game: String,
    pub boot: String,
    pub psr: String,
}

/// Build `root/<id>/` from the disc's members, which `read` gives by PSR name; `members` is the
/// PSR's directory. `mxu` is MX Unleashed's 57 names, without which the pack cannot say what is new
/// (it says so in `todo`). A folder already there is replaced, but only if it is this pack's.
pub fn build(
    pack: &Pack,
    origin: &Origin,
    members: &[String],
    read: &mut dyn FnMut(&str) -> Result<Vec<u8>, String>,
    mxu: Option<&[String]>,
    root: &Path,
) -> Result<Built, String> {
    let name_of = |want: &str| members.iter().find(|m| m.eq_ignore_ascii_case(want)).cloned();
    let dir = root.join(pack.id);
    clear(&dir, pack.id)?;
    let mut copied: Vec<String> = Vec::new();
    let mut put = |member: &str, bytes: &[u8]| -> Result<(), String> {
        let path = dir.join(member.replace('\\', "/"));
        std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| format!("{}: {e}", path.display()))?;
        std::fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        copied.push(member.to_owned());
        Ok(())
    };
    let mut todo = Vec::new();

    // The tables.
    let mut tables = Vec::new();
    for &t in pack.tables {
        let member = name_of(t).ok_or_else(|| format!("{t} is not in {}", origin.psr))?;
        let bytes = read(&member)?;
        put(&member, &bytes)?;
        let text = ini_text(&member, &bytes)?;
        let table = Table::parse(&text, pack).map_err(|e| format!("{member}: {e}"))?;
        let openmx = openmx_reads(&text);
        if let Err(e) = &openmx {
            todo.push(format!(
                "{}: openmx's trick::Stunts::parse reads MX Unleashed's 57 sections and refuses this table of {} ({e}); notes/openmx-asks.md",
                file(&member),
                table.stunts.len()
            ));
        }
        tables.push(Read { member, table, openmx });
    }

    // What MX Unleashed lacks, by name.
    let mut new: Vec<String> = Vec::new();
    match mxu {
        Some(names) => {
            for r in &tables {
                for s in &r.table.stunts {
                    if !names.iter().any(|n| n.eq_ignore_ascii_case(&s.name)) && !new.iter().any(|n| n.eq_ignore_ascii_case(&s.name)) {
                        new.push(s.name.clone());
                    }
                }
            }
        }
        None => todo.push("new: MX Unleashed's disc was not found, so the stunts it lacks are not listed".into()),
    }
    for n in &new {
        todo.push(format!("{n}: MX Unleashed has no such stunt and openmx none of its own ({}); notes/tricks.md", moves(n)));
    }

    // The animations: each rider set's .mcf and .MLB, every clip the .mcf names, the bike's clips.
    let mut clips = Vec::new();
    let mut seen = BTreeSet::new();
    let mut riders_json = Vec::new();
    for r in pack.riders {
        let mcf_member = name_of(r.mcf).ok_or_else(|| format!("{} is not in {}", r.mcf, origin.psr))?;
        let mcf_bytes = read(&mcf_member)?;
        put(&mcf_member, &mcf_bytes)?;
        let mcf = Mcf::parse(&ini_text(&mcf_member, &mcf_bytes)?);
        let mlb = name_of(r.mlb);
        match &mlb {
            Some(m) => {
                let b = read(m)?;
                put(m, &b)?;
            }
            None => todo.push(format!("{}: {} names it, and it is not on the disc", file(r.mlb), origin.boot)),
        }
        let mut wanted: Vec<String> = mcf.motions.iter().filter_map(|(n, _)| mcf.member(n)).collect();
        let table = tables.iter().find(|t| t.member.eq_ignore_ascii_case(r.table)).map(|t| &t.table);
        let mut missing = Vec::new();
        if let Some(t) = table {
            for s in &t.stunts {
                if s.has_animation != Some(false) && mcf.member(&format!("{}_rider_{}", r.prefix, s.name)).is_none() {
                    missing.push(s.name.clone());
                }
                let bike = format!("Veh\\Anim\\{}bike_{}.mot", r.bike, s.name);
                if let Some(m) = name_of(&bike) {
                    wanted.push(m);
                }
            }
        }
        if !missing.is_empty() {
            todo.push(format!("{}: no {}_rider_<name> motion for {}", file(&mcf_member), r.prefix, missing.join(", ")));
        }
        for w in wanted {
            let Some(m) = name_of(&w) else {
                todo.push(format!("{}: names {}, which is not on the disc", file(&mcf_member), file(&w)));
                continue;
            };
            if !seen.insert(m.to_ascii_lowercase()) {
                continue;
            }
            let bytes = read(&m)?;
            put(&m, &bytes)?;
            let got = read_clip(&bytes);
            if let Err(e) = &got {
                todo.push(format!(
                    "{}: .mot version {} (MX Unleashed's are 3); openmx's skin::Clip::parse refuses it: {e}",
                    file(&m),
                    mot_version(&bytes).map(|v| v.to_string()).unwrap_or_else(|| "?".into())
                ));
            }
            clips.push(Clip { member: m, read: got });
        }
        riders_json.push(format!(
            "{{\"rider\": {}, \"table\": {}, \"mcf\": {}, \"mlb\": {}, \"bike_clips\": {}}}",
            json_str(r.prefix),
            json_str(&file(r.table)),
            json_str(&file(&mcf_member)),
            mlb.as_deref().map(|m| json_str(&file(m))).unwrap_or_else(|| "null".into()),
            json_str(&format!("Veh/Anim/{}bike_<name>.mot", r.bike))
        ));
    }

    // The rotations' tuning.
    let mut phys = Vec::new();
    for glob in pack.phy {
        for m in members.iter().filter(|m| discx::glob_match(glob, m)) {
            let b = read(m)?;
            put(m, &b)?;
            phys.push(m.clone());
        }
    }
    if phys.is_empty() {
        todo.push("no .phy: the rotations' tuning is not in the pack".into());
    }

    todo.push(format!(
        "{}: not yet ridden against the game in PCSX2 (T7): notes/tricks.md has what was captured",
        pack.id
    ));

    let source_members: Vec<String> = copied.clone();
    let json = mod_json(pack, origin, &source_members, &tables, &riders_json, &new, &phys, &todo);
    std::fs::write(dir.join("mod.json"), json).map_err(|e| format!("{}: {e}", dir.display()))?;
    Ok(Built { dir, tables, clips, new, members: source_members, todo })
}

/// A member's path in the pack.
pub fn file(member: &str) -> String {
    member.replace('\\', "/")
}

/// Make way for a pack: a folder that is not there, or one whose `mod.json` is this pack's, removed.
fn clear(dir: &Path, id: &str) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }
    let ours = std::fs::read_to_string(dir.join("mod.json"))
        .is_ok_and(|t| t.contains(&format!("\"id\": {}", json_str(id))) && t.contains("\"kind\": \"tricks\""));
    if !ours {
        return Err(format!("{}: there already, and not this tool's {id} pack; not touched", dir.display()));
    }
    std::fs::remove_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))
}

/// A JSON string.
pub fn json_str(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// A float as JSON: the shortest text that reads back to the same f32.
fn num(v: f32) -> String {
    format!("{v}")
}

#[allow(clippy::too_many_arguments)]
fn mod_json(
    pack: &Pack,
    origin: &Origin,
    members: &[String],
    tables: &[Read],
    riders: &[String],
    new: &[String],
    phys: &[String],
    todo: &[String],
) -> String {
    let list = |items: Vec<String>, indent: &str| {
        if items.is_empty() {
            "[]".to_string()
        } else {
            format!("[\n{indent}  {}\n{indent}]", items.join(&format!(",\n{indent}  ")))
        }
    };
    let tables_json: Vec<String> = tables
        .iter()
        .map(|r| {
            format!(
                "{{\"file\": {}, \"from\": {}, \"stunts\": {}, \"key\": {}}}",
                json_str(&file(&r.member)),
                json_str(pack.res),
                r.table.stunts.len(),
                json_str(pack.key_field)
            )
        })
        .collect();
    let new_json: Vec<String> = new
        .iter()
        .map(|n| {
            let (t, s) = tables
                .iter()
                .find_map(|t| t.table.by_name(n).map(|s| (t, s)))
                .expect("a new name comes from a table");
            format!(
                "{{\"name\": {}, \"display_name\": {}, \"table\": {}, \"section\": {}, \"hold_time\": {}, \"safe_time\": {}, \"duration\": {}, \"point_value\": {}, \"hold_point_value\": {}, \"gesture\": {}, \"moves\": {}}}",
                json_str(&s.name),
                json_str(&s.display_name),
                json_str(&file(&t.member)),
                json_str(&s.section),
                num(s.hold_time),
                num(s.safe_time),
                num(s.duration),
                num(s.point_value),
                num(s.hold_point_value),
                s.key.as_deref().map(json_str).unwrap_or_else(|| "null".into()),
                json_str(moves(&s.name))
            )
        })
        .collect();
    let rotations = [
        format!(
            "{{\"name\": \"backflip\", \"stunts\": [\"backflip\"], \"phy\": \"[Backflip]\", \"buttons\": {}, \"stick\": \"y past -0.8 on the lip, then past +0.8\"}}",
            json_str(&describe(R1))
        ),
        format!(
            "{{\"name\": \"threesixty\", \"stunts\": [\"threesixtyL\", \"threesixtyR\"], \"phy\": \"[Three Sixty]\", \"buttons\": {}, \"stick\": \"x past 0.7 one way on the lip, then past 0.7 the other\"}}",
            json_str(&describe(R1))
        ),
    ];
    format!(
        "{{\n  \"standard\": 1,\n  \"kind\": \"tricks\",\n  \"id\": {},\n  \"name\": {},\n  \"source\": {{\n    \"game\": {},\n    \"boot\": {},\n    \"psr\": {},\n    \"members\": {}\n  }},\n  \"version\": \"1.0.0\",\n  \"author\": {},\n  \"permission\": \"own-disc\",\n  \"tables\": {},\n  \"riders\": {},\n  \"new\": {},\n  \"rotations\": {},\n  \"phy\": {},\n  \"todo\": {}\n}}\n",
        json_str(pack.id),
        json_str(pack.name),
        json_str(&origin.game),
        json_str(&origin.boot),
        json_str(&origin.psr),
        list(members.iter().map(|m| json_str(m)).collect(), "    "),
        json_str(AUTHOR),
        list(tables_json, "  "),
        list(riders.to_vec(), "  "),
        list(new_json, "  "),
        list(rotations.to_vec(), "  "),
        list(phys.iter().map(|m| json_str(&file(m))).collect(), "  "),
        list(todo.iter().map(|t| json_str(t)).collect(), "  "),
    )
}

/// Build a pack off a disc image, or off MX vs. ATV Unleashed's PC paks when `image` is those
/// (`discx::Disc::open_any`; `source.psr` then says `data\*.pak`); MX Unleashed's names are read off
/// its disc in `iso` if it is there.
pub fn import(image: &Path, pack: &Pack, iso: &Path, root: &Path) -> Result<Built, String> {
    let mut disc = discx::Disc::open_any(image)?;
    let game = disc.game.ok_or_else(|| format!("{}: not a disc this repo knows", image.display()))?;
    if game.key != pack.game {
        return Err(format!("{}: is {} ({}), not {}", image.display(), game.title, game.boot, pack.name));
    }
    let origin = Origin { game: game.title.to_string(), boot: game.boot.to_string(), psr: disc.psr_name.clone() };
    let members: Vec<String> = disc.entries().iter().map(|e| e.name.clone()).collect();
    let mxu = mxu_names_from(iso).ok();
    let mut read = |name: &str| {
        let e = disc.find(name).cloned().ok_or_else(|| format!("{name}: not on the disc"))?;
        disc.read(&e)
    };
    build(pack, &origin, &members, &mut read, mxu.as_deref(), root)
}

/// A built pack read back from its folder: every table its `mod.json` lists with [`Table::parse`]
/// (and openmx's loader), every `.mot` in it with openmx's clip reader.
pub fn check(dir: &Path) -> Result<(Vec<Read>, Vec<Clip>), String> {
    let id = dir.file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_default();
    let pack = Pack::by_game(&id).ok_or_else(|| format!("{id}: not a pack this tool builds"))?;
    check_with(dir, pack)
}

/// [`check`] for a pack given rather than found by the folder's name.
pub fn check_with(dir: &Path, pack: &Pack) -> Result<(Vec<Read>, Vec<Clip>), String> {
    let text = std::fs::read_to_string(dir.join("mod.json")).map_err(|e| format!("{}: {e}", dir.display()))?;
    if !text.contains("\"kind\": \"tricks\"") {
        return Err(format!("{}: mod.json is not a trick pack's", dir.display()));
    }
    let mut tables = Vec::new();
    for &t in pack.tables {
        let path = dir.join(file(t));
        let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let text = ini_text(t, &bytes)?;
        let table = Table::parse(&text, pack).map_err(|e| format!("{t}: {e}"))?;
        tables.push(Read { member: t.to_owned(), openmx: openmx_reads(&text), table });
    }
    let mut clips = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).map_err(|e| format!("{}: {e}", d.display()))? {
            let p = e.map_err(|e| e.to_string())?.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("mot")) {
                let bytes = std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?;
                let rel = p.strip_prefix(dir).unwrap().to_string_lossy().replace('/', "\\");
                clips.push(Clip { member: rel, read: read_clip(&bytes) });
            }
        }
    }
    clips.sort_by(|a, b| a.member.cmp(&b.member));
    Ok((tables, clips))
}
