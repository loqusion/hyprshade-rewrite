use proc_macros::hyprland_test;

use crate::common::{hyprshade_cmd_snapshot, Space};

#[hyprland_test]
fn smoke() {
    let space = Space::new();
    hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("off"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    "###);
}
