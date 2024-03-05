use std::path::Path;
use crate::*;

pub fn cli_args_to_fft_args(cli_args: CLIArgs) -> FFTArgs {
    if !Path::new(&cli_args.file_path).is_file() {
        println!("File \"{}\" not found!", cli_args.file_path);
        std::process::exit(1);
    }

    bar_smoothness_constraint(cli_args.bar_smoothness);
    fft_fps_constraint(cli_args.fft_fps);
    freq_resolution_constraint(cli_args.freq_resolution);
    averaging_window_constraint(cli_args.averaging_window);

    FFTArgs {
        file_path: Path::new(&cli_args.file_path).to_path_buf(),
        border_size: cli_args.border_size as i32,
        border_color: cli_args.border_color,
        bar_color: cli_args.bar_color,
        disable_title: cli_args.disable_title,
        text_color: cli_args.text_color,
        font_size: cli_args.font_size as i32,
        background_color: cli_args.background_color,
        bar_smoothness: cli_args.bar_smoothness,
        fft_fps: cli_args.fft_fps,
        freq_resolution: cli_args.freq_resolution,
        window_height: cli_args.window_height,
        window_width: cli_args.window_width,
        averaging_window: cli_args.averaging_window,
        min_freq: cli_args.min_freq,
        max_freq: cli_args.max_freq
    }
}

pub fn bar_smoothness_constraint(v: u32) { 
    if v > 3 {
        println!("bar-smoothness must be between 1 and 3 inclusive.");
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

fn averaging_window_constraint(v: u32) { 
    if v < 1 || v > 5 {
        println!("averaging-window must be between 1 and 5 inclusive.");
        std::process::exit(1);
    }
}
