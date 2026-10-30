//! Each imported map's category (openmx MD66): the source game's event type, from the id.

use import_mxvatv::{MXVATV, category, mxu::MXU, untamed::UNTAMED};

#[test]
fn each_id_names_its_games_event_type() {
    for (id, want) in [
        ("mxvatv_fs01", "Freestyle"),
        ("mxvatv_sx18", "Supercross"),
        ("mxvatv_nat17", "National"),
        ("mxvatv_sm03", "Supermoto"),
        ("mxvatv_mc10", "Machine Challenge"),
    ] {
        assert_eq!(category(&MXVATV, id), want, "{id}");
    }
    for (id, want) in [
        ("untamed_fs_a", "Freestyle"),
        ("untamed_sx_l", "Supercross"),
        ("untamed_nat_sdca", "National"),
        ("untamed_sm_s1", "Supermoto"),
        ("untamed_bt_b", "Opencross"),
        ("untamed_am_gfuc", "Free Roam"),
        ("untamed_mt_a", "Free Roam"),
    ] {
        assert_eq!(category(&UNTAMED, id), want, "{id}");
    }
    assert_eq!(category(&MXU, "mxu_free06"), "Free Roam");
    assert_eq!(category(&MXU, "mxu_nat01"), "National");
    assert_eq!(category(&MXVATV, "mxvatv_zz01"), "Other");
}
