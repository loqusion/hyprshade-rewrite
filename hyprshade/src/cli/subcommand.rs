mod auto;
use auto::Auto;
mod current;
use current::Current;
mod install;
use install::Install;
mod ls;
use ls::Ls;
mod off;
use off::Off;
mod on;
use on::On;
mod toggle;
use toggle::Toggle;

#[derive(Debug, clap::Subcommand)]
pub enum HyprshadeSubcommand {
    Auto(Auto),
    Current(Current),
    Install(Install),
    Ls(Ls),
    Off(Off),
    On(On),
    Toggle(Toggle),
}
