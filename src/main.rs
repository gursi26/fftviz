mod utils;
mod fft;

use utils::*;
use fft::*;

use clap::{ArgAction, Parser};
use raylib::prelude::*;
use rodio::{source::Source, Decoder, OutputStream};
use std::ffi::OsString;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;


// Constants
const RENDERING_FPS: u32 = 60;
const SCREEN_WIDTH: i32 = 1780;
const SCREEN_HEIGHT: i32 = 1000;

const FREQUENCY_RESOLUTION: u32 = 100;
const FFT_FPS: u32 = 12;
const FREQ_WINDOW_LOW: f32 = 0.0;
const FREQ_WINDOW_HIGH: f32 = 5000.0;
const FFT_WINDOW: i32 =
    ((256 as u64 / 107 as u64) * FREQUENCY_RESOLUTION as u64).next_power_of_two() as i32;
const BAR_INTERPOLATION_FACTOR: u32 = 2;
const RESCALING_THRESHOLDS: &[f32] = &[0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];
const RESCALING_FACTOR: &[f32] = &[2.0, 1.7, 1.3, 1.2, 1.1, 1.0, 0.9, 0.8, 0.7];
const CACHE_FFT: bool = false;
const FORCE_CACHE_REFRESH: bool = false;


#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct CLIArgs {
    /// File path to Audio file
    #[arg()]
    file_path: String,

    /// Border size for each bar
    #[arg(long = "border-size", default_value_t = 1)]
    border_size: u32,

    /// Border color for each bar
    #[arg(long = "border-color", default_value_t = String::from("000000"))]
    border_color: String,

    /// Color for each bar
    #[arg(long = "bar-color", default_value_t = String::from("FF0000"))]
    bar_color: String,

    /// Set to false to disable printing currently playing song title
    #[arg(long = "print-text", action = ArgAction::SetFalse)]
    print_text: bool,

    /// Text of currently playing label
    #[arg(long = "text-color", default_value_t = String::from("FFFFFF"))]
    text_color: String,

    /// Font size of currently playing label
    #[arg(long = "font-size", default_value_t = 25)]
    font_size: u32,
}

struct FFTArgs {
    file_path: String,
    border_size: i32,
    border_color: Color,
    bar_color: Color,
    print_text: bool,
    text_color: Color,
    font_size: i32,
}

fn main() {
    let args = cli_args_to_fft_args(CLIArgs::parse());

    let p = PathBuf::from(OsString::from(&args.file_path));

    let file_name = p.file_stem().unwrap().to_str().unwrap();
    let mut cache_path = p.parent().unwrap().to_path_buf();
    cache_path.push(format!(".{}.fft", file_name));

    println!("Computing FFT...");
    let mut fft;
    if CACHE_FFT {
        fft = compute_and_cache_fft(&p);
    } else if cache_path.is_file() {
        fft = read_fft_from_binary_file(&cache_path).unwrap();
    } else {
        fft = compute_fft(&p);
    }

    fft = normalize_fft(fft, RESCALING_THRESHOLDS, RESCALING_FACTOR);
    let mut fft_vec = fft.fft;

    for c in fft_vec.iter_mut() {
        let mut reversed = c.clone();
        reversed.reverse();
        reversed.append(c);
        *c = reversed;
    }

    let mut fft = fft_vec.into_iter().peekable();
    let mut i = 0;

    let (mut rl, thread) = raylib::init().title("fft-visualizer").build();
    rl.set_target_fps(RENDERING_FPS);
    rl.set_window_size(SCREEN_WIDTH, SCREEN_HEIGHT);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let file = BufReader::new(File::open(&p).unwrap());
    let source = Decoder::new(file).unwrap();
    stream_handle.play_raw(source.convert_samples()).unwrap();

    let num_frame_gen = RENDERING_FPS / FFT_FPS;
    let mut fft_chunk: Vec<f32> = Vec::new();

    while !rl.window_should_close() && fft.peek().is_some() && !rl.is_key_down(KeyboardKey::KEY_Q) {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        if i as u32 % num_frame_gen == 0 {
            fft_chunk = fft.next().unwrap();
        } else {
            let next_chunk = fft.peek().unwrap();
            fft_chunk = time_interpolate(&fft_chunk, next_chunk);
        }

        let mut new_chunk = fft_chunk.clone();
        space_interpolate(&mut new_chunk, BAR_INTERPOLATION_FACTOR);

        let (h, w) = (d.get_screen_height(), d.get_screen_width());
        let bar_start_idxs = (0..((w as i32) + 1))
            .step_by(w as usize / new_chunk.len() as usize)
            .collect::<Vec<i32>>();

        for j in 0..(bar_start_idxs.len() - 1 as usize) {
            let curr_fft_value =
                (new_chunk[j.clamp(0, new_chunk.len() - 1)] * h as f32 * 0.5) as i32;
            let (start_x_id, end_x_id) = (bar_start_idxs[j], bar_start_idxs[j + 1]);
            d.draw_rectangle(
                start_x_id,
                (h / 2) - curr_fft_value,
                end_x_id - start_x_id,
                curr_fft_value * 2,
                args.border_color,
            );

            d.draw_rectangle(
                start_x_id + args.border_size,
                (h / 2) - curr_fft_value + args.border_size,
                end_x_id - start_x_id - (args.border_size * 2),
                curr_fft_value * 2 - (args.border_size * 2),
                args.bar_color,
            );
        }

        if args.print_text {
            d.draw_text(
                &format!("Playing: {:?}", p.file_stem().unwrap().to_str().unwrap())[..],
                10,
                10,
                args.font_size,
                args.text_color,
            );
        }
        i += 1;
    }
}
