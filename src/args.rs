use std::path::Path;
use bevy::prelude::*;
use crate::*;
use clap::{ArgAction, Parser};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct CLIArgs {
    /// File path to Audio file
    #[arg()]
    file_path: String,

    /// Smoothing factor for spatial interpolation between bars
    #[clap(long = "smoothness", default_value = None)]
    pub smoothness: Option<u32>,

    /// Number of individual frequencies detected by the FFT
    #[arg(long = "freq-resolution", default_value = None)]
    pub freq_resolution: Option<u32>,

    /// Maximum frequency detected by FFT
    #[arg(long = "min-freq", default_value = None)]
    pub min_freq: Option<f32>,

    /// Minimum frequency detected by FFT
    #[arg(long = "max-freq", default_value = None)]
    pub max_freq: Option<f32>,

    /// Volume
    #[arg(long = "volume", default_value = None)]
    pub volume: Option<f32>,

    /// Window width
    #[arg(long = "width", default_value = None)]
    pub window_width: Option<f32>,

    /// Window height
    #[arg(long = "height", default_value = None)]
    pub window_height: Option<f32>,

    /// Border size for each bar
    #[arg(long = "border-size", default_value = None)]
    pub border_size: Option<i32>,

    /// Border color for each bar (in hex)
    #[arg(long = "border-color", default_value = None)]
    pub border_color: Option<String>,

    /// Color for each bar (in hex)
    #[arg(long = "bar-color", default_value = None)]
    pub bar_color: Option<String>,

    /// Use if you want track name to be printed
    #[arg(long = "track-name", action = ArgAction::SetTrue)]
    pub track_name: Option<bool>,

    /// Use if you want the gui to be open when launched
    #[arg(long = "display-gui", action = ArgAction::SetTrue)]
    pub display_gui: Option<bool>,

    /// Color for currently playing text (in hex)
    #[arg(long = "text-color", default_value = None)]
    pub text_color: Option<String>,

    /// Font size of currently playing label
    #[arg(long = "font-size", default_value = None)]
    pub font_size: Option<i32>,

    // Background color (in hex)
    #[arg(long = "background-color", default_value = None)]
    pub background_color: Option<String>,
}

pub fn cli_args_to_fft_args(mut cli_args: CLIArgs) -> FFTArgs {
    if !Path::new(&cli_args.file_path).is_file() {
        println!("File \"{}\" not found!", cli_args.file_path);
        std::process::exit(1);
    }

    // Merges cli args with args in config.yaml.
    // Precendence: Cli args > config.yaml args > default values
    merge_config_with_cli_args(&mut cli_args);

    bar_smoothness_constraint(cli_args.smoothness.unwrap());
    freq_resolution_constraint(cli_args.freq_resolution.unwrap());

    FFTArgs {
        file_path: Path::new(&cli_args.file_path).to_path_buf(),
        border_size: cli_args.border_size.unwrap(),
        border_color: Color::hex(cli_args.border_color.unwrap()).unwrap(),
        bar_color: Color::hex(cli_args.bar_color.unwrap()).unwrap(),
        track_name: cli_args.track_name.unwrap(),
        text_color: Color::hex(cli_args.text_color.unwrap()).unwrap(),
        font_size: cli_args.font_size.unwrap(),
        background_color: Color::hex(cli_args.background_color.unwrap()).unwrap(),
        smoothness: cli_args.smoothness.unwrap(),
        freq_resolution: cli_args.freq_resolution.unwrap(),
        window_height: cli_args.window_height.unwrap(),
        window_width: cli_args.window_width.unwrap(),
        min_freq: cli_args.min_freq.unwrap(),
        max_freq: cli_args.max_freq.unwrap(),
        display_gui: cli_args.display_gui.unwrap(),
        volume: cli_args.volume.unwrap(),
        paused: false,
        fft_fps: FFT_FPS
    }
}

pub fn parse_cli_args() -> FFTArgs {
    cli_args_to_fft_args(args::CLIArgs::parse())
}
// Value constraints
pub fn bar_smoothness_constraint(v: u32) { 
    if v > 3 {
        println!("smoothness must be between 0 and 3 inclusive.");
        std::process::exit(1);
    }
}

fn freq_resolution_constraint(v: u32) { 
    if v < 10 || v > 300 {
        println!("freq-resolution must be between 10 and 300 inclusive.");
        std::process::exit(1);
    }
}
