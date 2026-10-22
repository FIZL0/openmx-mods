//! What each source game keeps where: its terrain object, its terrain queries and its camera's viewport.
//!
//! These are research addresses, found with the kit (`notes/accuracy.md` says how), not product: the
//! harness needs them to run the original's own code on the original's own RAM. Only MX vs. ATV
//! Unleashed has been proved end to end (T7); the MX Unleashed row is copied from openmx's
//! `research/oracle.txt` and `crates/game-sim/tests/contact_search.rs` and has not been run here.

/// The fields of the terrain object the queries read, as offsets from the object.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerrainObject {
    /// The box the terrain covers, two quadwords.
    pub min: u32,
    pub max: u32,
    /// A float the point is kept inside the box by (MX vs. ATV); `None` where the query does not clamp.
    pub border: Option<u32>,
    /// The cell reader's heights are multiplied by this (the `.trn`'s scale).
    pub height_scale: u32,
    /// The tree's root node.
    pub tree: u32,
    /// `== 1`: every cell split the same way, else by the cell's parity.
    pub split: u32,
    /// Samples per world unit.
    pub per_sample: u32,
    /// Non-zero: cells two samples wide.
    pub coarse: u32,
    /// Zero makes the next query empty the shared ten-cell cache.
    pub cache_ready: u32,
}

/// A tree node in RAM: the four floats the file's node starts with sit at this offset.
pub const NODE_SCALES: u32 = 0x0c;

/// The viewport the follow camera writes each frame, as offsets from it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Viewport {
    /// The eye, `w = 1`.
    pub eye: u32,
    /// Where the camera looks: a unit vector.
    pub forward: u32,
    pub up: u32,
    /// The field of view, degrees.
    pub fov: u32,
    /// The width over the height, 4/3 on this camera.
    pub aspect: u32,
    /// A second copy of the eye the viewport keeps: how [`crate::ram::find_viewports`] tells one apart.
    pub eye_copy: u32,
}

/// One source game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Source {
    /// The name the CLI takes.
    pub name: &'static str,
    pub game: &'static str,
    /// The boot ELF (SYSTEM.CNF), which is how a disc is told from another (AGENTS.md).
    pub boot: &'static str,
    /// `(terrain, buf)`: buf +0 the point in and its `y` out, +0x10 the normal, +0x20 the cell's four
    /// surfaces, +0x30 the corner weights. The same buffer as MX Unleashed's `terrain_point_query`,
    /// which openmx ports as `contact_search::ground`.
    pub query: u32,
    /// `(terrain, point, normal out, -, surface out)`: the query a wheel uses, which writes the nearest
    /// corner's surface (openmx's `contact_search::nearest_surface`). `None` where it is not known.
    pub ride_query: Option<u32>,
    pub object: TerrainObject,
    pub viewport: Option<Viewport>,
    pub camera: Option<Camera>,
    pub normals: Normals,
}

/// How to hold the ride camera still: its class's update, `(dt in f12, cam)`, is one vtable slot, and
/// pointing the slot at a function that returns at once stops it writing the viewport. The viewport
/// then draws whatever eye, forward, up and fov are written into it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Camera {
    /// The vtable slot (an address in the ELF's data) and what it holds.
    pub update_slot: u32,
    pub update: u32,
    /// A function that only returns (`jr ra; move v0, a0`).
    pub stub: u32,
    /// More words to hold, in the order they are written: (address, what the game has there, what it
    /// holds instead); put back in the reverse order. Untamed (M13) stops its viewport's setter and
    /// look-at in the code itself.
    pub also: &'static [(u32, u32, u32)],
}

/// How the cell reader hands back a corner's normal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Normals {
    /// The file's three bytes as floats, x and z times the node's vertical scale (MX Unleashed, and
    /// openmx's `Terrain::cell`).
    Raw,
    /// The same over 127: `k = 1.0 / 127.0`, then x and z times `vertical scale × k`, y times `k`.
    /// When the word at `packed` is non-zero the game takes its normals from a table instead, which the
    /// harness does not model.
    Over127 { packed: u32 },
}

/// MX vs. ATV Unleashed (SLUS_211.04 1.02). The queries are in `GAME_REL.BIN`, the gameplay module the
/// boot ELF loads at `0x45c580`, so they are in a RAM image of a ride and not in the ELF. Every field of
/// the object from `+0x80` on sits 0x10 further than in MX Unleashed.
pub const MXVATV: Source = Source {
    name: "mxvatv",
    game: "MX vs. ATV Unleashed",
    boot: "SLUS_211.04",
    query: 0x004838f0,
    ride_query: Some(0x004833c0),
    object: TerrainObject {
        min: 0x30,
        max: 0x40,
        border: Some(0x50),
        height_scale: 0x90,
        tree: 0x94,
        split: 0x770,
        per_sample: 0x788,
        coarse: 0x8b8,
        cache_ready: 0x9e0,
    },
    viewport: Some(Viewport { eye: 0x140, forward: 0x150, up: 0x160, fov: 0x170, aspect: 0x17c, eye_copy: 0x1c0 }),
    // The follow camera's vtable is 0x3cc6f0 (its preset loader, boot 0x1c5e00, sits at +0x12c); the
    // update is game 0x48b490, at +0x40.
    camera: Some(Camera { update_slot: 0x003cc730, update: 0x0048b490, stub: 0x001288a0, also: &[] }),
    normals: Normals::Over127 { packed: 0x00541518 },
};

/// MX Unleashed (SLUS_207.67), from openmx's research. Not run by this tool yet.
pub const MXU: Source = Source {
    name: "mxu",
    game: "MX Unleashed",
    boot: "SLUS_207.67",
    query: 0x001dce80,
    ride_query: None,
    object: TerrainObject {
        min: 0x30,
        max: 0x40,
        border: None,
        height_scale: 0x80,
        tree: 0x84,
        split: 0x760,
        per_sample: 0x778,
        coarse: 0x8a8,
        cache_ready: 0x9d0,
    },
    viewport: None,
    camera: None,
    normals: Normals::Raw,
};

/// MX vs. ATV Untamed (SLUS_217.01 1.01, M13). The queries are in its `GAME_REL.BIN`, loaded at
/// `0x48ea80` (header and all, where `UI_REL.BIN` sits in the menus). They, the cell reader (`0x4a8ea0`)
/// and the terrain object are MX vs. ATV Unleashed's, field for field; only the addresses moved. The ten-cell
/// cache is at `0x544120` (count `0x53d2e0`), the packed normal table at `0x3bd6f0`.
pub const UNTAMED: Source = Source {
    name: "untamed",
    game: "MX vs. ATV Untamed",
    boot: "SLUS_217.01",
    query: 0x004b0e30,
    ride_query: Some(0x004b0910),
    object: MXVATV.object,
    // The viewport object starts 0x20 before these offsets (its vtable `0x3d6b50` at +4).
    viewport: MXVATV.viewport,
    // The viewport's setter (boot `0x111700`, `(vp, &eye, &forward, &up, -, &fov)`, Unleashed's
    // `0x111c30`) and its look-at (`0x1117d0`, which makes forward from a target each frame) are the
    // only writers of the ride's eye and forward: each first word becomes `jr ra` while held, one word
    // each, so the game never runs a half-written function. The delay slots left are harmless: the
    // setter's `move $10, $8`, the look-at's `addiu $3, $4, 0x160` (`$v1` only; its `addiu sp` is the
    // word replaced). A first try that also made the look-at's second word a `nop` let the game run it
    // between the two writes (PCSX2's log: a TLB miss at `0x1117d0`), and PCSX2's recompiler failed
    // a few maps later. No slot of a vtable does it: the follow camera's `0x3ef174` (`0x4b6670`) also
    // builds the view matrices, and the viewport's own slots are not on the ride camera's path.
    camera: Some(Camera {
        update_slot: 0x00111700,
        update: 0x10a00004,
        stub: 0x03e00008,
        also: &[(0x001117d0, 0x27bdffd0, 0x03e00008)],
    }),
    normals: Normals::Over127 { packed: 0x005440e8 },
};

pub const ALL: [Source; 3] = [MXVATV, MXU, UNTAMED];

/// A source by its CLI name or its boot ELF.
pub fn find(name: &str) -> Option<Source> {
    ALL.into_iter().find(|s| s.name.eq_ignore_ascii_case(name) || s.boot.eq_ignore_ascii_case(name))
}

/// The ps2-decomp-kit config for a source game (AGENTS.md: a kit config per game under
/// `extracted/<game>/`): its disc, the gameplay module where it has one, PINE on, and the kit's private
/// folders beside the config. Absolute paths, so the kit can be run from anywhere.
pub fn kit_toml(src: &Source, iso: &std::path::Path, dir: &std::path::Path) -> String {
    let serial = src.boot.replace('_', "-").replace('.', "");
    let d = dir.display();
    let overlay = match src.name {
        // The gameplay code, an "MWo3" module whose header (+8) says where it goes: into the boot
        // ELF's BSS at 0x45c580, header and all. The terrain queries and the ride camera are in it.
        "mxvatv" => "[[overlay]]\nname = \"game\"\nfile = \"GAME_REL.BIN\"\nload = \"0x45c580\"\n\n",
        // Untamed's, the same kind of module, at 0x48ea80 (M13).
        "untamed" => "[[overlay]]\nname = \"game\"\nfile = \"GAME_REL.BIN\"\nload = \"0x48ea80\"\n\n",
        _ => "",
    };
    format!(
        "# ps2-decomp-kit config for {game} (tools/accuracy setup, T7). Use it as\n\
         #   KIT_CONFIG={d}/kit.toml KIT_PCSX2=9 ../ps2-decomp-kit/kit ...\n\n\
         [game]\nname    = \"{game}\"\nserial  = \"{serial}\"\nregion  = \"US\"\nversion = \"\"\niso     = \"{iso}\"\n\n\
         [emu]\npcsx2      = \"pcsx2-qt\"\npine_slot  = 28011\nvsync_addr = \"auto\"\nspeed      = 1.0\ninstances  = 1\nram_mb     = 32\n\n\
         [boot]\nelf  = \"{boot}\"\nload = \"\"\n\n{overlay}\
         [ghidra]\nhome    = \"/opt/ghidra\"\nproject = \"{d}/ghidra\"\nname    = \"{name}\"\ntimeout = 7200\n\n\
         [build]\ntest = \"\"\n\n[git]\npush = false\n\n\
         [paths]\ncontext = \"{d}/context\"\njournal = \"{d}/journal\"\nplan    = \"{d}/plan\"\n\n\
         [oracle]\naddresses = \"{d}/oracle.txt\"\n\n[states]\n8 = \"scratch\"\n9 = \"scratch\"\n\n[symbols]\n",
        game = src.game,
        iso = iso.display(),
        boot = src.boot,
        name = src.name,
    )
}
