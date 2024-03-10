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

    /// Temporal resolution for FFT calculation (rendering always occurs at 60 fps with interpolation)
    #[arg(long = "fft-fps", default_value_t = 12)]
    fft_fps: u32,

    /// Smoothing factor for spatial interpolation between bars
    #[clap(long = "smoothness", default_value_t = 1)]
    smoothness: u32,

    /// Number of individual frequencies detected by the FFT
    #[arg(long = "freq-resolution", default_value_t = 90)]
    freq_resolution: u32,

    /// Maximum frequency detected by FFT
    #[arg(long = "min-freq", default_value_t = 0.0)]
    min_freq: f32,

    /// Minimum frequency detected by FFT
    #[arg(long = "max-freq", default_value_t = 5000.0)]
    max_freq: f32,

    /// Volume
    #[arg(long = "volume", default_value_t = 0.7)]
    volume: f32,

    /// Window width
    #[arg(long = "width", default_value_t = 1000.0)]
    window_width: f32,

    /// Window height
    #[arg(long = "height", default_value_t = 700.0)]
    window_height: f32,

    /// Border size for each bar
    #[arg(long = "border-size", default_value_t = 1)]
    border_size: u32,

    /// Border color for each bar (in hex)
    #[arg(long = "border-color", default_value_t = String::from("000000"))]
    border_color: String,

    /// Color for each bar (in hex)
    #[arg(long = "bar-color", default_value_t = String::from("FF0000"))]
    bar_color: String,

    /// Use if you want track name to be printed
    #[arg(long = "track-name", action = ArgAction::SetTrue)]
    track_name: bool,

    /// Use if you want the gui to be open when launched
    #[arg(long = "display-gui", action = ArgAction::SetTrue)]
    display_gui: bool,

    /// Color for currently playing text (in hex)
    #[arg(long = "text-color", default_value_t = String::from("FFFFFF"))]
    text_color: String,

    /// Font size of currently playing label
    #[arg(long = "font-size", default_value_t = 25)]
    font_size: u32,

    // Background color (in hex)
    #[arg(long = "background-color", default_value_t = String::from("000000"))]
    background_color: String,
}

pub fn cli_args_to_fft_args(cli_args: CLIArgs) -> FFTArgs {
    if !Path::new(&cli_args.file_path).is_file() {
        println!("File \"{}\" not found!", cli_args.file_path);
        std::process::exit(1);
    }

    bar_smoothness_constraint(cli_args.smoothness);
    fft_fps_constraint(cli_args.fft_fps);
    freq_resolution_constraint(cli_args.freq_resolution);

    FFTArgs {
        file_path: Path::new(&cli_args.file_path).to_path_buf(),
        border_size: cli_args.border_size as i32,
        border_color: Color::hex(cli_args.border_color).unwrap(),
        bar_color: Color::hex(cli_args.bar_color).unwrap(),
        track_name: cli_args.track_name,
        text_color: Color::hex(cli_args.text_color).unwrap(),
        font_size: cli_args.font_size as i32,
        background_color: Color::hex(cli_args.background_color).unwrap(),
        smoothness: cli_args.smoothness,
        fft_fps: cli_args.fft_fps,
        freq_resolution: cli_args.freq_resolution,
        window_height: cli_args.window_height,
        window_width: cli_args.window_width,
        min_freq: cli_args.min_freq,
        max_freq: cli_args.max_freq,
        display_gui: cli_args.display_gui,
        volume: cli_args.volume,
        paused: false,
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

pub fn fft_fps_constraint(v: u32) { 
    if v < 6 || v > 60 || RENDERING_FPS % v != 0 {
        println!("fft-fps must be between 6 and 60 inclusive and divide RENDERING_FPS = {}.", RENDERING_FPS);
        std::process::exit(1);
    }
}

fn freq_resolution_constraint(v: u32) { 
    if v < 10 || v > 300 {
        println!("freq-resolution must be between 10 and 300 inclusive.");
        std::process::exit(1);
    }
}
