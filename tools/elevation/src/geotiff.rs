//! GeoTIFF: a TIFF (the `tiff` crate decodes strips, tiles, LZW, deflate and the predictors) plus the
//! tags that place it on the ground.
//!
//! ```text
//! 33550  ModelPixelScale   f64 × 3   metres (or feet) a pixel across and down, and 0
//! 33922  ModelTiepoint     f64 × 6   pixel (i, j, k) is at model (x, y, z): the image's corner at (0, 0)
//! 34735  GeoKeyDirectory   u16 × 4n  1, 1, 0, n, then n keys: id, where (0 = the value itself), count, value
//! 42113  GDAL_NODATA       ASCII     the value that means "no data"
//! ```
//!
//! The keys read: 1024 the model (1 projected, 2 geographic), 1025 the raster type (1 the tiepoint is a
//! pixel's corner, 2 its centre), 3072 the projected EPSG code, 2048 the geographic one, 3076 and 2052 the
//! linear units (9001 metre, 9002 foot, 9003 US survey foot), 4096 the vertical datum and 4099 its units.
//!
//! USGS 3DEP's 1 m tiles (checked on `USGS_1M_11_x67y366_CA_SaltonSea_EarthMRI_2021_D21.tif`) are one
//! band of f32, 512×512 tiles, LZW with the floating-point predictor, EPSG 26911, metres, the tiepoint at
//! the corner, nodata −999999, and no vertical keys. NAIP's are four bands of u8 (R, G, B, near infrared).

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tiff::decoder::{ChunkType, Decoder, DecodingResult, Limits};
use tiff::tags::Tag;

pub const PIXEL_SCALE: u16 = 33550;
pub const TIEPOINT: u16 = 33922;
pub const GEO_KEYS: u16 = 34735;
pub const GDAL_NODATA: u16 = 42113;

/// Linear unit codes (EPSG).
pub const METRE: u16 = 9001;
pub const FOOT: u16 = 9002;
pub const US_FOOT: u16 = 9003;

/// Where a raster lies and what its numbers mean, without its pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct Geo {
    pub width: u32,
    pub height: u32,
    pub bands: u32,
    /// The model coordinate of the image's outer corner: the left edge of column 0 and the top edge of
    /// row 0 (a centre tiepoint is moved there by half a pixel).
    pub left: f64,
    pub top: f64,
    /// Units a pixel across and down (both positive; rows run down, towards smaller y).
    pub dx: f64,
    pub dy: f64,
    /// 1 projected, 2 geographic, 0 not given.
    pub model: u16,
    /// The projected (3072) or, failing that, the geographic (2048) EPSG code.
    pub epsg: Option<u16>,
    /// Horizontal units (3076, or 2052 for a geographic model's linear unit); None if not given.
    pub linear_units: Option<u16>,
    /// Vertical units (4099); None if not given.
    pub vertical_units: Option<u16>,
    pub vertical_datum: Option<u16>,
    pub nodata: Option<f64>,
}

impl Geo {
    /// The model x of column `i`'s centre and the y of row `j`'s.
    pub fn x(&self, i: f64) -> f64 {
        self.left + (i + 0.5) * self.dx
    }
    pub fn y(&self, j: f64) -> f64 {
        self.top - (j + 0.5) * self.dy
    }
}

/// An open GeoTIFF: its geography, and the decoder to read pixels from.
pub struct GeoTiff {
    pub geo: Geo,
    decoder: Decoder<BufReader<File>>,
}

fn err(path: &Path, e: impl std::fmt::Display) -> String {
    format!("{}: {e}", path.display())
}

impl GeoTiff {
    pub fn open(path: &Path) -> Result<GeoTiff, String> {
        let f = File::open(path).map_err(|e| err(path, e))?;
        let mut d = Decoder::new(BufReader::new(f))
            .map_err(|e| err(path, e))?
            .with_limits(Limits::unlimited());
        let (width, height) = d.dimensions().map_err(|e| err(path, e))?;
        let bands = d
            .find_tag_unsigned::<u32>(Tag::SamplesPerPixel)
            .map_err(|e| err(path, e))?
            .unwrap_or(1);
        let scale = d
            .find_tag(Tag::Unknown(PIXEL_SCALE))
            .map_err(|e| err(path, e))?
            .ok_or_else(|| err(path, "no ModelPixelScale tag: not a GeoTIFF"))?
            .into_f64_vec()
            .map_err(|e| err(path, e))?;
        let tie = d
            .find_tag(Tag::Unknown(TIEPOINT))
            .map_err(|e| err(path, e))?
            .ok_or_else(|| err(path, "no ModelTiepoint tag: not a GeoTIFF"))?
            .into_f64_vec()
            .map_err(|e| err(path, e))?;
        if scale.len() < 2 || tie.len() < 6 {
            return Err(err(path, "short ModelPixelScale or ModelTiepoint"));
        }
        if !(scale[0] > 0.0 && scale[1] > 0.0) {
            return Err(err(path, format!("pixel scale {scale:?}")));
        }
        let keys = match d
            .find_tag(Tag::Unknown(GEO_KEYS))
            .map_err(|e| err(path, e))?
        {
            Some(v) => v.into_u16_vec().map_err(|e| err(path, e))?,
            None => Vec::new(),
        };
        let key = |id: u16| -> Option<u16> {
            // Header of four, then id, location, count, value; only keys held in place are numbers.
            keys.get(4..)?
                .chunks_exact(4)
                .find(|k| k[0] == id && k[1] == 0)
                .map(|k| k[3])
        };
        let nodata = match d
            .find_tag(Tag::Unknown(GDAL_NODATA))
            .map_err(|e| err(path, e))?
        {
            Some(v) => {
                let s = v.into_string().map_err(|e| err(path, e))?;
                let s = s.trim_matches(|c: char| c == '\0' || c.is_whitespace());
                Some(
                    s.parse::<f64>()
                        .map_err(|e| err(path, format!("GDAL_NODATA {s:?}: {e}")))?,
                )
            }
            None => None,
        };
        let (dx, dy) = (scale[0], scale[1]);
        // The tiepoint names pixel (i, j); with raster type 2 it is the pixel's centre.
        let centre = key(1025) == Some(2);
        let shift = if centre { 0.5 } else { 0.0 };
        let left = tie[3] - (tie[0] + shift) * dx;
        let top = tie[4] + (tie[1] + shift) * dy;
        let geo = Geo {
            width,
            height,
            bands,
            left,
            top,
            dx,
            dy,
            model: key(1024).unwrap_or(0),
            epsg: key(3072)
                .filter(|c| *c != 32767)
                .or(key(2048).filter(|c| *c != 32767)),
            linear_units: key(3076).or(key(2052)),
            vertical_units: key(4099),
            vertical_datum: key(4096),
            nodata,
        };
        Ok(GeoTiff { geo, decoder: d })
    }

    /// The pixels of a window, `w × h` from column `i0` and row `j0`, band-interleaved as the file has
    /// them (`(i + w j) × bands + band`), as f32. Only the strips or tiles under the window are read.
    pub fn window(&mut self, i0: u32, j0: u32, w: u32, h: u32) -> Result<Vec<f32>, String> {
        let g = &self.geo;
        if i0 + w > g.width || j0 + h > g.height {
            return Err(format!(
                "window {w}×{h} at ({i0}, {j0}) is outside the {}×{} image",
                g.width, g.height
            ));
        }
        let bands = g.bands as usize;
        let (cw, ch) = self.decoder.chunk_dimensions();
        let across = match self.decoder.get_chunk_type() {
            ChunkType::Tile => g.width.div_ceil(cw),
            ChunkType::Strip => 1,
        };
        let mut out = vec![0f32; w as usize * h as usize * bands];
        for cj in j0 / ch..(j0 + h).div_ceil(ch) {
            for ci in i0 / cw..(i0 + w).div_ceil(cw).min(across) {
                let index = cj * across + ci;
                let (dw, dh) = self.decoder.chunk_data_dimensions(index);
                let data = to_f32(self.decoder.read_chunk(index).map_err(|e| e.to_string())?)?;
                // A tile's buffer is the full tile width; a strip's the image's.
                let stride = data.len() / (dh as usize * bands).max(1);
                let (x0, y0) = (ci * cw, cj * ch);
                for y in y0.max(j0)..(y0 + dh).min(j0 + h) {
                    for x in x0.max(i0)..(x0 + dw).min(i0 + w) {
                        let from = ((y - y0) as usize * stride + (x - x0) as usize) * bands;
                        let to = ((y - j0) as usize * w as usize + (x - i0) as usize) * bands;
                        out[to..to + bands].copy_from_slice(&data[from..from + bands]);
                    }
                }
            }
        }
        Ok(out)
    }
}

fn to_f32(r: DecodingResult) -> Result<Vec<f32>, String> {
    Ok(match r {
        DecodingResult::U8(v) => v.into_iter().map(f32::from).collect(),
        DecodingResult::U16(v) => v.into_iter().map(f32::from).collect(),
        DecodingResult::I8(v) => v.into_iter().map(f32::from).collect(),
        DecodingResult::I16(v) => v.into_iter().map(f32::from).collect(),
        DecodingResult::I32(v) => v.into_iter().map(|x| x as f32).collect(),
        DecodingResult::U32(v) => v.into_iter().map(|x| x as f32).collect(),
        DecodingResult::F32(v) => v,
        DecodingResult::F64(v) => v.into_iter().map(|x| x as f32).collect(),
        _ => {
            return Err(
                "a sample type this tool does not take (16- or 64-bit integers wider than f32)"
                    .into(),
            );
        }
    })
}
