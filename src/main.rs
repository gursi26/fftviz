#![allow(unused)]

mod args;
mod fft;
mod systems;
mod config;

use args::*;
use fft::*;
use config::*;
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

// TODO : Add to other package managers
// TODO : Add yaml config file for changing default settings
// TODO : Add a button to gui to write current state to config file
// TODO : Add a button to gui to reset to default 

// Constants
const RENDERING_FPS: u32 = 60;
const RESCALING_THRESHOLDS: &[f32] = &[0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];
const INTENSITY_RESCALING: &[f32] = &[0.4, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.6, 0.5];
const FREQ_RESCALING: &[f32] = &[0.9, 1.2, 1.2, 1.2, 1.0];
const AVERAGING_WINDOW: u32 = 1;
const FFT_FPS: u32 = 12;

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
    smoothness: u32,
    freq_resolution: u32,
    window_width: f32,
    window_height: f32,
    min_freq: f32,
    max_freq: f32,
    display_gui: bool,
    volume: u32,
    paused: bool,
    fft_fps: u32,
}

#[derive(Resource)]
struct AppState {
    sink: rodio::Sink,
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

    fft = smooth_fft(fft, AVERAGING_WINDOW);
    fft = intensity_normalize_fft(fft, RESCALING_THRESHOLDS, INTENSITY_RESCALING);
    fft = frequency_normalize_fft(fft, FREQ_RESCALING);

    let mut fft_vec = fft.fft;

    // Reverses bar order and prepends
    for c in fft_vec.iter_mut() {
        let mut reversed = c.clone();
        reversed.reverse();
        reversed.append(c);
        *c = reversed;
    }

    fft_vec
        .par_iter_mut()
        .for_each(|x| space_interpolate(x, args.smoothness));

    fft_vec
}

fn main() {
    std::env::set_var("RUST_LOG", "none");

    // Parse CLI args
    let args = parse_cli_args();
    let fp = PathBuf::from(OsString::from(&args.file_path));

    // Compute and preprocess FFT (spatial + temporal interpolation and normalization)
    let fft_vec = compute_and_preprocess_fft(&fp, &args);

    let volume = args.volume;

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
    let file = BufReader::new(File::open(&fp).unwrap());
    let source = Decoder::new(file).unwrap();
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&stream_handle).unwrap();
    sink.set_volume(volume as f32 / 100.0);
    sink.append(source);

    app.insert_resource(AppState {
        sink,
        fft: fft_vec,
        curr_bars: Vec::new(),
        despawn_handles: Vec::new(),
        fft_frame_counter: 0,
        total_frame_counter: 0,
    });

    app.run();
}

