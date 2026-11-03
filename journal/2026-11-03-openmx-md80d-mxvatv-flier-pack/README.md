# openmx MD80d: MX vs. ATV's biplane and Kiowa as a machine pack

`import-atv fliers [OUT]` (and `--disc IMAGE --out OUT`, which build.sh runs, now builds this pack
after the quad pack) writes `mods/machines/mxvatv_fliers/` off SLUS_211.04. That is 135 files plus
`mod.json`: eight skins, each with its model, its garage `_ui` model, its collision pair and its
textures; and each flier's `.phy`, cameras, sounds and particles. The layout is in the standard's §7a.

Where it comes from:

- `Veh\vehicletype_2.res` is the biplane's archive, copied whole: biplane1-4 and the propcones,
  Biplane*.col (with the grown BiplaneM), biplane.phy, the materials, biplaneparticles.dat and its
  emitters, mxparticlezones.dat, wreckchunks.emt and event.aud.
- The Kiowa is loose on the PSR: kiowa1-4, the blades and tail rotors, Kiowa*.col, kiowa.phy,
  WreckBlade.phy and helicopterparticles.dat.
- The garage sets `UI\Res\Veh_BiPlane1-4.res` and `Veh_Helicopter1-4.res` are unpacked whole. They
  hold the `_ui` models and the only copy of most skins' textures (`Pitts_body01.pst`,
  `SpitFire_01.pst`, …). No two sources give one name different bytes; the importer would stop if
  they did.
- The cameras FC_/RC_BiplaneDefault.pb and FC_/RC_HeliDefault.pb come out of `Cam\CameraPresets.res`.
- The sounds are the three `.aud` tables and every sound the fliers' sections name: carengine.aud's
  `[biplane]` (7 loops), otherengine.aud's `[Helicopter]` EngineLoop, and machine.aud's
  `[biplaneWreck]`, `[biplaneTireTouchdown]` and `[helicopterWreck]`. That is 14 VAGs, none with the
  `VAGp` magic, as MX vs. ATV's are.
- Skins 1 and 2 are model one (MX Unleashed's machine). 3 and 4 are model two. The `.phy` keys each
  model by `ModelOne`/`ModelTwo`.

modcheck allows a machine pack with no builds when it has `skins`, and checks that every file a skin
and a flier name exists.

## How it was proved

- `cargo test --release -p import-atv -- --ignored`: the quad test and the new
  `the_fliers_pack_is_the_discs_and_openmx_reads_it` pass. It checks 8 skins and their models, the
  two standing todo entries, modcheck clean, nothing refused, 16 collision files, 14 sounds, 2 clips,
  ≥28 models, ≥24 textures, ≥20 emitters, and every skin's flier nodes.
- `modcheck`: ok, 174 files compared byte for byte with the SLUS_211.04 disc. Control: one byte of
  `Veh/Anim/SH_BiPlane2.pst` changed → FAIL "differs … first at byte 100"; restored → ok.
- `import-atv fliers check`: 29 models, 16 collision files, 2 clips, 25 materials, 25 textures,
  14 sounds and 21 emitters read by openmx's readers. Every skin has every node.
- openmx (slot s1, task/MD80d), pack installed in its `mods/machines/`:
  - `cargo test -p openmx --test machine_packs`: 11 passed, including the two new flier tests.
    Controls: a truncated kiowa3.psg fails the skin test, and a missing mchn_wreck3.VAG fails the
    sounds test.
  - `openmx --machine-packs mods` reads every skin's nodes, collision and textures with nothing
    refused. It also reads both `.phy`s (17 keys a model for the biplane, 3 for the Kiowa), 8 follow
    and 8 replay cameras each, every sound, and 9 and 12 emitters, none without its `.emt`.
- `./check.sh`: ok, 150 passed, 0 failed.

## Not verified

- Nothing was drawn, flown or heard. openmx loads the pack and flies none of it until MD80b/MD80c and
  MD80e.
- kiowa3 and kiowa4 spell three seat dummies `Dummy_Foot_R_pos01`, `Dummy_Hand_L_pos01` and
  `Dummy_Hand_R_pos01`. The readers accept the `01`. Whether MX vs. ATV seats its pilot by those
  names, or by some other lookup, was not checked.
- `sh_biplane2.psg` / `SH_BiPlane2.pst` are taken by name only. That they are the biplane's shadow,
  and which skin uses them, is a guess.
- Whether MX vs. ATV draws a ridden skin from the garage set's textures or from somewhere else was not
  checked. The pack has both wherever both exist.
- Not looked at beside MX vs. ATV in PCSX2 (T7).
