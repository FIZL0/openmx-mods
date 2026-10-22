//! A sky, fog and lights for a map whose data has none (M17), taken whole from one disc map of the
//! same kind: its scene's `CubeFile`, its `[Fog]`, `[Lights]` and `[Light<n>]` sections, copied line
//! for line, and the `.cub` the `CubeFile` names, copied byte for byte from the map's own archive.
//! Nothing is mixed or tuned: the lights, the fog and the sky (whose lower half is painted in the fog's
//! colour) stay the one map's set. The donor is named in `mod.json`'s `sky_from` and in its `todo`.
//!
//! The donors are MX vs. ATV Unleashed's (`notes/elevation.md`, *Sky, fog and lights*), so a mod built
//! with one holds that disc's bytes and its permission is `own-disc`.

use discx::Disc;
use std::path::{Path, PathBuf};

/// The game the donors come from, by its boot ELF (AGENTS.md's table).
pub const BOOT: &str = "SLUS_211.04";
pub const GAME: &str = "MX vs. ATV Unleashed";

/// The scene sections copied whole. `[Lights]` only counts them (`NumberOfLights`), and is kept with
/// them as the source has it.
fn copied(section: &str) -> bool {
    let s = section.to_ascii_lowercase();
    s == "fog"
        || s == "lights"
        || s.strip_prefix("light")
            .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

/// What is taken from one disc map.
#[derive(Clone, Debug, PartialEq)]
pub struct Donor {
    /// The map as the disc names its folder (`Nat03`), and its name in the game (`Glamis`).
    pub map: String,
    pub name: String,
    /// The PSR member the scene and the sky are read from: `Env\Nat03\nat03.res`.
    pub res: String,
    /// The scene's and the sky's member paths in it, as the archive spells them.
    pub scene: String,
    pub cube_member: String,
    /// `[Environment] CubeFile`'s value, as written (`sky16.cub`).
    pub cube_file: String,
    pub cube: Vec<u8>,
    /// The sections copied, in the donor's order, and their text: each section's header and lines as
    /// the donor has them (CRLF), blank lines between sections dropped.
    pub sections: Vec<String>,
    pub text: String,
    /// The donor's whole scene, for the proof.
    pub donor_scene: String,
}

/// The `[section]` name of a header line, if it is one.
fn header(line: &str) -> Option<&str> {
    line.trim()
        .strip_prefix('[')
        .and_then(|l| l.strip_suffix(']'))
}

impl Donor {
    /// Take a map's sky, fog and lights from its scene text and a reader of its archive's members.
    pub fn from_scene(
        map: &str,
        name: &str,
        res: &str,
        scene_member: &str,
        scene: &str,
        mut member: impl FnMut(&str) -> Option<(String, Vec<u8>)>,
    ) -> Result<Donor, String> {
        let mut section = String::new();
        let mut cube_file = None;
        let mut sections: Vec<String> = Vec::new();
        let mut text = String::new();
        for line in scene.lines() {
            let line = line.strip_suffix('\r').unwrap_or(line);
            if let Some(h) = header(line) {
                section = h.to_owned();
                if copied(h) {
                    if sections.iter().any(|s| s.eq_ignore_ascii_case(h)) {
                        return Err(format!("{scene_member}: [{h}] twice"));
                    }
                    sections.push(h.to_owned());
                    text.push_str(line);
                    text.push_str("\r\n");
                }
                continue;
            }
            if copied(&section) {
                if !line.trim().is_empty() {
                    text.push_str(line);
                    text.push_str("\r\n");
                }
            } else if section.eq_ignore_ascii_case("Environment")
                && let Some((k, v)) = line.split_once('=')
                && k.trim().eq_ignore_ascii_case("CubeFile")
            {
                cube_file = Some(v.trim().to_owned());
            }
        }
        let cube_file = cube_file.ok_or(format!("{scene_member}: no CubeFile"))?;
        if !sections.iter().any(|s| s.eq_ignore_ascii_case("Fog")) {
            return Err(format!("{scene_member}: no [Fog]"));
        }
        if !sections.iter().any(|s| {
            copied(s) && !s.eq_ignore_ascii_case("Fog") && !s.eq_ignore_ascii_case("Lights")
        }) {
            return Err(format!("{scene_member}: no [Light<n>]"));
        }
        // The member game-render's `Sky::load` would find: `<stem>.ps2.cub`, then the name as written.
        let low = cube_file.to_ascii_lowercase();
        let stem = low.strip_suffix(".cub").unwrap_or(&low);
        let (cube_member, cube) = member(&format!("{stem}.ps2.cub"))
            .or_else(|| member(&low))
            .ok_or(format!("{res}: no {stem}.ps2.cub for CubeFile={cube_file}"))?;
        game_render::sky::Sky::parse(&cube)
            .map_err(|e| format!("{res}:{cube_member}: game-render's Sky::parse: {e}"))?;
        Ok(Donor {
            map: map.to_owned(),
            name: name.to_owned(),
            res: res.to_owned(),
            scene: scene_member.to_owned(),
            cube_member,
            cube_file,
            cube,
            sections,
            text,
            donor_scene: scene.to_owned(),
        })
    }

    /// Take a map's from the disc: `Env\<map>\<map>.res`, its one `.scn`, and the `.cub` it names.
    pub fn take(disc: &mut Disc, map: &str, name: &str) -> Result<Donor, String> {
        let want = format!("Env\\{map}\\{}.res", map.to_ascii_lowercase());
        let e = disc
            .find(&want)
            .cloned()
            .ok_or(format!("{}: no {want}", disc.path.display()))?;
        let res = disc.res(&e)?;
        let scenes = res.matching("**.scn");
        let [scn] = scenes[..] else {
            return Err(format!("{}: {} scenes", e.name, scenes.len()));
        };
        let scene_member = scn.name.clone();
        let text = String::from_utf8(res.read(scn)?)
            .map_err(|_| format!("{}:{scene_member}: not UTF-8", e.name))?;
        Donor::from_scene(map, name, &e.name, &scene_member, &text, |suffix| {
            let m = res.dir.entries.iter().find(|m| {
                let n = m.name.to_ascii_lowercase().replace('/', "\\");
                n == suffix || n.ends_with(&format!("\\{suffix}"))
            })?;
            Some((m.name.clone(), res.read(m).ok()?))
        })
    }

    /// The mod's file for the sky: the member's path with `\` turned to `/`, its spelling kept (§2).
    pub fn cube_path(&self) -> String {
        self.cube_member.replace('\\', "/")
    }

    /// `scene` (a generated scene, which ends in its `[Environment]` section) with `CubeFile` added to
    /// that section and the donor's sections after it.
    pub fn scene_with(&self, scene: &str) -> Result<String, String> {
        let last = scene.lines().filter_map(header).last().unwrap_or_default();
        if !last.eq_ignore_ascii_case("Environment") {
            return Err(format!("the scene ends in [{last}], not [Environment]"));
        }
        let mut s = scene.trim_end_matches(['\r', '\n']).to_owned();
        s.push_str(&format!("\r\nCubeFile={}\r\n\r\n", self.cube_file));
        // One blank line between sections, as the generated scene writes them.
        let mut first = true;
        for line in self.text.split("\r\n").filter(|l| !l.is_empty()) {
            if header(line).is_some() && !first {
                s.push_str("\r\n");
            }
            first = false;
            s.push_str(line);
            s.push_str("\r\n");
        }
        Ok(s)
    }

    /// The scene as game-render reads it has this donor's fog, lights and sky: every number of `[Fog]`
    /// and every light the same as in the donor's own scene, and its `CubeFile` the one whose file the
    /// mod holds.
    pub fn prove(&self, scene: &str) -> Result<(), String> {
        use game_render::scene::Scene;
        let (a, b) = (Scene::parse(scene), Scene::parse(&self.donor_scene));
        let fog = |s: &Scene| {
            (
                s.fog_color,
                s.fog_near.to_bits(),
                s.fog_far.to_bits(),
                s.fog_visibility.to_bits(),
                s.fog_haziness.to_bits(),
            )
        };
        if fog(&a) != fog(&b) {
            return Err(format!(
                "the scene's fog {:?} is not {}'s {:?}",
                fog(&a),
                self.map,
                fog(&b)
            ));
        }
        let lights =
            |s: &Scene| -> Vec<String> { s.lights.iter().map(|l| format!("{l:?}")).collect() };
        if lights(&a) != lights(&b) || a.lights.is_empty() {
            return Err(format!(
                "the scene's lights {:?} are not {}'s {:?}",
                lights(&a),
                self.map,
                lights(&b)
            ));
        }
        let cube = game_render::ini::parse(scene)
            .iter()
            .find(|s| s.name.eq_ignore_ascii_case("Environment"))
            .and_then(|s| s.get("CubeFile").map(|v| v.trim().to_ascii_lowercase()));
        let Some(cube) = cube else {
            return Err("the scene's [Environment] has no CubeFile".into());
        };
        let stem = cube.strip_suffix(".cub").unwrap_or(&cube).to_owned();
        let path = self.cube_path().to_ascii_lowercase();
        if !(path.ends_with(&format!("{stem}.ps2.cub")) || path.ends_with(&cube)) {
            return Err(format!(
                "CubeFile={cube} does not find {}",
                self.cube_path()
            ));
        }
        Ok(())
    }
}

/// The MX vs. ATV Unleashed image in `dir`, found by its boot ELF.
pub fn find_disc(dir: &Path) -> Option<PathBuf> {
    discx::images(dir)
        .ok()?
        .into_iter()
        .find(|(_, b)| b.as_deref().is_some_and(|b| b.eq_ignore_ascii_case(BOOT)))
        .map(|(p, _)| p)
}
