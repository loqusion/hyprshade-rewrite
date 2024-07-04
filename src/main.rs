mod cli;
mod hyprctl;

use clap::Parser;
use cli::Cli;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    cli.run()?;

    Ok(())
}
