use proc_macros::hyprland_test;

use crate::common::{CommandExt, Space, hyprshade_cmd_snapshot};

#[test]
fn empty_arg_fails_without_config() {
    let mut space = Space::new();
    space.with_any_time();

    hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("toggle"), @r"
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
    ");
}

#[hyprland_test]
fn empty_arg_fails_resolving_default_shader() {
    let mut space = Space::new();
    space.with_any_time().with_config(
        r#"
            [[shader]]
            name = "invalid"
            default = true
        "#,
    );
    space.hyprshade_cmd().arg("off").run();

    hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("toggle"), @r#"
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
    "#);
}

#[hyprland_test]
fn empty_arg_fails_resolving_scheduled_shader() {
    let mut space = Space::new();
    space.with_time("01:00:00").with_config(
        r#"
            [[shader]]
            name = "invalid"
            start_time = 00:00:00
            end_time = 02:00:00
        "#,
    );

    let _stash = space.stash_runtime_shader("vibrance");
    space.hyprshade_cmd().arg("on").arg("vibrance").run();

    hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("toggle"), @r#"
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
    "#);
}

#[hyprland_test]
fn empty_arg_with_active_shader_turns_off() {
    let mut space = Space::new();
    space.with_any_time().with_config(r#""#);

    let _stash = space.stash_runtime_shader("vibrance");
    space.hyprshade_cmd().arg("on").arg("vibrance").run();

    hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("toggle"), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    assert!(space.current_shader().is_none());
}

#[hyprland_test]
fn empty_arg_with_no_active_shader_turns_on_default() {
    let mut space = Space::new();
    space.with_any_time().with_config(
        r#"
            [[shader]]
            name = "vibrance"
            default = true
        "#,
    );

    let _stash = space.stash_runtime_shader("vibrance");
    space.hyprshade_cmd().arg("off").run();

    hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("toggle"), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    assert_eq!(space.current_shader().as_deref(), Some("vibrance"));
}

#[test]
fn fallback_default_fails_without_config() {
    let mut space = Space::new();
    space.with_any_time();

    hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("toggle").args(["vibrance", "--fallback-default"]), @r"
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
    ");
}

#[test]
fn fallback_default_fails_without_default_shader() {
    let mut space = Space::new();
    space.with_any_time().with_config(r#""#);

    hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("toggle").args(["vibrance", "--fallback-default"]), @r"
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
    ");
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

    let _stash = space.stash_runtime_shader("vibrance");
    space.hyprshade_cmd().arg("on").arg("vibrance").run();

    hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("toggle").args(["vibrance", "--fallback-default"]), @r#"
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
    "#);
}

#[hyprland_test]
fn fallback_default_with_active_shader_turns_on_default() {
    let mut space = Space::new();
    space.with_any_time().with_config(
        r#"
            [[shader]]
            name = "color-filter"
            default = true
        "#,
    );

    let _stash = space.stash_runtime_shaders(["color-filter", "vibrance"]);
    space.hyprshade_cmd().arg("on").arg("vibrance").run();

    hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("toggle").args(["vibrance", "--fallback-default"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    assert_eq!(space.current_shader().as_deref(), Some("color-filter"));
}

#[hyprland_test]
fn fallback_default_with_default_turns_on_positional_arg() {
    let mut space = Space::new();
    space.with_any_time().with_config(
        r#"
            [[shader]]
            name = "color-filter"
            default = true
        "#,
    );

    let _stash = space.stash_runtime_shaders(["color-filter", "vibrance"]);
    space.hyprshade_cmd().arg("on").arg("color-filter").run();

    hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("toggle").args(["vibrance", "--fallback-default"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    assert_eq!(space.current_shader().as_deref(), Some("vibrance"));
}

#[test]
fn fallback_auto_fails_without_config() {
    let mut space = Space::new();
    space.with_any_time();

    hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("toggle").args(["vibrance", "--fallback-auto"]), @r"
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
    ");
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

    let _stash = space.stash_runtime_shader("vibrance");
    space.hyprshade_cmd().arg("on").arg("vibrance").run();

    hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("toggle").args(["vibrance", "--fallback-auto"]), @r#"
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
    "#);
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

    let _stash = space.stash_runtime_shader("vibrance");
    space.hyprshade_cmd().arg("on").arg("vibrance").run();

    hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("toggle").args(["vibrance", "--fallback-auto"]), @r#"
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
    "#);
}

#[hyprland_test]
fn fallback_auto_success() {
    #[derive(Clone, Copy, Debug)]
    enum TestVariant {
        WithDefault,
        NoDefault,
    }

    #[derive(Clone, Debug)]
    struct Case {
        time: &'static str,
        initial_shader: &'static str,
        expected_shader: ExpectedShader,
    }
    #[derive(Clone, Debug)]
    struct ExpectedShader {
        with_default: Option<&'static str>,
        no_default: Option<&'static str>,
    }
    impl Case {
        fn snapshot_name(&self, variant: TestVariant) -> String {
            let suffix = match &variant {
                TestVariant::NoDefault => "no-default",
                TestVariant::WithDefault => "with-default",
            };
            format!(
                "fallback_auto_success-{}-{}-{}-{}-{}",
                self.time,
                self.initial_shader,
                self.expected_shader.with_default.unwrap_or("--NONE--"),
                self.expected_shader.no_default.unwrap_or("--NONE--"),
                suffix
            )
        }
    }

    const STASHED_SHADERS: &[&str] = &[
        "blue-light-filter",
        "color-filter",
        "invert-colors",
        "vibrance",
    ];
    const CONFIG_WITH_DEFAULT: &str = r#"
        [[shader]]
        name = "color-filter"
        default = true

        [[shader]]
        name = "blue-light-filter"
        start_time = 01:00:00
        end_time = 03:00:00

        [[shader]]
        name = "invert-colors"
        start_time = 23:00:00
        end_time = 01:00:00
    "#;
    const CONFIG_NO_DEFAULT: &str = r#"
        [[shader]]
        name = "blue-light-filter"
        start_time = 01:00:00
        end_time = 03:00:00

        [[shader]]
        name = "invert-colors"
        start_time = 23:00:00
        end_time = 01:00:00
    "#;

    #[track_caller]
    fn check(case: Case) {
        #[track_caller]
        fn _check(case: &Case, variant: TestVariant) {
            let (config, expected_shader) = match variant {
                TestVariant::WithDefault => {
                    (CONFIG_WITH_DEFAULT, &case.expected_shader.with_default)
                }
                TestVariant::NoDefault => (CONFIG_NO_DEFAULT, &case.expected_shader.no_default),
            };

            let Case {
                time,
                initial_shader,
                expected_shader: _,
            } = &case;

            let mut space = Space::new();
            let _stash = space.stash_runtime_shaders(STASHED_SHADERS);
            space.with_time(time).with_config(config);

            space.hyprshade_cmd().arg("on").arg(initial_shader).run();

            hyprshade_cmd_snapshot!(
                case.snapshot_name(variant),
                space
                    .hyprshade_cmd()
                    .arg("toggle")
                    .args(["vibrance", "--fallback-auto"])
            );

            assert_eq!(
                space.current_shader().as_deref(),
                *expected_shader,
                "{:?}",
                variant
            );
        }

        _check(&case, TestVariant::WithDefault);
        _check(&case, TestVariant::NoDefault);
    }

    check(Case {
        time: "00:00:00",
        initial_shader: "vibrance",
        expected_shader: ExpectedShader {
            with_default: Some("invert-colors"),
            no_default: Some("invert-colors"),
        },
    });
    check(Case {
        time: "00:00:00",
        initial_shader: "blue-light-filter",
        expected_shader: ExpectedShader {
            with_default: Some("vibrance"),
            no_default: Some("vibrance"),
        },
    });
    check(Case {
        time: "00:00:00",
        initial_shader: "invert-colors",
        expected_shader: ExpectedShader {
            with_default: Some("vibrance"),
            no_default: Some("vibrance"),
        },
    });
    check(Case {
        time: "00:00:00",
        initial_shader: "color-filter",
        expected_shader: ExpectedShader {
            with_default: Some("vibrance"),
            no_default: Some("vibrance"),
        },
    });

    check(Case {
        time: "00:59:59.999999999",
        initial_shader: "vibrance",
        expected_shader: ExpectedShader {
            with_default: Some("invert-colors"),
            no_default: Some("invert-colors"),
        },
    });
    check(Case {
        time: "00:59:59.999999999",
        initial_shader: "blue-light-filter",
        expected_shader: ExpectedShader {
            with_default: Some("vibrance"),
            no_default: Some("vibrance"),
        },
    });
    check(Case {
        time: "00:59:59.999999999",
        initial_shader: "invert-colors",
        expected_shader: ExpectedShader {
            with_default: Some("vibrance"),
            no_default: Some("vibrance"),
        },
    });
    check(Case {
        time: "00:59:59.999999999",
        initial_shader: "color-filter",
        expected_shader: ExpectedShader {
            with_default: Some("vibrance"),
            no_default: Some("vibrance"),
        },
    });

    check(Case {
        time: "01:00:00",
        initial_shader: "vibrance",
        expected_shader: ExpectedShader {
            with_default: Some("blue-light-filter"),
            no_default: Some("blue-light-filter"),
        },
    });
    check(Case {
        time: "01:00:00",
        initial_shader: "blue-light-filter",
        expected_shader: ExpectedShader {
            with_default: Some("vibrance"),
            no_default: Some("vibrance"),
        },
    });
    check(Case {
        time: "01:00:00",
        initial_shader: "invert-colors",
        expected_shader: ExpectedShader {
            with_default: Some("vibrance"),
            no_default: Some("vibrance"),
        },
    });
    check(Case {
        time: "01:00:00",
        initial_shader: "color-filter",
        expected_shader: ExpectedShader {
            with_default: Some("vibrance"),
            no_default: Some("vibrance"),
        },
    });

    check(Case {
        time: "02:00:00",
        initial_shader: "vibrance",
        expected_shader: ExpectedShader {
            with_default: Some("blue-light-filter"),
            no_default: Some("blue-light-filter"),
        },
    });
    check(Case {
        time: "02:00:00",
        initial_shader: "blue-light-filter",
        expected_shader: ExpectedShader {
            with_default: Some("vibrance"),
            no_default: Some("vibrance"),
        },
    });
    check(Case {
        time: "02:00:00",
        initial_shader: "invert-colors",
        expected_shader: ExpectedShader {
            with_default: Some("vibrance"),
            no_default: Some("vibrance"),
        },
    });
    check(Case {
        time: "02:00:00",
        initial_shader: "color-filter",
        expected_shader: ExpectedShader {
            with_default: Some("vibrance"),
            no_default: Some("vibrance"),
        },
    });

    check(Case {
        time: "02:59:59.999999999",
        initial_shader: "vibrance",
        expected_shader: ExpectedShader {
            with_default: Some("blue-light-filter"),
            no_default: Some("blue-light-filter"),
        },
    });
    check(Case {
        time: "02:59:59.999999999",
        initial_shader: "blue-light-filter",
        expected_shader: ExpectedShader {
            with_default: Some("vibrance"),
            no_default: Some("vibrance"),
        },
    });
    check(Case {
        time: "02:59:59.999999999",
        initial_shader: "invert-colors",
        expected_shader: ExpectedShader {
            with_default: Some("vibrance"),
            no_default: Some("vibrance"),
        },
    });
    check(Case {
        time: "02:59:59.999999999",
        initial_shader: "color-filter",
        expected_shader: ExpectedShader {
            with_default: Some("vibrance"),
            no_default: Some("vibrance"),
        },
    });

    check(Case {
        time: "03:00:00",
        initial_shader: "vibrance",
        expected_shader: ExpectedShader {
            with_default: Some("color-filter"),
            no_default: None,
        },
    });
    check(Case {
        time: "03:00:00",
        initial_shader: "blue-light-filter",
        expected_shader: ExpectedShader {
            with_default: Some("vibrance"),
            no_default: Some("vibrance"),
        },
    });
    check(Case {
        time: "03:00:00",
        initial_shader: "invert-colors",
        expected_shader: ExpectedShader {
            with_default: Some("vibrance"),
            no_default: Some("vibrance"),
        },
    });
    check(Case {
        time: "03:00:00",
        initial_shader: "color-filter",
        expected_shader: ExpectedShader {
            with_default: Some("vibrance"),
            no_default: Some("vibrance"),
        },
    });
}

#[test]
fn fallback_flags_are_mutually_exclusive() {
    let space = Space::new();

    hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("toggle").args([
        "--fallback",
        "vibrance",
        "--fallback-auto",
        "blue-light-filter",
    ]), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: the argument '--fallback <FALLBACK>' cannot be used with '--fallback-auto'

    Usage: hyprshade toggle --fallback <FALLBACK> <SHADER>

    For more information, try '--help'.
    ");

    hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("toggle").args([
        "--fallback",
        "vibrance",
        "--fallback-default",
        "blue-light-filter",
    ]), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: the argument '--fallback <FALLBACK>' cannot be used with '--fallback-default'

    Usage: hyprshade toggle --fallback <FALLBACK> <SHADER>

    For more information, try '--help'.
    ");

    hyprshade_cmd_snapshot!(space.hyprshade_cmd().arg("toggle").args([
        "--fallback-auto",
        "--fallback-default",
        "blue-light-filter",
    ]), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: the argument '--fallback-auto' cannot be used with '--fallback-default'

    Usage: hyprshade toggle --fallback-auto <SHADER>

    For more information, try '--help'.
    ");
}

#[test]
#[ignore = "already tested in test_on::error::fails_merging_var"]
fn fails_merging_var() {
    unimplemented!()
}

#[test]
#[ignore = "should be near-identical to fails_merging_var"]
fn fails_merging_fallback_var() {
    unimplemented!()
}

#[test]
#[ignore = "already tested in test_on::error::fails_parsing_var"]
fn fails_parsing_var() {
    unimplemented!()
}

#[test]
#[ignore = "should be near-identical to fails_parsing_var"]
fn fails_parsing_fallback_var() {
    unimplemented!()
}
