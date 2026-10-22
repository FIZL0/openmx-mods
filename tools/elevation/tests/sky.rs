//! M17: a disc map's sky, fog and lights in a generated scene, on a scene written in the disc's style and
//! a generated `.cub` that game-render's `Sky::parse` takes; read back with openmx's readers and the
//! standard's check.

use elevation::sky::Donor;
use elevation::{Request, build_mod, write_mod};
use std::path::PathBuf;

/// A scene laid out as MX vs. ATV Unleashed's are (Nat03's keys, CRLF), with a section after the
/// lights that must not be copied.
const DONOR: &str = "[SceneInfo]\r\nSceneName=Nat99\r\nDefaultStartPosition=10.00,20.00,30.00\r\n\r\n\
[Environment]\r\nTerrainFile=nat99.trn\r\nEcosystemFile=Nat99.esb\r\nCubeFile=sky16.cub\r\nDetailMapTilingMultiplier=6\r\n\r\n\
[Fog]\r\nColorRGB=120,94,64\r\nVisibility=1.00\r\nHaziness=0.80\r\nMinimumFogDistance=5.00\r\nMaximumFogDistance=768.00\r\n\r\n\
[Lights]\r\nNumberOfLights = 2\r\n\r\n\
[Light1]\r\nType=Ambient\r\nColorRGB=137,141,170\r\n\r\n\
[Light2]\r\n;  The Sun\r\nType=Directional\r\nColorRGB=212,179,122\r\nPosition=1869.634521,928.181274,2133.083740\r\n\
TargetPosition=2880.000000,400.000000,2880.000000\r\nShadowRGB=25,25,25\r\nCastsShadows=T\r\n\r\n\
[StaticModels]\r\nNumberOfStaticModels=1\r\n\r\n[Model1]\r\nSLT=rock\r\nPosition=1,2,3\r\n";

/// 96 packets of one three-corner strip each: positions, then as many colours.
fn cub() -> Vec<u8> {
    let mut d = vec![0x55; 92];
    for p in 0..96u8 {
        d.extend_from_slice(&[0, 0, 0, 0, 9, 2, 0, 0]);
        d.extend_from_slice(&[1, 0xc0, 3, 0x66, 0, 0, 128, 0, 0, 128]);
        d.extend_from_slice(&[2, 0xc0, 3, 0x6a, p, 10, 20, p, 10, 20, p, 10, 20]);
    }
    d
}

fn donor(scene: &str, bytes: Vec<u8>) -> Result<Donor, String> {
    Donor::from_scene(
        "Nat99",
        "Test Dunes",
        "Env\\Nat99\\nat99.res",
        "Env\\nat99\\Nat99.scn",
        scene,
        |s| (s == "sky16.ps2.cub").then(|| ("Skies\\sky16.ps2.cub".to_owned(), bytes.clone())),
    )
}

fn dir(name: &str) -> PathBuf {
    let d = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("elevation-sky")
        .join(name);
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// A one-band f32 GeoTIFF of 300 × 300 one-metre pixels, low ripples, in EPSG 26911 metres (as
/// tests/generated.rs writes them).
fn dem(path: &std::path::Path) {
    use tiff::encoder::{TiffEncoder, colortype};
    use tiff::tags::Tag;
    let (w, h) = (300u32, 300u32);
    let px: Vec<f32> = (0..w * h).map(|k| 100.0 + (k % 7) as f32).collect();
    let mut e = TiffEncoder::new(std::fs::File::create(path).unwrap()).unwrap();
    let mut img = e.new_image::<colortype::Gray32Float>(w, h).unwrap();
    let t = img.encoder();
    t.write_tag(Tag::Unknown(33550), &[1.0f64, 1.0, 0.0][..])
        .unwrap();
    t.write_tag(
        Tag::Unknown(33922),
        &[0.0f64, 0.0, 0.0, 0.0, 300.0, 0.0][..],
    )
    .unwrap();
    t.write_tag(
        Tag::Unknown(34735),
        &[
            1u16, 1, 0, 4, 1024, 0, 1, 1, 1025, 0, 1, 1, 3072, 0, 1, 26911, 3076, 0, 1, 9001,
        ][..],
    )
    .unwrap();
    img.write_data(&px).unwrap();
}

#[test]
fn a_donor_takes_the_sections_line_for_line_and_its_sky_byte_for_byte() {
    let d = donor(DONOR, cub()).unwrap();
    assert_eq!(d.cube_file, "sky16.cub");
    assert_eq!(d.cube_path(), "Skies/sky16.ps2.cub");
    assert_eq!(d.cube, cub());
    assert_eq!(d.sections, ["Fog", "Lights", "Light1", "Light2"]);
    assert!(d.text.contains(";  The Sun\r\nType=Directional\r\n"));
    assert!(d.text.contains("NumberOfLights = 2\r\n"));
    assert!(
        !d.text.contains("StaticModels") && !d.text.contains("rock") && !d.text.contains("Esb")
    );

    // Refused: no CubeFile, no fog, no light, no sky file, a sky game-render does not decode.
    assert!(donor(&DONOR.replace("CubeFile=sky16.cub\r\n", ""), cub()).is_err());
    assert!(donor(&DONOR.replace("[Fog]", "[Fig]"), cub()).is_err());
    assert!(
        donor(
            &DONOR
                .replace("[Light1]", "[Lamp1]")
                .replace("[Light2]", "[Lamp2]"),
            cub()
        )
        .is_err()
    );
    assert!(donor(&DONOR.replace("sky16", "sky17"), cub()).is_err());
    assert!(donor(DONOR, cub()[..cub().len() / 2].to_vec()).is_err());
}

#[test]
fn a_generated_scene_with_a_donor_reads_as_the_donor_lights_it() {
    let tmp = dir("build");
    dem(&tmp.join("dem.tif"));
    let d = donor(DONOR, cub()).unwrap();
    let req = Request {
        id: "elev_test".into(),
        name: "Generated with a sky".into(),
        dem: vec![tmp.join("dem.tif")],
        naip: vec![],
        centre: (150.0, 150.0),
        cells: (1, 1),
        texture: 64,
        surface: 0,
        data: "generated GeoTIFF (test)".into(),
        date: "2026-10-22".into(),
        sky: Some(d.clone()),
    };
    let b = build_mod(&req).unwrap();
    let file = |p: &str| b.files.iter().find(|(n, _)| n == p).map(|(_, v)| v.clone());
    assert_eq!(file("tracks/elev_test/Skies/sky16.ps2.cub"), Some(cub()));
    let scene = String::from_utf8(file("tracks/elev_test/elev_test.scn").unwrap()).unwrap();
    assert!(scene.contains("DetailMapTilingMultiplier=8\r\nCubeFile=sky16.cub\r\n\r\n[Fog]\r\n"));
    assert!(!scene.contains("StaticModels") && !scene.contains("EcosystemFile"));

    // game-render reads the donor's numbers; game-sim still reads the generated spawn and terrain.
    let g = game_render::scene::Scene::parse(&scene);
    assert_eq!(g.fog_color, [120, 94, 64]);
    // Read as the original does (strtod, then to single): 0.80 is 0.79999995.
    assert_eq!((g.fog_near, g.fog_far, g.fog_visibility), (5.0, 768.0, 1.0));
    assert!((g.fog_haziness - 0.8).abs() < 1e-6);
    assert_eq!(g.lights.len(), 2);
    assert!(!g.lights[0].directional && g.lights[1].directional);
    assert_eq!(
        g.lights[1].color,
        [212.0 / 255.0, 179.0 / 255.0, 122.0 / 255.0]
    );
    let s = game_sim::track::Scene::parse(scene.as_bytes()).unwrap();
    assert_eq!(s.terrain_file.to_ascii_lowercase(), "elev_test.trn");

    // A changed number is caught by the proof.
    assert!(
        d.prove(&scene.replace("Haziness=0.80", "Haziness=0.70"))
            .is_err()
    );
    assert!(
        d.prove(&scene.replace("ColorRGB=212,179,122", "ColorRGB=212,179,123"))
            .is_err()
    );
    assert!(
        d.prove(&scene.replace("CubeFile=sky16.cub", "CubeFile=sky17.cub"))
            .is_err()
    );

    let json = String::from_utf8(file("mod.json").unwrap()).unwrap();
    assert!(json.contains("\"permission\": \"own-disc\""));
    assert!(json.contains("\"sky_from\": {\n    \"game\": \"MX vs. ATV Unleashed\",\n    \"boot\": \"SLUS_211.04\",\n    \"map\": \"Nat99\""));
    assert!(json.contains("\"sections\": [\"Fog\", \"Lights\", \"Light1\", \"Light2\"]"));
    assert!(
        b.todo
            .iter()
            .any(|t| t.contains("Nat99 (Test Dunes)") && t.contains("Skies/sky16.ps2.cub"))
    );
    assert!(!b.todo.iter().any(|t| t.contains("no sky (CubeFile)")));

    let out = tmp.join("elev_test");
    write_mod(&out, &b).unwrap();
    let r = modcheck::check(&out, &mut modcheck::Discs::new(None));
    assert!(r.ok(), "{:?}", r.fail);
    let _ = std::fs::remove_dir_all(&tmp);
}
