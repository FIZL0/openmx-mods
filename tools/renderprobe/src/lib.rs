//! A map's render-side files through openmx game-render's readers (M9).
//!
//! ```text
//! <map>.res      .scn  .psg  .cub  .esb  .toy  .ps2.trn (its texture pages)
//! <map>_mat.res  .psm  .pst
//! the PSR        Geo\  Tex\  Skies\  Eco\  …   loose, where the map's archives do not hold a name
//! ```
//!
//! game-render opens a disc only as `MX.PSR` in an ISO root, with its directory in the first 64 KiB,
//! and makes a [`game_render::Res`] only from there. MX vs. ATV's PSR has another name and a 20k-member
//! directory, so its loaders (`Assets::load`, `Sky::load`, `Grove::load`, `Splat::load`), which take
//! `&[Res]`, cannot be handed one. [`render_res`] gets one anyway without a second reader: it puts the
//! `.res` bytes, unchanged, as the one member of a PSR in a minimal ISO in memory, and opens that with
//! game-render's own `Disc::read_from` and `Disc::res`.
//!
//! The terrain's pages are not `.pst` files, but a `.pst` is the same record behind a palette:
//! [`page_pst`] puts the terrain's own palette in front of a page so `Texture::parse` decodes it.

use game_render::disc::Disc as RenderDisc;
use std::io::Cursor;

const SECTOR: usize = 2048;

/// A game-render `Res` over these `.res` bytes, through game-render's own disc path.
pub fn render_res(bytes: Vec<u8>) -> Result<game_render::Res, String> {
    let mut disc = RenderDisc::read_from(Box::new(Cursor::new(image(&bytes))))?;
    disc.res(MEMBER)
}

const MEMBER: &str = "map.res";

/// A minimal ISO 9660 image: a primary volume descriptor at sector 16, a root directory at 17 naming
/// `MX.PSR`, and at 18 a version-2 PSR whose one member, at its sector 1, is `payload`.
pub fn image(payload: &[u8]) -> Vec<u8> {
    let psr_sector = 18usize;
    let mut psr = Vec::new();
    let name = MEMBER.as_bytes();
    let dir_bytes = 12 + name.len();
    for w in [2u32, 1, 0, 0, dir_bytes as u32, 0] {
        psr.extend_from_slice(&w.to_le_bytes());
    }
    psr.extend_from_slice(&(name.len() as u32).to_le_bytes());
    psr.extend_from_slice(name);
    psr.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    psr.extend_from_slice(&1u32.to_le_bytes());
    psr.resize(SECTOR, 0);
    psr.extend_from_slice(payload);

    let mut img = vec![0u8; psr_sector * SECTOR];
    let pvd = &mut img[16 * SECTOR..17 * SECTOR];
    pvd[0] = 1;
    pvd[1..6].copy_from_slice(b"CD001");
    pvd[156] = 34;
    pvd[156 + 2..156 + 6].copy_from_slice(&17u32.to_le_bytes());
    pvd[156 + 10..156 + 14].copy_from_slice(&(SECTOR as u32).to_le_bytes());
    let file = b"MX.PSR;1";
    let rec = &mut img[17 * SECTOR..17 * SECTOR + 34 + file.len()];
    rec[0] = (33 + file.len() + 1) as u8;
    rec[2..6].copy_from_slice(&(psr_sector as u32).to_le_bytes());
    rec[10..14].copy_from_slice(&(psr.len() as u32).to_le_bytes());
    rec[32] = file.len() as u8;
    rec[33..33 + file.len()].copy_from_slice(file);
    img.extend_from_slice(&psr);
    img
}

/// A `.pst` made of a terrain's 1024-byte palette and one of its texture pages: the `.pst` magic
/// (`0x00020002`, `0x28`), the palette, then the page from its mip count on. The page's own first
/// word, its format (`0x27`), is the one thing left out.
pub fn page_pst(palette: &[u8], page: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(8 + palette.len() + page.len());
    v.extend_from_slice(&0x0002_0002u32.to_le_bytes());
    v.extend_from_slice(&0x28u32.to_le_bytes());
    v.extend_from_slice(palette);
    v.extend_from_slice(&page[4..]);
    v
}

/// The last path segment of a member name, lower-cased: how the loose files are looked up.
pub fn base(name: &str) -> String {
    name.rsplit(['\\', '/'])
        .next()
        .unwrap_or(name)
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_image_hands_game_render_the_payload_unchanged() {
        let payload: Vec<u8> = (0..5000u32).map(|i| (i * 7) as u8).collect();
        let mut disc = RenderDisc::read_from(Box::new(Cursor::new(image(&payload)))).unwrap();
        assert_eq!(disc.member(MEMBER).unwrap(), payload);
    }

    #[test]
    fn a_page_decodes_as_a_pst_through_its_palette() {
        let mut palette = vec![0u8; 1024];
        // Entry 3 is stored at slot 3 (the swizzle leaves 0..8 where they are): red, opaque.
        palette[12..16].copy_from_slice(&[255, 0, 0, 0x80]);
        let mut page = Vec::new();
        for w in [0x27u32, 1, 2, 2, 4, 0] {
            page.extend_from_slice(&w.to_le_bytes());
        }
        page.extend_from_slice(&[3, 0, 0, 3]);
        let t = game_render::Texture::parse(&page_pst(&palette, &page)).unwrap();
        assert_eq!((t.width, t.height), (2, 2));
        assert_eq!(&t.rgba[0..4], &[255, 0, 0, 255]);
        assert_eq!(&t.rgba[12..16], &[255, 0, 0, 255]);
    }
}
