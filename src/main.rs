#![allow(dead_code, unused_imports, unused)]
mod utils;
mod fft;

use bevy::log::tracing_subscriber::filter::targets::IntoIter;
use bevy::render::mesh::VertexAttributeValues;
use bevy::utils::dbg;
use rayon::iter::IntoParallelRefIterator;
use utils::*;
use fft::*;

use clap::{ArgAction, Parser};
use rodio::{source::Source, Decoder, OutputStream};
use std::time::Duration;
use std::{ffi::OsString, iter::Peekable};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};



// Constants
const RENDERING_FPS: u32 = 60;
const SCREEN_WIDTH: i32 = 1000;
const SCREEN_HEIGHT: i32 = 700;

const FREQUENCY_RESOLUTION: u32 = 100;
const FFT_FPS: u32 = 12;
const FREQ_WINDOW_LOW: f32 = 0.0;
const FREQ_WINDOW_HIGH: f32 = 5000.0;
const FFT_WINDOW: i32 =
    ((256 as u64 / 107 as u64) * FREQUENCY_RESOLUTION as u64).next_power_of_two() as i32;
const BAR_INTERPOLATION_FACTOR: u32 = 2;
const RESCALING_THRESHOLDS: &[f32] = &[0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];
const RESCALING_FACTOR: &[f32] = &[2.0, 1.7, 1.3, 1.2, 1.1, 1.0, 0.9, 0.8, 0.7];


#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct CLIArgs {
    /// File path to Audio file
    #[arg()]
    file_path: String,

    /// Border size for each bar
    #[arg(long = "border-size", default_value_t = 1)]
    border_size: u32,

    /// Border color for each bar (in hex)
    #[arg(long = "border-color", default_value_t = String::from("000000"))]
    border_color: String,

    /// Color for each bar (in hex)
    #[arg(long = "bar-color", default_value_t = String::from("FF0000"))]
    bar_color: String,

    /// Whether to disable printing
    #[arg(long = "disable-title", action = ArgAction::SetTrue)]
    disable_title: bool,

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

struct FFTArgs {
    file_path: PathBuf,
    border_size: i32,
    border_color: String,
    bar_color: String,
    disable_title: bool,
    text_color: String,
    font_size: i32,
    background_color: String
}

#[derive(Resource)]
struct FFTQueue {
    fft: Vec<Vec<f32>>,
    curr_bars: Vec<Handle<Mesh>>,
    i: usize
}

fn update_bars (
    mut commands: Commands,
    mut window: Query<&mut Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut fft_queue: ResMut<FFTQueue>
) {
    let (h, w) = (window.single_mut().physical_height(), window.single_mut().physical_width());
    let curr_fft = &fft_queue.fft[fft_queue.i];
    let num_bars = fft_queue.fft[0].len();
    let bar_size = w as f32 / num_bars as f32;

    for (handle, new_value) in fft_queue.curr_bars.iter().zip(curr_fft.iter()) {
        let rect = meshes.get_mut(handle).unwrap();
        let dims = rect.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap();
        match dims {
            VertexAttributeValues::Float32x3(x) => {
                x[0][1] = new_value.clone() * (h / 2) as f32;
                x[1][1] = new_value.clone() * (h / 2) as f32;
                x[2][1] = - new_value.clone() * (h / 2) as f32;
                x[3][1] = - new_value.clone() * (h / 2) as f32;
            },
            _ => {}
        }
    }

    fft_queue.i += 1;
}

fn startup (
    mut commands: Commands,
    mut window: Query<&mut Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut fft_queue: ResMut<FFTQueue>
) {
    commands.spawn(Camera2dBundle::default());

    let w = window.single_mut().physical_width();
    let num_bars = fft_queue.fft[0].len();
    let bar_size = w as f32 / num_bars as f32;
    let mut handle_vec = Vec::new();

    for i in 0..num_bars {
        let handle = meshes.add(Rectangle::new(bar_size, i as f32 * 10.0));
        handle_vec.push(handle.clone());

        commands.spawn(MaterialMesh2dBundle {
            mesh: Mesh2dHandle(handle),
            material: materials.add(Color::GREEN),
            transform: Transform::from_xyz(
                bar_size * i as f32 + (bar_size / 2.0) - (w / 2) as f32,
                0.0,
                0.0,
            ),
            ..default()
        });

    }
    fft_queue.curr_bars = handle_vec;
}

fn main() {
    let args = cli_args_to_fft_args(CLIArgs::parse());

    let p = PathBuf::from(OsString::from(&args.file_path));

    let file_name = p.file_stem().unwrap().to_str().unwrap();
    let mut cache_path = p.parent().unwrap().to_path_buf();
    cache_path.push(format!(".{}.fft", file_name));

    println!("Computing FFT...");
    let mut fft = compute_fft(&p);

    fft = normalize_fft(fft, RESCALING_THRESHOLDS, RESCALING_FACTOR);
    let mut fft_vec = fft.fft;

    for c in fft_vec.iter_mut() {
        let mut reversed = c.clone();
        reversed.reverse();
        reversed.append(c);
        *c = reversed;
    }

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let file = BufReader::new(File::open(&p).unwrap());
    let source = Decoder::new(file).unwrap();
    stream_handle.play_raw(source.convert_samples()).unwrap();

    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "fftviz".into(),
                    name: Some("fftviz".into()),
                    resolution: (SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32).into(),
                    prevent_default_event_handling: false,
                    enabled_buttons: bevy::window::EnabledButtons {
                        maximize: false,
                        ..Default::default()
                    },
                    visible: true,
                    ..default()
                }),
                ..default()
            }),
        ))
        .insert_resource(FFTQueue {
            fft: fft_vec,
            curr_bars: Vec::new(),
            i: 0
        }).add_systems(Startup, startup)
        .add_systems(
            Update,
            (
                update_bars.run_if(bevy::time::common_conditions::on_timer(Duration::from_millis(80))),
            )
        )
        .run();

    // let num_frame_gen = RENDERING_FPS / FFT_FPS;
    // let mut fft_chunk: Vec<f32> = Vec::new();

    // while !rl.window_should_close() && fft.peek().is_some() && !rl.is_key_down(KeyboardKey::KEY_Q) {
    //     let mut d = rl.begin_drawing(&thread);
    //     d.clear_background(args.background_color);

    //     if i as u32 % num_frame_gen == 0 {
    //         fft_chunk = fft.next().unwrap();
    //     } else {
    //         let next_chunk = fft.peek().unwrap();
    //         fft_chunk = time_interpolate(&fft_chunk, next_chunk);
    //     }

    //     let mut new_chunk = fft_chunk.clone();
    //     space_interpolate(&mut new_chunk, BAR_INTERPOLATION_FACTOR);

    //     let (h, w) = (d.get_screen_height(), d.get_screen_width());
    //     let bar_start_idxs = (0..((w as i32) + 1))
    //         .step_by(w as usize / new_chunk.len() as usize)
    //         .collect::<Vec<i32>>();

    //     for j in 0..(bar_start_idxs.len() - 1 as usize) {
    //         let curr_fft_value =
    //             (new_chunk[j.clamp(0, new_chunk.len() - 1)] * h as f32 * 0.5) as i32;
    //         let (start_x_id, end_x_id) = (bar_start_idxs[j], bar_start_idxs[j + 1]);
    //         d.draw_rectangle(
    //             start_x_id,
    //             (h / 2) - curr_fft_value,
    //             end_x_id - start_x_id,
    //             curr_fft_value * 2,
    //             args.border_color,
    //         );

    //         d.draw_rectangle(
    //             start_x_id + args.border_size,
    //             (h / 2) - curr_fft_value + args.border_size,
    //             end_x_id - start_x_id - (args.border_size * 2),
    //             curr_fft_value * 2 - (args.border_size * 2),
    //             args.bar_color,
    //         );
    //     }

    //     if !args.disable_title {
    //         d.draw_text(
    //             &format!("Playing: {:?}", p.file_stem().unwrap().to_str().unwrap())[..],
    //             10,
    //             10,
    //             args.font_size,
    //             args.text_color,
    //         );
    //     }
    //     i += 1;
    // }
}
