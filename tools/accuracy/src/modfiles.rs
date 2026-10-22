//! A built track mod's terrain and scene, as openmx will read them.

use game_sim::track::{Scene, Terrain, TerrainLayout};
use modcheck::json::Json;
use std::path::{Path, PathBuf};

pub struct TrackMod {
    pub folder: PathBuf,
    pub id: String,
    /// `map` in mod.json (the source's own map name), else the id.
    pub map: String,
    /// `source.boot`: which game the mod came from.
    pub boot: String,
    /// The `.trn` as openmx's `Terrain::parse` takes it today (`modcheck::terrain::md1_view` where the
    /// file has texture pages, openmx-asks MD1).
    pub terrain: Vec<u8>,
    pub layout: TerrainLayout,
    /// The scene's spawn and the way it faces: where a pose list starts.
    pub start: [f32; 3],
    pub dir: [f32; 3],
}

fn lookup(json: &Json, key: &str) -> Option<String> {
    json.get("lookups")?.get(key)?.str().map(str::to_owned)
}

impl TrackMod {
    pub fn load(dir: &Path) -> Result<TrackMod, String> {
        let text = std::fs::read_to_string(dir.join("mod.json")).map_err(|e| format!("{}/mod.json: {e}", dir.display()))?;
        let json = Json::parse(&text).map_err(|e| format!("{}/mod.json: {e}", dir.display()))?;
        let id = json.get("id").and_then(Json::str).ok_or("mod.json has no id")?.to_owned();
        let map = json.get("map").and_then(Json::str).unwrap_or(&id).to_owned();
        let boot = json.get("source").and_then(|s| s.get("boot")).and_then(Json::str).unwrap_or("").to_owned();
        let (Some(trn), Some(scn)) = (lookup(&json, "terrain"), lookup(&json, "scene")) else {
            return Err(format!("{id}: mod.json's lookups name no terrain or no scene"));
        };
        let read = |rel: &str| std::fs::read(dir.join(rel)).map_err(|e| format!("{id}: {rel}: {e}"));
        let raw = read(&trn)?;
        let terrain = match Terrain::parse(&raw) {
            Ok(_) => raw,
            Err(_) => modcheck::terrain::md1_view(&raw).map_err(|e| format!("{id}: {trn}: {e}"))?.bytes,
        };
        Terrain::parse(&terrain).map_err(|e| format!("{id}: {trn}: {e}"))?;
        let scene = Scene::parse(&read(&scn)?).map_err(|e| format!("{id}: {scn}: {e}"))?;
        let start = scene.start.map(|c| c.0);
        let facing = scene.start_direction.map(|c| c.0);
        Ok(TrackMod { folder: dir.to_owned(), id, map, boot, terrain, layout: scene.terrain, start, dir: facing })
    }

    pub fn parse(&self) -> Terrain<'_> {
        Terrain::parse(&self.terrain).expect("parsed once in load")
    }

    /// The first 16 bytes of the root node and the scale, which is how the original's object is found.
    pub fn root_key(&self) -> ([u8; 16], u32) {
        let t = self.parse();
        let at = t.root_at() as usize;
        (self.terrain[at..at + 16].try_into().expect("16 bytes"), t.scale.to_bits())
    }
}
