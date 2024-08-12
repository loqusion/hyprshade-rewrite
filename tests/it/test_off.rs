use proc_macros::hyprland_test;

use crate::common::Space;

#[hyprland_test]
fn smoke() {
    let space = Space::new();
    insta_cmd::assert_cmd_snapshot!(space.hyprshade_cmd().arg("off"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    "###);
}
