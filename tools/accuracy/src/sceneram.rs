//! What the original made of a map's scene (M12): the `[SceneInfo]`, `[Environment]` and `[Fog]` the
//! game holds in RAM in a ride, against the mod's `.scn` text, field by field and bit for bit.
//!
//! MX vs. ATV Unleashed reads each section with its own small reader object, called through a
//! one-slot vtable (SLUS_211.04's table at `0x3c5bb0..0x3c5c20`). Metrowerks lays the vtable word after
//! the members, so each block is found in the image by its vtable and read backwards from it:
//!
//! | section | reader | vtable | members (from the vtable word) |
//! |---|---|---|---|
//! | `[SceneInfo]` | `0x31d4f0` | `0x3c5c10` | -0x24 `DefaultStartPosition` (w = 1), -0x14 `DefaultStartDirection`, -0x4 `DefaultStartZone` - 1 |
//! | `[Fog]` | `0x31f0c0` | `0x3c5bd0` | -0x20 `ColorRGB` as four floats (A 255), -0x10 `Visibility`, -0xc `Haziness`, -0x8 `MinimumFogDistance`, -0x4 `MaximumFogDistance` |
//! | `[Environment]` | `0x31e0c0` | `0x3c5be0` | words of the reader's object, the vtable at word 0x169 ([`ENV`]) |
//!
//! The `[Light<n>]`s (`0x31e7d0`) become light objects, vtable `0x3b3cd0` at +4: +0x30 the colour over
//! 255 (A 1), +0x40 the position (w 1), +0x50 the direction, `TargetPosition - Position` made unit, and
//! +0x70 the type (`Ambient` 4, `Directional` 2, `ObjectPoint` 1, `Spotlight` 3). Each scene light is
//! matched to the first light object of its type.
//!
//! A key the scene leaves out takes the reader's default (the constants in its calls, given here). The
//! values are compared as the game's bits, the text read the way openmx reads a scene's numbers
//! (`strtod`, then the EE's truncating double-to-float): read as Rust reads it, rounding to nearest,
//! Lago's start and haziness come out one ulp above the game's. A field one ulp off is still told
//! apart from a real difference.

use crate::ram::{f32_at, u32_at};
use std::collections::BTreeMap;

pub const SCENE_INFO_VT: u32 = 0x3c5c10;
pub const FOG_VT: u32 = 0x3c5bd0;
pub const ENV_VT: u32 = 0x3c5be0;
pub const LIGHT_VT: u32 = 0x3b3cd0;
/// The Environment reader's vtable word, in words from the start of its object.
pub const ENV_VT_WORD: u32 = 0x169;

/// The `[Environment]` keys read as floats: (key, word in the object, the reader's default).
pub const ENV: [(&str, u32, f32); 18] = [
    ("EcosystemWindStrength", 0x148, 0.0),
    ("EcosystemWindGustRate", 0x149, 0.15),
    ("EcoMaxPhysicsEco", 0x14b, 50.0),
    ("Eco2dFadeDistance", 0x14c, 0.0),
    ("Eco2dClipDistance", 0x14d, 0.0),
    ("TerrainWidth", 0x14e, 0.0),
    ("TerrainBreadth", 0x14f, 0.0),
    ("TerrainWidthScale", 0x150, 0.0),
    ("TerrainBreadthScale", 0x151, 0.0),
    ("TerrainBorderWidth", 0x152, 0.0),
    ("Xmin", 0x154, 0.0),
    ("Xmax", 0x155, 0.0),
    ("Zmin", 0x156, 0.0),
    ("Zmax", 0x157, 0.0),
    ("MinDivisionSize", 0x158, 96.0),
    ("EcoDistance2D", 0x159, 180.0),
    ("EcoDistance3D", 0x15a, 120.0),
    ("ShrubberyDistLimit", 0x15b, 400.0),
];
/// Read only when another key is there (`EcosystemWindStrength`, `UseEcoPhysics`): left out of the
/// comparison when the scene does not give them.
pub const ENV_CONDITIONAL: [&str; 2] = ["EcosystemWindGustRate", "EcoMaxPhysicsEco"];
/// Read only when `EcosystemFile` names one (the reader's default is `none`): the words are left as
/// they were, so they are not compared on a scene without an ecosystem.
pub const ENV_ECO: [&str; 5] = ["EcosystemWindStrength", "EcosystemWindGustRate", "EcoMaxPhysicsEco", "Eco2dFadeDistance", "Eco2dClipDistance"];

/// `AmountOfTransmittedLight` (word 0x15c, default 0), `DetailMapTilingMultiplier` (an int, word 0x153,
/// default 8) and the two vec4 scales (words 0x160 and 0x164, default the vector at `0x396fd0`).
pub const ENV_TRANSMITTED: (&str, u32) = ("AmountOfTransmittedLight", 0x15c);
pub const ENV_DETAIL: (&str, u32, i32) = ("DetailMapTilingMultiplier", 0x153, 8);
pub const ENV_SCALES: [(&str, u32); 2] = [("EcoAmbientScale", 0x160), ("EcoDiffuseScale", 0x164)];

/// The `[Fog]` reader's defaults: visibility, haziness, minimum and maximum distance. A missing
/// `ColorRGB` is 255, 255, 255.
pub const FOG_DEFAULTS: [f32; 4] = [1.0, 0.5, 64.0, 768.0];

/// A scene's text as sections of keys, both in lower case, the last of a repeated key winning.
pub fn sections(text: &str) -> BTreeMap<String, BTreeMap<String, String>> {
    let mut out: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut cur = String::new();
    for line in text.lines() {
        let line = line.split(';').next().unwrap_or("").trim();
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            cur = name.trim().to_ascii_lowercase();
            out.entry(cur.clone()).or_default();
        } else if let Some((k, v)) = line.split_once('=') {
            out.entry(cur.clone()).or_default().insert(k.trim().to_ascii_lowercase(), v.trim().trim_matches('"').to_owned());
        }
    }
    out
}

/// The numbers of a value as the game reads them, and as openmx does: `strtod`, then the EE's
/// truncating double-to-float (`game_sim::phy::number`).
fn floats(v: &str) -> Vec<f32> {
    v.split(',')
        .map(|w| w.trim().trim_end_matches('f'))
        .filter(|w| w.parse::<f32>().is_ok())
        .map(|w| game_sim::phy::number(w).0)
        .collect()
}

/// One field: the scene's value (or the default, said so), the game's, and how they compare.
#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub what: String,
    /// The value the scene gives, or the reader's default when it gives none.
    pub scene: Vec<f32>,
    pub defaulted: bool,
    pub game: Vec<f32>,
    /// Worked out here rather than read from the text (the sun's direction): a few ulps are the
    /// game's arithmetic, not the data, and are called close.
    pub computed: bool,
}

impl Field {
    /// The largest distance in ulps between the two, lane by lane (`u32::MAX` if they differ in length
    /// or sign).
    pub fn ulps(&self) -> u32 {
        if self.scene.len() != self.game.len() {
            return u32::MAX;
        }
        self.scene
            .iter()
            .zip(&self.game)
            .map(|(a, b)| {
                if a.to_bits() == b.to_bits() || (*a == 0.0 && *b == 0.0) {
                    0
                } else if a.is_sign_negative() != b.is_sign_negative() {
                    u32::MAX
                } else {
                    a.to_bits().abs_diff(b.to_bits())
                }
            })
            .max()
            .unwrap_or(0)
    }
    pub fn verdict(&self) -> &'static str {
        match self.ulps() {
            0 => "equal",
            1 => "1 ulp",
            2..=8 if self.computed => "close",
            _ => "DIFFERS",
        }
    }
    /// Whether the field counts against the map: more than an ulp off, or more than 8 on a computed one.
    pub fn differs(&self) -> bool {
        self.verdict() == "DIFFERS"
    }
}

/// Every word-aligned address holding `vt`.
pub fn with_vtable(ram: &[u8], vt: u32) -> Vec<u32> {
    let mut out = Vec::new();
    for k in 0..ram.len() / 4 {
        if u32_at(ram, (k * 4) as u32) == Some(vt) {
            out.push((k * 4) as u32);
        }
    }
    out
}

fn read(ram: &[u8], at: u32, n: u32) -> Vec<f32> {
    (0..n).map(|i| f32_at(ram, at + 4 * i).unwrap_or(f32::NAN)).collect()
}

/// Where one game's scene readers leave what they read: [`MXVATV`] (the constants above) and
/// [`UNTAMED`] (M13).
#[derive(Clone, Copy, Debug)]
pub struct Readers {
    pub scene_info_vt: u32,
    pub fog_vt: u32,
    pub env_vt: u32,
    pub light_vt: u32,
    pub env_vt_word: u32,
    pub env: &'static [(&'static str, u32, f32)],
    pub transmitted: (&'static str, u32),
    pub detail: (&'static str, u32, i32),
    pub scales: [(&'static str, u32); 2],
    /// `[Fog]`: `ColorRGB`, then `Visibility`, `Haziness`, `MinimumFogDistance`, `MaximumFogDistance`,
    /// as offsets back from the vtable word.
    pub fog_rgb: u32,
    pub fog: [u32; 4],
    /// Untamed's `FogStart` and `FogEnd`, whose defaults are the minimum and maximum distance.
    pub fog_start_end: Option<(u32, u32)>,
    /// Untamed's `[Environment]` ints past the detail multiplier: (key, word, default, minus one when
    /// above zero) (`EcosystemZone`).
    pub env_ints: &'static [(&'static str, u32, i32, bool)],
    /// The `[SceneInfo]` reader makes `DefaultStartDirection` unit (Untamed's, with VU0's `vrsqrt`):
    /// the text's vector is made unit here and compared as a computed field.
    pub unit_direction: bool,
}

const UNIT_DIRECTION: &str = "SceneInfo DefaultStartDirection, made unit";

/// MX vs. ATV Unleashed's (M12).
pub const MXVATV: Readers = Readers {
    scene_info_vt: SCENE_INFO_VT,
    fog_vt: FOG_VT,
    env_vt: ENV_VT,
    light_vt: LIGHT_VT,
    env_vt_word: ENV_VT_WORD,
    env: &ENV,
    transmitted: ENV_TRANSMITTED,
    detail: ENV_DETAIL,
    scales: ENV_SCALES,
    fog_rgb: 0x20,
    fog: [0x10, 0xc, 0x8, 0x4],
    fog_start_end: None,
    env_ints: &[],
    unit_direction: false,
};

/// MX vs. ATV Untamed's (SLUS_217.01, M13). The same readers, grown: `[SceneInfo]` `0x30e490` (vtable
/// `0x3e7800`, the members as Unleashed's; the direction is made unit as it is read), `[Fog]` `0x310180`
/// (`0x3e77c0`) adds `FogStart` and `FogEnd` between the haziness and the distances, `[Environment]`
/// `0x30f0b0` (`0x3e77d0` at word 0x169) adds `DetailMapLODBias` (word 0x153), which moves the
/// detail multiplier to 0x154 and the box and the eco distances one word on, and `EcosystemZone` (0x168,
/// stored less one when above zero). Lights: vtable `0x3d647c`, Unleashed's members.
pub const UNTAMED: Readers = Readers {
    scene_info_vt: 0x3e7800,
    fog_vt: 0x3e77c0,
    env_vt: 0x3e77d0,
    light_vt: 0x3d647c,
    env_vt_word: 0x169,
    env: &UNTAMED_ENV,
    transmitted: ("AmountOfTransmittedLight", 0x15d),
    detail: ("DetailMapTilingMultiplier", 0x154, 8),
    scales: ENV_SCALES,
    fog_rgb: 0x28,
    fog: [0x18, 0x14, 0x8, 0x4],
    fog_start_end: Some((0x10, 0xc)),
    env_ints: &[("EcosystemZone", 0x168, 1, true)],
    unit_direction: true,
};

/// Untamed's `[Environment]` floats (`0x30f0b0`).
pub const UNTAMED_ENV: [(&str, u32, f32); 19] = [
    ("EcosystemWindStrength", 0x148, 0.0),
    ("EcosystemWindGustRate", 0x149, 0.15),
    ("EcoMaxPhysicsEco", 0x14b, 50.0),
    ("Eco2dFadeDistance", 0x14c, 0.0),
    ("Eco2dClipDistance", 0x14d, 0.0),
    ("TerrainWidth", 0x14e, 0.0),
    ("TerrainBreadth", 0x14f, 0.0),
    ("TerrainWidthScale", 0x150, 0.0),
    ("TerrainBreadthScale", 0x151, 0.0),
    ("TerrainBorderWidth", 0x152, 0.0),
    ("DetailMapLODBias", 0x153, 0.0),
    ("Xmin", 0x155, 0.0),
    ("Xmax", 0x156, 0.0),
    ("Zmin", 0x157, 0.0),
    ("Zmax", 0x158, 0.0),
    ("MinDivisionSize", 0x159, 96.0),
    ("EcoDistance2D", 0x15a, 180.0),
    ("EcoDistance3D", 0x15b, 120.0),
    ("ShrubberyDistLimit", 0x15c, 400.0),
];

/// The readers of the game with this boot ELF.
pub fn readers(boot: &str) -> Option<Readers> {
    if boot.eq_ignore_ascii_case(crate::source::MXVATV.boot) {
        Some(MXVATV)
    } else if boot.eq_ignore_ascii_case(crate::source::UNTAMED.boot) {
        Some(UNTAMED)
    } else {
        None
    }
}

/// The fields of the scene's `[SceneInfo]`, `[Fog]` and `[Environment]` against MX vs. ATV Unleashed's,
/// or why they could not be found.
pub fn compare(ram: &[u8], scene_text: &str) -> Result<Vec<Field>, String> {
    compare_with(ram, scene_text, &MXVATV)
}

/// [`compare`] with one game's [`Readers`].
pub fn compare_with(ram: &[u8], scene_text: &str, r: &Readers) -> Result<Vec<Field>, String> {
    let s = sections(scene_text);
    let empty = BTreeMap::new();
    let sec = |n: &str| s.get(n).unwrap_or(&empty);
    let one = |vt: u32, what: &str| -> Result<u32, String> {
        match with_vtable(ram, vt).as_slice() {
            [] => Err(format!("no {what} reader (vtable {vt:#x}) in the image")),
            [first, ..] => Ok(*first),
        }
    };
    let mut out = Vec::new();
    let mut push = |what: String, scene: Option<Vec<f32>>, default: Vec<f32>, game: Vec<f32>| {
        let defaulted = scene.is_none();
        out.push(Field { what, scene: scene.unwrap_or(default), defaulted, game, computed: false });
    };

    let info = sec("sceneinfo");
    // Untamed's reader makes the direction unit, and on SX_C a block of the class that was never read,
    // all zeros, comes first in the image: the scene's is the first with a unit direction, else the first.
    let unit = |v: u32| {
        let d = read(ram, v.saturating_sub(0x14), 3);
        ((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]) - 1.0).abs() < 1e-3
    };
    let blocks = with_vtable(ram, r.scene_info_vt);
    let v = match blocks.iter().find(|&&v| unit(v)) {
        Some(&v) => v,
        None => one(r.scene_info_vt, "[SceneInfo]")?,
    };
    let start = info.get("defaultstartposition").map(|t| floats(t));
    push("SceneInfo DefaultStartPosition".into(), start, vec![0.0; 3], read(ram, v - 0x24, 3));
    let dir = info.get("defaultstartdirection").map(|t| floats(t));
    let game_dir = read(ram, v - 0x14, 3);
    match dir {
        // Marked computed below, once `push` is done with `out`.
        Some(d) if r.unit_direction && d.len() == 3 => {
            let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            push(UNIT_DIRECTION.into(), Some(d.iter().map(|c| c / len).collect()), vec![], game_dir);
        }
        _ => push("SceneInfo DefaultStartDirection".into(), dir, vec![0.0, 0.0, 1.0], game_dir),
    }

    let fog = sec("fog");
    let fog_vt = r.fog_vt;
    // A zone's fog is an object of the same class whose distances are not set: the scene's is the first
    // with a distance pair (0 <= min <= max, max >= 1).
    let v = with_vtable(ram, fog_vt)
        .into_iter()
        .find(|&v| {
            let (lo, hi) = (f32_at(ram, v - 8).unwrap_or(f32::NAN), f32_at(ram, v - 4).unwrap_or(f32::NAN));
            (lo == 0.0 || lo.is_normal()) && lo >= 0.0 && hi.is_normal() && hi >= 1.0 && hi >= lo
        })
        .ok_or_else(|| format!("no [Fog] reader (vtable {fog_vt:#x}) with distances in the image"))?;
    let rgb = fog.get("colorrgb").map(|t| {
        let mut c = floats(t);
        c.resize(3, 255.0);
        c.push(255.0);
        c
    });
    push("Fog ColorRGB".into(), rgb, vec![255.0; 4], read(ram, v - r.fog_rgb, 4));
    let mut fog_vals = [0.0f32; 4];
    for (i, k) in ["Visibility", "Haziness", "MinimumFogDistance", "MaximumFogDistance"].iter().enumerate() {
        let val = fog.get(&k.to_ascii_lowercase()).and_then(|t| floats(t).first().copied());
        fog_vals[i] = val.unwrap_or(FOG_DEFAULTS[i]);
        push(format!("Fog {k}"), val.map(|f| vec![f]), vec![FOG_DEFAULTS[i]], read(ram, v - r.fog[i], 1));
    }
    if let Some((start, end)) = r.fog_start_end {
        for (k, off, d) in [("FogStart", start, fog_vals[2]), ("FogEnd", end, fog_vals[3])] {
            let val = fog.get(&k.to_ascii_lowercase()).and_then(|t| floats(t).first().copied()).map(|f| vec![f]);
            push(format!("Fog {k}"), val, vec![d], read(ram, v - off, 1));
        }
    }

    let env = sec("environment");
    let v = one(r.env_vt, "[Environment]")?;
    let base = v - 4 * r.env_vt_word;
    let word = |w: u32| base + 4 * w;
    let key = |k: &str| env.get(&k.to_ascii_lowercase()).and_then(|t| floats(t).first().copied());
    let border = key("TerrainBorderWidth").unwrap_or(0.0);
    let eco = env.get("ecosystemfile").is_some_and(|f| !f.is_empty() && !f.eq_ignore_ascii_case("none"));
    for &(k, w, d) in r.env {
        let val = key(k);
        if (val.is_none() && ENV_CONDITIONAL.contains(&k)) || (!eco && ENV_ECO.contains(&k)) {
            continue;
        }
        // The box's defaults are the terrain's own: -border, and width x scale x 256 + border.
        let d = match k {
            "Xmin" | "Zmin" => -border,
            "Xmax" => key("TerrainWidth").unwrap_or(0.0) * key("TerrainWidthScale").unwrap_or(0.0) * 256.0 + border + 0.0,
            "Zmax" => key("TerrainBreadth").unwrap_or(0.0) * key("TerrainBreadthScale").unwrap_or(0.0) * 256.0 + border + 0.0,
            _ => d,
        };
        push(format!("Environment {k}"), val.map(|f| vec![f]), vec![d], read(ram, word(w), 1));
    }
    let (k, w) = r.transmitted;
    let val = env.get(&k.to_ascii_lowercase()).and_then(|t| floats(t).first().copied()).map(|f| vec![f]);
    push(format!("Environment {k}"), val, vec![0.0], read(ram, word(w), 1));
    let int = |k: &str| env.get(&k.to_ascii_lowercase()).and_then(|t| t.trim().parse::<i32>().ok());
    let (k, w, d) = r.detail;
    let game = u32_at(ram, word(w)).map_or(f32::NAN, |i| i as i32 as f32);
    push(format!("Environment {k}"), int(k).map(|i| vec![i as f32]), vec![d as f32], vec![game]);
    for &(k, w, d, less_one) in r.env_ints {
        let held = |i: i32| if less_one && i > 0 { i - 1 } else { i };
        let game = u32_at(ram, word(w)).map_or(f32::NAN, |i| i as i32 as f32);
        let what = if less_one { format!("Environment {k} (less one)") } else { format!("Environment {k}") };
        push(what, int(k).map(|i| vec![held(i) as f32]), vec![held(d) as f32], vec![game]);
    }
    for (k, w) in r.scales {
        let val = env.get(&k.to_ascii_lowercase()).map(|t| {
            let mut c = floats(t);
            c.resize(3, 1.0);
            c.push(1.0);
            c
        });
        push(format!("Environment {k}"), val, vec![1.0; 4], read(ram, word(w), 4));
    }

    let mut computed = Vec::new();
    let lights: Vec<u32> = with_vtable(ram, r.light_vt).into_iter().map(|v| v - 4).collect();
    let n: usize = sec("lights").get("numberoflights").and_then(|t| t.trim().parse().ok()).unwrap_or(0);
    for i in 1..=n {
        let l = sec(&format!("light{i}"));
        let kind = l.get("type").map(|t| t.to_ascii_lowercase()).unwrap_or_default();
        let code = match kind.as_str() {
            "ambient" => 4,
            "directional" => 2,
            "objectpoint" => 1,
            "spotlight" => 3,
            _ => 4,
        };
        let Some(&at) = lights.iter().find(|&&a| u32_at(ram, a + 0x70) == Some(code)) else {
            return Err(format!("no light object of type {code} ([Light{i}] {kind}) in the image"));
        };
        let rgb = l.get("colorrgb").map(|t| {
            let mut c: Vec<f32> = floats(t).iter().map(|v| v / 255.0).collect();
            c.resize(3, 1.0);
            c.push(1.0);
            c
        });
        push(format!("Light{i} ({kind}) ColorRGB / 255"), rgb, vec![1.0; 4], read(ram, at + 0x30, 4));
        if code == 2 {
            let pos = l.get("position").map(|t| floats(t));
            let target = l.get("targetposition").map(|t| floats(t));
            push(format!("Light{i} ({kind}) Position"), pos.clone(), vec![0.0; 3], read(ram, at + 0x40, 3));
            if let (Some(p), Some(t)) = (pos, target)
                && p.len() == 3
                && t.len() == 3
            {
                let d = [t[0] - p[0], t[1] - p[1], t[2] - p[2]];
                let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                computed.push(Field {
                    what: format!("Light{i} ({kind}) direction, Target - Position made unit"),
                    scene: d.iter().map(|c| c / len).collect(),
                    defaulted: false,
                    game: read(ram, at + 0x50, 3),
                    computed: true,
                });
            }
        }
    }
    out.extend(computed);
    for f in out.iter_mut().filter(|f| f.what == UNIT_DIRECTION) {
        f.computed = true;
    }
    Ok(out)
}

/// The report: one line a field, then a verdict line first.
pub fn text(id: &str, fields: &[Field]) -> String {
    let differ = fields.iter().filter(|f| f.differs()).count();
    let near = fields.iter().filter(|f| matches!(f.verdict(), "1 ulp" | "close")).count();
    let mut s = format!(
        "{id}: scene in RAM: {} fields, {} equal, {near} within an ulp (or 8 on a computed one), {differ} differ\n",
        fields.len(),
        fields.len() - differ - near
    );
    for f in fields {
        let bits = |v: &[f32]| v.iter().map(|x| format!("{:08x}", x.to_bits())).collect::<Vec<_>>().join(" ");
        s.push_str(&format!(
            "  {:<40} {:<7} scene{} {:?} [{}]  game {:?} [{}]\n",
            f.what,
            f.verdict(),
            if f.defaulted { " (default)" } else { "" },
            f.scene,
            bits(&f.scene),
            f.game,
            bits(&f.game)
        ));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put(ram: &mut [u8], at: u32, v: &[f32]) {
        for (i, f) in v.iter().enumerate() {
            let a = at as usize + 4 * i;
            ram[a..a + 4].copy_from_slice(&f.to_bits().to_le_bytes());
        }
    }
    fn put_u32(ram: &mut [u8], at: u32, v: u32) {
        ram[at as usize..at as usize + 4].copy_from_slice(&v.to_le_bytes());
    }

    #[test]
    fn reads_each_block_back_from_its_vtable() {
        let mut ram = vec![0u8; 0x4000];
        // SceneInfo at 0x100 (vtable word at 0x124), fog at 0x200 (0x220), environment at 0x1000.
        put(&mut ram, 0x100, &[10.0, 20.0, 30.0, 1.0, 0.5, 0.0, -0.5, 0.0]);
        put_u32(&mut ram, 0x124, SCENE_INFO_VT);
        put(&mut ram, 0x200, &[121.0, 118.0, 111.0, 255.0, 1.0, f32::from_bits(0x3f4ccccc), 5.0, 768.0]);
        put_u32(&mut ram, 0x220, FOG_VT);
        let env = 0x1000u32;
        put_u32(&mut ram, env + 4 * ENV_VT_WORD, ENV_VT);
        put(&mut ram, env + 4 * 0x14e, &[5.0, 5.0, 4.5, 4.5]);
        // The box by default: -border and 5 x 4.5 x 256 + border; MinDivisionSize 96 by default.
        put(&mut ram, env + 4 * 0x154, &[-0.0, 5760.0, -0.0, 5760.0, 96.0]);
        put(&mut ram, env + 4 * 0x159, &[300.0, 120.0, 400.0]);
        put(&mut ram, env + 4 * 0x149, &[0.15]);
        put(&mut ram, env + 4 * 0x14b, &[50.0]);
        put_u32(&mut ram, env + 4 * 0x153, 8);
        put(&mut ram, env + 4 * 0x160, &[1.0, 1.0, 1.0, 1.0, 1.3, 1.3, 1.3, 1.0]);
        let scene = "[SceneInfo]\nDefaultStartPosition=10,20,30\nDefaultStartDirection=0.5,0,-0.5\n\n[Environment]\n\
            TerrainWidth=5\nTerrainBreadth=5\nTerrainWidthScale=4.5\nTerrainBreadthScale=4.5\nEcoDistance2D=300\n\
            EcoDiffuseScale=1.3,1.3,1.3\n\n[Fog]\nColorRGB=121,118,111\nVisibility=1.00\nHaziness=0.80\nMinimumFogDistance=5.00\nMaximumFogDistance=768.00\n";
        let f = compare(&ram, scene).unwrap();
        let get = |w: &str| f.iter().find(|x| x.what == w).unwrap();
        assert_eq!(get("SceneInfo DefaultStartPosition").verdict(), "equal");
        assert_eq!(get("Fog ColorRGB").verdict(), "equal");
        // 0.80 rounds to 3f4ccccd; the game's parser, which truncates, gives 3f4ccccc.
        assert_eq!(get("Fog Haziness").verdict(), "equal");
        assert_eq!(get("Environment EcoDistance3D").verdict(), "equal");
        assert!(get("Environment EcoDistance3D").defaulted);
        assert_eq!(get("Environment DetailMapTilingMultiplier").verdict(), "equal");
        assert_eq!(get("Environment EcoDiffuseScale").verdict(), "equal");
        assert!(f.iter().all(|x| x.ulps() == 0), "{}", text("t", &f));
    }

    #[test]
    fn reads_untamed_blocks_with_their_extra_keys() {
        let mut ram = vec![0u8; 0x4000];
        // A SceneInfo block never read (zeros, as SX_C's image has one) before the scene's.
        put_u32(&mut ram, 0x0a4, UNTAMED.scene_info_vt);
        put(&mut ram, 0x100, &[10.0, 20.0, 30.0, 1.0, 0.6, 0.0, 0.8, 1.0]);
        put_u32(&mut ram, 0x124, UNTAMED.scene_info_vt);
        // Fog: colour, visibility, haziness, FogStart, FogEnd, then the distances, then the vtable.
        put(&mut ram, 0x200, &[114.0, 117.0, 123.0, 255.0, 1.0, 0.0, 2000.0, 4000.0, 2000.0, 4000.0]);
        put_u32(&mut ram, 0x228, UNTAMED.fog_vt);
        let env = 0x1000u32;
        put_u32(&mut ram, env + 4 * 0x169, UNTAMED.env_vt);
        put(&mut ram, env + 4 * 0x14e, &[1.0, 1.0, 3.5, 3.5, 1000.0, 0.0]);
        put_u32(&mut ram, env + 4 * 0x154, 8);
        put(&mut ram, env + 4 * 0x155, &[-1000.0, 1896.0, -1000.0, 1896.0, 96.0, 180.0, 120.0, 400.0, 0.0]);
        put(&mut ram, env + 4 * 0x160, &[1.0; 8]);
        put_u32(&mut ram, env + 4 * 0x168, 2);
        let scene = "[SceneInfo]\nDefaultStartPosition=10,20,30\nDefaultStartDirection=3,0,4\n\n[Environment]\n\
            TerrainWidth=1\nTerrainBreadth=1\nTerrainWidthScale=3.5\nTerrainBreadthScale=3.5\nTerrainBorderWidth=1000\n\
            EcosystemZone=3\n\n[Fog]\nColorRGB=114,117,123\nHaziness=0.0\nMinimumFogDistance=2000\nMaximumFogDistance=4000\n";
        let f = compare_with(&ram, scene, &UNTAMED).unwrap();
        let get = |w: &str| f.iter().find(|x| x.what == w).unwrap_or_else(|| panic!("no {w}"));
        assert!(f.iter().all(|x| x.ulps() == 0), "{}", text("t", &f));
        // The reader made the direction unit: the text's 3,0,4 is held as 0.6,0,0.8.
        assert!(get("SceneInfo DefaultStartDirection, made unit").computed);
        // FogStart and FogEnd default to the distances; EcosystemZone is held less one.
        assert!(get("Fog FogStart").defaulted && get("Fog FogStart").game == vec![2000.0]);
        assert_eq!(get("Environment EcosystemZone (less one)").game, vec![2.0]);
        assert_eq!(get("Environment Xmax").scene, vec![1896.0]);
        assert_eq!(get("Environment DetailMapLODBias").game, vec![0.0]);
        // Unleashed's readers do not find Untamed's blocks.
        assert!(compare_with(&ram, scene, &MXVATV).is_err());
        assert_eq!(readers("SLUS_217.01").unwrap().env_vt, UNTAMED.env_vt);
        assert!(readers("SLUS_207.67").is_none());
    }
}
