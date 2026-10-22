//! Several GeoTIFFs on one pixel grid, read as one: USGS 3DEP's 1 m tiles are 10 km squares with six
//! pixels of overlap on each side, all on the same UTM grid, so a map that crosses a tile edge is cut out
//! of the mosaic without moving any pixel.

use crate::geotiff::{Geo, GeoTiff};
use std::path::Path;

/// Rasters that share a grid: the same EPSG code, the same pixel size, and corners whole pixels apart.
pub struct Mosaic {
    pub rasters: Vec<GeoTiff>,
    /// The first raster's geography; every other one is placed against it.
    pub geo: Geo,
    /// Each raster's corner in the first one's pixels.
    pub at: Vec<(i64, i64)>,
}

/// What reading a window found where two rasters overlap.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Overlap {
    /// Pixels inside the window that more than one raster has data for.
    pub pixels: u64,
    /// Of those, the ones where a later raster's value is not the first one's (the first is kept).
    pub differ: u64,
    pub largest: f32,
}

/// How far from a whole pixel two corners may be and still count as one grid: 3DEP's corners are a
/// quarter of a millimetre off the metre (669993.999734), the same on every tile of a project.
const ALIGN: f64 = 1e-3;

impl Mosaic {
    pub fn open(paths: &[impl AsRef<Path>]) -> Result<Mosaic, String> {
        let mut rasters = Vec::new();
        for p in paths {
            rasters.push(GeoTiff::open(p.as_ref())?);
        }
        Mosaic::new(rasters)
    }

    pub fn new(rasters: Vec<GeoTiff>) -> Result<Mosaic, String> {
        let first = rasters.first().ok_or("no rasters")?.geo.clone();
        let mut at = Vec::new();
        for (n, r) in rasters.iter().enumerate() {
            let g = &r.geo;
            if g.epsg != first.epsg
                || g.dx != first.dx
                || g.dy != first.dy
                || g.bands != first.bands
            {
                return Err(format!(
                    "raster {n} is EPSG {:?} at {}×{} with {} bands, the first is EPSG {:?} at {}×{} with {}: \
                     not one grid (joining them would resample)",
                    g.epsg, g.dx, g.dy, g.bands, first.epsg, first.dx, first.dy, first.bands
                ));
            }
            let (fi, fj) = (
                (g.left - first.left) / first.dx,
                (first.top - g.top) / first.dy,
            );
            let (i, j) = (fi.round(), fj.round());
            if (fi - i).abs() > ALIGN || (fj - j).abs() > ALIGN {
                return Err(format!(
                    "raster {n}'s corner is {fi}, {fj} pixels from the first's: not on the same grid"
                ));
            }
            at.push((i as i64, j as i64));
        }
        Ok(Mosaic {
            rasters,
            geo: first,
            at,
        })
    }

    /// The pixel (column, row), in the first raster's frame, whose area holds model point (x, y).
    pub fn pixel_of(&self, x: f64, y: f64) -> (i64, i64) {
        (
            ((x - self.geo.left) / self.geo.dx).floor() as i64,
            ((self.geo.top - y) / self.geo.dy).floor() as i64,
        )
    }

    /// `w × h` pixels from column `i0` and row `j0` (first raster's frame), band-interleaved. Each pixel
    /// is the first raster's that has data there. Pixels nobody has are NaN.
    pub fn window(
        &mut self,
        i0: i64,
        j0: i64,
        w: u32,
        h: u32,
    ) -> Result<(Vec<f32>, Overlap), String> {
        let bands = self.geo.bands as usize;
        let mut out = vec![f32::NAN; w as usize * h as usize * bands];
        let mut overlap = Overlap::default();
        for (r, &(ri, rj)) in self.rasters.iter_mut().zip(&self.at) {
            let g = r.geo.clone();
            // The window in this raster's own pixels, clipped to it.
            let (a0, b0) = ((i0 - ri).max(0), (j0 - rj).max(0));
            let a1 = (i0 + i64::from(w) - ri).min(i64::from(g.width));
            let b1 = (j0 + i64::from(h) - rj).min(i64::from(g.height));
            if a0 >= a1 || b0 >= b1 {
                continue;
            }
            let (cw, ch) = ((a1 - a0) as u32, (b1 - b0) as u32);
            let px = r.window(a0 as u32, b0 as u32, cw, ch)?;
            for b in 0..ch as i64 {
                for a in 0..cw as i64 {
                    let from = (a + b * i64::from(cw)) as usize * bands;
                    let v = &px[from..from + bands];
                    if !has_data(v, g.nodata) {
                        continue;
                    }
                    let (oi, oj) = (a0 + a + ri - i0, b0 + b + rj - j0);
                    let to = (oi + oj * i64::from(w)) as usize * bands;
                    if out[to].is_nan() {
                        out[to..to + bands].copy_from_slice(v);
                    } else {
                        overlap.pixels += 1;
                        let d = out[to..to + bands]
                            .iter()
                            .zip(v)
                            .map(|(p, q)| (p - q).abs())
                            .fold(0.0, f32::max);
                        if d > 0.0 {
                            overlap.differ += 1;
                            overlap.largest = overlap.largest.max(d);
                        }
                    }
                }
            }
        }
        Ok((out, overlap))
    }
}

fn has_data(v: &[f32], nodata: Option<f64>) -> bool {
    v.iter().all(|x| x.is_finite())
        && nodata.is_none_or(|n| f64::from(v[0]) != n && f64::from(v[0]) != (n as f32) as f64)
}
