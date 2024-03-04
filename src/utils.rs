use std::path::Path;
use crate::*;

pub fn cli_args_to_fft_args(cli_args: CLIArgs) -> FFTArgs {
    if !Path::new(&cli_args.file_path).is_file() {
        println!("File \"{}\" not found!", cli_args.file_path);
        std::process::exit(1);
    }
    FFTArgs {
        file_path: cli_args.file_path,
        border_size: cli_args.border_size as i32,
        border_color: Color::from_hex(&cli_args.border_color[..]).unwrap(),
        bar_color: Color::from_hex(&cli_args.bar_color[..]).unwrap(),
        disable_title: cli_args.disable_title,
        text_color: Color::from_hex(&cli_args.text_color[..]).unwrap(),
        font_size: cli_args.font_size as i32,
        background_color: Color::from_hex(&cli_args.background_color[..]).unwrap()
    }
}
