//! An EE RAM image of the source game in a ride, and finding things in it.
//!
//! The image comes from a save state (`./kit state ram`) or straight over PINE (`./kit pine block 0
//! 0x2000000`); either is the running game's memory, code and data, at one frame.

use crate::source::{NODE_SCALES, Source, Viewport};

/// 32 MB, a retail console's EE RAM.
pub const RAM_BYTES: usize = 32 << 20;

pub fn u32_at(ram: &[u8], at: u32) -> Option<u32> {
    let at = at as usize;
    ram.get(at..at + 4).map(|w| u32::from_le_bytes(w.try_into().expect("4 bytes")))
}

pub fn f32_at(ram: &[u8], at: u32) -> Option<f32> {
    u32_at(ram, at).map(f32::from_bits)
}

pub fn quad(ram: &[u8], at: u32) -> Option<[f32; 4]> {
    Some([f32_at(ram, at)?, f32_at(ram, at + 4)?, f32_at(ram, at + 8)?, f32_at(ram, at + 12)?])
}

/// The terrain object of the map in the image, found by what it points at: a word holding a node whose
/// four leading floats are the file's root node's (`root`, the first 16 bytes of the `.trn`'s tree),
/// with the file's scale beside it at the object's height-scale field.
///
/// Every match is returned. More than one means two objects share the tree; none means the image is not
/// of this map (or not of a ride at all).
pub fn find_terrain(ram: &[u8], src: &Source, root: &[u8; 16], scale_bits: u32) -> Vec<u32> {
    let o = src.object;
    let mut found = Vec::new();
    let words = ram.len() / 4;
    for k in 0..words {
        let at = (k * 4) as u32;
        if at < o.tree {
            continue;
        }
        let Some(node) = u32_at(ram, at) else { continue };
        let n = node as usize + NODE_SCALES as usize;
        if node == 0 || n + 16 > ram.len() || ram[n..n + 16] != root[..] {
            continue;
        }
        let object = at - o.tree;
        if u32_at(ram, object + o.height_scale) == Some(scale_bits) {
            found.push(object);
        }
    }
    found
}

/// Every viewport-shaped thing in the image: an eye with `w = 1` kept twice, unit forward and up
/// vectors, and a field of view between 1 and 179 degrees. The camera that draws the ride is the one
/// whose eye is over the terrain; [`pick_viewport`] chooses it.
pub fn find_viewports(ram: &[u8], vp: &Viewport) -> Vec<u32> {
    let mut found = Vec::new();
    for k in 0..ram.len() / 16 {
        let at = (k * 16) as u32;
        let Some(eye) = quad(ram, at + vp.eye) else { break };
        if eye[3] != 1.0 || eye[..3].iter().all(|&c| c == 0.0) || !eye.iter().all(|c| c.is_finite()) {
            continue;
        }
        let (e, c) = ((at + vp.eye) as usize, (at + vp.eye_copy) as usize);
        if c + 16 > ram.len() || ram[e..e + 16] != ram[c..c + 16] {
            continue;
        }
        let unit = |off: u32| {
            quad(ram, at + off).is_some_and(|v| ((v[0] * v[0] + v[1] * v[1] + v[2] * v[2]) - 1.0).abs() < 1e-3)
        };
        let fov = f32_at(ram, at + vp.fov).unwrap_or(0.0);
        if unit(vp.forward) && unit(vp.up) && (1.0..179.0).contains(&fov) {
            found.push(at);
        }
    }
    found
}

/// Of [`find_viewports`]' finds, the one whose eye is inside the box `[min, max]` (x and z), if exactly
/// one is.
pub fn pick_viewport(ram: &[u8], vp: &Viewport, found: &[u32], min: [f32; 4], max: [f32; 4]) -> Result<u32, String> {
    let inside: Vec<u32> = found
        .iter()
        .copied()
        .filter(|&at| {
            quad(ram, at + vp.eye)
                .is_some_and(|e| e[0] >= min[0] && e[0] <= max[0] && e[2] >= min[2] && e[2] <= max[2])
        })
        .collect();
    match inside.as_slice() {
        [one] => Ok(*one),
        [] => Err(format!("no viewport's eye is over the terrain ({} viewports found)", found.len())),
        many => Err(format!("{} viewports have their eye over the terrain: {many:x?}", many.len())),
    }
}
