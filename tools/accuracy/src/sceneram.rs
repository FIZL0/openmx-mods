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
            _ => "DIFFERS",
        }
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

/// The fields of the scene's `[SceneInfo]`, `[Fog]` and `[Environment]` against the game's, or why they
/// could not be found.
pub fn compare(ram: &[u8], scene_text: &str) -> Result<Vec<Field>, String> {
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
        out.push(Field { what, scene: scene.unwrap_or(default), defaulted, game });
    };

    let info = sec("sceneinfo");
    let v = one(SCENE_INFO_VT, "[SceneInfo]")?;
    let start = info.get("defaultstartposition").map(|t| floats(t));
    push("SceneInfo DefaultStartPosition".into(), start, vec![0.0; 3], read(ram, v - 0x24, 3));
    let dir = info.get("defaultstartdirection").map(|t| floats(t));
    push("SceneInfo DefaultStartDirection".into(), dir, vec![0.0, 0.0, 1.0], read(ram, v - 0x14, 3));

    let fog = sec("fog");
    let v = one(FOG_VT, "[Fog]")?;
    let rgb = fog.get("colorrgb").map(|t| {
        let mut c = floats(t);
        c.resize(3, 255.0);
        c.push(255.0);
        c
    });
    push("Fog ColorRGB".into(), rgb, vec![255.0; 4], read(ram, v - 0x20, 4));
    for (i, k) in ["Visibility", "Haziness", "MinimumFogDistance", "MaximumFogDistance"].iter().enumerate() {
        let val = fog.get(&k.to_ascii_lowercase()).and_then(|t| floats(t).first().copied()).map(|f| vec![f]);
        push(format!("Fog {k}"), val, vec![FOG_DEFAULTS[i]], read(ram, v - 0x10 + 4 * i as u32, 1));
    }

    let env = sec("environment");
    let v = one(ENV_VT, "[Environment]")?;
    let base = v - 4 * ENV_VT_WORD;
    let word = |w: u32| base + 4 * w;
    let key = |k: &str| env.get(&k.to_ascii_lowercase()).and_then(|t| floats(t).first().copied());
    let border = key("TerrainBorderWidth").unwrap_or(0.0);
    let eco = env.get("ecosystemfile").is_some_and(|f| !f.is_empty() && !f.eq_ignore_ascii_case("none"));
    for (k, w, d) in ENV {
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
    let (k, w) = ENV_TRANSMITTED;
    let val = env.get(&k.to_ascii_lowercase()).and_then(|t| floats(t).first().copied()).map(|f| vec![f]);
    push(format!("Environment {k}"), val, vec![0.0], read(ram, word(w), 1));
    let (k, w, d) = ENV_DETAIL;
    let val = env.get(&k.to_ascii_lowercase()).and_then(|t| t.trim().parse::<i32>().ok()).map(|i| vec![i as f32]);
    let game = u32_at(ram, word(w)).map_or(f32::NAN, |i| i as i32 as f32);
    push(format!("Environment {k}"), val, vec![d as f32], vec![game]);
    for (k, w) in ENV_SCALES {
        let val = env.get(&k.to_ascii_lowercase()).map(|t| {
            let mut c = floats(t);
            c.resize(3, 1.0);
            c.push(1.0);
            c
        });
        push(format!("Environment {k}"), val, vec![1.0; 4], read(ram, word(w), 4));
    }
    Ok(out)
}

/// The report: one line a field, then a verdict line first.
pub fn text(id: &str, fields: &[Field]) -> String {
    let differ = fields.iter().filter(|f| f.ulps() > 1).count();
    let ulp = fields.iter().filter(|f| f.ulps() == 1).count();
    let mut s = format!(
        "{id}: scene in RAM: {} fields, {} equal, {ulp} 1 ulp off, {differ} differ\n",
        fields.len(),
        fields.len() - differ - ulp
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
}
