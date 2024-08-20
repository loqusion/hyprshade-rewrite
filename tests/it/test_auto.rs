mod error {
    use crate::common::{hyprshade_cmd_snapshot, Space};

    #[test]
    fn fails_without_config() {
        let mut space = Space::new();
        space.with_any_time();
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

    #[test]
    fn fails_resolving_shader() {
        let mut space = Space::new();
        space.with_any_time().with_config(
            r#"
                [[shader]]
                name = "invalid"
                default = true
            "#,
        );
        hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("auto"), @r###"
        success: false
        exit_code: 1
        ----- stdout -----

        ----- stderr -----
        Error: 
           0: [91mresolving shader in config[0m
           1: [91mshader named "invalid" not found[0m

        Location:
           [LOCATION]

        Configuration:
           [33m[HYPRSHADE_CONFIG][39m

        [96mSuggestion[0m: Change the shader name in your configuration, or make sure a shader by that name exists
        [96mSuggestion[0m: For more information, see [URL]
        "###);
    }
}
