//! The look: screenshot pairs from the same poses, the original's and openmx's, side by side in
//! `out/accuracy/<map>/` for a person or an agent to read.
//!
//! The folder holds `poses.txt` (one [`Pose`] a line), `<pose>.original.png` (`./kit shot` of the
//! source game in PCSX2), `<pose>.openmx.png` (`openmx --shot` from the same pose, openmx MD10) and
//! `<pose>.pair.png`, the two side by side, original on the left; `index.md` lists them. Shots are game
//! data: `out/` is never committed.

use crate::pose::Pose;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const POSES: &str = "poses.txt";

pub fn original_png(dir: &Path, name: &str) -> PathBuf {
    dir.join(format!("{name}.original.png"))
}
pub fn openmx_png(dir: &Path, name: &str) -> PathBuf {
    dir.join(format!("{name}.openmx.png"))
}
pub fn pair_png(dir: &Path, name: &str) -> PathBuf {
    dir.join(format!("{name}.pair.png"))
}

/// Whether a file is a whole PNG: it ends in the `IEND` chunk (its name and CRC, the last 8 bytes).
pub fn png_complete(path: &Path) -> bool {
    std::fs::read(path).is_ok_and(|b| b.len() > 12 && b.starts_with(b"\x89PNG") && &b[b.len() - 8..b.len() - 4] == b"IEND")
}

/// The folder's poses, in file order.
pub fn read_poses(dir: &Path) -> Result<Vec<(String, Pose)>, String> {
    let p = dir.join(POSES);
    let text = match std::fs::read_to_string(&p) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("{}: {e}", p.display())),
    };
    text.lines().filter(|l| !l.trim().is_empty() && !l.starts_with('#')).map(Pose::parse_line).collect()
}

/// Add or replace one pose.
pub fn write_pose(dir: &Path, name: &str, pose: &Pose) -> Result<(), String> {
    if name.is_empty() || name.contains(|c: char| c.is_whitespace() || c == '/' || c == '\\') {
        return Err(format!("{name:?}: a pose name is one word, no slashes"));
    }
    let mut all = read_poses(dir)?;
    all.retain(|(n, _)| n != name);
    all.push((name.to_owned(), *pose));
    let mut text = String::from("# name  eye x y z  forward x y z  up x y z  fov (tools/accuracy, look.rs)\n");
    for (n, p) in &all {
        text.push_str(&p.line(n));
        text.push('\n');
    }
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    std::fs::write(dir.join(POSES), text).map_err(|e| format!("{}: {e}", dir.display()))
}

/// The two shots side by side, the original on the left, both at the original's height. ImageMagick does
/// the pixels (`magick`), which keeps PNG out of this workspace.
pub fn compose(original: &Path, openmx: &Path, out: &Path) -> Result<(), String> {
    let size = Command::new("magick")
        .args(["identify", "-format", "%h"])
        .arg(original)
        .output()
        .map_err(|e| format!("magick (ImageMagick 7) is needed for the pairs: {e}"))?;
    let h = String::from_utf8_lossy(&size.stdout).trim().to_owned();
    if !size.status.success() || h.parse::<u32>().is_err() {
        return Err(format!("{}: not an image magick reads", original.display()));
    }
    let st = Command::new("magick")
        .arg(original)
        .arg("(")
        .arg(openmx)
        .args(["-resize", &format!("x{h}"), ")"])
        .args(["-background", "black", "+append"])
        .arg(out)
        .status()
        .map_err(|e| format!("magick: {e}"))?;
    if st.success() { Ok(()) } else { Err(format!("magick could not join {} and {}", original.display(), openmx.display())) }
}

/// What one pose has so far.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub name: String,
    pub pose: Pose,
    pub original: bool,
    pub openmx: bool,
    pub pair: bool,
}

pub fn rows(dir: &Path) -> Result<Vec<Row>, String> {
    Ok(read_poses(dir)?
        .into_iter()
        .map(|(name, pose)| Row {
            original: original_png(dir, &name).exists(),
            openmx: openmx_png(dir, &name).exists(),
            pair: pair_png(dir, &name).exists(),
            name,
            pose,
        })
        .collect())
}

/// `index.md`: every pose, its `--shot-pose`, and what is there. `heights` is the heights report's
/// first line, when there is one.
pub fn index(dir: &Path, map: &str, game: &str, waiting: Option<&str>) -> Result<String, String> {
    let mut s = format!("# {map}: {game} against openmx\n\nOriginal on the left, openmx on the right (tools/accuracy).\n\n");
    if let Ok(h) = std::fs::read_to_string(dir.join("heights.txt")) {
        s.push_str(&format!("Heights: {}\n\n", h.lines().next().unwrap_or("")));
    }
    if let Some(w) = waiting {
        s.push_str(&format!("openmx's shots: {w}\n\n"));
    }
    s.push_str("| pose | --shot-pose | original | openmx | pair |\n|---|---|---|---|---|\n");
    let mark = |b: bool, f: PathBuf| if b { format!("[yes]({})", f.file_name().unwrap().to_string_lossy()) } else { "no".into() };
    for r in rows(dir)? {
        s.push_str(&format!(
            "| {} | `{}` | {} | {} | {} |\n",
            r.name,
            r.pose.md10(),
            mark(r.original, original_png(dir, &r.name)),
            mark(r.openmx, openmx_png(dir, &r.name)),
            mark(r.pair, pair_png(dir, &r.name)),
        ));
    }
    Ok(s)
}

/// `pair`'s default `--openmx-args`: `{mods}` the mods folder (the one holding `tracks/<id>/`), `{id}`
/// the mod's id (its folder's name, which openmx MD2 requires mod.json's id to be), `{shot}` the PNG to
/// write, `{pose}` the pose as [`Pose::md10`] writes it. openmx's own flags (T9): MD2's `--mods DIR
/// --track ID`, MD10's `--shot-pose`. Headless, two frames (the shot is the first), 4:3 like the
/// original's shots, no HUD (the original's carries one: read the pairs past it), and the original's
/// fog and far clip (`--graphics original`), not openmx's longer view.
pub const OPENMX_ARGS: &str = "--mods {mods} --track {id} --headless --no-sound --frames 2 --size 640x480 --overlay none --graphics original --shot {shot} --shot-pose {pose}";

/// openmx's command line for one pose of the mod in `folder` (`<mods>/tracks/<id>`): the template split
/// on whitespace first, then each word's placeholders filled, so a path with spaces stays one argument.
/// `{mod}` is the folder itself, for a template of the caller's own.
pub fn openmx_args(template: &str, folder: &Path, shot: &Path, pose: &Pose) -> Vec<String> {
    let id = folder.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let mods = folder.parent().and_then(Path::parent).unwrap_or(Path::new(""));
    template
        .split_whitespace()
        .map(|w| {
            w.replace("{mods}", &mods.to_string_lossy())
                .replace("{mod}", &folder.to_string_lossy())
                .replace("{id}", &id)
                .replace("{shot}", &shot.to_string_lossy())
                .replace("{pose}", &pose.md10())
        })
        .collect()
}

/// Whether openmx can find the mod in `folder` by the `--mods` it is given: it looks in `<mods>/tracks/`.
pub fn in_a_mods_folder(folder: &Path) -> Result<(), String> {
    match folder.parent().and_then(Path::file_name) {
        Some(n) if n == "tracks" => Ok(()),
        _ => Err(format!("{}: openmx takes a mod from <mods>/tracks/<id>/, and this is not in a tracks folder", folder.display())),
    }
}

/// Whether `help` names `flag` as a word of its own: `--mod` is not in `--modifiers`.
pub fn help_has_flag(help: &str, flag: &str) -> bool {
    help.match_indices(flag).any(|(i, _)| {
        let before = help[..i].chars().next_back();
        let after = help[i + flag.len()..].chars().next();
        let word = |c: char| c.is_alphanumeric() || c == '-' || c == '_';
        !before.is_some_and(word) && !after.is_some_and(word)
    })
}

/// What an openmx `--help` lacks of the flags `template` uses (`--shot-pose`, MD10; `--mods`, MD2), or
/// nothing.
pub fn openmx_missing(help: &str, template: &str) -> Vec<String> {
    template
        .split_whitespace()
        .filter(|w| w.starts_with("--"))
        .map(|w| w.split('=').next().unwrap_or(w))
        .filter(|f| !help_has_flag(help, f))
        .map(|f| match f {
            "--shot-pose" => "--shot-pose (openmx MD10)".to_owned(),
            "--mods" => "--mods, tracks from a mods folder (openmx MD2)".to_owned(),
            f => f.to_owned(),
        })
        .collect()
}

/// Whether an openmx binary takes every flag of `template`, from its `--help`.
pub fn openmx_ready(openmx: &Path, template: &str) -> Result<(), String> {
    let out = Command::new(openmx).arg("--help").output().map_err(|e| format!("{}: {e}", openmx.display()))?;
    let help = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    let missing = openmx_missing(&help, template);
    if missing.is_empty() { Ok(()) } else { Err(format!("{} has no {}", openmx.display(), missing.join(" and no "))) }
}

/// The ride camera's field of view on MX vs. ATV, which the generated poses keep.
pub const FOV: f32 = 85.0;

/// The ride camera's field of view in the game with this boot ELF: 75 on Untamed (its Free Ride camera's
/// viewport on every map ridden, M13), else [`FOV`].
pub fn ride_fov(boot: &str) -> f32 {
    if boot.eq_ignore_ascii_case(crate::source::UNTAMED.boot) { 75.0 } else { FOV }
}

/// The terrain's box as the game's object has it (M12): as far as the file's leaves reach, 16 samples a
/// leaf at the file's scale. On every MX vs. ATV map it is the object's own box (`heights`' box check).
/// openmx's `world::Ground` takes the scene's `TerrainWidth x 16^levels` samples instead, which on a
/// one-level terrain (the arenas) is 16, not 256, and on Nat04 (56 x 80 leaves, the scene says 5 x 5)
/// is 1280 across, not 896 (notes/openmx-asks.md).
/// The product is the EE's (`terrain.scale` is an `F`), which truncates: Untamed's Nat_OR, at
/// scale 4.499999, has a box of 5759.9985, where a rounding multiply gives 5759.999 (M13).
pub fn terrain_box(_m: &crate::modfiles::TrackMod, terrain: &game_sim::track::Terrain) -> (f32, f32) {
    let (x, z) = leaf_extent(terrain);
    let s = terrain.scale;
    ((s * ps2_float::F::from(x as i32)).0, (s * ps2_float::F::from(z as i32)).0)
}

/// How many samples across and along the file's leaves reach: the farthest leaf's far edge.
pub fn leaf_extent(t: &game_sim::track::Terrain) -> (u32, u32) {
    fn walk(t: &game_sim::track::Terrain, node: &game_sim::track::TerrainNode, left: u32, ox: u32, oz: u32, max: &mut (u32, u32)) {
        if left == 0 {
            max.0 = max.0.max(ox + 16);
            max.1 = max.1.max(oz + 16);
            return;
        }
        let span = 16u32.pow(left);
        for cz in 0..16 {
            for cx in 0..16 {
                if let Some(child) = t.child(node, cx, cz).and_then(|at| t.node(at)) {
                    walk(t, &child, left - 1, ox + cx * span, oz + cz * span, max);
                }
            }
        }
    }
    let mut max = (0, 0);
    if let Some(root) = t.root() {
        walk(t, &root, t.levels, 0, 0, &mut max);
    }
    max
}

/// The ground's height at `x, z`: the port's terrain query (`contact_search::ground`, which the heights
/// check proves equal to the original's), 0 off the terrain.
pub fn height_at(field: &game_sim::contact_search::TerrainField, x: f32, z: f32) -> f32 {
    let at = ps2_float::Vf::new(x, 0.0, z, 1.0);
    let g = game_sim::contact_search::ground(field, at);
    if g.point.y().is_finite() { g.point.y() } else { 0.0 }
}

/// A pose list for a map, from its own terrain: the scene's start seen from behind the rider, the map's
/// centre looking north, east, south and west, an overview from high over the centre, and the highest
/// ground found (a sample in 32 across) looking across to the centre. Heights are openmx's ground under
/// each eye, which on a copied terrain is the original's (the heights check says so).
pub fn standard_poses(m: &crate::modfiles::TrackMod) -> Vec<(String, Pose)> {
    let terrain = m.parse();
    let ground = game_sim::world::Ground::new(&terrain, &m.layout);
    let field = ground.field();
    let (ex, ez) = terrain_box(m, &terrain);
    let y_at = |x: f32, z: f32| height_at(&field, x, z);
    let mut out = Vec::new();

    let [sx, sy, sz] = m.start;
    let dir = m.dir;
    let yaw = dir[0].atan2(dir[2]).to_degrees();
    let back = [sx - 20.0 * dir[0], 0.0, sz - 20.0 * dir[2]];
    out.push(("start".to_owned(), Pose::from_angles([back[0], sy.max(y_at(back[0], back[2])) + 8.0, back[2]], yaw, -12.0, FOV)));

    let (cx, cz) = (ex / 2.0, ez / 2.0);
    let cy = y_at(cx, cz) + 25.0;
    for (name, yaw) in [("centre-n", 180.0), ("centre-e", 270.0), ("centre-s", 0.0), ("centre-w", 90.0)] {
        // North is -z on these maps' radar; the names say which way the camera faces.
        out.push((name.to_owned(), Pose::from_angles([cx, cy, cz], yaw, -8.0, FOV)));
    }
    // Not higher: the fog closes in a few hundred feet out, and from 400 ft up there is only fog.
    out.push(("overview".to_owned(), Pose::from_angles([cx * 0.6, cy + 150.0, cz * 0.6], 45.0, -25.0, FOV)));

    let step = (ex.min(ez) / 32.0).max(1.0);
    let mut best = (f32::MIN, 0.0, 0.0);
    let mut x = step;
    while x < ex - step {
        let mut z = step;
        while z < ez - step {
            let h = y_at(x, z);
            if h > best.0 {
                best = (h, x, z);
            }
            z += step;
        }
        x += step;
    }
    let (h, rx, rz) = best;
    let to_centre = (cx - rx).atan2(cz - rz).to_degrees();
    out.push(("ridge".to_owned(), Pose::from_angles([rx, h + 25.0, rz], to_centre, -10.0, FOV)));
    out
}
