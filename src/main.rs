#![allow(unused)]

mod fft;
mod utils;

use bevy_egui::egui::{Align2, Color32, Stroke};
use fft::*;
use utils::*;

use bevy::render::mesh::VertexAttributeValues;
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

// Constants
const RENDERING_FPS: u32 = 60;
const RESCALING_THRESHOLDS: &[f32] = &[0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];
const RESCALING_FACTOR: &[f32] = &[0.4, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.6, 0.5];

const MIN_BAR_HEIGHT: f32 = 0.001;
const MAX_BAR_HEIGHT: f32 = 0.45;

#[derive(Component)]
struct ColorText;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct CLIArgs {
    /// File path to Audio file
    #[arg()]
    file_path: String,

    /// Temporal resolution for FFT calculation (rendering always occurs at 60 fps with interpolation)
    #[arg(long = "fft-fps", default_value_t = 12)]
    fft_fps: u32,

    /// Smoothing factor for spatial interpolation between bars
    #[clap(long = "bar-smoothness", default_value_t = 1)]
    bar_smoothness: u32,

    /// Number of individual frequencies detected by the FFT
    #[arg(long = "freq-resolution", default_value_t = 90)]
    freq_resolution: u32,

    /// Size of averaging window (larger = less movement)
    #[arg(long = "min-freq", default_value_t = 0.0)]
    min_freq: f32,

    /// Size of averaging window (larger = less movement)
    #[arg(long = "max-freq", default_value_t = 5000.0)]
    max_freq: f32,

    /// Size of averaging window (larger = less movement)
    #[arg(long = "averaging-window", default_value_t = 1)]
    averaging_window: u32,

    /// Window width
    #[arg(long = "width", default_value_t = 1000)]
    window_width: i32,

    /// Window height
    #[arg(long = "height", default_value_t = 700)]
    window_height: i32,

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
    #[arg(long = "track-name", action = ArgAction::SetTrue)]
    track_name: bool,

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

#[derive(Resource)]
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
}

#[derive(Resource)]
struct AppState {
    fft: Vec<Vec<f32>>,
    curr_bars: Vec<(Handle<Mesh>, Handle<ColorMaterial>)>,
    display_gui: bool,
    despawn_handles: Vec<Entity>,
    c: usize,
    i: usize,
}

fn update_bars(
    mut commands: Commands,
    mut window: Query<&mut Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut fft_queue: ResMut<AppState>,
    mut args: ResMut<FFTArgs>,
    mut clear_color: ResMut<ClearColor>,
    mut text_query: Query<&mut Text, With<ColorText>>
) {
    let h = window.single_mut().height();
    let mut update_i = false;
    let interval = RENDERING_FPS / args.fft_fps;

    let curr_fft = match fft_queue.c as u32 % interval {
        0 => {
            if fft_queue.i > fft_queue.fft.len() {
                std::process::exit(0);
            }
            update_i = true;
            fft_queue.fft[fft_queue.i].clone()
        }
        rem => time_interpolate(
            &(fft_queue.fft[fft_queue.i - 1]),
            &(fft_queue.fft[fft_queue.i]),
            rem as f32 / interval as f32,
        ),
    };

    for mut text in &mut text_query {
        if args.track_name {
            text.sections[0].style.color = args.text_color;
        } else {
            text.sections[0].style.color = Color::rgba(0.0, 0.0, 0.0, 0.0);
        }
    }

    clear_color.0 = args.background_color;

    for (handle, new_value) in fft_queue
        .curr_bars
        .chunks(2)
        .zip(curr_fft.iter())
    {
        let ((handle1, color_handle1), (handle2, color_handle2)) = (handle[0].clone(), handle[1].clone());

        let color = materials.get_mut(color_handle1).unwrap();
        color.color = args.border_color;

        let rect = meshes.get_mut(handle1).unwrap();
        let dims = rect.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap();
        let bar_value_1 =
            (new_value.clone() * (h / 2.0) as f32).clamp(h * MIN_BAR_HEIGHT, h * MAX_BAR_HEIGHT);
        match dims {
            VertexAttributeValues::Float32x3(x) => {
                x[0][1] = bar_value_1;
                x[1][1] = bar_value_1;
                x[2][1] = -bar_value_1;
                x[3][1] = -bar_value_1;
            }
            _ => {}
        }
        let color = rect.attribute(Mesh::ATTRIBUTE_COLOR);
 
        let color = materials.get_mut(color_handle2).unwrap();
        color.color = args.bar_color;

        let rect = meshes.get_mut(handle2).unwrap();
        let dims = rect.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap();
        let bar_value_2 = (new_value.clone() * (h / 2.0) as f32 - args.border_size as f32)
            .clamp(h * MIN_BAR_HEIGHT, h * MAX_BAR_HEIGHT);

        match dims {
            VertexAttributeValues::Float32x3(x) => {
                x[0][1] = bar_value_2;
                x[1][1] = bar_value_2;
                x[2][1] = -bar_value_2;
                x[3][1] = -bar_value_2;
            }
            _ => {}
        }
    }

    if update_i {
        fft_queue.i += 1;
    }
    fft_queue.c += 1;
}

fn startup(
    mut commands: Commands,
    mut window: Query<&mut Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut fft_queue: ResMut<AppState>,
    clear_color: Res<ClearColor>,
    args: Res<FFTArgs>,
) {
    commands.spawn(Camera2dBundle::default());

    let w = window.single_mut().width();
    let h = window.single_mut().height();

    let text_style = TextStyle {
        font: Default::default(),
        font_size: args.font_size as f32,
        color: args.text_color,
    };

    commands.spawn((Text2dBundle {
        text: Text::from_section(
            format!(
                "Playing: \"{}\"",
                args.file_path.file_name().unwrap().to_str().unwrap()
            ),
            text_style.clone(),
        ),
        transform: Transform::from_xyz(-(w as f32 / 2.0) + 10.0, (h as f32 / 2.0) - 10.0, 0.0),
        text_anchor: Anchor::TopLeft,
        ..default()
    }, ColorText));

    let num_bars = fft_queue.fft[0].len();

    let (mesh_handles, despawn_handles) = spawn_bars(
        num_bars as u32,
        w,
        &args,
        &mut commands,
        &mut meshes,
        &mut materials,
    );

    fft_queue.curr_bars = mesh_handles;
    fft_queue.despawn_handles = despawn_handles;
}

fn check_quit(keyboard_input: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        exit.send(AppExit);
    }
}

fn spawn_bars(
    num_bars: u32,
    w: f32,
    args: &FFTArgs,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) -> (Vec<(Handle<Mesh>, Handle<ColorMaterial>)>, Vec<Entity>) {
    let bar_size = w as f32 / num_bars as f32;
    let mut handle_vec = Vec::new();
    let mut despawn_handles = Vec::new();

    for i in 0..num_bars {
        let handle1 = meshes.add(Rectangle::new(bar_size, 0.0));
        let color_handle = materials.add(args.border_color);
        handle_vec.push((handle1.clone(), color_handle.clone()));

        let dh = commands
            .spawn(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(handle1),
                material: color_handle,
                transform: Transform::from_xyz(
                    bar_size * i as f32 + (bar_size / 2.0) - (w / 2.0) as f32,
                    0.0,
                    0.0,
                ),
                ..default()
            })
            .id();
        despawn_handles.push(dh);

        let handle2 = meshes.add(Rectangle::new(bar_size - args.border_size as f32, 0.0));
        let color_handle = materials.add(args.bar_color);
        handle_vec.push((handle2.clone(), color_handle.clone()));

        let dh = commands
            .spawn(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(handle2),
                material: color_handle,
                transform: Transform::from_xyz(
                    bar_size * i as f32 + (bar_size / 2.0) - (w / 2.0) as f32,
                    0.0,
                    0.0,
                ),
                ..default()
            })
            .id();
        despawn_handles.push(dh);
    }
    (handle_vec, despawn_handles)
}

fn main() {
    std::env::set_var("RUST_LOG", "none");
    let args = cli_args_to_fft_args(CLIArgs::parse());

    let p = PathBuf::from(OsString::from(&args.file_path));

    let file_name = p.file_stem().unwrap().to_str().unwrap();
    let mut cache_path = p.parent().unwrap().to_path_buf();
    cache_path.push(format!(".{}.fft", file_name));

    println!("Computing FFT...");
    let mut fft = compute_fft(
        &p,
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

    let mut binding = App::new();
    let app = binding
        .insert_resource(ClearColor(args.background_color))
        .add_plugins((DefaultPlugins.set(WindowPlugin {
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
        }),))
        .add_plugins(EguiPlugin)
        .insert_resource(AppState {
            fft: fft_vec,
            curr_bars: Vec::new(),
            display_gui: false,
            despawn_handles: Vec::new(),
            i: 0,
            c: 0,
        })
        .insert_resource(args)
        .add_systems(Startup, startup)
        .add_systems(Update, ui_example_system)
        .add_systems(
            Update,
            (update_bars.run_if(bevy::time::common_conditions::on_timer(
                Duration::from_secs_f64(1.0 / RENDERING_FPS as f64),
            )),),
        )
        .add_systems(Update, check_quit);
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let file = BufReader::new(File::open(&p).unwrap());
    let source = Decoder::new(file).unwrap();
    stream_handle.play_raw(source.convert_samples()).unwrap();

    app.run();
}

fn ui_example_system(
    mut contexts: EguiContexts,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut app_state: ResMut<AppState>,
    mut args: ResMut<FFTArgs>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyE) {
        app_state.display_gui = !app_state.display_gui;
    }

    if app_state.display_gui {
        let window_handle = egui::Window::new("")
            .fixed_size(egui::Vec2 { x: 100.0, y: 100.0 })
            .anchor(Align2::RIGHT_TOP, egui::Vec2::new(-10.0, 10.0))
            // .fixed_pos(egui::Pos2 { x: 10.0, y: 10.0 })
            .collapsible(false);

        window_handle.show(contexts.ctx_mut(), |ui| {
            ui.checkbox(&mut args.track_name, "Display title: ");
            if args.track_name {
                ui.horizontal(|ui| {
                    ui.label("Text color: ");
                    color_picker_widget(ui, &mut args.text_color);
                });
            }

            ui.horizontal(|ui| {
                ui.label("Bar color: ");
                color_picker_widget(ui, &mut args.bar_color);
            });

            ui.horizontal(|ui| {
                ui.label("Border color: ");
                color_picker_widget(ui, &mut args.border_color);
            });

            ui.horizontal(|ui| {
                ui.label("Background color: ");
                color_picker_widget(ui, &mut args.background_color);
            });
        });
    }
}

fn color_picker_widget(ui: &mut egui::Ui, color: &mut Color) -> egui::Response {
    let [r, g, b, a] = color.as_rgba_f32();
    let mut egui_color: egui::Rgba = egui::Rgba::from_srgba_unmultiplied(
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
        (a * 255.0) as u8,
    );
    let res = egui::widgets::color_picker::color_edit_button_rgba(
        ui,
        &mut egui_color,
        egui::color_picker::Alpha::Opaque,
    );
    let [r, g, b, a] = egui_color.to_srgba_unmultiplied();
    *color = Color::rgba(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    );
    res
}
