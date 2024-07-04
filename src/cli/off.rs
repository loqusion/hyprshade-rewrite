use crate::hyprctl;

pub fn run() -> anyhow::Result<()> {
    hyprctl::shader::clear()?;
    Ok(())
}
