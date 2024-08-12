mod hooks;
mod insta;
mod test;

use std::process::ExitCode;

use xshell::Shell;

fn usage() {
    eprint!(
        "\
        Usage: cargo xtask <COMMAND>\n\
        \n\
        Commands:\n\
        \x20\x20insta  Run `cargo insta` with hooks\n\
        \x20\x20test   Run `cargo test` with hooks\n\
        "
    );
}

fn main() -> eyre::Result<ExitCode> {
    let shell = Shell::new()?;

    let args = std::env::args().collect::<Vec<_>>();
    match &args[1..] {
        [subcommand, args @ ..] if subcommand == "test" => test::main(shell, args),
        [subcommand, args @ ..] if subcommand == "insta" => insta::main(shell, args),
        [unrecognized_subcommand, ..] => {
            eprintln!("error: unrecognized subcommand '{unrecognized_subcommand}'");
            eprintln!();
            usage();
            Ok(ExitCode::FAILURE)
        }
        [] => {
            usage();
            Ok(ExitCode::FAILURE)
        }
    }
}
