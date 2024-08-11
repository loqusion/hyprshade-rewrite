mod test;

use std::process::ExitCode;

use xshell::Shell;

fn usage() {
    eprintln!(
        "\
        Usage: cargo xtask <COMMAND>\n\
        \n\
        Commands:\n\
        \x20\x20test  Run `cargo test` with hooks\
        "
    );
}

fn main() -> eyre::Result<ExitCode> {
    let shell = Shell::new()?;

    match std::env::args().nth(1).as_deref() {
        Some("test") => test::main(shell),
        Some(invalid) => {
            eprintln!("error: unrecognized subcommand '{invalid}'");
            eprintln!();
            usage();
            Ok(ExitCode::FAILURE)
        }
        None => {
            usage();
            Ok(ExitCode::FAILURE)
        }
    }
}
