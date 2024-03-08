#![allow(unused)]

mod args;
mod fft;
mod systems;

use args::*;
use fft::*;
use systems::get_keyboard_input::*;
use systems::egui::*;
use systems::startup::*;
use systems::update_fft::*;
use systems::update_view_settings::*;

use bevy::render::mesh::VertexAttributeValues;
use bevy_egui::egui::{Align2, Color32, Stroke};
use bevy::sprite::Anchor;
use bevy::{
    app::AppExit,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use clap::{ArgAction, Parser};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use rodio::{source::Source, Decoder, OutputStream};
use std::ffi::OsString;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::time::Duration;

// TODO : Add to brew, and whatever linux/windows uses
// TODO : Fix indexing error 
// TODO : Add border resizing, font resizing at runtime
// TODO : Add yaml config file for changing default settings
// TODO : Add a button to gui to write current state to config file
// TODO : Add instructions somewhere (e for gui, q to quit)
// TODO : Combine FFTArgs with AppState?

// Constants
const RENDERING_FPS: u32 = 60;
const RESCALING_THRESHOLDS: &[f32] = &[0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];
const RESCALING_FACTOR: &[f32] = &[0.4, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.6, 0.5];

const MIN_BAR_HEIGHT: f32 = 0.001;
const MAX_BAR_HEIGHT: f32 = 0.45;

#[derive(Resource, Clone, Component, Debug)]
struct FFTArgs {
    file_path: PathBuf,
    border_size: i32,
    border_color: Color,
    bar_color: Color,
    track_name: bool,
    text_color: Color,
    font_size: i32,
    background_color: Color,
    fft_fps: u32,
    bar_smoothness: u32,
    freq_resolution: u32,
    window_width: i32,
    window_height: i32,
    averaging_window: u32,
    min_freq: f32,
    max_freq: f32,
    display_gui: bool,
}

#[derive(Resource)]
struct AppState {
    fft: Vec<Vec<f32>>,
    curr_bars: Vec<(Handle<Mesh>, Handle<ColorMaterial>)>,
    despawn_handles: Vec<Entity>,
    total_frame_counter: usize,
    fft_frame_counter: usize,
}


fn compute_and_preprocess_fft(fp: &PathBuf, args: &FFTArgs) -> Vec<Vec<f32>> {
    println!("Computing FFT...");
    let mut fft = compute_fft(
        fp,
        args.fft_fps,
        args.freq_resolution,
        args.min_freq,
        args.max_freq,
    );

    fft = smooth_fft(fft, args.averaging_window);
    fft = normalize_fft(fft, RESCALING_THRESHOLDS, RESCALING_FACTOR);

    let mut fft_vec = fft.fft;

    for c in fft_vec.iter_mut() {
        let mut reversed = c.clone();
        reversed.reverse();
        reversed.append(c);
        *c = reversed;
    }

    fft_vec
        .par_iter_mut()
        .for_each(|x| space_interpolate(x, args.bar_smoothness));

    fft_vec
}

fn main() {
    std::env::set_var("RUST_LOG", "none");

    // Parse CLI args
    let args = parse_cli_args();
    let fp = PathBuf::from(OsString::from(&args.file_path));

    // Compute and preprocess FFT (spatial + temporal interpolation and normalization)
    let fft_vec = compute_and_preprocess_fft(&fp, &args);

    // Initialize Bevy app
    let mut binding = App::new();
    let app = binding

        // Insert plugins
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: format!(
                        "fftviz - {}",
                        args.file_path.file_stem().unwrap().to_str().unwrap()
                    )
                    .into(),
                    name: Some("fftviz".into()),
                    resolution: (args.window_width as f32, args.window_height as f32).into(),
                    resizable: false,
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
        )
        .add_plugins(EguiPlugin)

        // Insert resources
        .insert_resource(ClearColor(args.background_color))
        .insert_resource(AppState {
            fft: fft_vec,
            curr_bars: Vec::new(),
            despawn_handles: Vec::new(),
            fft_frame_counter: 0,
            total_frame_counter: 0,
        })
        .insert_resource(args)

        // Insert systems
        .add_systems(Startup, startup)
        .add_systems(Update, ui_example_system)
        .add_systems(
            Update,
            (update_fft.run_if(bevy::time::common_conditions::on_timer(
                Duration::from_secs_f64(1.0 / RENDERING_FPS as f64),
            )),),
        )
        .add_systems(Update, get_keyboard_input)
        .add_systems(Update, update_view_settings);

    // Play audio and start app
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let file = BufReader::new(File::open(&fp).unwrap());
    let source = Decoder::new(file).unwrap();
    stream_handle.play_raw(source.convert_samples()).unwrap();

    app.run();
}

