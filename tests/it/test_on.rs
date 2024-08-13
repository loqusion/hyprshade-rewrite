mod error {
    use crate::common::{hyprshade_cmd_snapshot, Space};

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
        exit_code: 1
        ----- stdout -----

        ----- stderr -----
        Error: 
           0: [91m--var: 'strength=0.6' conflicts with 'strength=0.5'[0m

        Location:
           [LOCATION]
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
