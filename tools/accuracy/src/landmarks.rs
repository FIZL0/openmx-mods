//! A map's landmarks (M12): the props a rider would know the map by, found in its own scene, and a
//! pose looking at each.
//!
//! A landmark is a `[Model<n>]` the scene places once (its `SLT` appears in no other section), that
//! carries no activity keys (`Hide`, `Activity`, `Type`: start zones, waypoints and gates, which a Free
//! Ride does not draw or draws only for an event), ranked by the size of its model file (`<SLT>.psg`
//! anywhere in the mod), the biggest first: on MX vs. ATV that is the courthouse, the church, the
//! stadium, never a crate. Two picked are at least [`APART`] ft from each other.

use crate::look::FOV;
use crate::modfiles::TrackMod;
use crate::pose::Pose;
use std::collections::HashMap;
use std::path::Path;

/// How far apart two landmarks of one map are, at least (ft).
pub const APART: f32 = 300.0;
/// How far the camera stands from a landmark, on the side of the map's centre (ft).
pub const BACK: f32 = 90.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Landmark {
    /// The scene's `SLT`, as it spells it.
    pub slt: String,
    pub position: [f32; 3],
    /// The model file's size in bytes.
    pub bytes: u64,
}

/// One `[Model<n>]` section of a scene's text: its SLT, position, and whether it has activity keys.
fn models(scene: &str) -> Vec<(String, [f32; 3], bool)> {
    let mut out = Vec::new();
    let mut cur: Option<(Option<String>, Option<[f32; 3]>, bool)> = None;
    let mut flush = |c: &mut Option<(Option<String>, Option<[f32; 3]>, bool)>| {
        if let Some((Some(s), Some(p), act)) = c.take() {
            out.push((s, p, act));
        }
    };
    for line in scene.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            flush(&mut cur);
            let name = line.trim_matches(|c| c == '[' || c == ']');
            let is_model = name.strip_prefix("Model").is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()));
            cur = is_model.then_some((None, None, false));
            continue;
        }
        let Some(c) = cur.as_mut() else { continue };
        let Some((k, v)) = line.split_once('=') else { continue };
        let (k, v) = (k.trim(), v.trim());
        match k.to_ascii_lowercase().as_str() {
            "slt" => c.0 = Some(v.to_owned()),
            "position" => {
                let f: Vec<f32> = v.split(',').filter_map(|w| w.trim().parse().ok()).collect();
                if let [x, y, z] = f[..] {
                    c.1 = Some([x, y, z]);
                }
            }
            "hide" | "activity" | "type" | "istrigger" | "invisible" => c.2 = true,
            _ => {}
        }
    }
    flush(&mut cur);
    out
}

/// Every `.psg` in the mod, by its stem in lower case, with its size.
fn model_sizes(dir: &Path) -> HashMap<String, u64> {
    let mut out = HashMap::new();
    let mut stack = vec![dir.to_owned()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("psg"))
                && let (Some(stem), Ok(md)) = (p.file_stem(), e.metadata())
            {
                let k = stem.to_string_lossy().to_ascii_lowercase();
                let v = out.entry(k).or_insert(0);
                *v = (*v).max(md.len());
            }
        }
    }
    out
}

/// Up to `n` landmarks from the scene's own text, the biggest model first.
pub fn find(scene: &str, sizes: &HashMap<String, u64>, n: usize) -> Vec<Landmark> {
    let all = models(scene);
    let mut count: HashMap<String, usize> = HashMap::new();
    for (s, _, _) in &all {
        *count.entry(s.to_ascii_lowercase()).or_default() += 1;
    }
    let mut once: Vec<Landmark> = all
        .into_iter()
        .filter(|(s, _, act)| !act && count[&s.to_ascii_lowercase()] == 1)
        .filter_map(|(s, p, _)| sizes.get(&s.to_ascii_lowercase()).map(|&b| Landmark { slt: s, position: p, bytes: b }))
        .collect();
    once.sort_by(|a, b| b.bytes.cmp(&a.bytes).then_with(|| a.slt.cmp(&b.slt)));
    let mut out: Vec<Landmark> = Vec::new();
    for l in once {
        if out.len() == n {
            break;
        }
        let far = out.iter().all(|o| {
            let (dx, dz) = (o.position[0] - l.position[0], o.position[2] - l.position[2]);
            (dx * dx + dz * dz).sqrt() >= APART
        });
        if far {
            out.push(l);
        }
    }
    out
}

/// The mod's landmarks: its scene from mod.json's `lookups.scene`.
pub fn of_mod(m: &TrackMod, n: usize) -> Result<Vec<Landmark>, String> {
    let text = std::fs::read_to_string(m.folder.join("mod.json")).map_err(|e| format!("{}: {e}", m.folder.display()))?;
    let json = modcheck::json::Json::parse(&text).map_err(|e| format!("{}/mod.json: {e}", m.folder.display()))?;
    let scn = json.get("lookups").and_then(|l| l.get("scene")).and_then(|s| s.str()).ok_or("mod.json names no scene")?;
    let bytes = std::fs::read(m.folder.join(scn)).map_err(|e| format!("{scn}: {e}"))?;
    Ok(find(&String::from_utf8_lossy(&bytes), &model_sizes(&m.folder), n))
}

/// A pose name for a landmark: `lm-` and its SLT in lower case.
pub fn pose_name(l: &Landmark) -> String {
    let s: String = l.slt.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c.to_ascii_lowercase() } else { '_' }).collect();
    format!("lm-{s}")
}

/// A pose looking at each landmark from [`BACK`] ft out on the side of the map's centre, at least 20 ft
/// over its base and 10 ft over the ground under the eye.
pub fn poses(m: &TrackMod, marks: &[Landmark]) -> Vec<(String, Pose)> {
    let terrain = m.parse();
    let ground = game_sim::world::Ground::new(&terrain, &m.layout);
    let (ex, ez) = ground.extent();
    let (cx, cz) = (ex.0 / 2.0, ez.0 / 2.0);
    let y_at = |x: f32, z: f32| ground.at(ps2_float::F(x), ps2_float::F(z)).map_or(f32::MIN, |h| h.y.0);
    marks
        .iter()
        .map(|l| {
            let [lx, ly, lz] = l.position;
            let (dx, dz) = (cx - lx, cz - lz);
            let d = (dx * dx + dz * dz).sqrt();
            let (ux, uz) = if d < 1.0 { (0.0, -1.0) } else { (dx / d, dz / d) };
            let (x, z) = (lx + BACK * ux, lz + BACK * uz);
            let y = (ly + 20.0).max(y_at(x, z) + 10.0);
            let (fx, fy, fz) = (lx - x, ly + 8.0 - y, lz - z);
            let yaw = fx.atan2(fz).to_degrees();
            let pitch = fy.atan2((fx * fx + fz * fz).sqrt()).to_degrees();
            (pose_name(l), Pose::from_angles([x, y, z], yaw, pitch, FOV))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_the_biggest_single_plain_models_apart() {
        let scene = "[SceneInfo]\nSceneName=x\n\n[Model1]\nSLT=Crate\nPosition=0,0,0\n\n[Model2]\nSLT=Crate\nPosition=10,0,0\n\n\
            [Model3]\nSLT=Church\nPosition=100,5,100\n\n[Model4]\nSLT=Gate\nPosition=500,0,500\nType=StartZone\nHide=True\n\n\
            [Model5]\nSLT=Barn\nPosition=150,0,100\n\n[Model6]\nSLT=Mill\nPosition=900,0,900\n\n[Lights]\n";
        let sizes: HashMap<String, u64> =
            [("crate", 9000), ("church", 5000), ("gate", 8000), ("barn", 4000), ("mill", 1000)].map(|(k, v)| (k.to_owned(), v)).into();
        let got = find(scene, &sizes, 3);
        let names: Vec<&str> = got.iter().map(|l| l.slt.as_str()).collect();
        // Crate is placed twice, Gate is an activity's, Barn is 50 ft from Church.
        assert_eq!(names, ["Church", "Mill"]);
        assert_eq!(got[0].position, [100.0, 5.0, 100.0]);
        assert_eq!(pose_name(&got[0]), "lm-church");
    }
}
