//! BPB: MX vs. ATV Untamed's binary INI. Every text INI of the earlier games (`.scn`, `.toy`, `.phy`,
//! `.psm`, ...) is on Untamed's disc as `<name>.bpb`, the same sections and keys compiled to a table.
//!
//! ```text
//! header   0x00 "BPB\0"
//!          0x04 u32 offset of the section records (always 0x20)
//!          0x08 u32 offset of the key records
//!          0x0c u16 sections   0x0e u16 keys
//!          0x10 u32 offset of the name table      0x14 u32 its length
//!          0x18 u32 offset of the value strings   0x1c u32 their length
//! section  16 bytes: u32 hash, u32 name (offset in the name table), u16 index (0xffff: none),
//!          u16 keys, u32 first key
//! key      16 bytes: u32 hash, u32 name, u16 type, u16 0, then 4 bytes by type:
//!          1 a string, u32 offset in the value strings;  2 a string of up to five bytes, held in the
//!          record's last six (NUL-padded);  4 an i32;  8 an f32
//! ```
//!
//! A section's text name is its name followed by its index when it has one: `Model` index 17 is
//! `[Model17]`. The hash is djb2 (`h = h * 33 + c` from 5381) over the name with ASCII letters lowered,
//! so `HasCOL` and `HasCol` (both on the disc) hash alike. The name table holds each distinct spelling
//! once, in first-use order, NUL-terminated, padded with zeros to 16 bytes; the value strings likewise,
//! and the file is padded to 16. Both are as SLUS_217.01's reader takes them (notes/mxvatv-later.md).

pub const MAGIC: &[u8; 4] = b"BPB\0";
const HEADER: usize = 0x20;
const RECORD: usize = 16;
/// The bytes a type-2 value has in its record, its NUL included when shorter.
pub const INLINE: usize = 6;

/// The name hash: djb2 over the name with ASCII letters lowered.
pub fn hash(name: &[u8]) -> u32 {
    name.iter().fold(5381u32, |h, &c| {
        h.wrapping_mul(33).wrapping_add(u32::from(c.to_ascii_lowercase()))
    })
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// Type 1: a string in the value strings.
    Text(Vec<u8>),
    /// Type 2: a string of at most five bytes, inside the record.
    Short(Vec<u8>),
    /// Type 4.
    Int(i32),
    /// Type 8.
    Float(f32),
}

impl Value {
    pub fn kind(&self) -> u16 {
        match self {
            Value::Text(_) => 1,
            Value::Short(_) => 2,
            Value::Int(_) => 4,
            Value::Float(_) => 8,
        }
    }

    /// The value as INI text. A float is `%.06f`, as the game's own get-as-text prints one and as the
    /// earlier games' text files hold them, when that reads back to the same bits both through the
    /// game's text reader (`strtod`, then a truncating double-to-single) and through a round-to-nearest
    /// one. Otherwise it is the shortest decimal of the float widened to a double, which is exact under
    /// any of them: the parse gives that double, and the double is the float itself.
    pub fn text(&self) -> Vec<u8> {
        match self {
            Value::Text(s) | Value::Short(s) => s.clone(),
            Value::Int(i) => i.to_string().into_bytes(),
            Value::Float(f) => float_text(*f).into_bytes(),
        }
    }
}

/// See [`Value::text`].
pub fn float_text(f: f32) -> String {
    let six = format!("{f:.6}");
    if six.parse::<f64>().is_ok_and(|d| reads_back(d, f)) {
        return six;
    }
    let mut s = format!("{}", f64::from(f));
    if f.is_finite() && !s.contains(['.', 'e']) {
        s.push_str(".0");
    }
    s
}

/// Whether the double `d` gives the float `f` both rounded to nearest and truncated toward zero.
fn reads_back(d: f64, f: f32) -> bool {
    let nearest = d as f32;
    let truncated = if f64::from(nearest).abs() > d.abs() {
        f32::from_bits(nearest.to_bits() - 1)
    } else {
        nearest
    };
    nearest.to_bits() == f.to_bits() && truncated.to_bits() == f.to_bits()
}

#[derive(Clone, Debug, PartialEq)]
pub struct Key {
    pub hash: u32,
    pub name: Vec<u8>,
    pub value: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Section {
    pub hash: u32,
    pub name: Vec<u8>,
    pub index: Option<u16>,
    pub keys: Vec<Key>,
}

impl Section {
    /// `[Model17]`'s `Model17`: the game's `%s%d`, the index read as a signed 16-bit number (the disc's
    /// largest is 5713, so the sign never shows).
    pub fn title(&self) -> Vec<u8> {
        let mut t = self.name.clone();
        if let Some(i) = self.index {
            t.extend_from_slice((i as i16).to_string().as_bytes());
        }
        t
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bpb {
    pub sections: Vec<Section>,
}

fn u16_at(b: &[u8], at: usize) -> Result<u16, String> {
    b.get(at..at + 2)
        .map(|s| u16::from_le_bytes(s.try_into().unwrap()))
        .ok_or_else(|| format!("{at:#x}: past the end ({} bytes)", b.len()))
}

fn u32_at(b: &[u8], at: usize) -> Result<u32, String> {
    b.get(at..at + 4)
        .map(|s| u32::from_le_bytes(s.try_into().unwrap()))
        .ok_or_else(|| format!("{at:#x}: past the end ({} bytes)", b.len()))
}

/// The NUL-terminated string at `off` in a table of `table` bytes.
fn cstr<'a>(table: &'a [u8], off: u32, what: &str) -> Result<&'a [u8], String> {
    let rest = table
        .get(off as usize..)
        .ok_or_else(|| format!("{what} at {off:#x} is past its table ({:#x} bytes)", table.len()))?;
    let end = rest
        .iter()
        .position(|&c| c == 0)
        .ok_or_else(|| format!("{what} at {off:#x} has no NUL in its table"))?;
    Ok(&rest[..end])
}

fn region<'a>(b: &'a [u8], off: u32, len: usize, what: &str) -> Result<&'a [u8], String> {
    b.get(off as usize..off as usize + len)
        .ok_or_else(|| format!("{what}: {len:#x} bytes at {off:#x} pass the end ({:#x})", b.len()))
}

/// Read a BPB. The game's loader also takes one wrapped as `ZLIBME`, a u32 length at 12 and a zlib
/// stream at 16; no file on the disc is, and this refuses them.
pub fn parse(b: &[u8]) -> Result<Bpb, String> {
    if b.starts_with(b"ZLIBME") {
        return Err("a ZLIBME (zlib-wrapped) BPB: not read here, and none is on the disc".into());
    }
    if b.get(..4) != Some(MAGIC) {
        return Err(format!("not a BPB (starts {:02x?})", &b[..b.len().min(4)]));
    }
    let sec_off = u32_at(b, 4)?;
    let key_off = u32_at(b, 8)?;
    let (n_sec, n_key) = (u16_at(b, 0xc)? as usize, u16_at(b, 0xe)? as usize);
    let names = region(b, u32_at(b, 0x10)?, u32_at(b, 0x14)? as usize, "the name table")?;
    let values = region(b, u32_at(b, 0x18)?, u32_at(b, 0x1c)? as usize, "the value strings")?;
    let secs = region(b, sec_off, n_sec * RECORD, "the section records")?;
    let keys = region(b, key_off, n_key * RECORD, "the key records")?;

    let mut out = Bpb::default();
    for (s, r) in secs.chunks_exact(RECORD).enumerate() {
        let name = cstr(names, u32_at(r, 4)?, "a section name")?.to_vec();
        let index = u16_at(r, 8)?;
        let (count, first) = (u16_at(r, 10)? as usize, u32_at(r, 12)? as usize);
        if first + count > n_key {
            return Err(format!("section {s} ({}): keys {first}+{count} of {n_key}", show(&name)));
        }
        let mut section = Section {
            hash: u32_at(r, 0)?,
            name,
            index: (index != 0xffff).then_some(index),
            keys: Vec::with_capacity(count),
        };
        for k in first..first + count {
            let r = &keys[k * RECORD..(k + 1) * RECORD];
            let name = cstr(names, u32_at(r, 4)?, "a key name")?.to_vec();
            let value = match u16_at(r, 8)? {
                1 => Value::Text(cstr(values, u32_at(r, 12)?, "a string value")?.to_vec()),
                2 => {
                    let raw = &r[10..16];
                    Value::Short(raw[..raw.iter().position(|&c| c == 0).unwrap_or(INLINE)].to_vec())
                }
                4 => Value::Int(u32_at(r, 12)? as i32),
                8 => Value::Float(f32::from_bits(u32_at(r, 12)?)),
                t => return Err(format!("key {k} ({}): type {t} is not one of 1, 2, 4, 8", show(&name))),
            };
            section.keys.push(Key { hash: u32_at(r, 0)?, name, value });
        }
        out.sections.push(section);
    }
    Ok(out)
}

fn show(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

impl Bpb {
    /// The INI: CRLF lines, `[Title]`, `Key=value`, a blank line between sections (the layout of the
    /// earlier games' text files, and what openmx's `Scene::parse` reads).
    pub fn to_ini(&self) -> Vec<u8> {
        let mut t = Vec::new();
        for (i, s) in self.sections.iter().enumerate() {
            if i > 0 {
                t.extend_from_slice(b"\r\n");
            }
            t.push(b'[');
            t.extend_from_slice(&s.title());
            t.extend_from_slice(b"]\r\n");
            for k in &s.keys {
                t.extend_from_slice(&k.name);
                t.push(b'=');
                t.extend_from_slice(&k.value.text());
                t.extend_from_slice(b"\r\n");
            }
        }
        t
    }

    /// Every section and key whose stored hash is not its name's.
    pub fn bad_hashes(&self) -> Vec<String> {
        let mut bad = Vec::new();
        for s in &self.sections {
            if hash(&s.name) != s.hash {
                bad.push(format!("[{}]: {:#010x}, the name gives {:#010x}", show(&s.title()), s.hash, hash(&s.name)));
            }
            for k in &s.keys {
                if hash(&k.name) != k.hash {
                    bad.push(format!(
                        "[{}] {}: {:#010x}, the name gives {:#010x}",
                        show(&s.title()),
                        show(&k.name),
                        k.hash,
                        hash(&k.name)
                    ));
                }
            }
        }
        bad
    }

    pub fn keys(&self) -> usize {
        self.sections.iter().map(|s| s.keys.len()).sum()
    }
}

/// A string table built in first-use order, each distinct string once.
#[derive(Default)]
struct Table {
    bytes: Vec<u8>,
    at: std::collections::HashMap<Vec<u8>, u32>,
}

impl Table {
    fn add(&mut self, s: &[u8]) -> u32 {
        if let Some(&o) = self.at.get(s) {
            return o;
        }
        let o = self.bytes.len() as u32;
        self.bytes.extend_from_slice(s);
        self.bytes.push(0);
        self.at.insert(s.to_vec(), o);
        o
    }
}

fn pad16(b: &mut Vec<u8>) {
    b.resize(b.len().div_ceil(16) * 16, 0);
}

/// The BPB of `bpb`, laid out as the disc's are (the hashes are written as held, so a test can build a
/// file with a wrong one). Fails on what the format cannot hold.
pub fn write(bpb: &Bpb) -> Result<Vec<u8>, String> {
    let n_sec = u16::try_from(bpb.sections.len()).map_err(|_| "more than 65535 sections")?;
    let n_key = u16::try_from(bpb.keys()).map_err(|_| "more than 65535 keys")?;
    let (mut names, mut values) = (Table::default(), Table::default());
    let (mut secs, mut keys) = (Vec::new(), Vec::new());
    for s in &bpb.sections {
        let count = u16::try_from(s.keys.len()).map_err(|_| "a section with more than 65535 keys")?;
        if s.index == Some(0xffff) {
            return Err("a section index of 65535 is the format's 'none'".into());
        }
        secs.extend_from_slice(&s.hash.to_le_bytes());
        secs.extend_from_slice(&names.add(&s.name).to_le_bytes());
        secs.extend_from_slice(&s.index.unwrap_or(0xffff).to_le_bytes());
        secs.extend_from_slice(&count.to_le_bytes());
        secs.extend_from_slice(&((keys.len() / RECORD) as u32).to_le_bytes());
        for k in &s.keys {
            keys.extend_from_slice(&k.hash.to_le_bytes());
            keys.extend_from_slice(&names.add(&k.name).to_le_bytes());
            keys.extend_from_slice(&k.value.kind().to_le_bytes());
            let mut tail = [0u8; INLINE];
            match &k.value {
                Value::Text(s) => tail[2..].copy_from_slice(&values.add(s).to_le_bytes()),
                Value::Short(s) => {
                    if s.len() >= INLINE || s.contains(&0) {
                        return Err(format!("{}: {:?} does not fit a short string", show(&k.name), show(s)));
                    }
                    tail[..s.len()].copy_from_slice(s);
                }
                Value::Int(i) => tail[2..].copy_from_slice(&i.to_le_bytes()),
                Value::Float(f) => tail[2..].copy_from_slice(&f.to_bits().to_le_bytes()),
            }
            keys.extend_from_slice(&tail);
        }
    }
    let key_off = HEADER + secs.len();
    let name_off = key_off + keys.len();
    let mut out = Vec::with_capacity(name_off + names.bytes.len() + values.bytes.len() + 32);
    out.extend_from_slice(MAGIC);
    for v in [HEADER as u32, key_off as u32] {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out.extend_from_slice(&n_sec.to_le_bytes());
    out.extend_from_slice(&n_key.to_le_bytes());
    let value_off = (name_off + names.bytes.len()).div_ceil(16) * 16;
    for v in [name_off, names.bytes.len(), value_off, values.bytes.len()] {
        out.extend_from_slice(&(v as u32).to_le_bytes());
    }
    out.extend_from_slice(&secs);
    out.extend_from_slice(&keys);
    out.extend_from_slice(&names.bytes);
    pad16(&mut out);
    out.extend_from_slice(&values.bytes);
    pad16(&mut out);
    Ok(out)
}
