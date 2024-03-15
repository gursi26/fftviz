use bevy::prelude::Color;
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use serde_yaml::{self};
use std::fs::{create_dir_all, read_to_string, File, OpenOptions};
use std::io::BufWriter;
use std::{
    fs::remove_file,
    io::Write,
    path::PathBuf,
};

use crate::{CLIArgs, FFTArgs};

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigFFTArgs {
    pub border_size: Option<i32>,
    pub border_color: Option<String>,
    pub bar_color: Option<String>,
    pub display_track_name: Option<bool>,
    pub text_color: Option<String>,
    pub font_size: Option<i32>,
    pub background_color: Option<String>,
    pub smoothness: Option<u32>,
    pub freq_resolution: Option<u32>,
    pub window_width: Option<f32>,
    pub window_height: Option<f32>,
    pub min_freq: Option<f32>,
    pub max_freq: Option<f32>,
    pub display_gui: Option<bool>,
    pub volume: Option<u32>,
}

impl Default for ConfigFFTArgs {
    fn default() -> Self {
        ConfigFFTArgs {
            border_size: Some(1),
            border_color: Some(String::from("000000")),
            bar_color: Some(String::from("FF0000")),
            display_track_name: Some(false),
            text_color: Some(String::from("FFFFFF")),
            font_size: Some(25),
            background_color: Some(String::from("000000")),
            smoothness: Some(1),
            freq_resolution: Some(90),
            window_width: Some(1000.0),
            window_height: Some(700.0),
            min_freq: Some(0.0),
            max_freq: Some(5000.0),
            display_gui: Some(false),
            volume: Some(50),
        }
    }
}

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

pub fn read_config() -> ConfigFFTArgs {
    let config_file = File::open(config_path()).expect("Could not find file lmao");
    let user_config_yaml =
        serde_yaml::from_reader(config_file).expect("Could not read values lmao");
    user_config_yaml
}

macro_rules! update_cli_arg {
    ($cli_arg: expr, $config_arg: expr, $default_config_arg: expr) => {
        if let None = *$cli_arg {
            if let None = $config_arg {
                *$cli_arg = $default_config_arg;
            } else {
                *$cli_arg = $config_arg;
            }
        }
    };
}

macro_rules! overwrite_non_default_args {
    ($user_config_arg: expr, $fft_arg: expr) => {
        *$user_config_arg = Some($fft_arg);
    };
}

pub fn convert_color_to_hex(c: &Color) -> String {
    let c_vec = c.rgb_to_vec3();
    let (r, g, b) = (
        (c_vec.x * 255.0) as u32,
        (c_vec.y * 255.0) as u32,
        (c_vec.z * 255.0) as u32,
    );
    format!(
        "{}{}{}",
        format!("{:02X}", r),
        format!("{:02X}", g),
        format!("{:02X}", b)
    )
}

pub fn write_fftargs_to_config(args: &FFTArgs) {
    let mut default_args = ConfigFFTArgs::default();
    overwrite_non_default_args!(
        &mut default_args.background_color,
        convert_color_to_hex(&args.background_color)
    );
    overwrite_non_default_args!(
        &mut default_args.border_color,
        convert_color_to_hex(&args.border_color)
    );
    overwrite_non_default_args!(
        &mut default_args.bar_color,
        convert_color_to_hex(&args.bar_color)
    );
    overwrite_non_default_args!(
        &mut default_args.text_color,
        convert_color_to_hex(&args.text_color)
    );
    overwrite_non_default_args!(&mut default_args.border_size, args.border_size);
    overwrite_non_default_args!(&mut default_args.display_track_name, args.track_name);
    overwrite_non_default_args!(&mut default_args.font_size, args.font_size);
    overwrite_non_default_args!(&mut default_args.smoothness, args.smoothness);
    overwrite_non_default_args!(&mut default_args.freq_resolution, args.freq_resolution);
    overwrite_non_default_args!(&mut default_args.window_width, args.window_width);
    overwrite_non_default_args!(&mut default_args.window_height, args.window_height);
    overwrite_non_default_args!(&mut default_args.min_freq, args.min_freq);
    overwrite_non_default_args!(&mut default_args.max_freq, args.max_freq);
    overwrite_non_default_args!(&mut default_args.volume, args.volume);

    let cfg_path = config_path();
    create_dir_all(cfg_path.as_path().parent().unwrap()).unwrap();

    let config_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&cfg_path)
        .expect("Could not open file.");
    serde_yaml::to_writer(config_file, &default_args).unwrap();

    let mut cfg_yaml: Vec<String> = read_to_string(&cfg_path)
        .unwrap() // panic on possible file-reading errors
        .lines() // split the string into an iterator of string slices
        .map(String::from) // make each slice into a string
        .collect(); // gather them together into a vector

    cfg_yaml.retain(|x| x.contains(":"));

    remove_file(&cfg_path).unwrap();

    let f = File::create(&cfg_path).expect("Unable to create file");
    let mut f = BufWriter::new(f);
    f.write_all(cfg_yaml.join("\n").as_bytes())
        .expect("Unable to write data");
}

#[allow(dead_code)]
pub fn reset_config_file() {
    let default_user_config = ConfigFFTArgs::default();

    let cfg_path = config_path();
    create_dir_all(cfg_path.as_path().parent().unwrap()).unwrap();

    let config_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(cfg_path)
        .expect("Could not open file.");
    serde_yaml::to_writer(config_file, &default_user_config).unwrap();
}

pub fn merge_config_with_cli_args(args: &mut CLIArgs, use_default: bool) {
    let default_user_config = ConfigFFTArgs::default();
    if !config_exists() {
        update_cli_arg!(
            &mut args.background_color,
            None::<String>,
            default_user_config.background_color
        );
        update_cli_arg!(
            &mut args.bar_color,
            None::<String>,
            default_user_config.bar_color
        );
        update_cli_arg!(
            &mut args.border_color,
            None::<String>,
            default_user_config.border_color
        );
        update_cli_arg!(
            &mut args.border_size,
            None::<i32>,
            default_user_config.border_size
        );
        update_cli_arg!(
            &mut args.font_size,
            None::<i32>,
            default_user_config.font_size
        );
        update_cli_arg!(
            &mut args.freq_resolution,
            None::<u32>,
            default_user_config.freq_resolution
        );
        update_cli_arg!(
            &mut args.max_freq,
            None::<f32>,
            default_user_config.max_freq
        );
        update_cli_arg!(
            &mut args.min_freq,
            None::<f32>,
            default_user_config.min_freq
        );
        update_cli_arg!(
            &mut args.smoothness,
            None::<u32>,
            default_user_config.smoothness
        );
        update_cli_arg!(
            &mut args.text_color,
            None::<String>,
            default_user_config.text_color
        );
        update_cli_arg!(&mut args.volume, None::<u32>, default_user_config.volume);
        update_cli_arg!(
            &mut args.window_width,
            None::<f32>,
            default_user_config.window_width
        );
        update_cli_arg!(
            &mut args.window_height,
            None::<f32>,
            default_user_config.window_height
        );
        return;
    }

    let user_config_yaml: ConfigFFTArgs;
    if use_default {
        user_config_yaml = ConfigFFTArgs::default();
    } else {
        user_config_yaml = read_config();
    }

    if let Some(x) = user_config_yaml.display_track_name {
        args.track_name = Some(x);
    }
    if let Some(x) = user_config_yaml.display_gui {
        args.display_gui = Some(x);
    }

    update_cli_arg!(
        &mut args.background_color,
        user_config_yaml.background_color,
        default_user_config.background_color
    );
    update_cli_arg!(
        &mut args.bar_color,
        user_config_yaml.bar_color,
        default_user_config.bar_color
    );
    update_cli_arg!(
        &mut args.border_color,
        user_config_yaml.border_color,
        default_user_config.border_color
    );
    update_cli_arg!(
        &mut args.border_size,
        user_config_yaml.border_size,
        default_user_config.border_size
    );
    update_cli_arg!(
        &mut args.font_size,
        user_config_yaml.font_size,
        default_user_config.font_size
    );
    update_cli_arg!(
        &mut args.freq_resolution,
        user_config_yaml.freq_resolution,
        default_user_config.freq_resolution
    );
    update_cli_arg!(
        &mut args.max_freq,
        user_config_yaml.max_freq,
        default_user_config.max_freq
    );
    update_cli_arg!(
        &mut args.min_freq,
        user_config_yaml.min_freq,
        default_user_config.min_freq
    );
    update_cli_arg!(
        &mut args.smoothness,
        user_config_yaml.smoothness,
        default_user_config.smoothness
    );
    update_cli_arg!(
        &mut args.text_color,
        user_config_yaml.text_color,
        default_user_config.text_color
    );
    update_cli_arg!(
        &mut args.volume,
        user_config_yaml.volume,
        default_user_config.volume
    );
    update_cli_arg!(
        &mut args.window_width,
        user_config_yaml.window_width,
        default_user_config.window_width
    );
    update_cli_arg!(
        &mut args.window_height,
        user_config_yaml.window_height,
        default_user_config.window_height
    );
}
