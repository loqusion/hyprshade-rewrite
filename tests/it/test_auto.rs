use insta_cmd::assert_cmd_snapshot;

use crate::common::Space;

#[test]
fn fails_without_config() {
    let space = Space::new();
    assert_cmd_snapshot!(space.hyprshade_cmd().arg("auto"), @r###"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: 
       0: [91mno configuration file found[0m

    Location:
       [35msrc/cli/subcommand/auto.rs[0m:[35m36[0m

    [93mWarning[0m: A configuration file is required to call this command
    [96mSuggestion[0m: For more information, see https://github.com/loqusion/hyprshade#configuration

    Backtrace omitted. Run with RUST_BACKTRACE=1 environment variable to display it.
    Run with RUST_BACKTRACE=full to include source snippets.
    "###);
}
