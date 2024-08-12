mod error {
    use crate::common::{hyprshade_cmd_snapshot, Space};

    #[test]
    fn fails_without_config() {
        let mut space = Space::new();
        space.with_time("02:00:00");
        hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("auto"), @r###"
        success: false
        exit_code: 1
        ----- stdout -----

        ----- stderr -----
        Error: 
           0: [91mno configuration file found[0m

        Location:
           [LOCATION]

        [93mWarning[0m: A configuration file is required to call this command
        [96mSuggestion[0m: For more information, see [URL]
        "###);
    }
}
