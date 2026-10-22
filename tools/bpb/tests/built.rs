//! A BPB put together byte by byte here, the way the disc's are laid out, decodes to the INI it holds;
//! the encoder gives the same bytes back; the hash is the name's.

use bpb::{Bpb, Key, Section, Value, float_text, hash, parse, write};
use game_sim::track::Scene;

fn name_off(names: &[u8], s: &str) -> u32 {
    let mut at = 0;
    for n in names.split(|&c| c == 0) {
        if n == s.as_bytes() {
            return at;
        }
        at += n.len() as u32 + 1;
    }
    panic!("{s} not in the name table")
}

/// A small scene: `[SceneInfo]`, `[Environment]`, `[Object1]`, `[Object2]`, every value type.
fn built() -> Vec<u8> {
    let names = b"SceneInfo\0SceneName\0DefaultStartPosition\0Environment\0TerrainFile\0TerrainWidth\0\
TerrainWidthScale\0Object\0COL\0Position\0Hide\0";
    // First-use order, as the disc's are: the start position comes before the terrain.
    let values = b"1.5,2.25,-3\0Free06.trn\0Rock.col\0";
    let sections: [(&str, u16, u16, u32); 4] =
        [("SceneInfo", 0xffff, 2, 0), ("Environment", 0xffff, 3, 2), ("Object", 1, 3, 5), ("Object", 2, 1, 8)];
    // (name, type, the last six bytes)
    let mut keys: Vec<(&str, u16, [u8; 6])> = Vec::new();
    let off = |v: u32| {
        let mut t = [0u8; 6];
        t[2..].copy_from_slice(&v.to_le_bytes());
        t
    };
    keys.push(("SceneName", 2, *b"Ex\0\0\0\0"));
    keys.push(("DefaultStartPosition", 1, off(0)));
    keys.push(("TerrainFile", 1, off(12)));
    keys.push(("TerrainWidth", 4, off(3)));
    keys.push(("TerrainWidthScale", 8, off(4.5f32.to_bits())));
    keys.push(("COL", 1, off(23)));
    keys.push(("Position", 1, off(0)));
    keys.push(("Hide", 2, *b"False\0"));
    keys.push(("Position", 1, off(0)));

    let key_off = 0x20 + 16 * sections.len();
    let name_at = key_off + 16 * keys.len();
    let value_at = (name_at + names.len()).div_ceil(16) * 16;
    let mut b = Vec::new();
    b.extend_from_slice(b"BPB\0");
    for v in [0x20u32, key_off as u32] {
        b.extend_from_slice(&v.to_le_bytes());
    }
    b.extend_from_slice(&(sections.len() as u16).to_le_bytes());
    b.extend_from_slice(&(keys.len() as u16).to_le_bytes());
    for v in [name_at, names.len(), value_at, values.len()] {
        b.extend_from_slice(&(v as u32).to_le_bytes());
    }
    for (n, index, count, first) in sections {
        b.extend_from_slice(&hash(n.as_bytes()).to_le_bytes());
        b.extend_from_slice(&name_off(names, n).to_le_bytes());
        b.extend_from_slice(&index.to_le_bytes());
        b.extend_from_slice(&count.to_le_bytes());
        b.extend_from_slice(&first.to_le_bytes());
    }
    for (n, t, tail) in &keys {
        b.extend_from_slice(&hash(n.as_bytes()).to_le_bytes());
        b.extend_from_slice(&name_off(names, n).to_le_bytes());
        b.extend_from_slice(&t.to_le_bytes());
        b.extend_from_slice(tail);
    }
    b.extend_from_slice(names);
    b.resize(value_at, 0);
    b.extend_from_slice(values);
    b.resize(b.len().div_ceil(16) * 16, 0);
    b
}

const INI: &str = "[SceneInfo]\r\nSceneName=Ex\r\nDefaultStartPosition=1.5,2.25,-3\r\n\r\n\
[Environment]\r\nTerrainFile=Free06.trn\r\nTerrainWidth=3\r\nTerrainWidthScale=4.500000\r\n\r\n\
[Object1]\r\nCOL=Rock.col\r\nPosition=1.5,2.25,-3\r\nHide=False\r\n\r\n\
[Object2]\r\nPosition=1.5,2.25,-3\r\n";

#[test]
fn a_built_bpb_decodes_to_its_ini() {
    let b = built();
    let bpb = parse(&b).expect("parses");
    assert_eq!(String::from_utf8(bpb.to_ini()).unwrap(), INI);
    assert!(bpb.bad_hashes().is_empty());
    assert_eq!(bpb.sections[2].index, Some(1));
    assert_eq!(bpb.sections[1].keys[2].value, Value::Float(4.5));
    assert_eq!(bpb.sections[1].keys[1].value, Value::Int(3));
    assert_eq!(bpb.sections[0].keys[0].value, Value::Short(b"Ex".to_vec()));

    // The encoder lays it out the same way, byte for byte.
    assert_eq!(write(&bpb).unwrap(), b);

    // And openmx reads the text.
    let scene = Scene::parse(&bpb.to_ini()).expect("Scene::parse");
    assert_eq!(scene.name, "Ex");
    assert_eq!(scene.terrain_file, "Free06.trn");
    assert_eq!(scene.terrain.width, 3);
    // Object2 names no COL, so it is not a placement.
    assert_eq!(scene.collision.len(), 1);
    assert_eq!(scene.collision[0].file, "Rock.col");
}

#[test]
fn the_hash_is_djb2_of_the_lowered_name() {
    // Values off the disc (Nat_AZ.scn.bpb).
    assert_eq!(hash(b"SceneInfo"), 0x0a29_2e9f);
    assert_eq!(hash(b"SceneName"), 0x0a2b_b614);
    assert_eq!(hash(b"Type"), 0x7c9e_bd07);
    assert_eq!(hash(b"HasCOL"), hash(b"HasCol"));
    assert_eq!(hash(b""), 5381);
}

#[test]
fn a_wrong_hash_is_reported() {
    let mut b = built();
    b[0x20] ^= 1; // the first section's hash
    let bad = parse(&b).unwrap().bad_hashes();
    assert_eq!(bad.len(), 1, "{bad:?}");
    assert!(bad[0].starts_with("[SceneInfo]"), "{bad:?}");
}

#[test]
fn broken_files_are_refused() {
    let b = built();
    assert!(parse(&b[..0x30]).is_err(), "cut short");
    assert!(parse(b"ZLIBME\0\0").unwrap_err().contains("ZLIBME"));
    assert!(parse(b"INI\0").is_err());
    let mut t2 = b.clone();
    let key_off = 0x20 + 4 * 16;
    t2[key_off + 8] = 3; // the first key's type
    assert!(parse(&t2).unwrap_err().contains("type 3"));
    let mut t3 = b;
    t3[0x20 + 12] = 200; // first key past the end
    assert!(parse(&t3).is_err());
}

#[test]
fn floats_read_back_to_their_bits() {
    assert_eq!(float_text(4.5), "4.500000");
    assert_eq!(float_text(400.0), "400.000000");
    assert_eq!(float_text(-0.25), "-0.250000");
    // 0.1 as a single is above 0.1: "0.100000" would truncate to the single below.
    assert_eq!(float_text(0.1), "0.10000000149011612");
    // Six places cannot hold it at all.
    assert_eq!(float_text(1e-7), "0.00000010000000116860974");
    for f in [0.1f32, 1e-7, 4.499999, 3.0e9, -7.77, f32::MIN_POSITIVE] {
        let d: f64 = float_text(f).parse().unwrap();
        let nearest = d as f32;
        let truncated = if f64::from(nearest).abs() > d.abs() {
            f32::from_bits(nearest.to_bits() - 1)
        } else {
            nearest
        };
        assert_eq!((nearest, truncated), (f, f), "{f}: {}", float_text(f));
    }
}

#[test]
fn sections_without_keys_and_long_strings_survive_the_encoder() {
    let bpb = Bpb {
        sections: vec![Section {
            hash: hash(b"Empty"),
            name: b"Empty".to_vec(),
            index: Some(0),
            keys: vec![],
        }, Section {
            hash: hash(b"S"),
            name: b"S".to_vec(),
            index: None,
            keys: vec![Key { hash: hash(b"K"), name: b"K".to_vec(), value: Value::Text(vec![b'x'; 300]) }],
        }],
    };
    let b = write(&bpb).unwrap();
    assert_eq!(parse(&b).unwrap(), bpb);
    assert_eq!(String::from_utf8(bpb.to_ini()).unwrap(), format!("[Empty0]\r\n\r\n[S]\r\nK={}\r\n", "x".repeat(300)));
    let too_long = Bpb { sections: vec![Section { keys: vec![Key { hash: 0, name: b"K".to_vec(), value: Value::Short(b"Sixsix".to_vec()) }], ..bpb.sections[0].clone() }] };
    assert!(write(&too_long).is_err());
}
