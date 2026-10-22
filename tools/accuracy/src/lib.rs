//! The accuracy harness (T7): a track mod against its source game running in PCSX2.
//!
//! - [`heights`]: the ground at a thousand points and more, the original's own query on a RAM image of
//!   the game in a ride against openmx's port on the mod's file — bit for bit, since the terrain is
//!   copied.
//! - [`look`]: screenshot pairs from the same camera poses, the original's (`./kit shot`, the pose read
//!   out of its viewport over PINE) and openmx's (`--shot-pose`, openmx MD10), side by side.
//!
//! - [`sceneram`] and [`loaded`] (M12): the scene's fog, environment and start as the game holds them,
//!   and the files it opened for the map, against the mod.
//!
//! [`source`] says where each game keeps the things this reads; `notes/accuracy.md` how they were found
//! and how to run it.

pub mod heights;
pub mod landmarks;
pub mod loaded;
pub mod look;
pub mod modfiles;
pub mod pose;
pub mod ram;
pub mod sceneram;
pub mod source;

use std::fmt::Write as _;

fn fields(r: &heights::Report) -> String {
    let d: Vec<String> = heights::FIELDS
        .iter()
        .filter(|f| r.differ(f) > 0)
        .map(|f| format!("{f} at {}", r.differ(f)))
        .collect();
    if d.is_empty() { "every field equal".into() } else { format!("{} differ", d.join(", ")) }
}

fn section(s: &mut String, title: &str, r: &heights::Report) {
    let _ = writeln!(s, "{title}: {} points, {} with a difference: {}", r.total_points(), r.total_mismatches(), fields(r));
    for (mode, t) in &r.modes {
        let _ = writeln!(s, "  {mode}: {} points, {} on a cell, {} ride queries, {} differ", t.points, t.cells, t.rides, t.mismatches);
    }
    for x in &r.mismatches {
        let _ = writeln!(s, "  at {:?} ({}): {}: original {}, port {}", x.at, x.mode, x.what, x.original, x.port);
    }
}

/// The heights report as text. The first line is the verdict: the heights (the query's and the ride
/// query's `y`) at every point, then what else differs.
pub fn heights_text(
    m: &modfiles::TrackMod,
    src: &source::Source,
    terrain: u32,
    checks: &[(String, bool)],
    as_is: &heights::Report,
    with_step: Option<&heights::Report>,
) -> String {
    let mut s = String::new();
    let heights = as_is.differ("height") + as_is.differ("ride height");
    let _ = writeln!(
        s,
        "{}: heights {} at all {} points ({} modes); openmx as it is: {}{}",
        m.id,
        if heights == 0 { "equal" } else { "DIFFER" },
        as_is.total_points(),
        as_is.modes.len(),
        fields(as_is),
        with_step.map_or(String::new(), |r| format!("; with {}'s normal step: {}", src.game, fields(r))),
    );
    let _ = writeln!(s, "source {} ({}), terrain object at {terrain:#x}, query {:#x}, ride query {}", src.game, src.boot, src.query, src.ride_query.map_or("none".into(), |a| format!("{a:#x}")));
    for (c, ok) in checks {
        let _ = writeln!(s, "  {} {c}", if *ok { "same" } else { "DIFFERS" });
    }
    section(&mut s, "openmx as it is (contact_search::ground, nearest_surface)", as_is);
    if let Some(r) = with_step {
        section(&mut s, &format!("openmx with {}'s cell reader normals (over 127)", src.game), r);
    }
    s
}
