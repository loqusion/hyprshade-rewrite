mod error {
    use proc_macros::hyprland_test;

    use crate::common::{hyprshade_cmd_snapshot, CommandExt, Space};

    #[test]
    fn empty_arg_fails_without_config() {
        let mut space = Space::new();
        space.with_any_time();
        hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("toggle"), @r###"
        success: false
        exit_code: 1
        ----- stdout -----

        ----- stderr -----
        Error: 
           0: [91mno configuration file found[0m

        Location:
           [LOCATION]

        [93mWarning[0m: A configuration file is required to call this command without SHADER
        [96mSuggestion[0m: For more information, see [URL]
        "###);
    }

    #[test]
    fn fallback_default_fails_without_config() {
        let mut space = Space::new();
        space.with_any_time();
        hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("toggle").args(["vibrance", "--fallback-default"]), @r###"
        success: false
        exit_code: 1
        ----- stdout -----

        ----- stderr -----
        Error: 
           0: [91mno configuration file found[0m

        Location:
           [LOCATION]

        [93mWarning[0m: A configuration file is required to use --fallback-default
        [96mSuggestion[0m: For more information, see [URL]
        "###);
    }

    #[test]
    fn fallback_auto_fails_without_config() {
        let mut space = Space::new();
        space.with_any_time();
        hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("toggle").args(["vibrance", "--fallback-auto"]), @r###"
        success: false
        exit_code: 1
        ----- stdout -----

        ----- stderr -----
        Error: 
           0: [91mno configuration file found[0m

        Location:
           [LOCATION]

        [93mWarning[0m: A configuration file is required to use --fallback-auto
        [96mSuggestion[0m: For more information, see [URL]
        "###);
    }

    #[test]
    fn fallback_default_fails_without_default_shader() {
        let mut space = Space::new();
        space.with_any_time().with_config(r#""#);
        hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("toggle").args(["vibrance", "--fallback-default"]), @r###"
        success: false
        exit_code: 1
        ----- stdout -----

        ----- stderr -----
        Error: 
           0: [91mno default shader found in config[0m

        Location:
           [LOCATION]

        Configuration:
           [33m[HYPRSHADE_CONFIG][39m

        [96mSuggestion[0m: Make sure a default shader is defined (default = true)
        [96mSuggestion[0m: For more information, see [URL]
        "###);
    }

    #[hyprland_test]
    fn empty_arg_fails_resolving_shader() {
        let mut space = Space::new();
        space.with_any_time().with_config(
            r#"
            [[shader]]
            name = "invalid"
            default = true
            "#,
        );
        space.hyprshade_cmd().arg("off").run();

        hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("toggle"), @r###"
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

        [96mNote[0m: Since you omitted SHADER from cli arguments, it was inferred from the schedule in your configuration
        [96mSuggestion[0m: Change the shader name in your configuration, or make sure a shader by that name exists
        [96mSuggestion[0m: For more information, see [URL]
        "###);
    }

    #[hyprland_test]
    fn fallback_default_fails_resolving_shader() {
        let mut space = Space::new();
        space.with_any_time().with_config(
            r#"
            [[shader]]
            name = "invalid"
            default = true
            "#,
        );
        space.hyprshade_cmd().arg("on").arg("vibrance").run();

        hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("toggle").args(["vibrance", "--fallback-default"]), @r###"
        success: false
        exit_code: 1
        ----- stdout -----

        ----- stderr -----
        Error: 
           0: [91mresolving default shader in config[0m
           1: [91mshader named "invalid" not found[0m

        Location:
           [LOCATION]

        Configuration:
           [33m[HYPRSHADE_CONFIG][39m

        [96mSuggestion[0m: Change the shader name in your configuration, or make sure a shader by that name exists
        [96mSuggestion[0m: For more information, see [URL]
        "###);
    }

    #[hyprland_test]
    fn fallback_auto_fails_resolving_scheduled_shader() {
        let mut space = Space::new();
        space.with_time("01:00:00").with_config(
            r#"
            [[shader]]
            name = "invalid"
            start_time = 00:00:00
            end_time = 02:00:00
            "#,
        );
        space.hyprshade_cmd().arg("on").arg("vibrance").run();

        hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("toggle").args(["vibrance", "--fallback-auto"]), @r###"
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

        [96mNote[0m: Tried to resolve scheduled shader because of --fallback-auto
        [96mSuggestion[0m: Change the shader name in your configuration, or make sure a shader by that name exists
        [96mSuggestion[0m: For more information, see [URL]
        "###);
    }

    #[hyprland_test]
    fn fallback_auto_fails_resolving_default_shader() {
        let mut space = Space::new();
        space.with_time("01:00:00").with_config(
            r#"
            [[shader]]
            name = "invalid"
            default = true
            [[shader]]
            name = "vibrance"
            start_time = 00:00:00
            end_time = 02:00:00
            "#,
        );
        space.hyprshade_cmd().arg("on").arg("vibrance").run();

        hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("toggle").args(["vibrance", "--fallback-auto"]), @r###"
        success: false
        exit_code: 1
        ----- stdout -----

        ----- stderr -----
        Error: 
           0: [91mresolving default shader in config[0m
           1: [91mshader named "invalid" not found[0m

        Location:
           [LOCATION]

        Configuration:
           [33m[HYPRSHADE_CONFIG][39m

        [96mNote[0m: Tried to resolve default shader because of --fallback-auto
        [96mSuggestion[0m: Change the shader name in your configuration, or make sure a shader by that name exists
        [96mSuggestion[0m: For more information, see [URL]
        "###);
    }
}
