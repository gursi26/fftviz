use std::path::Path;
use crate::*;

pub fn cli_args_to_fft_args(cli_args: CLIArgs) -> FFTArgs {
    if !Path::new(&cli_args.file_path).is_file() {
        println!("File \"{}\" not found!", cli_args.file_path);
        std::process::exit(1);
    }
    FFTArgs {
        file_path: Path::new(&cli_args.file_path).to_path_buf(),
        border_size: cli_args.border_size as i32,
        border_color: cli_args.border_color,
        bar_color: cli_args.bar_color,
        disable_title: cli_args.disable_title,
        text_color: cli_args.text_color,
        font_size: cli_args.font_size as i32,
        background_color: cli_args.background_color
    }
}
