//! The ground's picture: an aerial image (NAIP) draped over the whole map as one texture, the way MX
//! Unleashed's freeride tracks lay their whole-map masks (`terrainscreenspacemesh.psm`, read by
//! game-render's `splat`).
//!
//! free09's `Free09_Sand_Mask` pass is the model: one texture, `UScale = 1/6912`, `VScale = −1/6912`
//! (6912 ft is its 768 samples at 9 ft), so the texture's first row is the map's far edge in z, which
//! here is north, the image's own first row. The pass here is that pass with blending off, as free09's
//! first pass has it, so the picture is the ground's colour.
//!
//! The `.pst` is game-render's `pst` layout: `0x00020002`, `0x28`, a 256-colour palette in the GS's
//! order, a mip count, then each mip's head (height, width, bytes, 0) and its 8-bit pixels. The image is
//! cut down to 256 colours by median cut.

/// An RGB picture, rows from the top.
#[derive(Clone, Debug, PartialEq)]
pub struct Rgb {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<[u8; 3]>,
}

/// The `.pst`'s first two words on every texture of MXU's disc (game-render `pst`).
const PST_MAGIC: [u32; 2] = [0x0002_0002, 0x28];

/// The GS's palette order: blocks of eight, the middle two of each group of four swapped. Its own inverse.
pub fn swizzle(i: usize) -> usize {
    (i & 0xE7) | ((i & 0x08) << 1) | ((i & 0x10) >> 1)
}

/// A palette of up to 256 colours for these pixels, by median cut: the box with the widest channel is
/// split at its median until there are 256 boxes (or no box can be split), each box's colour its mean.
pub fn median_cut(pixels: &[[u8; 3]]) -> Vec<[u8; 3]> {
    let mut boxes: Vec<Vec<[u8; 3]>> = vec![pixels.to_vec()];
    while boxes.len() < 256 {
        // The box with the widest single channel.
        let widest = boxes
            .iter()
            .enumerate()
            .filter(|(_, b)| b.len() > 1)
            .map(|(i, b)| {
                let (c, r) = widest_channel(b);
                (r, c, i)
            })
            .filter(|(r, _, _)| *r > 0)
            .max_by_key(|(r, _, i)| (*r, std::cmp::Reverse(*i)));
        let Some((_, c, i)) = widest else { break };
        let mut b = boxes.swap_remove(i);
        b.sort_unstable_by_key(|p| p[c]);
        let upper = b.split_off(b.len() / 2);
        boxes.push(b);
        boxes.push(upper);
    }
    boxes
        .iter()
        .filter(|b| !b.is_empty())
        .map(|b| {
            let mut s = [0u64; 3];
            for p in b {
                for k in 0..3 {
                    s[k] += u64::from(p[k]);
                }
            }
            let n = b.len() as u64;
            s.map(|v| ((v + n / 2) / n) as u8)
        })
        .collect()
}

fn widest_channel(b: &[[u8; 3]]) -> (usize, u8) {
    (0..3)
        .map(|c| {
            let (lo, hi) = b
                .iter()
                .fold((255u8, 0u8), |(lo, hi), p| (lo.min(p[c]), hi.max(p[c])));
            (c, hi - lo)
        })
        .max_by_key(|(c, r)| (*r, std::cmp::Reverse(*c)))
        .unwrap()
}

/// The nearest palette entry to each pixel (squared RGB distance, the lower index on a tie).
pub fn index(pixels: &[[u8; 3]], palette: &[[u8; 3]]) -> Vec<u8> {
    let mut cache = std::collections::HashMap::new();
    pixels
        .iter()
        .map(|p| {
            *cache.entry(*p).or_insert_with(|| {
                let d = |q: &[u8; 3]| -> i32 {
                    (0..3)
                        .map(|k| (i32::from(p[k]) - i32::from(q[k])).pow(2))
                        .sum()
                };
                (0..palette.len())
                    .min_by_key(|&i| (d(&palette[i]), i))
                    .unwrap() as u8
            })
        })
        .collect()
}

/// An 8-bit `.pst` of one mip: the picture through its median-cut palette, alpha 0x80 (opaque on the GS).
pub fn pst(img: &Rgb) -> Vec<u8> {
    let palette = median_cut(&img.pixels);
    let pixels = index(&img.pixels, &palette);
    let mut d = Vec::with_capacity(1052 + pixels.len());
    for w in PST_MAGIC {
        d.extend_from_slice(&w.to_le_bytes());
    }
    let mut pal = [0u8; 1024];
    for (i, c) in palette.iter().enumerate() {
        let at = 4 * swizzle(i);
        pal[at..at + 4].copy_from_slice(&[c[0], c[1], c[2], 0x80]);
    }
    d.extend_from_slice(&pal);
    for w in [1, img.height, img.width, img.width * img.height, 0] {
        d.extend_from_slice(&w.to_le_bytes());
    }
    d.extend_from_slice(&pixels);
    d
}

/// The `.psm`: one pass laying `texture` (a `.pst` stem) once over a map `extent` feet across (x) and
/// along (z), its first row at the far z edge. Keys and spelling are free09's `ShaderPass_1`, CRLF.
pub fn psm(texture: &str, extent: (f32, f32)) -> String {
    let lines = [
        "[General] ".to_owned(),
        "TotalShaderPasses             = 1".to_owned(),
        String::new(),
        "[ShaderPass_1]".to_owned(),
        "microProgramName    = Vu1StripConstantColorTexTransformUncompressed.vuo".to_owned(),
        format!("TextureMap_1        = {texture}"),
        "ShaderRGBA          = 128.0, 128.0, 128.0, 128.0".to_owned(),
        String::new(),
        "lodCalculationMethod= 0".to_owned(),
        "MipMap_1            = 2".to_owned(),
        "MagFilter           = 1".to_owned(),
        "MinFilter           = 5".to_owned(),
        "K                   = -65.0".to_owned(),
        String::new(),
        "inputColorA         = 0".to_owned(),
        "inputColorB         = 1".to_owned(),
        "inputAlphaC         = 0".to_owned(),
        "inputColorD         = 1".to_owned(),
        "fixedAlphaValue     =".to_owned(),
        String::new(),
        "alphaBlendingOnOff  = 0 ".to_owned(),
        "alphaTestOnOff      = 0".to_owned(),
        "foggingOnOff        = 0".to_owned(),
        "zWriteDisabled      = 1".to_owned(),
        "depthTestMethod     = 1".to_owned(),
        "wrapModeHorizontal  = 0".to_owned(),
        "wrapModeVertical    = 0".to_owned(),
        String::new(),
        "VertexAnimation     =  unique".to_owned(),
        "TexGenMatrixRow1    =  1.0, 0.0, 0.0, 0.0".to_owned(),
        "TexGenMatrixRow2    =  0.0, 1.0, 0.0, 0.0".to_owned(),
        "TexGenMatrixRow3    =  0.0, 0.0, 1.0, 0.0".to_owned(),
        "TexGenMatrixRow4    =  0.0, 0.0, 0.0, 1.0".to_owned(),
        String::new(),
        "NumAnimators = 1".to_owned(),
        "[ShaderPass_1_Animator_1]".to_owned(),
        "Algorithm           = UVAnimation".to_owned(),
        format!("UScale              = {:.10}", 1.0 / extent.0),
        format!("VScale              = {:.10}", -1.0 / extent.1),
        String::new(),
    ];
    lines.join("\r\n")
}

/// Bilinear sample of a band-interleaved picture at pixel coordinates (centres at whole numbers), or
/// None where any of the four is missing (NaN) or outside.
pub fn bilinear(px: &[f32], w: u32, h: u32, bands: usize, fi: f64, fj: f64) -> Option<[f32; 3]> {
    let (i0, j0) = (fi.floor(), fj.floor());
    let (fx, fy) = ((fi - i0) as f32, (fj - j0) as f32);
    let (i0, j0) = (i0 as i64, j0 as i64);
    if i0 < 0 || j0 < 0 || i0 + 1 >= i64::from(w) || j0 + 1 >= i64::from(h) {
        return None;
    }
    let at = |i: i64, j: i64, k: usize| px[(i + j * i64::from(w)) as usize * bands + k];
    let mut out = [0f32; 3];
    for (k, o) in out.iter_mut().enumerate() {
        let v = (1.0 - fx) * (1.0 - fy) * at(i0, j0, k)
            + fx * (1.0 - fy) * at(i0 + 1, j0, k)
            + (1.0 - fx) * fy * at(i0, j0 + 1, k)
            + fx * fy * at(i0 + 1, j0 + 1, k);
        if !v.is_finite() {
            return None;
        }
        *o = v;
    }
    Some(out)
}
