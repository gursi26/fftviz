use dirs::home_dir;
use std::path::PathBuf;

pub fn config_path() -> PathBuf {
    let mut config_path = home_dir().unwrap();
    config_path.push(".config");
    config_path.push("fftviz");
    config_path.push("config.yaml");
    config_path
}

pub fn config_exists() -> bool {
    let cfg_path = config_path();
    cfg_path.exists()
}
