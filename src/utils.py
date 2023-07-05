import os
from os import path


def get_shaders_dir() -> str:
    config_home = os.getenv("XDG_CONFIG_HOME")
    if config_home is None:
        home = os.getenv("HOME")
        if home is None:
            raise ValueError("$HOME environment variable is not set")
        config_home = path.join(home, ".config")
    shaders_dir = path.join(config_home, "hypr", "shaders")
    if not path.isdir(shaders_dir):
        raise FileNotFoundError(f"Shaders directory {shaders_dir} does not exist")
    return path.join(config_home, "hypr", "shaders")


def get_shader_path(shader: str) -> str:
    shader_path = shader
    if not path.isfile(shader_path):
        shaders_dir = get_shaders_dir()
        shader_path = path.join(shaders_dir, glsl_ext(shader))
        if not path.isfile(shader_path):
            raise FileNotFoundError(
                f"Shader {shader} does not exist; check contents of {shaders_dir}"
            )
    return shader_path


def glsl_ext(pathname: str) -> str:
    if pathname.endswith(".glsl"):
        return pathname
    return f"{pathname}.glsl"
