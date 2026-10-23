//! The stunt table, the gesture code and the stunt `.mcf` on generated files, and a whole pack built
//! from generated members and read back. The discs' own tables are `tests/disc.rs`.

use bpb::{Bpb, Key, Section, Value, hash};
use import_tricks::*;
use std::collections::HashMap;

/// A table in MX vs. ATV Unleashed's layout: comments, quoted display names, `[Stunt%02d]`.
const TEXT: &str = "\
;; The first 24 stunts are the defualt stunts
; R1       = \"1\"

;Circle Up

[Stunt01]
Name=Taunt
DisplayName=\"Taunt\"
HoldTime = 0.93
SafeTime = 1.50
Duration = 1.67
LinkModTime = 0.0
UsedForBike = T
HasAnimation = T
DefaultKeyMapping = \"CU\"   ; a comment after a value
PointValue = 1000.0

[Stunt02]
Name=threesixtyL
DisplayName=\"Three Sixty Left\"
HoldTime = 0.72
SafeTime = 1.26
Duration = 1.50
LinkModTime = 0.0
UsedForBike = T
HasAnimation = T
DefaultKeyMapping = \"\"
PointValue = 2000.0
HoldPointValue = 15000.0

[Stunt03]
Name=wheelie
DisplayName=\"Wheelie\"
HoldTime = 0.0
UsedForBike = F
HasAnimation = F
HoldPointValue = 1000.0

[Depreciation]
DepreciationPct = 0.8
NumStuntsToRestoreOne = 4
NumStuntsToMaxDepletion = 4

[Stunt04]
Name=unread
";

#[test]
fn a_table_reads_as_the_loader_reads_it() {
    let t = Table::parse_with(TEXT, 3, 0x20, "DefaultKeyMapping").unwrap();
    assert_eq!(t.stunts.len(), 3);
    let s = &t.stunts[0];
    assert_eq!((s.section.as_str(), s.name.as_str(), s.display_name.as_str()), ("Stunt01", "Taunt", "Taunt"));
    let n = |v: &str| game_sim::phy::number(v).0;
    assert_eq!((s.hold_time, s.safe_time, s.duration), (n("0.93"), 1.5, n("1.67")));
    assert!(s.hold_time < 0.93, "truncated, not rounded: {}", s.hold_time);
    assert_eq!((s.used_for_bike, s.has_animation), (Some(true), Some(true)));
    assert_eq!(s.key.as_deref(), Some("CU"), "the comment after the value is not part of it");
    assert_eq!((s.point_value, s.point_value_given, s.hold_point_value, s.hold_point_value_given), (1000.0, true, 1000.0, false));

    let s = t.by_name("THREESIXTYL").unwrap();
    assert_eq!(s.key, None, "an empty mapping is no gesture");
    assert_eq!((s.point_value, s.hold_point_value), (2000.0, 15000.0));

    // Missing keys take the loader's defaults: zero clocks, 1000 points.
    let s = &t.stunts[2];
    assert_eq!((s.safe_time, s.duration, s.link_mod_time), (0.0, 0.0, 0.0));
    assert_eq!((s.point_value, s.point_value_given), (1000.0, false));

    let d = t.depreciation.as_ref().unwrap();
    assert_eq!((d.restore_one, d.max_depletion), (4, 4));
    assert_eq!(t.unread, vec!["Stunt04".to_string()], "the loader reads only its count");
}

#[test]
fn the_clocks_truncate_like_the_ee() {
    // 1.33 is not an f32; the loader's double-to-float truncates, so it is the one below nearest.
    let t = Table::parse_with("[Stunt01]\nDuration = 1.33\n", 1, 0x20, "DefaultKeyMapping").unwrap();
    let nearest = 1.33f32;
    assert!(t.stunts[0].duration <= nearest);
    assert_eq!(t.stunts[0].duration, game_sim::phy::number("1.33").0);
}

#[test]
fn a_missing_section_and_a_bad_number_are_errors() {
    assert!(Table::parse_with(TEXT, 4 + 1, 0x20, "DefaultKeyMapping").unwrap_err().contains("[Stunt05]"));
    let bad = "[Stunt01]\nHoldTime = soon\n";
    assert!(Table::parse_with(bad, 1, 0x20, "DefaultKeyMapping").unwrap_err().contains("Stunt01"));
}

#[test]
fn names_are_cut_to_the_loaders_buffer() {
    let long = "[Stunt01]\nName=abcdefghijklmnopqrstuvwxyz0123456789\nDisplayName=\"abcdefghijklmnopqrstuvwxyz0123456789\"\n";
    let t = Table::parse_with(long, 1, 0x20, "DefaultKeyMapping").unwrap();
    assert_eq!(t.stunts[0].name.len(), 31, "MX vs. ATV Unleashed: 0x20 bytes with the terminator");
    let t = Table::parse_with(long, 1, 0x1b, "DefaultKeyMapping").unwrap();
    assert_eq!(t.stunts[0].display_name.len(), 26, "Untamed: 0x1b bytes");
}

/// The same table as Untamed keeps it: BPB, `[Stunt1]` (BPB's `%s%d`), floats stored as floats.
fn table_bpb() -> Vec<u8> {
    let sec = |name: &str, index: Option<u16>, keys: &[(&str, Value)]| Section {
        hash: hash(name.as_bytes()),
        name: name.as_bytes().to_vec(),
        index,
        keys: keys.iter().map(|(k, v)| Key { hash: hash(k.as_bytes()), name: k.as_bytes().to_vec(), value: v.clone() }).collect(),
    };
    let t = |s: &str| Value::Text(s.as_bytes().to_vec());
    let short = |s: &str| Value::Short(s.as_bytes().to_vec());
    bpb::write(&Bpb {
        sections: vec![
            sec(
                "Stunt",
                Some(1),
                &[
                    ("Name", short("Taunt")),
                    ("DisplayName", short("Taunt")),
                    ("HoldTime", Value::Float(0.93)),
                    ("SafeTime", Value::Float(1.5)),
                    ("Duration", Value::Float(1.67)),
                    ("UsedForBike", short("T")),
                    ("DefaultKeyMappingClassic", short("CU")),
                    ("DefaultKeyMappingStyle1", short("MR")),
                    ("PointValue", Value::Float(1000.0)),
                ],
            ),
            sec(
                "Stunt",
                Some(2),
                &[
                    ("Name", t("threesixtyL")),
                    ("DisplayName", t("Three Sixty Left")),
                    ("HoldTime", Value::Float(0.72)),
                    ("PointValue", Value::Float(2000.0)),
                    ("HoldPointValue", Value::Float(15000.0)),
                ],
            ),
        ],
    })
    .unwrap()
}

#[test]
fn a_bpb_table_reads_like_its_text() {
    let text = ini_text("MX_Rider_Stunts.ini.bpb", &table_bpb()).unwrap();
    let t = Table::parse_with(&text, 2, 0x1b, "DefaultKeyMappingClassic").unwrap();
    assert_eq!(t.stunts[0].section, "Stunt1");
    assert_eq!((t.stunts[0].name.as_str(), t.stunts[0].hold_time, t.stunts[0].duration), ("Taunt", 0.93, 1.67));
    assert_eq!(t.stunts[0].key.as_deref(), Some("CU"), "the PS2's mapping is the Classic one, not a Wii style");
    assert_eq!((t.stunts[1].name.as_str(), t.stunts[1].hold_time, t.stunts[1].hold_point_value), ("threesixtyL", 0.72, 15000.0));
    // openmx's loader builds `stunt01`, so BPB's `Stunt1` is not found even before the count.
    assert!(openmx_reads(&text).unwrap_err().contains("stunt01"));
}

#[test]
fn openmx_reads_57_sections_and_no_fewer() {
    let mut t = String::new();
    for i in 1..=57 {
        t.push_str(&format!("[Stunt{i:02}]\nName=s{i}\nHoldTime=1\nSafeTime=1\nDuration=1\n"));
        if i == 46 {
            assert!(openmx_reads(&t).unwrap_err().contains("[stunt47]"), "MX vs. ATV's 46 are refused");
        }
    }
    assert_eq!(openmx_reads(&t), Ok(57));
}

#[test]
fn the_gesture_code_decodes_as_the_loader_does() {
    assert_eq!(key_bits("CU"), CIRCLE | UP);
    assert_eq!(key_bits("1CTR"), R1 | CIRCLE | TRIANGLE | RIGHT);
    assert_eq!(key_bits("TCD"), key_bits("CTD"), "order does not matter");
    // Untamed's Wii letters are skipped by the PS2 loader's decoder (`3 4 S W B`) or unknown (M, H).
    assert_eq!(key_bits("3MHSWB4"), 0);
    assert_eq!(describe(key_bits("1CTU")), "R1+Circle+Triangle, up");
    assert_eq!(describe(R1), "R1", "the backflip's and the 360's button");
    assert_eq!(describe(0), "none");
}

#[test]
fn a_stunt_mcf_names_its_clips() {
    let mcf = Mcf::parse(
        "[General info]\nNumberofMotions = 2\nContentDirectory=Rider\\Anim\\\n[MOTION 1]\nNAME=MX_rider_Taunt\nVUE=MX_Taunt.MOT\nLooping=no\n[MOTION 2]\nNAME=MX_rider_ThreeSixtyL\nVUE=MX_ThreeSixtyL.vue\n",
    );
    assert_eq!(mcf.motions.len(), 2);
    assert_eq!(mcf.member("mx_rider_taunt").as_deref(), Some("Rider\\Anim\\MX_Taunt.mot"));
    assert_eq!(mcf.member("MX_rider_threesixtyl").as_deref(), Some("Rider\\Anim\\MX_ThreeSixtyL.mot"), "the written extension is replaced");
    assert_eq!(mcf.member("MX_rider_varial"), None);
}

/// A pack of the generated table, built from generated members and read back.
const TEST_PACK: Pack = Pack {
    game: "mxvatv",
    id: "mxvatv_tricks",
    name: "test",
    res: "StuntInfo.res",
    tables: &["Rider\\Anim\\MX_Rider_Stunts.ini"],
    stunts: 3,
    name_len: 0x20,
    key_field: "DefaultKeyMapping",
    riders: &[Rider {
        prefix: "MX",
        table: "Rider\\Anim\\MX_Rider_Stunts.ini",
        mcf: "Rider\\Anim\\MX_RiderStunts.mcf",
        mlb: "Rider\\Anim\\MX_RiderStunts.MLB",
        bike: "",
    }],
    phy: &["Veh\\mx_*.phy"],
};

#[test]
fn a_pack_builds_and_reads_back() {
    let mut files: HashMap<&str, Vec<u8>> = HashMap::new();
    files.insert("Rider\\Anim\\MX_Rider_Stunts.ini", TEXT.as_bytes().to_vec());
    files.insert(
        "Rider\\Anim\\MX_RiderStunts.mcf",
        b"[General info]\nContentDirectory=Rider\\Anim\\\n[MOTION 1]\nNAME=MX_rider_Taunt\nVUE=MX_Taunt.MOT\n[MOTION 2]\nNAME=MX_rider_threesixtyL\nVUE=MX_ThreeSixtyL.MOT\n".to_vec(),
    );
    files.insert("Rider\\Anim\\MX_Taunt.mot", b"not a clip".to_vec());
    files.insert("Rider\\Anim\\MX_ThreeSixtyL.mot", b"nor this".to_vec());
    files.insert("Veh\\Anim\\Bike_ThreeSixtyL.mot", b"a bike's".to_vec());
    files.insert("Veh\\mx_Pro250A.phy", b"[Three Sixty]\nMaxThreeSixtyTorqueX = 1100\n".to_vec());
    files.insert("Veh\\baja.phy", b"not a bike".to_vec());
    let members: Vec<String> = files.keys().map(|k| k.to_string()).collect();
    let mut read = |m: &str| files.get(m).cloned().ok_or_else(|| format!("{m}: no such member"));
    let origin = Origin { game: "MX vs. ATV Unleashed".into(), boot: "SLUS_211.04".into(), psr: "MXVSATV.PSR".into() };
    let mxu: Vec<String> = ["Taunt", "backflip"].iter().map(|s| s.to_string()).collect();
    let root = std::env::temp_dir().join(format!("import-tricks-table-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);

    let built = build(&TEST_PACK, &origin, &members, &mut read, Some(&mxu), &root).unwrap();
    assert_eq!(built.new, vec!["threesixtyL".to_string(), "wheelie".to_string()]);
    assert!(built.todo.iter().any(|t| t.contains("Rider/Anim/MX_Rider_Stunts.ini: openmx's trick::Stunts::parse")), "{:?}", built.todo);
    assert!(built.todo.iter().any(|t| t.contains("MX_RiderStunts.MLB: SLUS_211.04 names it, and it is not on the disc")));
    assert!(!built.todo.iter().any(|t| t.contains("motion for wheelie")), "wheelie has no animation");
    assert_eq!(built.clips.len(), 3, "two rider clips and the bike's 360");
    assert!(built.clips.iter().all(|c| c.read.is_err()) && built.todo.iter().filter(|t| t.contains("skin::Clip::parse refuses")).count() == 3);
    assert!(root.join("mxvatv_tricks/Veh/mx_Pro250A.phy").is_file() && !root.join("mxvatv_tricks/Veh/baja.phy").exists());

    let json = std::fs::read_to_string(root.join("mxvatv_tricks/mod.json")).unwrap();
    assert!(json.contains("\"kind\": \"tricks\"") && json.contains("\"display_name\": \"Three Sixty Left\""));
    assert!(json.contains("\"hold_point_value\": 15000"));

    let (tables, clips) = check_with(&root.join("mxvatv_tricks"), &TEST_PACK).unwrap();
    assert_eq!(tables[0].table, built.tables[0].table, "the table reads back as it was built");
    assert_eq!(clips.len(), 3);

    // A second build replaces its own folder; a folder of something else is left alone.
    build(&TEST_PACK, &origin, &members, &mut read, Some(&mxu), &root).unwrap();
    std::fs::write(root.join("mxvatv_tricks/mod.json"), "{\"kind\": \"music\"}").unwrap();
    assert!(build(&TEST_PACK, &origin, &members, &mut read, Some(&mxu), &root).unwrap_err().contains("not touched"));
    std::fs::remove_dir_all(&root).unwrap();
}
