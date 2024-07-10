use crate::hyprctl;

pub fn run() -> anyhow::Result<()> {
    eprintln!("Implementation is incomlete");

    if let Some(shader_path) = hyprctl::shader::get()? {
        println!("{shader_path}")
    }

    Ok(())
}
