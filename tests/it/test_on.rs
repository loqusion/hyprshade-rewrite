use insta::assert_snapshot;
use proc_macros::hyprland_test;

use crate::common::{hyprshade_cmd_snapshot, CommandExt, Space};

#[hyprland_test]
fn builtin_shader() {
    let space = Space::new();
    let _stash = space.stash_runtime_shader("vibrance");
    space.hyprshade_cmd().arg("off").run();

    hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("on").args(["vibrance"]), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    "###);

    assert_snapshot!(space.read_runtime_shader("vibrance"));
}

#[hyprland_test]
fn path_shader() {
    let space = Space::new();
    space.hyprshade_cmd().arg("off").run();
    let fixture_path = space.fixture_simple();

    hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("on").arg(&fixture_path), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    "###);
}

mod error {
    use crate::common::{hyprshade_cmd_snapshot, Space};

    #[test]
    fn fails_resolving_shader() {
        let space = Space::new();
        hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("on").args(["invalid"]), @r###"
        success: false
        exit_code: 1
        ----- stdout -----

        ----- stderr -----
        Error: 
           0: [91mshader named "invalid" not found[0m

        Location:
           [LOCATION]
        "###);
    }

    #[test]
    fn fails_merging_var() {
        let space = Space::new();
        hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("on").args([
            "vibrance",
            "--var",
            "strength=0.5",
            "--var",
            "strength=0.6",
        ]), @r###"
        success: false
        exit_code: 2
        ----- stdout -----

        ----- stderr -----
        error: the argument '--var strength=0.6' cannot be used with '--var strength=0.5'

          tip: '--var strength=0.6' would override '--var strength=0.5'

        For more information, try '--help'.
        "###);
    }

    #[test]
    fn fails_parsing_var() {
        let space = Space::new();
        const INVALID: &[&[u8]] = &[
            b"",
            b"strength",
            b"=0.3",
            b"strength=",
            b"=strength=",
            b"strength=0.3=0.4",
            b"balance..red=0.3",
            b".balance.red=0.3",
            b"balance.red.=0.3",
            b"\xFF\xFF\xFF",
        ];
        for invalid in INVALID.iter() {
            hyprshade_cmd_snapshot!(
                String::from_utf8_lossy(invalid).as_ref(),
                space
                    .hyprshade_cmd()
                    .arg("on")
                    .args(["vibrance", "--var"])
                    .arg(unsafe { std::ffi::OsStr::from_encoded_bytes_unchecked(invalid) })
            );
        }
    }
}
